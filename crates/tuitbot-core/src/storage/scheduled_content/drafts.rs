//! Draft Studio operations: archive, restore, duplicate, and metadata updates.

use super::{DbPool, ScheduledContent, StorageError};

/// Archive a draft (set `archived_at` timestamp). No-op if already archived.
pub async fn archive_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE scheduled_content \
         SET archived_at = datetime('now'), updated_at = datetime('now') \
         WHERE id = ? AND account_id = ? AND archived_at IS NULL",
    )
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected() > 0)
}

/// Restore an archived draft (clear `archived_at`). No-op if not archived.
pub async fn restore_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE scheduled_content \
         SET archived_at = NULL, updated_at = datetime('now') \
         WHERE id = ? AND account_id = ? AND archived_at IS NOT NULL",
    )
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected() > 0)
}

/// Duplicate a draft, creating a new row with the same content.
/// Returns the new row's ID, or `None` if the source draft was not found.
pub async fn duplicate_draft_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
) -> Result<Option<i64>, StorageError> {
    let original = sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content WHERE id = ? AND account_id = ?",
    )
    .bind(id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let Some(original) = original else {
        return Ok(None);
    };

    let new_title = original
        .title
        .as_deref()
        .map(|t| format!("{t} (copy)"))
        .or_else(|| Some("(copy)".to_string()));

    let result = sqlx::query(
        "INSERT INTO scheduled_content \
         (account_id, content_type, content, status, source, title) \
         VALUES (?, ?, ?, 'draft', 'manual', ?)",
    )
    .bind(account_id)
    .bind(&original.content_type)
    .bind(&original.content)
    .bind(&new_title)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(Some(result.last_insert_rowid()))
}

/// Update a draft's title and notes metadata.
pub async fn update_draft_meta_for(
    pool: &DbPool,
    account_id: &str,
    id: i64,
    title: Option<&str>,
    notes: Option<&str>,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE scheduled_content \
         SET title = ?, notes = ?, updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(title)
    .bind(notes)
    .bind(id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected() > 0)
}

/// List archived drafts for a specific account.
pub async fn list_archived_drafts_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ScheduledContent>, StorageError> {
    sqlx::query_as::<_, ScheduledContent>(
        "SELECT * FROM scheduled_content \
         WHERE account_id = ? AND archived_at IS NOT NULL \
         ORDER BY archived_at DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}
