//! CRUD operations for source_contexts.

use super::{SourceContext, SourceContextRow};
use crate::error::StorageError;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::DbPool;

// ============================================================================
// Account-scoped source context functions
// ============================================================================

/// Insert a new source context for a specific account and return its ID.
pub async fn insert_source_context_for(
    pool: &DbPool,
    account_id: &str,
    source_type: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO source_contexts (account_id, source_type, config_json) \
         VALUES (?, ?, ?) \
         RETURNING id",
    )
    .bind(account_id)
    .bind(source_type)
    .bind(config_json)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Insert a new source context and return its ID.
pub async fn insert_source_context(
    pool: &DbPool,
    source_type: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    insert_source_context_for(pool, DEFAULT_ACCOUNT_ID, source_type, config_json).await
}

/// Get a source context by ID.
pub async fn get_source_context(
    pool: &DbPool,
    id: i64,
) -> Result<Option<SourceContext>, StorageError> {
    let row: Option<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(SourceContext::from_row))
}

/// Get all active source contexts for a specific account.
pub async fn get_source_contexts_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<SourceContext>, StorageError> {
    let rows: Vec<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts \
             WHERE account_id = ? AND status = 'active' ORDER BY id",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(SourceContext::from_row).collect())
}

/// Get all active source contexts.
pub async fn get_source_contexts(pool: &DbPool) -> Result<Vec<SourceContext>, StorageError> {
    let rows: Vec<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts WHERE status = 'active' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(SourceContext::from_row).collect())
}

/// Get all source contexts regardless of status (for status APIs).
pub async fn get_all_source_contexts(pool: &DbPool) -> Result<Vec<SourceContext>, StorageError> {
    let rows: Vec<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(SourceContext::from_row).collect())
}

/// Update the sync cursor for a source context.
pub async fn update_sync_cursor(pool: &DbPool, id: i64, cursor: &str) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE source_contexts SET sync_cursor = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(cursor)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Update the status (and optional error message) of a source context.
pub async fn update_source_status(
    pool: &DbPool,
    id: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE source_contexts \
         SET status = ?, error_message = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(status)
    .bind(error_message)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Find a source context by path substring for a specific account.
pub async fn find_source_by_path_for(
    pool: &DbPool,
    account_id: &str,
    path: &str,
) -> Result<Option<SourceContext>, StorageError> {
    let row: Option<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts \
             WHERE account_id = ? AND source_type = 'local_fs' AND status = 'active' \
               AND config_json LIKE '%' || ? || '%' \
             LIMIT 1",
    )
    .bind(account_id)
    .bind(path)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(SourceContext::from_row))
}

/// Find a source context by source type and path substring in config_json.
pub async fn find_source_by_path(
    pool: &DbPool,
    path: &str,
) -> Result<Option<SourceContext>, StorageError> {
    find_source_by_path_for(pool, DEFAULT_ACCOUNT_ID, path).await
}

/// Ensure a "local_fs" source context exists for a specific account, returning its ID.
pub async fn ensure_local_fs_source_for(
    pool: &DbPool,
    account_id: &str,
    path: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    if let Some(ctx) = find_source_by_path_for(pool, account_id, path).await? {
        return Ok(ctx.id);
    }
    insert_source_context_for(pool, account_id, "local_fs", config_json).await
}

/// Ensure a "local_fs" source context exists for the given path, returning its ID.
pub async fn ensure_local_fs_source(
    pool: &DbPool,
    path: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    ensure_local_fs_source_for(pool, DEFAULT_ACCOUNT_ID, path, config_json).await
}

/// Find a source context by Google Drive folder ID for a specific account.
pub async fn find_source_by_folder_id_for(
    pool: &DbPool,
    account_id: &str,
    folder_id: &str,
) -> Result<Option<SourceContext>, StorageError> {
    let row: Option<SourceContextRow> = sqlx::query_as(
        "SELECT id, account_id, source_type, config_json, sync_cursor, \
                    status, error_message, created_at, updated_at \
             FROM source_contexts \
             WHERE account_id = ? AND source_type = 'google_drive' AND status = 'active' \
               AND config_json LIKE '%' || ? || '%' \
             LIMIT 1",
    )
    .bind(account_id)
    .bind(folder_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(SourceContext::from_row))
}

/// Find a source context by Google Drive folder ID in config_json.
pub async fn find_source_by_folder_id(
    pool: &DbPool,
    folder_id: &str,
) -> Result<Option<SourceContext>, StorageError> {
    find_source_by_folder_id_for(pool, DEFAULT_ACCOUNT_ID, folder_id).await
}

/// Ensure a "google_drive" source context exists for a specific account, returning its ID.
pub async fn ensure_google_drive_source_for(
    pool: &DbPool,
    account_id: &str,
    folder_id: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    if let Some(ctx) = find_source_by_folder_id_for(pool, account_id, folder_id).await? {
        return Ok(ctx.id);
    }
    insert_source_context_for(pool, account_id, "google_drive", config_json).await
}

/// Ensure a "google_drive" source context exists for the given folder ID, returning its ID.
pub async fn ensure_google_drive_source(
    pool: &DbPool,
    folder_id: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    ensure_google_drive_source_for(pool, DEFAULT_ACCOUNT_ID, folder_id, config_json).await
}

/// Ensure a "manual" source context exists for a specific account, returning its ID.
pub async fn ensure_manual_source_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<i64, StorageError> {
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM source_contexts \
         WHERE account_id = ? AND source_type = 'manual' AND status = 'active' \
         LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    match existing {
        Some((id,)) => Ok(id),
        None => insert_source_context_for(pool, account_id, "manual", "{}").await,
    }
}

/// Ensure a "manual" source context exists for inline ingestion, returning its ID.
pub async fn ensure_manual_source(pool: &DbPool) -> Result<i64, StorageError> {
    ensure_manual_source_for(pool, DEFAULT_ACCOUNT_ID).await
}
