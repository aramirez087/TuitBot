//! Tests for content_chunks CRUD operations (chunks.rs).
use super::chunks::*;
use super::*;
use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::init_test_db;

/// Helper: create a source + node, return (source_id, node_id).
async fn setup_node(pool: &crate::storage::DbPool) -> (i64, i64) {
    let source_id = insert_source_context(pool, "local_fs", "{}").await.unwrap();
    let result = upsert_content_node(
        pool,
        source_id,
        "notes/test.md",
        "hash-abc",
        Some("Test Note"),
        "body text for testing",
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(result, UpsertResult::Inserted);

    // Fetch the node_id via a query since upsert returns enum not id.
    let row: (i64,) =
        sqlx::query_as("SELECT id FROM content_nodes WHERE source_id = ? AND relative_path = ?")
            .bind(source_id)
            .bind("notes/test.md")
            .fetch_one(pool)
            .await
            .unwrap();

    (source_id, row.0)
}

#[tokio::test]
async fn insert_chunk_returns_id() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id = insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# Heading",
        "Some chunk text",
        "chunk-hash-1",
        0,
    )
    .await
    .unwrap();
    assert!(id > 0);
}

#[tokio::test]
async fn get_chunks_for_node_returns_ordered() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    // Insert chunks out of order
    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# B", "second", "h2", 1)
        .await
        .unwrap();
    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "first", "h1", 0)
        .await
        .unwrap();

    let chunks = get_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_index, 0);
    assert_eq!(chunks[1].chunk_index, 1);
}

#[tokio::test]
async fn get_chunk_by_id_found() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id = insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# H",
        "chunk text",
        "hash-x",
        0,
    )
    .await
    .unwrap();

    let chunk = get_chunk_by_id(&pool, DEFAULT_ACCOUNT_ID, id)
        .await
        .unwrap();
    assert!(chunk.is_some());
    let chunk = chunk.unwrap();
    assert_eq!(chunk.id, id);
    assert_eq!(chunk.chunk_text, "chunk text");
}

#[tokio::test]
async fn get_chunk_by_id_wrong_account_returns_none() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id = insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# H",
        "text",
        "hash-y",
        0,
    )
    .await
    .unwrap();

    let chunk = get_chunk_by_id(&pool, "wrong-account", id).await.unwrap();
    assert!(chunk.is_none());
}

#[tokio::test]
async fn get_chunks_by_ids_empty_returns_empty() {
    let pool = init_test_db().await.unwrap();
    let result = get_chunks_by_ids(&pool, DEFAULT_ACCOUNT_ID, &[])
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn get_chunks_by_ids_returns_matching() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id1 = insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "a", "ha", 0)
        .await
        .unwrap();
    let id2 = insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# B", "b", "hb", 1)
        .await
        .unwrap();
    let _id3 = insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# C", "c", "hc", 2)
        .await
        .unwrap();

    let chunks = get_chunks_by_ids(&pool, DEFAULT_ACCOUNT_ID, &[id1, id2])
        .await
        .unwrap();
    assert_eq!(chunks.len(), 2);
}

#[tokio::test]
async fn mark_chunks_stale_updates_status() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "a", "ha", 0)
        .await
        .unwrap();
    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# B", "b", "hb", 1)
        .await
        .unwrap();

    let affected = mark_chunks_stale(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    assert_eq!(affected, 2);

    // get_chunks_for_node only returns active chunks
    let active = get_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    assert!(active.is_empty());
}

#[tokio::test]
async fn mark_chunks_stale_idempotent() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "a", "ha", 0)
        .await
        .unwrap();

    mark_chunks_stale(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    let affected = mark_chunks_stale(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    assert_eq!(affected, 0); // already stale, no rows affected
}

#[tokio::test]
async fn upsert_chunks_for_node_inserts_new() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let chunks = vec![
        NewChunk {
            heading_path: "# A".to_string(),
            chunk_text: "aaa".to_string(),
            chunk_hash: "hash-aaa".to_string(),
            chunk_index: 0,
        },
        NewChunk {
            heading_path: "# B".to_string(),
            chunk_text: "bbb".to_string(),
            chunk_hash: "hash-bbb".to_string(),
            chunk_index: 1,
        },
    ];

    let ids = upsert_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id, &chunks)
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
}

#[tokio::test]
async fn upsert_chunks_for_node_reactivates_stale() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let chunks = vec![NewChunk {
        heading_path: "# A".to_string(),
        chunk_text: "aaa".to_string(),
        chunk_hash: "hash-aaa".to_string(),
        chunk_index: 0,
    }];

    let ids1 = upsert_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id, &chunks)
        .await
        .unwrap();

    // Mark stale
    mark_chunks_stale(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();

    // Upsert again with same hash — should reactivate, not create new
    let ids2 = upsert_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id, &chunks)
        .await
        .unwrap();
    assert_eq!(ids1[0], ids2[0]); // same ID reused

    let active = get_chunks_for_node(&pool, DEFAULT_ACCOUNT_ID, node_id)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].status, "active");
}

#[tokio::test]
async fn update_chunk_retrieval_boost_clamps() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id = insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "a", "ha", 0)
        .await
        .unwrap();

    // Set to above max
    update_chunk_retrieval_boost(&pool, DEFAULT_ACCOUNT_ID, id, 10.0)
        .await
        .unwrap();
    let chunk = get_chunk_by_id(&pool, DEFAULT_ACCOUNT_ID, id)
        .await
        .unwrap()
        .unwrap();
    assert!((chunk.retrieval_boost - 5.0).abs() < f64::EPSILON);

    // Set to below min
    update_chunk_retrieval_boost(&pool, DEFAULT_ACCOUNT_ID, id, 0.01)
        .await
        .unwrap();
    let chunk = get_chunk_by_id(&pool, DEFAULT_ACCOUNT_ID, id)
        .await
        .unwrap()
        .unwrap();
    assert!((chunk.retrieval_boost - 0.1).abs() < f64::EPSILON);

    // Set to valid value
    update_chunk_retrieval_boost(&pool, DEFAULT_ACCOUNT_ID, id, 2.5)
        .await
        .unwrap();
    let chunk = get_chunk_by_id(&pool, DEFAULT_ACCOUNT_ID, id)
        .await
        .unwrap()
        .unwrap();
    assert!((chunk.retrieval_boost - 2.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn search_chunks_by_keywords_empty_returns_empty() {
    let pool = init_test_db().await.unwrap();
    let result = search_chunks_by_keywords(&pool, DEFAULT_ACCOUNT_ID, &[], 10)
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn search_chunks_by_keywords_finds_matching() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# Intro",
        "Rust is a systems programming language",
        "hash-rust",
        0,
    )
    .await
    .unwrap();
    insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# Other",
        "Python is great for scripting",
        "hash-python",
        1,
    )
    .await
    .unwrap();

    let results = search_chunks_by_keywords(&pool, DEFAULT_ACCOUNT_ID, &["Rust"], 10)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].chunk_text.contains("Rust"));
}

#[tokio::test]
async fn search_chunks_by_keywords_respects_limit() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    for i in 0..5 {
        insert_chunk(
            &pool,
            DEFAULT_ACCOUNT_ID,
            node_id,
            &format!("# Section {i}"),
            &format!("keyword appears here {i}"),
            &format!("hash-{i}"),
            i,
        )
        .await
        .unwrap();
    }

    let results = search_chunks_by_keywords(&pool, DEFAULT_ACCOUNT_ID, &["keyword"], 2)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn find_best_chunk_by_heading_no_heading_returns_first() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "first", "h1", 0)
        .await
        .unwrap();
    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# B", "second", "h2", 1)
        .await
        .unwrap();

    // None heading → returns first chunk
    let best = find_best_chunk_by_heading_for(&pool, DEFAULT_ACCOUNT_ID, node_id, None)
        .await
        .unwrap();
    assert!(best.is_some());
    assert_eq!(best.unwrap().chunk_index, 0);
}

#[tokio::test]
async fn find_best_chunk_by_heading_empty_string_returns_first() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "first", "h1", 0)
        .await
        .unwrap();

    let best = find_best_chunk_by_heading_for(&pool, DEFAULT_ACCOUNT_ID, node_id, Some(""))
        .await
        .unwrap();
    assert!(best.is_some());
    assert_eq!(best.unwrap().chunk_index, 0);
}

#[tokio::test]
async fn find_best_chunk_by_heading_matches_prefix() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# Introduction",
        "intro text",
        "h-intro",
        0,
    )
    .await
    .unwrap();
    insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# Introduction > Details",
        "details text",
        "h-details",
        1,
    )
    .await
    .unwrap();

    let best = find_best_chunk_by_heading_for(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        Some("# Introduction > Details > Sub"),
    )
    .await
    .unwrap();
    assert!(best.is_some());
    assert_eq!(best.unwrap().heading_path, "# Introduction > Details");
}

#[tokio::test]
async fn find_best_chunk_by_heading_no_chunks_returns_none() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let best =
        find_best_chunk_by_heading_for(&pool, DEFAULT_ACCOUNT_ID, node_id, Some("# Nonexistent"))
            .await
            .unwrap();
    // No chunks exist → None
    assert!(best.is_none());
}

#[tokio::test]
async fn get_best_chunks_for_nodes_empty_ids_returns_empty() {
    let pool = init_test_db().await.unwrap();
    let result = get_best_chunks_for_nodes(&pool, DEFAULT_ACCOUNT_ID, &[])
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn get_best_chunks_for_nodes_returns_highest_boost() {
    let pool = init_test_db().await.unwrap();
    let (_src, node_id) = setup_node(&pool).await;

    let id1 = insert_chunk(&pool, DEFAULT_ACCOUNT_ID, node_id, "# A", "low", "h-low", 0)
        .await
        .unwrap();
    let id2 = insert_chunk(
        &pool,
        DEFAULT_ACCOUNT_ID,
        node_id,
        "# B",
        "high",
        "h-high",
        1,
    )
    .await
    .unwrap();

    // Boost second chunk higher
    update_chunk_retrieval_boost(&pool, DEFAULT_ACCOUNT_ID, id2, 3.0)
        .await
        .unwrap();
    // Keep first at default (1.0)
    let _ = id1;

    let best = get_best_chunks_for_nodes(&pool, DEFAULT_ACCOUNT_ID, &[node_id])
        .await
        .unwrap();
    assert_eq!(best.len(), 1);
    assert_eq!(best[0].id, id2);
}
