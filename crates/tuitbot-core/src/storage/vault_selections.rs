//! CRUD operations for the `vault_selections` transient table.
//!
//! Stores Ghostwriter selections sent from the Obsidian plugin with a
//! 30-minute TTL. Provides insert, lookup, rate-limit counting, and
//! expired-row cleanup.

use super::DbPool;
use crate::error::StorageError;

/// A row in the `vault_selections` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VaultSelection {
    pub id: i64,
    pub account_id: String,
    pub session_id: String,
    pub vault_name: String,
    pub file_path: String,
    pub selected_text: String,
    pub heading_context: Option<String>,
    pub selection_start_line: i64,
    pub selection_end_line: i64,
    pub note_title: Option<String>,
    pub frontmatter_tags: Option<String>,
    pub resolved_node_id: Option<i64>,
    pub resolved_chunk_id: Option<i64>,
    pub created_at: String,
    pub expires_at: String,
}

/// Insert a new vault selection. Returns the row ID.
#[allow(clippy::too_many_arguments)]
pub async fn insert_selection(
    pool: &DbPool,
    account_id: &str,
    session_id: &str,
    vault_name: &str,
    file_path: &str,
    selected_text: &str,
    heading_context: Option<&str>,
    selection_start_line: i64,
    selection_end_line: i64,
    note_title: Option<&str>,
    frontmatter_tags: Option<&str>,
    resolved_node_id: Option<i64>,
    resolved_chunk_id: Option<i64>,
    expires_at: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO vault_selections \
         (account_id, session_id, vault_name, file_path, selected_text, \
          heading_context, selection_start_line, selection_end_line, \
          note_title, frontmatter_tags, resolved_node_id, resolved_chunk_id, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(account_id)
    .bind(session_id)
    .bind(vault_name)
    .bind(file_path)
    .bind(selected_text)
    .bind(heading_context)
    .bind(selection_start_line)
    .bind(selection_end_line)
    .bind(note_title)
    .bind(frontmatter_tags)
    .bind(resolved_node_id)
    .bind(resolved_chunk_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get a selection by session_id, scoped to account.
///
/// Returns `None` if the selection does not exist, belongs to a different
/// account, or has expired.
pub async fn get_selection_by_session(
    pool: &DbPool,
    account_id: &str,
    session_id: &str,
) -> Result<Option<VaultSelection>, StorageError> {
    sqlx::query_as::<_, VaultSelection>(
        "SELECT * FROM vault_selections \
         WHERE account_id = ? AND session_id = ? AND expires_at > datetime('now')",
    )
    .bind(account_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Delete all expired selections. Returns the number of rows removed.
pub async fn cleanup_expired(pool: &DbPool) -> Result<u64, StorageError> {
    let result = sqlx::query("DELETE FROM vault_selections WHERE expires_at <= datetime('now')")
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Count selections created within the last `window_seconds` for an account.
///
/// Used for rate limiting (10 requests per 60 seconds per account).
pub async fn count_recent_for(
    pool: &DbPool,
    account_id: &str,
    window_seconds: i64,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vault_selections \
         WHERE account_id = ? AND created_at > datetime('now', '-' || ? || ' seconds')",
    )
    .bind(account_id)
    .bind(window_seconds)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    const ACCT: &str = "00000000-0000-0000-0000-000000000000";

    #[tokio::test]
    async fn insert_and_get_selection() {
        let pool = init_test_db().await.expect("init db");

        let id = insert_selection(
            &pool,
            ACCT,
            "sess-1",
            "marketing",
            "notes/test.md",
            "Some selected text",
            Some("# Title > ## Section"),
            10,
            15,
            Some("Test Note"),
            Some(r#"["tag1","tag2"]"#),
            None,
            None,
            "2099-12-31T23:59:59",
        )
        .await
        .expect("insert");

        assert!(id > 0);

        let sel = get_selection_by_session(&pool, ACCT, "sess-1")
            .await
            .expect("get")
            .expect("should exist");

        assert_eq!(sel.session_id, "sess-1");
        assert_eq!(sel.vault_name, "marketing");
        assert_eq!(sel.file_path, "notes/test.md");
        assert_eq!(sel.selected_text, "Some selected text");
        assert_eq!(sel.heading_context.as_deref(), Some("# Title > ## Section"));
        assert_eq!(sel.selection_start_line, 10);
        assert_eq!(sel.selection_end_line, 15);
        assert_eq!(sel.note_title.as_deref(), Some("Test Note"));
        assert!(sel.frontmatter_tags.is_some());
        assert!(sel.resolved_node_id.is_none());
        assert!(sel.resolved_chunk_id.is_none());
    }

    #[tokio::test]
    async fn get_selection_wrong_account() {
        let pool = init_test_db().await.expect("init db");

        insert_selection(
            &pool,
            ACCT,
            "sess-2",
            "vault",
            "note.md",
            "text",
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            "2099-12-31T23:59:59",
        )
        .await
        .expect("insert");

        let result = get_selection_by_session(&pool, "other-account", "sess-2")
            .await
            .expect("get");

        assert!(result.is_none(), "should not see other account's selection");
    }

    #[tokio::test]
    async fn cleanup_expired_removes_old() {
        let pool = init_test_db().await.expect("init db");

        // Insert an already-expired selection.
        insert_selection(
            &pool,
            ACCT,
            "sess-expired",
            "vault",
            "note.md",
            "text",
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            "2020-01-01T00:00:00",
        )
        .await
        .expect("insert expired");

        // Insert a valid selection.
        insert_selection(
            &pool,
            ACCT,
            "sess-valid",
            "vault",
            "note.md",
            "text",
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            "2099-12-31T23:59:59",
        )
        .await
        .expect("insert valid");

        let deleted = cleanup_expired(&pool).await.expect("cleanup");
        assert_eq!(deleted, 1);

        // Valid selection should still be retrievable.
        let valid = get_selection_by_session(&pool, ACCT, "sess-valid")
            .await
            .expect("get valid");
        assert!(valid.is_some());

        // Expired selection should be gone.
        let expired = get_selection_by_session(&pool, ACCT, "sess-expired")
            .await
            .expect("get expired");
        assert!(expired.is_none());
    }

    #[tokio::test]
    async fn count_recent_for_within_window() {
        let pool = init_test_db().await.expect("init db");

        // Insert two selections (created_at defaults to now).
        for i in 0..2 {
            insert_selection(
                &pool,
                ACCT,
                &format!("sess-rate-{i}"),
                "vault",
                "note.md",
                "text",
                None,
                0,
                0,
                None,
                None,
                None,
                None,
                "2099-12-31T23:59:59",
            )
            .await
            .expect("insert");
        }

        let count = count_recent_for(&pool, ACCT, 60).await.expect("count");
        assert_eq!(count, 2);

        // Different account should see 0.
        let other = count_recent_for(&pool, "other-acct", 60)
            .await
            .expect("count other");
        assert_eq!(other, 0);
    }
}
