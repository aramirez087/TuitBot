//! Activity log storage for content lifecycle events.

use super::{ContentActivity, DbPool, StorageError};

/// Insert an activity log entry for a content item.
pub async fn insert_activity_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
    action: &str,
    detail: Option<&str>,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO content_activity \
         (content_id, account_id, action, detail) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(content_id)
    .bind(account_id)
    .bind(action)
    .bind(detail)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// List activity log entries for a content item, newest first.
pub async fn list_activity_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
) -> Result<Vec<ContentActivity>, StorageError> {
    sqlx::query_as::<_, ContentActivity>(
        "SELECT * FROM content_activity \
         WHERE content_id = ? AND account_id = ? \
         ORDER BY id DESC",
    )
    .bind(content_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}
