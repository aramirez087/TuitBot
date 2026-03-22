//! CRUD operations for content_nodes.

use super::{ContentNode, ContentNodeRow, UpsertResult};
use crate::error::StorageError;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::DbPool;

// ============================================================================
// Account-scoped content node functions
// ============================================================================

/// Upsert a content node for a specific account by (source_id, relative_path).
#[allow(clippy::too_many_arguments)]
pub async fn upsert_content_node_for(
    pool: &DbPool,
    account_id: &str,
    source_id: i64,
    relative_path: &str,
    content_hash: &str,
    title: Option<&str>,
    body_text: &str,
    front_matter_json: Option<&str>,
    tags: Option<&str>,
) -> Result<UpsertResult, StorageError> {
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
            .bind(account_id)
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

/// Upsert a content node by (source_id, relative_path).
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
    upsert_content_node_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        source_id,
        relative_path,
        content_hash,
        title,
        body_text,
        front_matter_json,
        tags,
    )
    .await
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

    Ok(row.map(ContentNode::from_row))
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

    Ok(rows.into_iter().map(ContentNode::from_row).collect())
}

/// Get content nodes with status='pending' for a specific account.
pub async fn get_pending_content_nodes_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<ContentNode>, StorageError> {
    let rows: Vec<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes \
             WHERE account_id = ? AND status = 'pending' \
             ORDER BY ingested_at ASC \
             LIMIT ?",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentNode::from_row).collect())
}

/// Get content nodes with status='pending' that need seed generation.
pub async fn get_pending_content_nodes(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<ContentNode>, StorageError> {
    get_pending_content_nodes_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}

/// Mark a content node as 'processed' for a specific account.
pub async fn mark_node_processed_for(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE content_nodes SET status = 'processed', updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(node_id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Mark a content node as 'processed' after seed generation.
pub async fn mark_node_processed(pool: &DbPool, node_id: i64) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE content_nodes SET status = 'processed', updated_at = datetime('now') WHERE id = ?",
    )
    .bind(node_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Search content nodes by title or path for a specific account.
///
/// Returns nodes matching the query (LIKE-based), without loading body_text
/// into the API response layer. The caller should omit body_text when serializing.
pub async fn search_nodes_for(
    pool: &DbPool,
    account_id: &str,
    query: &str,
    limit: u32,
) -> Result<Vec<ContentNode>, StorageError> {
    let rows: Vec<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes \
             WHERE account_id = ? AND (title LIKE '%' || ? || '%' OR relative_path LIKE '%' || ? || '%') \
             ORDER BY updated_at DESC \
             LIMIT ?",
    )
    .bind(account_id)
    .bind(query)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentNode::from_row).collect())
}

/// Get all content nodes for a specific account and source.
pub async fn get_nodes_for_source_for(
    pool: &DbPool,
    account_id: &str,
    source_id: i64,
    limit: u32,
) -> Result<Vec<ContentNode>, StorageError> {
    let rows: Vec<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes \
             WHERE account_id = ? AND source_id = ? \
             ORDER BY updated_at DESC \
             LIMIT ?",
    )
    .bind(account_id)
    .bind(source_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentNode::from_row).collect())
}

/// Get a content node by ID, scoped to account.
pub async fn get_content_node_for(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<Option<ContentNode>, StorageError> {
    let row: Option<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes WHERE id = ? AND account_id = ?",
    )
    .bind(node_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ContentNode::from_row))
}

/// Count active chunks for a node, scoped to account.
pub async fn count_chunks_for_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM content_chunks \
         WHERE account_id = ? AND node_id = ? AND status = 'active'",
    )
    .bind(account_id)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Count content nodes for a source, scoped to account.
pub async fn count_nodes_for_source(
    pool: &DbPool,
    account_id: &str,
    source_id: i64,
) -> Result<i64, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM content_nodes WHERE account_id = ? AND source_id = ?")
            .bind(account_id)
            .bind(source_id)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Find a content node by `relative_path` for a given account (across all sources).
///
/// Returns the most recently updated match if multiple sources contain the same path.
pub async fn find_node_by_path_for(
    pool: &DbPool,
    account_id: &str,
    relative_path: &str,
) -> Result<Option<ContentNode>, StorageError> {
    let row: Option<ContentNodeRow> = sqlx::query_as(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                    title, body_text, front_matter_json, tags, status, \
                    ingested_at, updated_at \
             FROM content_nodes \
             WHERE account_id = ? AND relative_path = ? \
             ORDER BY updated_at DESC \
             LIMIT 1",
    )
    .bind(account_id)
    .bind(relative_path)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ContentNode::from_row))
}

/// Batch lookup content nodes by IDs, scoped to account.
///
/// Uses parameterized `WHERE IN` clause. Only returns nodes owned by the account.
pub async fn get_nodes_by_ids(
    pool: &DbPool,
    account_id: &str,
    node_ids: &[i64],
) -> Result<Vec<ContentNode>, StorageError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = node_ids.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");
    let sql = format!(
        "SELECT id, account_id, source_id, relative_path, content_hash, \
                title, body_text, front_matter_json, tags, status, \
                ingested_at, updated_at \
         FROM content_nodes \
         WHERE account_id = ? AND id IN ({in_clause}) \
         ORDER BY id"
    );

    let mut q = sqlx::query_as::<_, ContentNodeRow>(&sql);
    q = q.bind(account_id);
    for id in node_ids {
        q = q.bind(id);
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentNode::from_row).collect())
}

/// Mark a content node as 'chunked' for a specific account.
pub async fn mark_node_chunked(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE content_nodes SET status = 'chunked', updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(node_id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}
