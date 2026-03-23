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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{init_test_db, watchtower};

    async fn setup_test_data(pool: &DbPool) -> (i64, i64) {
        let source_id = watchtower::insert_source_context(pool, "local_fs", "{}")
            .await
            .expect("insert source");
        watchtower::upsert_content_node(
            pool,
            source_id,
            "test.md",
            "hash1",
            Some("Test Note"),
            "Content here.",
            None,
            None,
        )
        .await
        .expect("upsert node");

        let chunk_id =
            watchtower::insert_chunk(pool, "default", 1, "# Test", "chunk text", "chunkhash1", 0)
                .await
                .expect("insert chunk");

        (source_id, chunk_id)
    }

    #[test]
    fn vec_to_bytes_roundtrip() {
        let original = vec![1.0_f32, 2.5, -3.0, 0.0];
        let bytes = vec_to_bytes(&original);
        assert_eq!(bytes.len(), 16);
        let restored = bytes_to_vec(&bytes);
        assert_eq!(original, restored);
    }

    #[test]
    fn empty_vec_roundtrip() {
        let empty: Vec<f32> = vec![];
        let bytes = vec_to_bytes(&empty);
        assert!(bytes.is_empty());
        let restored = bytes_to_vec(&bytes);
        assert!(restored.is_empty());
    }

    #[tokio::test]
    async fn upsert_creates_new_row() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        let embedding = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(
            &pool,
            chunk_id,
            "default",
            &embedding,
            "test-model",
            3,
            "chunkhash1",
            1,
        )
        .await
        .expect("upsert");

        let rows = get_all_embeddings_for(&pool, "default")
            .await
            .expect("get all");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chunk_id, chunk_id);
        assert_eq!(rows[0].model_id, "test-model");
        assert_eq!(rows[0].dimension, 3);
    }

    #[tokio::test]
    async fn upsert_updates_existing_row() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        let emb1 = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(&pool, chunk_id, "default", &emb1, "model-v1", 3, "hash1", 1)
            .await
            .expect("first upsert");

        let emb2 = vec_to_bytes(&vec![0.2_f32; 3]);
        upsert_chunk_embedding(&pool, chunk_id, "default", &emb2, "model-v2", 3, "hash2", 2)
            .await
            .expect("second upsert");

        let rows = get_all_embeddings_for(&pool, "default")
            .await
            .expect("get all");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].model_id, "model-v2");
        assert_eq!(rows[0].generation, 2);
    }

    #[tokio::test]
    async fn get_dirty_chunks_returns_unembedded() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, _chunk_id) = setup_test_data(&pool).await;

        let dirty = get_dirty_chunks_for(&pool, "default", 10)
            .await
            .expect("get dirty");
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].chunk_text, "chunk text");
    }

    #[tokio::test]
    async fn get_dirty_chunks_returns_hash_mismatch() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        // Embed with old hash
        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(&pool, chunk_id, "default", &emb, "model", 3, "oldhash", 1)
            .await
            .expect("upsert");

        // Chunk has hash "chunkhash1" but embedding has "oldhash" → dirty
        let dirty = get_dirty_chunks_for(&pool, "default", 10)
            .await
            .expect("get dirty");
        assert_eq!(dirty.len(), 1);
    }

    #[tokio::test]
    async fn get_dirty_chunks_empty_when_fresh() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        // Embed with matching hash
        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(
            &pool,
            chunk_id,
            "default",
            &emb,
            "model",
            3,
            "chunkhash1",
            1,
        )
        .await
        .expect("upsert");

        let dirty = get_dirty_chunks_for(&pool, "default", 10)
            .await
            .expect("get dirty");
        assert!(dirty.is_empty());
    }

    #[tokio::test]
    async fn account_scoping() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(
            &pool,
            chunk_id,
            "default",
            &emb,
            "model",
            3,
            "chunkhash1",
            1,
        )
        .await
        .expect("upsert");

        // Different account should see nothing
        let rows = get_all_embeddings_for(&pool, "other-account")
            .await
            .expect("get all");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn get_index_stats_correct_counts() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        // Before embedding: 1 total, 0 embedded, 1 dirty
        let stats = get_index_stats_for(&pool, "default").await.expect("stats");
        assert_eq!(stats.total_chunks, 1);
        assert_eq!(stats.embedded_chunks, 0);
        assert_eq!(stats.dirty_chunks, 1);
        assert!((stats.freshness_pct - 0.0).abs() < f64::EPSILON);

        // After embedding with matching hash: 1 total, 1 embedded, 0 dirty
        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(
            &pool,
            chunk_id,
            "default",
            &emb,
            "model",
            3,
            "chunkhash1",
            1,
        )
        .await
        .expect("upsert");

        let stats = get_index_stats_for(&pool, "default").await.expect("stats");
        assert_eq!(stats.total_chunks, 1);
        assert_eq!(stats.embedded_chunks, 1);
        assert_eq!(stats.dirty_chunks, 0);
        assert!((stats.freshness_pct - 100.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn delete_embeddings_by_model_removes_target() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(&pool, chunk_id, "default", &emb, "model-a", 3, "h", 1)
            .await
            .expect("upsert");

        let deleted = delete_embeddings_by_model(&pool, "default", "model-a")
            .await
            .expect("delete");
        assert_eq!(deleted, 1);

        let rows = get_all_embeddings_for(&pool, "default")
            .await
            .expect("get all");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn cascade_delete_on_chunk_removal() {
        let pool = init_test_db().await.expect("init db");
        let (_source_id, chunk_id) = setup_test_data(&pool).await;

        let emb = vec_to_bytes(&vec![0.1_f32; 3]);
        upsert_chunk_embedding(&pool, chunk_id, "default", &emb, "model", 3, "h", 1)
            .await
            .expect("upsert");

        // Delete the chunk — CASCADE should remove the embedding
        sqlx::query("DELETE FROM content_chunks WHERE id = ?")
            .bind(chunk_id)
            .execute(&pool)
            .await
            .expect("delete chunk");

        let rows = get_all_embeddings_for(&pool, "default")
            .await
            .expect("get all");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn freshness_zero_when_no_chunks() {
        let pool = init_test_db().await.expect("init db");
        let stats = get_index_stats_for(&pool, "default").await.expect("stats");
        assert_eq!(stats.total_chunks, 0);
        assert!((stats.freshness_pct - 100.0).abs() < f64::EPSILON);
    }
}
