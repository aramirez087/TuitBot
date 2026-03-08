//! CRUD operations for content_chunks.

use super::{ContentChunk, ContentChunkRow};
use crate::error::StorageError;
use crate::storage::DbPool;

/// Input for bulk chunk creation.
#[derive(Debug, Clone)]
pub struct NewChunk {
    pub heading_path: String,
    pub chunk_text: String,
    pub chunk_hash: String,
    pub chunk_index: i64,
}

// ============================================================================
// Content chunks (all functions are account-scoped)
// ============================================================================

/// Insert a single chunk and return its ID.
pub async fn insert_chunk(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    heading_path: &str,
    chunk_text: &str,
    chunk_hash: &str,
    chunk_index: i64,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO content_chunks \
         (account_id, node_id, heading_path, chunk_text, chunk_hash, chunk_index) \
         VALUES (?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(account_id)
    .bind(node_id)
    .bind(heading_path)
    .bind(chunk_text)
    .bind(chunk_hash)
    .bind(chunk_index)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Batch insert/upsert chunks for a node.
///
/// Uses `chunk_hash` for dedup: if a chunk with the same hash already exists
/// for this node (active or stale), it is reactivated with updated metadata.
/// Otherwise a new chunk is inserted. Returns the IDs of all chunks.
pub async fn upsert_chunks_for_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    chunks: &[NewChunk],
) -> Result<Vec<i64>, StorageError> {
    let mut ids = Vec::with_capacity(chunks.len());

    for chunk in chunks {
        // Check if chunk already exists by hash for this node (any status).
        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM content_chunks \
             WHERE node_id = ? AND chunk_hash = ?",
        )
        .bind(node_id)
        .bind(&chunk.chunk_hash)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        match existing {
            Some((id,)) => {
                // Reactivate and update metadata (heading_path/index may have shifted).
                sqlx::query(
                    "UPDATE content_chunks \
                     SET status = 'active', heading_path = ?, chunk_index = ?, \
                         updated_at = datetime('now') \
                     WHERE id = ?",
                )
                .bind(&chunk.heading_path)
                .bind(chunk.chunk_index)
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| StorageError::Query { source: e })?;
                ids.push(id);
            }
            None => {
                let id = insert_chunk(
                    pool,
                    account_id,
                    node_id,
                    &chunk.heading_path,
                    &chunk.chunk_text,
                    &chunk.chunk_hash,
                    chunk.chunk_index,
                )
                .await?;
                ids.push(id);
            }
        }
    }

    Ok(ids)
}

/// Get all active chunks for a node, ordered by `chunk_index`.
pub async fn get_chunks_for_node(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<Vec<ContentChunk>, StorageError> {
    let rows: Vec<ContentChunkRow> = sqlx::query_as(
        "SELECT id, account_id, node_id, heading_path, chunk_text, chunk_hash, \
                chunk_index, retrieval_boost, status, created_at, updated_at \
         FROM content_chunks \
         WHERE account_id = ? AND node_id = ? AND status = 'active' \
         ORDER BY chunk_index ASC",
    )
    .bind(account_id)
    .bind(node_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentChunk::from_row).collect())
}

/// Get a single chunk by ID for a specific account.
pub async fn get_chunk_by_id(
    pool: &DbPool,
    account_id: &str,
    chunk_id: i64,
) -> Result<Option<ContentChunk>, StorageError> {
    let row: Option<ContentChunkRow> = sqlx::query_as(
        "SELECT id, account_id, node_id, heading_path, chunk_text, chunk_hash, \
                chunk_index, retrieval_boost, status, created_at, updated_at \
         FROM content_chunks \
         WHERE id = ? AND account_id = ?",
    )
    .bind(chunk_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(ContentChunk::from_row))
}

/// Batch lookup chunks by IDs for a specific account.
///
/// Uses parameterized `WHERE IN` clause. Only returns chunks owned by the account.
pub async fn get_chunks_by_ids(
    pool: &DbPool,
    account_id: &str,
    ids: &[i64],
) -> Result<Vec<ContentChunk>, StorageError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");
    let sql = format!(
        "SELECT id, account_id, node_id, heading_path, chunk_text, chunk_hash, \
                chunk_index, retrieval_boost, status, created_at, updated_at \
         FROM content_chunks \
         WHERE account_id = ? AND id IN ({in_clause}) \
         ORDER BY chunk_index ASC"
    );

    let mut q = sqlx::query_as::<_, ContentChunkRow>(&sql);
    q = q.bind(account_id);
    for id in ids {
        q = q.bind(id);
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentChunk::from_row).collect())
}

/// Set `status = 'stale'` for all chunks of a node. Returns rows affected.
pub async fn mark_chunks_stale(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
) -> Result<u64, StorageError> {
    let result = sqlx::query(
        "UPDATE content_chunks \
         SET status = 'stale', updated_at = datetime('now') \
         WHERE account_id = ? AND node_id = ? AND status = 'active'",
    )
    .bind(account_id)
    .bind(node_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Update a single chunk's retrieval boost, clamped to [0.1, 5.0].
pub async fn update_chunk_retrieval_boost(
    pool: &DbPool,
    account_id: &str,
    chunk_id: i64,
    new_boost: f64,
) -> Result<(), StorageError> {
    let clamped = new_boost.clamp(0.1, 5.0);
    sqlx::query(
        "UPDATE content_chunks \
         SET retrieval_boost = ?, updated_at = datetime('now') \
         WHERE id = ? AND account_id = ?",
    )
    .bind(clamped)
    .bind(chunk_id)
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Search chunks matching any of the given keywords, ordered by retrieval_boost.
///
/// Returns up to `limit` active chunks for the given account. Uses LIKE-based
/// matching as the minimum viable retrieval query (no embedding search).
pub async fn search_chunks_by_keywords(
    pool: &DbPool,
    account_id: &str,
    keywords: &[&str],
    limit: u32,
) -> Result<Vec<ContentChunk>, StorageError> {
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    let like_clauses: Vec<String> = keywords
        .iter()
        .map(|_| "chunk_text LIKE '%' || ? || '%'".to_string())
        .collect();
    let where_likes = like_clauses.join(" OR ");

    let sql = format!(
        "SELECT id, account_id, node_id, heading_path, chunk_text, chunk_hash, \
                chunk_index, retrieval_boost, status, created_at, updated_at \
         FROM content_chunks \
         WHERE account_id = ? AND status = 'active' AND ({where_likes}) \
         ORDER BY retrieval_boost DESC \
         LIMIT ?"
    );

    let mut q = sqlx::query_as::<_, ContentChunkRow>(&sql);
    q = q.bind(account_id);
    for kw in keywords {
        q = q.bind(*kw);
    }
    q = q.bind(limit);

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentChunk::from_row).collect())
}
