//! Tag management for content organization.

use super::{ContentTag, DbPool, StorageError};

/// Create a new tag for an account. Returns the tag ID.
pub async fn create_tag_for(
    pool: &DbPool,
    account_id: &str,
    name: &str,
    color: Option<&str>,
) -> Result<i64, StorageError> {
    let result = sqlx::query("INSERT INTO content_tags (account_id, name, color) VALUES (?, ?, ?)")
        .bind(account_id)
        .bind(name)
        .bind(color)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// List all tags for an account, ordered by name.
pub async fn list_tags_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ContentTag>, StorageError> {
    sqlx::query_as::<_, ContentTag>("SELECT * FROM content_tags WHERE account_id = ? ORDER BY name")
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Assign a tag to a content item. No-op if already assigned.
pub async fn assign_tag_for(
    pool: &DbPool,
    content_id: i64,
    tag_id: i64,
) -> Result<(), StorageError> {
    sqlx::query("INSERT OR IGNORE INTO content_tag_assignments (content_id, tag_id) VALUES (?, ?)")
        .bind(content_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// List tags assigned to a specific content item for an account.
pub async fn list_draft_tags_for(
    pool: &DbPool,
    account_id: &str,
    content_id: i64,
) -> Result<Vec<ContentTag>, StorageError> {
    sqlx::query_as::<_, ContentTag>(
        "SELECT ct.* FROM content_tags ct \
         INNER JOIN content_tag_assignments cta ON cta.tag_id = ct.id \
         WHERE cta.content_id = ? AND ct.account_id = ? \
         ORDER BY ct.name",
    )
    .bind(content_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Remove a tag assignment from a content item.
pub async fn unassign_tag_for(
    pool: &DbPool,
    content_id: i64,
    tag_id: i64,
) -> Result<bool, StorageError> {
    let result =
        sqlx::query("DELETE FROM content_tag_assignments WHERE content_id = ? AND tag_id = ?")
            .bind(content_id)
            .bind(tag_id)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected() > 0)
}
