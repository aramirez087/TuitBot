//! Session management backed by SQLite.
//!
//! Sessions are created on successful passphrase login and stored as
//! SHA-256 hashes of the raw token. This way, a database compromise
//! does not leak usable session tokens.

use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};

use super::error::AuthError;
use crate::storage::DbPool;

/// Session lifetime: 7 days.
const SESSION_LIFETIME_DAYS: i64 = 7;

/// A session record as stored in the database.
#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub csrf_token: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_accessed_at: String,
}

/// Result of creating a new session: the raw token (for the cookie)
/// and associated metadata.
pub struct NewSession {
    pub raw_token: String,
    pub csrf_token: String,
    pub expires_at: String,
}

/// SHA-256 hash a raw token for storage.
fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a cryptographically random hex string.
fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(&buf)
}

/// Create a new session in the database.
///
/// Returns the raw token (to set in the cookie) and the CSRF token.
pub async fn create_session(pool: &DbPool) -> Result<NewSession, AuthError> {
    let id = random_hex(16);
    let raw_token = random_hex(32);
    let csrf_token = random_hex(16);
    let token_hash = hash_token(&raw_token);
    let now = Utc::now();
    let expires_at = now + Duration::days(SESSION_LIFETIME_DAYS);
    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let expires_str = expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO sessions (id, token_hash, csrf_token, created_at, expires_at, last_accessed_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&token_hash)
    .bind(&csrf_token)
    .bind(&now_str)
    .bind(&expires_str)
    .bind(&now_str)
    .execute(pool)
    .await
    .map_err(|e| AuthError::Database { source: e })?;

    Ok(NewSession {
        raw_token,
        csrf_token,
        expires_at: expires_str,
    })
}

/// Validate a session by raw token. Returns the session if valid and not expired.
///
/// Updates `last_accessed_at` on success.
pub async fn validate_session(
    pool: &DbPool,
    raw_token: &str,
) -> Result<Option<Session>, AuthError> {
    let token_hash = hash_token(raw_token);
    let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let row = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id, csrf_token, created_at, expires_at, last_accessed_at
         FROM sessions WHERE token_hash = ? AND expires_at > ?",
    )
    .bind(&token_hash)
    .bind(&now_str)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::Database { source: e })?;

    let Some((id, csrf_token, created_at, expires_at, last_accessed_at)) = row else {
        return Ok(None);
    };

    // Update last_accessed_at
    sqlx::query("UPDATE sessions SET last_accessed_at = ? WHERE id = ?")
        .bind(&now_str)
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| AuthError::Database { source: e })?;

    Ok(Some(Session {
        id,
        csrf_token,
        created_at,
        expires_at,
        last_accessed_at,
    }))
}

/// Delete a session by raw token (logout).
pub async fn delete_session(pool: &DbPool, raw_token: &str) -> Result<(), AuthError> {
    let token_hash = hash_token(raw_token);
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(&token_hash)
        .execute(pool)
        .await
        .map_err(|e| AuthError::Database { source: e })?;
    Ok(())
}

/// Remove all expired sessions.
pub async fn cleanup_expired(pool: &DbPool) -> Result<u64, AuthError> {
    let now_str = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(&now_str)
        .execute(pool)
        .await
        .map_err(|e| AuthError::Database { source: e })?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn create_and_validate_session() {
        let pool = init_test_db().await.unwrap();
        let new = create_session(&pool).await.unwrap();
        assert!(!new.raw_token.is_empty());
        assert!(!new.csrf_token.is_empty());

        let session = validate_session(&pool, &new.raw_token).await.unwrap();
        assert!(session.is_some());
        let session = session.unwrap();
        assert_eq!(session.csrf_token, new.csrf_token);
    }

    #[tokio::test]
    async fn validate_invalid_token_returns_none() {
        let pool = init_test_db().await.unwrap();
        let session = validate_session(&pool, "nonexistent-token").await.unwrap();
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn delete_session_invalidates_token() {
        let pool = init_test_db().await.unwrap();
        let new = create_session(&pool).await.unwrap();
        delete_session(&pool, &new.raw_token).await.unwrap();
        let session = validate_session(&pool, &new.raw_token).await.unwrap();
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn cleanup_expired_removes_old_sessions() {
        let pool = init_test_db().await.unwrap();

        // Insert an already-expired session
        sqlx::query(
            "INSERT INTO sessions (id, token_hash, csrf_token, created_at, expires_at, last_accessed_at)
             VALUES ('old', 'oldhash', 'oldcsrf', '2020-01-01T00:00:00Z', '2020-01-02T00:00:00Z', '2020-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let removed = cleanup_expired(&pool).await.unwrap();
        assert_eq!(removed, 1);
    }
}
