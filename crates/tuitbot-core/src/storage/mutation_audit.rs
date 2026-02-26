//! DB-backed mutation audit trail for idempotency and incident review.
//!
//! Every mutation-capable MCP tool records an entry before executing.
//! The table serves dual purposes:
//!   1. **Idempotency** — detect and short-circuit recent identical mutations.
//!   2. **Audit** — every mutation attempt is traceable via `correlation_id`.

use sha2::{Digest, Sha256};

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;
use crate::error::StorageError;

/// An entry in the mutation audit trail.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct MutationAuditEntry {
    pub id: i64,
    pub correlation_id: String,
    pub idempotency_key: Option<String>,
    pub tool_name: String,
    pub params_hash: String,
    pub params_summary: String,
    pub status: String,
    pub result_summary: Option<String>,
    pub rollback_action: Option<String>,
    pub error_message: Option<String>,
    pub elapsed_ms: Option<i64>,
    pub account_id: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Compute a SHA-256 hash of the canonical params JSON.
pub fn compute_params_hash(tool_name: &str, params_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tool_name.as_bytes());
    hasher.update(b"|");
    hasher.update(params_json.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Truncate a JSON string for display (max 500 chars).
pub fn truncate_summary(json: &str, max_len: usize) -> String {
    if json.len() <= max_len {
        json.to_string()
    } else {
        format!("{}…", &json[..max_len])
    }
}

/// Check if a recent successful mutation with the same fingerprint exists.
///
/// Returns the cached entry if found within `window_seconds` (default 300 = 5 min).
pub async fn find_recent_duplicate(
    pool: &DbPool,
    tool_name: &str,
    params_hash: &str,
    window_seconds: u32,
) -> Result<Option<MutationAuditEntry>, StorageError> {
    let entry = sqlx::query_as::<_, MutationAuditEntry>(
        "SELECT * FROM mutation_audit
         WHERE tool_name = ? AND params_hash = ? AND status = 'success'
           AND created_at >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' seconds')
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(tool_name)
    .bind(params_hash)
    .bind(window_seconds)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(entry)
}

/// Check if a recent mutation with a specific idempotency key exists.
pub async fn find_by_idempotency_key(
    pool: &DbPool,
    key: &str,
) -> Result<Option<MutationAuditEntry>, StorageError> {
    let entry = sqlx::query_as::<_, MutationAuditEntry>(
        "SELECT * FROM mutation_audit
         WHERE idempotency_key = ?
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(entry)
}

/// Insert a new pending mutation audit record.
///
/// Returns the DB row ID.
pub async fn insert_pending(
    pool: &DbPool,
    correlation_id: &str,
    idempotency_key: Option<&str>,
    tool_name: &str,
    params_hash: &str,
    params_summary: &str,
) -> Result<i64, StorageError> {
    insert_pending_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        correlation_id,
        idempotency_key,
        tool_name,
        params_hash,
        params_summary,
    )
    .await
}

/// Insert a new pending mutation audit record for a specific account.
pub async fn insert_pending_for(
    pool: &DbPool,
    account_id: &str,
    correlation_id: &str,
    idempotency_key: Option<&str>,
    tool_name: &str,
    params_hash: &str,
    params_summary: &str,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO mutation_audit
            (correlation_id, idempotency_key, tool_name, params_hash, params_summary, account_id)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(correlation_id)
    .bind(idempotency_key)
    .bind(tool_name)
    .bind(params_hash)
    .bind(params_summary)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Mark a mutation as successfully completed.
pub async fn complete_success(
    pool: &DbPool,
    id: i64,
    result_summary: &str,
    rollback_action: Option<&str>,
    elapsed_ms: u64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE mutation_audit
         SET status = 'success',
             result_summary = ?,
             rollback_action = ?,
             elapsed_ms = ?,
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(result_summary)
    .bind(rollback_action)
    .bind(elapsed_ms as i64)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Mark a mutation as failed.
pub async fn complete_failure(
    pool: &DbPool,
    id: i64,
    error_message: &str,
    elapsed_ms: u64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE mutation_audit
         SET status = 'failure',
             error_message = ?,
             elapsed_ms = ?,
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(error_message)
    .bind(elapsed_ms as i64)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Mark a mutation as a duplicate (idempotency hit).
pub async fn mark_duplicate(
    pool: &DbPool,
    id: i64,
    original_correlation_id: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE mutation_audit
         SET status = 'duplicate',
             result_summary = ?,
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(format!(
        "{{\"duplicate_of\":\"{original_correlation_id}\"}}"
    ))
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Get recent mutations, newest first.
pub async fn get_recent(
    pool: &DbPool,
    limit: u32,
    tool_name: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<MutationAuditEntry>, StorageError> {
    let mut sql = String::from("SELECT * FROM mutation_audit WHERE 1=1");
    if tool_name.is_some() {
        sql.push_str(" AND tool_name = ?");
    }
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, MutationAuditEntry>(&sql);
    if let Some(t) = tool_name {
        query = query.bind(t);
    }
    if let Some(s) = status {
        query = query.bind(s);
    }
    query = query.bind(limit);

    query
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Get a single mutation by correlation ID.
pub async fn get_by_correlation_id(
    pool: &DbPool,
    correlation_id: &str,
) -> Result<Option<MutationAuditEntry>, StorageError> {
    sqlx::query_as::<_, MutationAuditEntry>("SELECT * FROM mutation_audit WHERE correlation_id = ?")
        .bind(correlation_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Get mutation counts grouped by status within a time window.
pub async fn get_status_counts(
    pool: &DbPool,
    since_hours: u32,
) -> Result<Vec<(String, i64)>, StorageError> {
    sqlx::query_as::<_, (String, i64)>(
        "SELECT status, COUNT(*) FROM mutation_audit
         WHERE created_at >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' hours')
         GROUP BY status ORDER BY COUNT(*) DESC",
    )
    .bind(since_hours)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn insert_and_complete_success() {
        let pool = init_test_db().await.expect("init db");

        let id = insert_pending(
            &pool,
            "corr-001",
            None,
            "x_post_tweet",
            "hash123",
            r#"{"text":"hello"}"#,
        )
        .await
        .expect("insert");

        complete_success(
            &pool,
            id,
            r#"{"tweet_id":"999"}"#,
            Some(r#"{"tool":"x_delete_tweet","params":{"tweet_id":"999"}}"#),
            150,
        )
        .await
        .expect("complete");

        let entry = get_by_correlation_id(&pool, "corr-001")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(entry.status, "success");
        assert_eq!(entry.tool_name, "x_post_tweet");
        assert!(entry.rollback_action.is_some());
        assert_eq!(entry.elapsed_ms, Some(150));
    }

    #[tokio::test]
    async fn insert_and_complete_failure() {
        let pool = init_test_db().await.expect("init db");

        let id = insert_pending(&pool, "corr-002", None, "x_like_tweet", "hash456", "{}")
            .await
            .expect("insert");

        complete_failure(&pool, id, "Rate limited", 50)
            .await
            .expect("complete");

        let entry = get_by_correlation_id(&pool, "corr-002")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(entry.status, "failure");
        assert_eq!(entry.error_message.as_deref(), Some("Rate limited"));
    }

    #[tokio::test]
    async fn find_recent_duplicate_within_window() {
        let pool = init_test_db().await.expect("init db");
        let hash = compute_params_hash("x_post_tweet", r#"{"text":"hi"}"#);

        let id = insert_pending(&pool, "corr-003", None, "x_post_tweet", &hash, "{}")
            .await
            .expect("insert");

        complete_success(&pool, id, r#"{"tweet_id":"111"}"#, None, 100)
            .await
            .expect("complete");

        let dup = find_recent_duplicate(&pool, "x_post_tweet", &hash, 300)
            .await
            .expect("find");
        assert!(dup.is_some());
        assert_eq!(dup.unwrap().correlation_id, "corr-003");
    }

    #[tokio::test]
    async fn no_duplicate_for_different_tool() {
        let pool = init_test_db().await.expect("init db");
        let hash = compute_params_hash("x_post_tweet", r#"{"text":"hi"}"#);

        let id = insert_pending(&pool, "corr-004", None, "x_post_tweet", &hash, "{}")
            .await
            .expect("insert");
        complete_success(&pool, id, "{}", None, 50)
            .await
            .expect("complete");

        let other_hash = compute_params_hash("x_like_tweet", r#"{"text":"hi"}"#);
        let dup = find_recent_duplicate(&pool, "x_like_tweet", &other_hash, 300)
            .await
            .expect("find");
        assert!(dup.is_none());
    }

    #[tokio::test]
    async fn idempotency_key_lookup() {
        let pool = init_test_db().await.expect("init db");

        let id = insert_pending(
            &pool,
            "corr-005",
            Some("user-key-abc"),
            "x_post_tweet",
            "hash789",
            "{}",
        )
        .await
        .expect("insert");
        complete_success(&pool, id, r#"{"tweet_id":"222"}"#, None, 75)
            .await
            .expect("complete");

        let found = find_by_idempotency_key(&pool, "user-key-abc")
            .await
            .expect("find")
            .expect("found");
        assert_eq!(found.correlation_id, "corr-005");

        let not_found = find_by_idempotency_key(&pool, "nonexistent")
            .await
            .expect("find");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn get_recent_with_filters() {
        let pool = init_test_db().await.expect("init db");

        for (tool, status_val) in [
            ("x_post_tweet", "success"),
            ("x_like_tweet", "success"),
            ("x_post_tweet", "failure"),
        ] {
            let id = insert_pending(
                &pool,
                &format!("c-{tool}-{status_val}"),
                None,
                tool,
                "h",
                "{}",
            )
            .await
            .expect("insert");
            if status_val == "success" {
                complete_success(&pool, id, "{}", None, 10)
                    .await
                    .expect("ok");
            } else {
                complete_failure(&pool, id, "err", 10).await.expect("ok");
            }
        }

        let all = get_recent(&pool, 10, None, None).await.expect("all");
        assert_eq!(all.len(), 3);

        let tweets = get_recent(&pool, 10, Some("x_post_tweet"), None)
            .await
            .expect("tweets");
        assert_eq!(tweets.len(), 2);

        let successes = get_recent(&pool, 10, None, Some("success"))
            .await
            .expect("successes");
        assert_eq!(successes.len(), 2);
    }

    #[tokio::test]
    async fn mark_duplicate_records_original() {
        let pool = init_test_db().await.expect("init db");

        let id = insert_pending(&pool, "corr-dup", None, "x_post_tweet", "h", "{}")
            .await
            .expect("insert");

        mark_duplicate(&pool, id, "corr-original")
            .await
            .expect("mark");

        let entry = get_by_correlation_id(&pool, "corr-dup")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(entry.status, "duplicate");
        assert!(entry
            .result_summary
            .as_deref()
            .unwrap()
            .contains("corr-original"));
    }

    #[tokio::test]
    async fn status_counts_aggregation() {
        let pool = init_test_db().await.expect("init db");

        for (i, status_val) in ["success", "success", "failure", "duplicate"]
            .iter()
            .enumerate()
        {
            let id = insert_pending(&pool, &format!("c-{i}"), None, "tool", "h", "{}")
                .await
                .expect("insert");
            match *status_val {
                "success" => {
                    complete_success(&pool, id, "{}", None, 10)
                        .await
                        .expect("ok");
                }
                "failure" => {
                    complete_failure(&pool, id, "err", 10).await.expect("ok");
                }
                "duplicate" => {
                    mark_duplicate(&pool, id, "other").await.expect("ok");
                }
                _ => {}
            }
        }

        let counts = get_status_counts(&pool, 24).await.expect("counts");
        let success_count = counts
            .iter()
            .find(|(s, _)| s == "success")
            .map(|(_, c)| *c)
            .unwrap_or(0);
        assert_eq!(success_count, 2);
    }

    #[tokio::test]
    async fn params_hash_deterministic() {
        let h1 = compute_params_hash("x_post_tweet", r#"{"text":"hello"}"#);
        let h2 = compute_params_hash("x_post_tweet", r#"{"text":"hello"}"#);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex

        let h3 = compute_params_hash("x_post_tweet", r#"{"text":"world"}"#);
        assert_ne!(h1, h3);
    }
}
