//! CRUD operations for chunk_embeddings.
//!
//! All functions are account-scoped. Embeddings are stored as BLOBs of
//! little-endian f32 values alongside metadata for dirty-state tracking.

use crate::error::StorageError;
use crate::storage::DbPool;

// ============================================================================
// Row types
// ============================================================================

/// A stored chunk embedding row.
#[derive(Debug, Clone)]
pub struct ChunkEmbeddingRow {
    pub id: i64,
    pub chunk_id: i64,
    pub account_id: String,
    pub embedding: Vec<u8>,
    pub model_id: String,
    pub dimension: i64,
    pub embedding_hash: String,
    pub generation: i64,
    pub created_at: String,
    pub updated_at: String,
}

type EmbeddingRowTuple = (
    i64,
    i64,
    String,
    Vec<u8>,
    String,
    i64,
    String,
    i64,
    String,
    String,
);

impl ChunkEmbeddingRow {
    fn from_row(r: EmbeddingRowTuple) -> Self {
        Self {
            id: r.0,
            chunk_id: r.1,
            account_id: r.2,
            embedding: r.3,
            model_id: r.4,
            dimension: r.5,
            embedding_hash: r.6,
            generation: r.7,
            created_at: r.8,
            updated_at: r.9,
        }
    }
}

/// A chunk that needs embedding (no embedding row, or hash mismatch).
#[derive(Debug, Clone)]
pub struct DirtyChunk {
    pub chunk_id: i64,
    pub chunk_text: String,
    pub chunk_hash: String,
    pub node_id: i64,
}

/// Aggregate statistics for the semantic index.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub total_chunks: i64,
    pub embedded_chunks: i64,
    pub dirty_chunks: i64,
    pub freshness_pct: f64,
    pub last_indexed_at: Option<String>,
    pub model_id: Option<String>,
}

// ============================================================================
// CRUD operations
// ============================================================================

/// Insert or update an embedding for a chunk.
///
/// Uses the UNIQUE(chunk_id, account_id) constraint for upsert behavior.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_chunk_embedding(
    pool: &DbPool,
    chunk_id: i64,
    account_id: &str,
    embedding_bytes: &[u8],
    model_id: &str,
    dimension: i64,
    embedding_hash: &str,
    generation: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO chunk_embeddings \
         (chunk_id, account_id, embedding, model_id, dimension, embedding_hash, generation) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(chunk_id, account_id) DO UPDATE SET \
             embedding = excluded.embedding, \
             model_id = excluded.model_id, \
             dimension = excluded.dimension, \
             embedding_hash = excluded.embedding_hash, \
             generation = excluded.generation, \
             updated_at = datetime('now')",
    )
    .bind(chunk_id)
    .bind(account_id)
    .bind(embedding_bytes)
    .bind(model_id)
    .bind(dimension)
    .bind(embedding_hash)
    .bind(generation)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Get all embeddings for an account, ordered by id for deterministic rebuild.
pub async fn get_all_embeddings_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ChunkEmbeddingRow>, StorageError> {
    let rows: Vec<EmbeddingRowTuple> = sqlx::query_as(
        "SELECT id, chunk_id, account_id, embedding, model_id, dimension, \
                embedding_hash, generation, created_at, updated_at \
         FROM chunk_embeddings \
         WHERE account_id = ? \
         ORDER BY id",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ChunkEmbeddingRow::from_row).collect())
}

/// Get chunks that need embedding: no embedding row or chunk_hash != embedding_hash.
///
/// Only returns active chunks. Limited to `limit` rows for batch processing.
pub async fn get_dirty_chunks_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<DirtyChunk>, StorageError> {
    let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT c.id, c.chunk_text, c.chunk_hash, c.node_id \
         FROM content_chunks c \
         LEFT JOIN chunk_embeddings e \
             ON c.id = e.chunk_id AND e.account_id = ? \
         WHERE c.account_id = ? \
           AND c.status = 'active' \
           AND (e.id IS NULL OR c.chunk_hash != e.embedding_hash) \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(chunk_id, chunk_text, chunk_hash, node_id)| DirtyChunk {
            chunk_id,
            chunk_text,
            chunk_hash,
            node_id,
        })
        .collect())
}

/// Get index statistics for an account.
pub async fn get_index_stats_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<IndexStats, StorageError> {
    let (total_chunks,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM content_chunks \
         WHERE account_id = ? AND status = 'active'",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let (embedded_chunks,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM chunk_embeddings e \
         INNER JOIN content_chunks c ON c.id = e.chunk_id \
         WHERE e.account_id = ? AND c.status = 'active' \
           AND c.chunk_hash = e.embedding_hash",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let dirty_chunks = total_chunks - embedded_chunks;
    let freshness_pct = if total_chunks > 0 {
        (embedded_chunks as f64 / total_chunks as f64) * 100.0
    } else {
        100.0
    };

    let last_indexed_at: Option<(String,)> =
        sqlx::query_as("SELECT MAX(updated_at) FROM chunk_embeddings WHERE account_id = ?")
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    let model_id: Option<(String,)> =
        sqlx::query_as("SELECT model_id FROM chunk_embeddings WHERE account_id = ? LIMIT 1")
            .bind(account_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    Ok(IndexStats {
        total_chunks,
        embedded_chunks,
        dirty_chunks,
        freshness_pct,
        last_indexed_at: last_indexed_at.map(|r| r.0),
        model_id: model_id.map(|r| r.0),
    })
}

/// Delete all embeddings for a specific model (used during model switch).
pub async fn delete_embeddings_by_model(
    pool: &DbPool,
    account_id: &str,
    model_id: &str,
) -> Result<u64, StorageError> {
    let result = sqlx::query("DELETE FROM chunk_embeddings WHERE account_id = ? AND model_id = ?")
        .bind(account_id)
        .bind(model_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.rows_affected())
}

/// Delete the embedding for a specific chunk.
pub async fn delete_embedding_for_chunk(
    pool: &DbPool,
    chunk_id: i64,
    account_id: &str,
) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM chunk_embeddings WHERE chunk_id = ? AND account_id = ?")
        .bind(chunk_id)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Serialize an f32 vector to little-endian bytes for BLOB storage.
pub fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for &val in v {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// Deserialize little-endian bytes back to an f32 vector.
pub fn bytes_to_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

// Tests moved to tests_embeddings.rs to stay under 500-line file limit.
