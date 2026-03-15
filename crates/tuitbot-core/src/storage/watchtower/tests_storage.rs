//! Additional storage tests for watchtower: sources, nodes, and chunks.
//!
//! Split from `tests.rs` to stay under the 500-line file limit.

use super::*;
use crate::storage::init_test_db;

const TEST_ACCOUNT: &str = "00000000-0000-0000-0000-000000000000";
const OTHER_ACCOUNT: &str = "11111111-1111-1111-1111-111111111111";

/// Helper: create a source + node, return (source_id, node_id).
async fn setup_source_and_node(pool: &crate::storage::DbPool) -> (i64, i64) {
    let source_id =
        insert_source_context_for(pool, TEST_ACCOUNT, "local_fs", r#"{"path":"~/notes"}"#)
            .await
            .expect("insert source");
    upsert_content_node_for(
        pool,
        TEST_ACCOUNT,
        source_id,
        "test.md",
        "h1",
        Some("Test Note"),
        "Body text here.",
        None,
        Some("rust"),
    )
    .await
    .expect("upsert node");
    let node = get_content_node(pool, 1)
        .await
        .expect("get")
        .expect("exists");
    (source_id, node.id)
}

// ============================================================================
// Sources — ensure_local_fs_source
// ============================================================================

#[tokio::test]
async fn ensure_local_fs_source_creates_and_dedup() {
    let pool = init_test_db().await.expect("init db");

    let config = r#"{"path":"~/projects"}"#;
    let id1 = ensure_local_fs_source(&pool, "~/projects", config)
        .await
        .expect("first ensure");
    let id2 = ensure_local_fs_source(&pool, "~/projects", config)
        .await
        .expect("second ensure");
    assert_eq!(id1, id2, "should return same source on duplicate path");

    let ctx = get_source_context(&pool, id1)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(ctx.source_type, "local_fs");
    assert!(ctx.config_json.contains("~/projects"));
}

#[tokio::test]
async fn ensure_local_fs_source_different_paths_differ() {
    let pool = init_test_db().await.expect("init db");

    let id1 = ensure_local_fs_source(&pool, "~/notes", r#"{"path":"~/notes"}"#)
        .await
        .expect("ensure notes");
    let id2 = ensure_local_fs_source(&pool, "~/docs", r#"{"path":"~/docs"}"#)
        .await
        .expect("ensure docs");
    assert_ne!(id1, id2, "different paths should create distinct sources");
}

// ============================================================================
// Sources — ensure_google_drive_source
// ============================================================================

#[tokio::test]
async fn ensure_google_drive_source_creates_and_dedup() {
    let pool = init_test_db().await.expect("init db");

    let config = r#"{"folder_id":"abc123","name":"My Drive"}"#;
    let id1 = ensure_google_drive_source(&pool, "abc123", config)
        .await
        .expect("first ensure");
    let id2 = ensure_google_drive_source(&pool, "abc123", config)
        .await
        .expect("second ensure");
    assert_eq!(id1, id2, "same folder_id should return same source");

    let ctx = get_source_context(&pool, id1)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(ctx.source_type, "google_drive");
}

#[tokio::test]
async fn ensure_google_drive_source_different_folders_differ() {
    let pool = init_test_db().await.expect("init db");

    let id1 = ensure_google_drive_source(&pool, "folder_a", r#"{"folder_id":"folder_a"}"#)
        .await
        .expect("a");
    let id2 = ensure_google_drive_source(&pool, "folder_b", r#"{"folder_id":"folder_b"}"#)
        .await
        .expect("b");
    assert_ne!(id1, id2);
}

// ============================================================================
// Sources — find_source_by_path
// ============================================================================

#[tokio::test]
async fn find_source_by_path_matches_substring() {
    let pool = init_test_db().await.expect("init db");

    insert_source_context(&pool, "local_fs", r#"{"path":"~/notes/journal"}"#)
        .await
        .expect("insert");

    let found = find_source_by_path(&pool, "journal").await.expect("find");
    assert!(found.is_some());
    assert!(found.unwrap().config_json.contains("journal"));
}

#[tokio::test]
async fn find_source_by_path_returns_none_for_no_match() {
    let pool = init_test_db().await.expect("init db");

    insert_source_context(&pool, "local_fs", r#"{"path":"~/notes"}"#)
        .await
        .expect("insert");

    let found = find_source_by_path(&pool, "nonexistent")
        .await
        .expect("find");
    assert!(found.is_none());
}

#[tokio::test]
async fn find_source_by_path_ignores_non_local_fs() {
    let pool = init_test_db().await.expect("init db");

    // Insert a google_drive source with "notes" in config
    insert_source_context(&pool, "google_drive", r#"{"folder_id":"notes"}"#)
        .await
        .expect("insert");

    // find_source_by_path only matches local_fs
    let found = find_source_by_path(&pool, "notes").await.expect("find");
    assert!(found.is_none());
}

// ============================================================================
// Sources — get_all_source_contexts / get_all_source_contexts_for
// ============================================================================

#[tokio::test]
async fn get_all_source_contexts_includes_all_statuses() {
    let pool = init_test_db().await.expect("init db");

    let id1 = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert");
    let id2 = insert_source_context(&pool, "manual", "{}")
        .await
        .expect("insert");

    update_source_status(&pool, id2, "paused", None)
        .await
        .expect("pause");

    let all = get_all_source_contexts(&pool).await.expect("get all");
    assert_eq!(all.len(), 2);

    // Verify both statuses present
    let statuses: Vec<&str> = all.iter().map(|s| s.status.as_str()).collect();
    assert!(statuses.contains(&"active"));
    assert!(statuses.contains(&"paused"));

    // get_source_contexts returns only active
    let active = get_source_contexts(&pool).await.expect("get active");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, id1);
}

#[tokio::test]
async fn get_all_source_contexts_for_scoped() {
    let pool = init_test_db().await.expect("init db");

    insert_source_context_for(&pool, "acct-a", "local_fs", "{}")
        .await
        .expect("insert a");
    insert_source_context_for(&pool, "acct-b", "manual", "{}")
        .await
        .expect("insert b");

    let all_a = get_all_source_contexts_for(&pool, "acct-a")
        .await
        .expect("get");
    assert_eq!(all_a.len(), 1);
    assert_eq!(all_a[0].source_type, "local_fs");

    let all_b = get_all_source_contexts_for(&pool, "acct-b")
        .await
        .expect("get");
    assert_eq!(all_b.len(), 1);
    assert_eq!(all_b[0].source_type, "manual");
}

// ============================================================================
// Nodes — search_nodes_for
// ============================================================================

#[tokio::test]
async fn search_nodes_for_matches_title() {
    let pool = init_test_db().await.expect("init db");
    let (source_id, _) = setup_source_and_node(&pool).await;

    // Add another node with different title
    upsert_content_node_for(
        &pool,
        TEST_ACCOUNT,
        source_id,
        "guide.md",
        "h2",
        Some("Async Guide"),
        "Async body",
        None,
        None,
    )
    .await
    .expect("upsert");

    let results = search_nodes_for(&pool, TEST_ACCOUNT, "Guide", 10)
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.as_deref(), Some("Async Guide"));
}

#[tokio::test]
async fn search_nodes_for_matches_path() {
    let pool = init_test_db().await.expect("init db");
    let (source_id, _) = setup_source_and_node(&pool).await;

    upsert_content_node_for(
        &pool,
        TEST_ACCOUNT,
        source_id,
        "deep/nested/file.md",
        "h2",
        None,
        "Content",
        None,
        None,
    )
    .await
    .expect("upsert");

    let results = search_nodes_for(&pool, TEST_ACCOUNT, "nested", 10)
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].relative_path, "deep/nested/file.md");
}

#[tokio::test]
async fn search_nodes_for_respects_account_scope() {
    let pool = init_test_db().await.expect("init db");
    let (_, _) = setup_source_and_node(&pool).await;

    let results = search_nodes_for(&pool, OTHER_ACCOUNT, "Test", 10)
        .await
        .expect("search");
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_nodes_for_respects_limit() {
    let pool = init_test_db().await.expect("init db");
    let src = insert_source_context_for(&pool, TEST_ACCOUNT, "local_fs", "{}")
        .await
        .expect("insert");

    for i in 0..5 {
        upsert_content_node_for(
            &pool,
            TEST_ACCOUNT,
            src,
            &format!("note_{i}.md"),
            &format!("h{i}"),
            Some("Note"),
            "Body",
            None,
            None,
        )
        .await
        .expect("upsert");
    }

    let results = search_nodes_for(&pool, TEST_ACCOUNT, "Note", 3)
        .await
        .expect("search");
    assert_eq!(results.len(), 3);
}

// ============================================================================
// Nodes — get_nodes_for_source_for
// ============================================================================

#[tokio::test]
async fn get_nodes_for_source_for_scoped() {
    let pool = init_test_db().await.expect("init db");
    let (source_id, _) = setup_source_and_node(&pool).await;

    let nodes = get_nodes_for_source_for(&pool, TEST_ACCOUNT, source_id, 10)
        .await
        .expect("get");
    assert_eq!(nodes.len(), 1);

    let nodes = get_nodes_for_source_for(&pool, OTHER_ACCOUNT, source_id, 10)
        .await
        .expect("get");
    assert!(nodes.is_empty());
}

// ============================================================================
// Nodes — get_content_node_for
// ============================================================================

#[tokio::test]
async fn get_content_node_for_scoped() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    let node = get_content_node_for(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("get");
    assert!(node.is_some());
    assert_eq!(node.unwrap().relative_path, "test.md");

    let node = get_content_node_for(&pool, OTHER_ACCOUNT, node_id)
        .await
        .expect("get");
    assert!(node.is_none());
}

// ============================================================================
// Nodes — count_chunks_for_node
// ============================================================================

#[tokio::test]
async fn count_chunks_for_node_counts_active_only() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    // No chunks yet
    let count = count_chunks_for_node(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("count");
    assert_eq!(count, 0);

    // Insert two chunks
    insert_chunk(&pool, TEST_ACCOUNT, node_id, "## A", "Chunk A", "ha", 0)
        .await
        .expect("insert");
    insert_chunk(&pool, TEST_ACCOUNT, node_id, "## B", "Chunk B", "hb", 1)
        .await
        .expect("insert");

    let count = count_chunks_for_node(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("count");
    assert_eq!(count, 2);

    // Mark stale -> count should be 0
    mark_chunks_stale(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("stale");
    let count = count_chunks_for_node(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn count_chunks_for_node_scoped_to_account() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "Text", "h1", 0)
        .await
        .expect("insert");

    let count = count_chunks_for_node(&pool, OTHER_ACCOUNT, node_id)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

// ============================================================================
// Nodes — count_nodes_for_source
// ============================================================================

#[tokio::test]
async fn count_nodes_for_source_works() {
    let pool = init_test_db().await.expect("init db");
    let (source_id, _) = setup_source_and_node(&pool).await;

    let count = count_nodes_for_source(&pool, TEST_ACCOUNT, source_id)
        .await
        .expect("count");
    assert_eq!(count, 1);

    // Add another node
    upsert_content_node_for(
        &pool,
        TEST_ACCOUNT,
        source_id,
        "second.md",
        "h2",
        None,
        "Second",
        None,
        None,
    )
    .await
    .expect("upsert");

    let count = count_nodes_for_source(&pool, TEST_ACCOUNT, source_id)
        .await
        .expect("count");
    assert_eq!(count, 2);

    // Other account sees 0
    let count = count_nodes_for_source(&pool, OTHER_ACCOUNT, source_id)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

// ============================================================================
// Nodes — mark_node_processed_for
// ============================================================================

#[tokio::test]
async fn mark_node_processed_for_scoped() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    mark_node_processed_for(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("mark");

    let node = get_content_node_for(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(node.status, "processed");
}

#[tokio::test]
async fn mark_node_processed_for_wrong_account_no_effect() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    // Try to mark with wrong account
    mark_node_processed_for(&pool, OTHER_ACCOUNT, node_id)
        .await
        .expect("mark");

    // Node should still be pending
    let node = get_content_node(&pool, node_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(node.status, "pending");
}

// ============================================================================
// Chunks — search_chunks_with_context
// ============================================================================

#[tokio::test]
async fn search_chunks_with_context_returns_node_metadata() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(
        &pool,
        TEST_ACCOUNT,
        node_id,
        "## Ownership",
        "Rust ownership model explained",
        "h_own",
        0,
    )
    .await
    .expect("insert");

    let results = search_chunks_with_context(&pool, TEST_ACCOUNT, &["ownership"], 10)
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].chunk.chunk_text,
        "Rust ownership model explained"
    );
    assert_eq!(results[0].relative_path, "test.md");
    assert_eq!(results[0].source_title.as_deref(), Some("Test Note"));
}

#[tokio::test]
async fn search_chunks_with_context_empty_keywords() {
    let pool = init_test_db().await.expect("init db");

    let results = search_chunks_with_context(&pool, TEST_ACCOUNT, &[], 10)
        .await
        .expect("search");
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_chunks_with_context_respects_account() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "secret data", "h1", 0)
        .await
        .expect("insert");

    let results = search_chunks_with_context(&pool, OTHER_ACCOUNT, &["secret"], 10)
        .await
        .expect("search");
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_chunks_with_context_skips_stale() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "findable text", "h1", 0)
        .await
        .expect("insert");
    mark_chunks_stale(&pool, TEST_ACCOUNT, node_id)
        .await
        .expect("stale");

    let results = search_chunks_with_context(&pool, TEST_ACCOUNT, &["findable"], 10)
        .await
        .expect("search");
    assert!(results.is_empty());
}

// ============================================================================
// Chunks — get_chunks_for_nodes_with_context
// ============================================================================

#[tokio::test]
async fn get_chunks_for_nodes_with_context_basic() {
    let pool = init_test_db().await.expect("init db");
    let (source_id, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(
        &pool,
        TEST_ACCOUNT,
        node_id,
        "## Intro",
        "Intro text",
        "h1",
        0,
    )
    .await
    .expect("insert");
    insert_chunk(
        &pool,
        TEST_ACCOUNT,
        node_id,
        "## Body",
        "Body text",
        "h2",
        1,
    )
    .await
    .expect("insert");

    // Create a second node
    upsert_content_node_for(
        &pool,
        TEST_ACCOUNT,
        source_id,
        "other.md",
        "h_other",
        Some("Other Note"),
        "Other body",
        None,
        None,
    )
    .await
    .expect("upsert");
    let node2 = get_content_node(&pool, 2)
        .await
        .expect("get")
        .expect("exists");
    insert_chunk(
        &pool,
        TEST_ACCOUNT,
        node2.id,
        "## Other",
        "Other chunk",
        "h3",
        0,
    )
    .await
    .expect("insert");

    let results = get_chunks_for_nodes_with_context(&pool, TEST_ACCOUNT, &[node_id, node2.id], 10)
        .await
        .expect("get");
    assert_eq!(results.len(), 3);

    // Verify node metadata is present
    let paths: Vec<&str> = results.iter().map(|r| r.relative_path.as_str()).collect();
    assert!(paths.contains(&"test.md"));
    assert!(paths.contains(&"other.md"));
}

#[tokio::test]
async fn get_chunks_for_nodes_with_context_empty_node_ids() {
    let pool = init_test_db().await.expect("init db");

    let results = get_chunks_for_nodes_with_context(&pool, TEST_ACCOUNT, &[], 10)
        .await
        .expect("get");
    assert!(results.is_empty());
}

#[tokio::test]
async fn get_chunks_for_nodes_with_context_respects_account() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "Data", "h1", 0)
        .await
        .expect("insert");

    let results = get_chunks_for_nodes_with_context(&pool, OTHER_ACCOUNT, &[node_id], 10)
        .await
        .expect("get");
    assert!(results.is_empty());
}

#[tokio::test]
async fn get_chunks_for_nodes_with_context_respects_limit() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    for i in 0..5 {
        insert_chunk(
            &pool,
            TEST_ACCOUNT,
            node_id,
            &format!("## H{i}"),
            &format!("Chunk {i}"),
            &format!("h{i}"),
            i,
        )
        .await
        .expect("insert");
    }

    let results = get_chunks_for_nodes_with_context(&pool, TEST_ACCOUNT, &[node_id], 3)
        .await
        .expect("get");
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn get_chunks_for_nodes_with_context_ordered_by_boost() {
    let pool = init_test_db().await.expect("init db");
    let (_, node_id) = setup_source_and_node(&pool).await;

    let low = insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "Low boost", "h1", 0)
        .await
        .expect("insert");
    let high = insert_chunk(&pool, TEST_ACCOUNT, node_id, "", "High boost", "h2", 1)
        .await
        .expect("insert");

    update_chunk_retrieval_boost(&pool, TEST_ACCOUNT, high, 4.0)
        .await
        .expect("boost");
    update_chunk_retrieval_boost(&pool, TEST_ACCOUNT, low, 0.5)
        .await
        .expect("boost");

    let results = get_chunks_for_nodes_with_context(&pool, TEST_ACCOUNT, &[node_id], 10)
        .await
        .expect("get");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].chunk.id, high);
    assert_eq!(results[1].chunk.id, low);
}

// ============================================================================
// Sources — find_source_by_folder_id
// ============================================================================

#[tokio::test]
async fn find_source_by_folder_id_matches() {
    let pool = init_test_db().await.expect("init db");

    insert_source_context(&pool, "google_drive", r#"{"folder_id":"xyz789"}"#)
        .await
        .expect("insert");

    let found = find_source_by_folder_id(&pool, "xyz789")
        .await
        .expect("find");
    assert!(found.is_some());
    assert_eq!(found.unwrap().source_type, "google_drive");
}

#[tokio::test]
async fn find_source_by_folder_id_no_match() {
    let pool = init_test_db().await.expect("init db");

    insert_source_context(&pool, "google_drive", r#"{"folder_id":"abc"}"#)
        .await
        .expect("insert");

    let found = find_source_by_folder_id(&pool, "nonexistent")
        .await
        .expect("find");
    assert!(found.is_none());
}

#[tokio::test]
async fn find_source_by_folder_id_ignores_local_fs() {
    let pool = init_test_db().await.expect("init db");

    // A local_fs source with folder_id in config should NOT be matched
    insert_source_context(&pool, "local_fs", r#"{"folder_id":"abc123"}"#)
        .await
        .expect("insert");

    let found = find_source_by_folder_id(&pool, "abc123")
        .await
        .expect("find");
    assert!(found.is_none());
}
