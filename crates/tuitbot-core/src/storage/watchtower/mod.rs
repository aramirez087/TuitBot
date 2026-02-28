//! CRUD operations for Watchtower ingestion tables.
//!
//! Manages source contexts, content nodes, and draft seeds for the
//! Cold-Start Watchtower RAG pipeline.

#[cfg(test)]
mod tests;

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;
use crate::error::StorageError;

/// Row type for source_contexts queries.
type SourceContextRow = (
    i64,
    String,
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    String,
    String,
);

/// Row type for content_nodes queries.
type ContentNodeRow = (
    i64,
    String,
    i64,
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
);

/// Row type for draft_seeds queries.
type DraftSeedRow = (
    i64,
    String,
    i64,
    String,
    Option<String>,
    f64,
    String,
    String,
    Option<String>,
);

// ============================================================================
// Row structs
// ============================================================================

/// A registered content source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceContext {
    pub id: i64,
    pub account_id: String,
    pub source_type: String,
    pub config_json: String,
    pub sync_cursor: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// An ingested content node from a source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentNode {
    pub id: i64,
    pub account_id: String,
    pub source_id: i64,
    pub relative_path: String,
    pub content_hash: String,
    pub title: Option<String>,
    pub body_text: String,
    pub front_matter_json: Option<String>,
    pub tags: Option<String>,
    pub status: String,
    pub ingested_at: String,
    pub updated_at: String,
}

/// A pre-computed draft seed derived from a content node.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DraftSeed {
    pub id: i64,
    pub account_id: String,
    pub node_id: i64,
    pub seed_text: String,
    pub archetype_suggestion: Option<String>,
    pub engagement_weight: f64,
    pub status: String,
    pub created_at: String,
    pub used_at: Option<String>,
}

/// Result of an upsert operation on a content node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertResult {
    /// A new node was inserted.
    Inserted,
    /// An existing node was updated (content hash changed).
    Updated,
    /// The node was skipped (content hash unchanged).
    Skipped,
}

// ============================================================================
// Source contexts
// ============================================================================

/// Insert a new source context and return its ID.
pub async fn insert_source_context(
    pool: &DbPool,
    source_type: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO source_contexts (account_id, source_type, config_json) \
         VALUES (?, ?, ?) \
         RETURNING id",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .bind(source_type)
    .bind(config_json)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
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

    Ok(row.map(|r| SourceContext {
        id: r.0,
        account_id: r.1,
        source_type: r.2,
        config_json: r.3,
        sync_cursor: r.4,
        status: r.5,
        error_message: r.6,
        created_at: r.7,
        updated_at: r.8,
    }))
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

    Ok(rows
        .into_iter()
        .map(|r| SourceContext {
            id: r.0,
            account_id: r.1,
            source_type: r.2,
            config_json: r.3,
            sync_cursor: r.4,
            status: r.5,
            error_message: r.6,
            created_at: r.7,
            updated_at: r.8,
        })
        .collect())
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

// ============================================================================
// Content nodes
// ============================================================================

/// Upsert a content node by (source_id, relative_path).
///
/// If the node does not exist, it is inserted. If it exists and the content
/// hash has changed, it is updated. If the hash is unchanged, the operation
/// is skipped.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_content_node(
    pool: &DbPool,
    source_id: i64,
    relative_path: &str,
    content_hash: &str,
    title: Option<&str>,
    body_text: &str,
    front_matter_json: Option<&str>,
    tags: Option<&str>,
) -> Result<UpsertResult, StorageError> {
    // Check if node exists and get its current hash.
    let existing: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, content_hash FROM content_nodes \
         WHERE source_id = ? AND relative_path = ?",
    )
    .bind(source_id)
    .bind(relative_path)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    match existing {
        Some((_id, ref existing_hash)) if existing_hash == content_hash => {
            Ok(UpsertResult::Skipped)
        }
        Some((id, _)) => {
            sqlx::query(
                "UPDATE content_nodes \
                 SET content_hash = ?, title = ?, body_text = ?, \
                     front_matter_json = ?, tags = ?, \
                     status = 'pending', updated_at = datetime('now') \
                 WHERE id = ?",
            )
            .bind(content_hash)
            .bind(title)
            .bind(body_text)
            .bind(front_matter_json)
            .bind(tags)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

            Ok(UpsertResult::Updated)
        }
        None => {
            sqlx::query(
                "INSERT INTO content_nodes \
                 (account_id, source_id, relative_path, content_hash, \
                  title, body_text, front_matter_json, tags) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(DEFAULT_ACCOUNT_ID)
            .bind(source_id)
            .bind(relative_path)
            .bind(content_hash)
            .bind(title)
            .bind(body_text)
            .bind(front_matter_json)
            .bind(tags)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

            Ok(UpsertResult::Inserted)
        }
    }
}

/// Get a content node by ID.
pub async fn get_content_node(pool: &DbPool, id: i64) -> Result<Option<ContentNode>, StorageError> {
    let row: Option<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| ContentNode {
        id: r.0,
        account_id: r.1,
        source_id: r.2,
        relative_path: r.3,
        content_hash: r.4,
        title: r.5,
        body_text: r.6,
        front_matter_json: r.7,
        tags: r.8,
        status: r.9,
        ingested_at: r.10,
        updated_at: r.11,
    }))
}

/// Get all content nodes for a source, optionally filtered by status.
pub async fn get_nodes_for_source(
    pool: &DbPool,
    source_id: i64,
    status_filter: Option<&str>,
) -> Result<Vec<ContentNode>, StorageError> {
    let rows: Vec<ContentNodeRow> = match status_filter {
        Some(status) => {
            sqlx::query_as(
                "SELECT id, account_id, source_id, relative_path, content_hash, \
                            title, body_text, front_matter_json, tags, status, \
                            ingested_at, updated_at \
                     FROM content_nodes WHERE source_id = ? AND status = ? ORDER BY id",
            )
            .bind(source_id)
            .bind(status)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as(
                "SELECT id, account_id, source_id, relative_path, content_hash, \
                            title, body_text, front_matter_json, tags, status, \
                            ingested_at, updated_at \
                     FROM content_nodes WHERE source_id = ? ORDER BY id",
            )
            .bind(source_id)
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| ContentNode {
            id: r.0,
            account_id: r.1,
            source_id: r.2,
            relative_path: r.3,
            content_hash: r.4,
            title: r.5,
            body_text: r.6,
            front_matter_json: r.7,
            tags: r.8,
            status: r.9,
            ingested_at: r.10,
            updated_at: r.11,
        })
        .collect())
}

// ============================================================================
// Draft seeds
// ============================================================================

/// Insert a new draft seed and return its ID.
pub async fn insert_draft_seed(
    pool: &DbPool,
    node_id: i64,
    seed_text: &str,
    archetype_suggestion: Option<&str>,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO draft_seeds (account_id, node_id, seed_text, archetype_suggestion) \
         VALUES (?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .bind(node_id)
    .bind(seed_text)
    .bind(archetype_suggestion)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Get pending draft seeds ordered by engagement weight descending.
pub async fn get_pending_seeds(pool: &DbPool, limit: u32) -> Result<Vec<DraftSeed>, StorageError> {
    let rows: Vec<DraftSeedRow> = sqlx::query_as(
        "SELECT id, account_id, node_id, seed_text, archetype_suggestion, \
                    engagement_weight, status, created_at, used_at \
             FROM draft_seeds \
             WHERE status = 'pending' \
             ORDER BY engagement_weight DESC \
             LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| DraftSeed {
            id: r.0,
            account_id: r.1,
            node_id: r.2,
            seed_text: r.3,
            archetype_suggestion: r.4,
            engagement_weight: r.5,
            status: r.6,
            created_at: r.7,
            used_at: r.8,
        })
        .collect())
}

/// Mark a draft seed as used.
pub async fn mark_seed_used(pool: &DbPool, id: i64) -> Result<(), StorageError> {
    sqlx::query("UPDATE draft_seeds SET status = 'used', used_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Find a source context by source type and path substring in config_json.
pub async fn find_source_by_path(
    pool: &DbPool,
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
    .bind(DEFAULT_ACCOUNT_ID)
    .bind(path)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| SourceContext {
        id: r.0,
        account_id: r.1,
        source_type: r.2,
        config_json: r.3,
        sync_cursor: r.4,
        status: r.5,
        error_message: r.6,
        created_at: r.7,
        updated_at: r.8,
    }))
}

/// Ensure a "local_fs" source context exists for the given path, returning its ID.
///
/// Creates the source if it does not exist. Used by the Watchtower to register
/// configured filesystem sources.
pub async fn ensure_local_fs_source(
    pool: &DbPool,
    path: &str,
    config_json: &str,
) -> Result<i64, StorageError> {
    if let Some(ctx) = find_source_by_path(pool, path).await? {
        return Ok(ctx.id);
    }
    insert_source_context(pool, "local_fs", config_json).await
}

/// Ensure a "manual" source context exists for inline ingestion, returning its ID.
///
/// Creates the source if it does not exist. This is used by the ingest API
/// when content is submitted directly (e.g. from Shortcuts or Telegram).
pub async fn ensure_manual_source(pool: &DbPool) -> Result<i64, StorageError> {
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM source_contexts \
         WHERE account_id = ? AND source_type = 'manual' AND status = 'active' \
         LIMIT 1",
    )
    .bind(DEFAULT_ACCOUNT_ID)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    match existing {
        Some((id,)) => Ok(id),
        None => insert_source_context(pool, "manual", "{}").await,
    }
}
