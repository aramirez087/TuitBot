//! Revision snapshot storage for content lifecycle events.

use super::{ContentRevision, DbPool, StorageError};

/// Insert a revision snapshot for a content item.
pub async fn insert_revision_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
    content: &str,
    content_type: &str,
    trigger_kind: &str,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO content_revisions \
         (content_id, account_id, content, content_type, trigger_kind) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(content_id)
    .bind(account_id)
    .bind(content)
    .bind(content_type)
    .bind(trigger_kind)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Fetch a single revision by ID, scoped to account + content item.
pub async fn get_revision_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
    revision_id: i64,
) -> Result<Option<ContentRevision>, StorageError> {
    sqlx::query_as::<_, ContentRevision>(
        "SELECT * FROM content_revisions \
         WHERE id = ? AND content_id = ? AND account_id = ?",
    )
    .bind(revision_id)
    .bind(content_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// List revision snapshots for a content item, newest first.
pub async fn list_revisions_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
) -> Result<Vec<ContentRevision>, StorageError> {
    sqlx::query_as::<_, ContentRevision>(
        "SELECT * FROM content_revisions \
         WHERE content_id = ? AND account_id = ? \
         ORDER BY id DESC",
    )
    .bind(content_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}
