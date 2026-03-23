//! CRUD operations for content_chunks.

use super::{ChunkWithNodeContext, ContentChunk, ContentChunkRow};
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

/// Find the best-matching chunk for a heading context within a node.
///
/// Uses longest prefix match on `heading_path`. If `heading_context` is `None`,
/// returns the first chunk (document root). Returns `None` if no chunks exist.
pub async fn find_best_chunk_by_heading_for(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    heading_context: Option<&str>,
) -> Result<Option<ContentChunk>, StorageError> {
    let chunks = get_chunks_for_node(pool, account_id, node_id).await?;

    if chunks.is_empty() {
        return Ok(None);
    }

    let heading = match heading_context {
        Some(h) if !h.is_empty() => h,
        _ => return Ok(Some(chunks.into_iter().next().unwrap())),
    };

    // Find the chunk whose heading_path is the longest prefix of the provided heading_context.
    let best = chunks
        .into_iter()
        .filter(|c| heading.starts_with(&c.heading_path) || c.heading_path.starts_with(heading))
        .max_by_key(|c| {
            // Score: length of the common prefix between chunk heading and query heading.
            let min_len = c.heading_path.len().min(heading.len());
            c.heading_path[..min_len]
                .chars()
                .zip(heading[..min_len].chars())
                .take_while(|(a, b)| a == b)
                .count()
        });

    Ok(best)
}

/// Get the highest-boost active chunk per node for a batch of node IDs.
///
/// Returns at most one chunk per node_id, selected by highest `retrieval_boost`
/// then lowest `chunk_index`. Only returns chunks owned by the account.
pub async fn get_best_chunks_for_nodes(
    pool: &DbPool,
    account_id: &str,
    node_ids: &[i64],
) -> Result<Vec<ContentChunk>, StorageError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = node_ids.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");

    // Use a correlated subquery to pick the best chunk per node.
    let sql = format!(
        "SELECT c.id, c.account_id, c.node_id, c.heading_path, c.chunk_text, c.chunk_hash, \
                c.chunk_index, c.retrieval_boost, c.status, c.created_at, c.updated_at \
         FROM content_chunks c \
         WHERE c.account_id = ? AND c.status = 'active' AND c.node_id IN ({in_clause}) \
           AND c.id = ( \
               SELECT c2.id FROM content_chunks c2 \
               WHERE c2.node_id = c.node_id AND c2.account_id = c.account_id \
                 AND c2.status = 'active' \
               ORDER BY c2.retrieval_boost DESC, c2.chunk_index ASC \
               LIMIT 1 \
           ) \
         ORDER BY c.node_id"
    );

    let mut q = sqlx::query_as::<_, ContentChunkRow>(&sql);
    q = q.bind(account_id);
    for id in node_ids {
        q = q.bind(id);
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ContentChunk::from_row).collect())
}

/// Row type for chunk + node context JOIN query.
type ChunkWithContextRow = (
    i64,            // cc.id
    String,         // cc.account_id
    i64,            // cc.node_id
    String,         // cc.heading_path
    String,         // cc.chunk_text
    String,         // cc.chunk_hash
    i64,            // cc.chunk_index
    f64,            // cc.retrieval_boost
    String,         // cc.status
    String,         // cc.created_at
    String,         // cc.updated_at
    String,         // cn.relative_path
    Option<String>, // cn.title
);

fn chunk_with_context_from_row(r: ChunkWithContextRow) -> ChunkWithNodeContext {
    let chunk = ContentChunk::from_row((r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8, r.9, r.10));
    ChunkWithNodeContext {
        chunk,
        relative_path: r.11,
        source_title: r.12,
    }
}

/// Search chunks matching keywords, joined with parent node metadata.
///
/// Returns up to `limit` active chunks for the given account with their
/// parent node's `relative_path` and `title` for citation display.
pub async fn search_chunks_with_context(
    pool: &DbPool,
    account_id: &str,
    keywords: &[&str],
    limit: u32,
) -> Result<Vec<ChunkWithNodeContext>, StorageError> {
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    let like_clauses: Vec<String> = keywords
        .iter()
        .map(|_| "cc.chunk_text LIKE '%' || ? || '%'".to_string())
        .collect();
    let where_likes = like_clauses.join(" OR ");

    let sql = format!(
        "SELECT cc.id, cc.account_id, cc.node_id, cc.heading_path, cc.chunk_text, \
                cc.chunk_hash, cc.chunk_index, cc.retrieval_boost, cc.status, \
                cc.created_at, cc.updated_at, cn.relative_path, cn.title \
         FROM content_chunks cc \
         JOIN content_nodes cn ON cn.id = cc.node_id AND cn.account_id = cc.account_id \
         WHERE cc.account_id = ? AND cc.status = 'active' AND ({where_likes}) \
         ORDER BY cc.retrieval_boost DESC \
         LIMIT ?"
    );

    let mut q = sqlx::query_as::<_, ChunkWithContextRow>(&sql);
    q = q.bind(account_id);
    for kw in keywords {
        q = q.bind(*kw);
    }
    q = q.bind(limit);

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(chunk_with_context_from_row).collect())
}

/// Get chunks by chunk IDs, joined with parent node metadata.
///
/// Returns active chunks for the given chunk IDs with their parent node's
/// `relative_path` and `title`. Used to enrich semantic search hits with
/// display metadata.
pub async fn get_chunks_with_context_by_ids(
    pool: &DbPool,
    account_id: &str,
    chunk_ids: &[i64],
) -> Result<Vec<ChunkWithNodeContext>, StorageError> {
    if chunk_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = chunk_ids.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");

    let sql = format!(
        "SELECT cc.id, cc.account_id, cc.node_id, cc.heading_path, cc.chunk_text, \
                cc.chunk_hash, cc.chunk_index, cc.retrieval_boost, cc.status, \
                cc.created_at, cc.updated_at, cn.relative_path, cn.title \
         FROM content_chunks cc \
         JOIN content_nodes cn ON cn.id = cc.node_id AND cn.account_id = cc.account_id \
         WHERE cc.account_id = ? AND cc.status = 'active' AND cc.id IN ({in_clause})"
    );

    let mut q = sqlx::query_as::<_, ChunkWithContextRow>(&sql);
    q = q.bind(account_id);
    for cid in chunk_ids {
        q = q.bind(cid);
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(chunk_with_context_from_row).collect())
}

/// Get chunks for specific nodes, joined with parent node metadata.
///
/// Returns active chunks for the given node IDs, ordered by retrieval_boost DESC.
pub async fn get_chunks_for_nodes_with_context(
    pool: &DbPool,
    account_id: &str,
    node_ids: &[i64],
    limit: u32,
) -> Result<Vec<ChunkWithNodeContext>, StorageError> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<&str> = node_ids.iter().map(|_| "?").collect();
    let in_clause = placeholders.join(", ");

    let sql = format!(
        "SELECT cc.id, cc.account_id, cc.node_id, cc.heading_path, cc.chunk_text, \
                cc.chunk_hash, cc.chunk_index, cc.retrieval_boost, cc.status, \
                cc.created_at, cc.updated_at, cn.relative_path, cn.title \
         FROM content_chunks cc \
         JOIN content_nodes cn ON cn.id = cc.node_id AND cn.account_id = cc.account_id \
         WHERE cc.account_id = ? AND cc.status = 'active' AND cc.node_id IN ({in_clause}) \
         ORDER BY cc.retrieval_boost DESC \
         LIMIT ?"
    );

    let mut q = sqlx::query_as::<_, ChunkWithContextRow>(&sql);
    q = q.bind(account_id);
    for nid in node_ids {
        q = q.bind(nid);
    }
    q = q.bind(limit);

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(chunk_with_context_from_row).collect())
}
