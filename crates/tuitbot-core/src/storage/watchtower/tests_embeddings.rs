//! Tests for `storage::watchtower::embeddings` module.

use crate::storage::watchtower::embeddings::*;
use crate::storage::{init_test_db, watchtower, DbPool};

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

    // Chunk has hash "chunkhash1" but embedding has "oldhash" -> dirty
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

    // Delete the chunk -- CASCADE should remove the embedding
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

#[tokio::test]
async fn delete_embedding_for_specific_chunk() {
    let pool = init_test_db().await.expect("init db");
    let (_source_id, chunk_id) = setup_test_data(&pool).await;

    let emb = vec_to_bytes(&vec![0.1_f32; 3]);
    upsert_chunk_embedding(&pool, chunk_id, "default", &emb, "model", 3, "h", 1)
        .await
        .expect("upsert");

    // Verify it exists
    let rows = get_all_embeddings_for(&pool, "default")
        .await
        .expect("get all");
    assert_eq!(rows.len(), 1);

    // Delete and verify it is gone
    delete_embedding_for_chunk(&pool, chunk_id, "default")
        .await
        .expect("delete");
    let rows = get_all_embeddings_for(&pool, "default")
        .await
        .expect("get all");
    assert!(rows.is_empty());
}

#[tokio::test]
async fn delete_embedding_for_chunk_nonexistent_is_ok() {
    let pool = init_test_db().await.expect("init db");
    // Deleting a nonexistent chunk embedding should not error
    delete_embedding_for_chunk(&pool, 99999, "default")
        .await
        .expect("delete nonexistent should be ok");
}

#[tokio::test]
async fn index_stats_includes_model_and_last_indexed() {
    let pool = init_test_db().await.expect("init db");
    let (_source_id, chunk_id) = setup_test_data(&pool).await;

    let emb = vec_to_bytes(&vec![0.1_f32; 3]);
    upsert_chunk_embedding(
        &pool,
        chunk_id,
        "default",
        &emb,
        "nomic-embed-text",
        3,
        "chunkhash1",
        1,
    )
    .await
    .expect("upsert");

    let stats = get_index_stats_for(&pool, "default").await.expect("stats");
    assert_eq!(stats.model_id.as_deref(), Some("nomic-embed-text"));
    assert!(stats.last_indexed_at.is_some());
}

#[tokio::test]
async fn index_stats_empty_account_has_no_model() {
    let pool = init_test_db().await.expect("init db");
    let stats = get_index_stats_for(&pool, "nonexistent")
        .await
        .expect("stats");
    assert!(stats.model_id.is_none());
    // No embeddings for this account, so total and embedded are both 0
    assert_eq!(stats.total_chunks, 0);
    assert_eq!(stats.embedded_chunks, 0);
}

#[tokio::test]
async fn delete_embeddings_by_model_wrong_model_deletes_nothing() {
    let pool = init_test_db().await.expect("init db");
    let (_source_id, chunk_id) = setup_test_data(&pool).await;

    let emb = vec_to_bytes(&vec![0.1_f32; 3]);
    upsert_chunk_embedding(&pool, chunk_id, "default", &emb, "model-a", 3, "h", 1)
        .await
        .expect("upsert");

    let deleted = delete_embeddings_by_model(&pool, "default", "model-b")
        .await
        .expect("delete");
    assert_eq!(deleted, 0);

    // Original still exists
    let rows = get_all_embeddings_for(&pool, "default")
        .await
        .expect("get all");
    assert_eq!(rows.len(), 1);
}

#[test]
fn bytes_to_vec_known_values() {
    // Manually encode 1.0f32 in little endian: 0x3F800000
    let bytes: Vec<u8> = vec![0x00, 0x00, 0x80, 0x3F];
    let result = bytes_to_vec(&bytes);
    assert_eq!(result.len(), 1);
    assert!((result[0] - 1.0).abs() < f32::EPSILON);
}

#[test]
fn bytes_to_vec_ignores_trailing_partial_chunk() {
    // 5 bytes: one full f32 + 1 trailing byte (ignored by chunks_exact)
    let mut bytes = vec![0x00, 0x00, 0x80, 0x3F, 0xFF];
    let result = bytes_to_vec(&bytes);
    assert_eq!(result.len(), 1);
    // The trailing byte is ignored
    bytes.truncate(4);
    let result2 = bytes_to_vec(&bytes);
    assert_eq!(result, result2);
}
