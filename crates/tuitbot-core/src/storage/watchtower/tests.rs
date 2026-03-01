use super::*;
use crate::storage::init_test_db;

#[tokio::test]
async fn migration_creates_new_tables() {
    let pool = init_test_db().await.expect("init db");

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' \
         AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .expect("query tables");

    let table_names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
    assert!(table_names.contains(&"source_contexts"));
    assert!(table_names.contains(&"content_nodes"));
    assert!(table_names.contains(&"draft_seeds"));
}

#[tokio::test]
async fn migration_adds_columns_to_performance() {
    let pool = init_test_db().await.expect("init db");

    // Check tweet_performance
    let cols: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('tweet_performance')")
            .fetch_all(&pool)
            .await
            .expect("pragma");
    let col_names: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();
    assert!(col_names.contains(&"archetype_vibe"));
    assert!(col_names.contains(&"engagement_score"));

    // Check reply_performance
    let cols: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('reply_performance')")
            .fetch_all(&pool)
            .await
            .expect("pragma");
    let col_names: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();
    assert!(col_names.contains(&"archetype_vibe"));
    assert!(col_names.contains(&"engagement_score"));
}

#[tokio::test]
async fn migration_adds_source_node_id_to_tweets() {
    let pool = init_test_db().await.expect("init db");

    let cols: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('original_tweets')")
            .fetch_all(&pool)
            .await
            .expect("pragma");
    let col_names: Vec<&str> = cols.iter().map(|c| c.0.as_str()).collect();
    assert!(col_names.contains(&"source_node_id"));
}

#[tokio::test]
async fn insert_and_get_source_context() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_source_context(&pool, "local_fs", r#"{"path":"~/notes"}"#)
        .await
        .expect("insert");
    assert!(id > 0);

    let ctx = get_source_context(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(ctx.source_type, "local_fs");
    assert_eq!(ctx.config_json, r#"{"path":"~/notes"}"#);
    assert_eq!(ctx.status, "active");
    assert!(ctx.sync_cursor.is_none());
}

#[tokio::test]
async fn get_source_contexts_returns_active_only() {
    let pool = init_test_db().await.expect("init db");

    let id1 = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert");
    let id2 = insert_source_context(&pool, "manual", "{}")
        .await
        .expect("insert");

    // Pause id2
    update_source_status(&pool, id2, "paused", None)
        .await
        .expect("update");

    let active = get_source_contexts(&pool).await.expect("get");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, id1);
}

#[tokio::test]
async fn update_sync_cursor_works() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert");

    update_sync_cursor(&pool, id, "2026-02-28T12:00:00Z")
        .await
        .expect("update");

    let ctx = get_source_context(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(ctx.sync_cursor.as_deref(), Some("2026-02-28T12:00:00Z"));
}

#[tokio::test]
async fn update_source_status_with_error() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert");

    update_source_status(&pool, id, "error", Some("path not found"))
        .await
        .expect("update");

    let ctx = get_source_context(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(ctx.status, "error");
    assert_eq!(ctx.error_message.as_deref(), Some("path not found"));
}

#[tokio::test]
async fn insert_content_node() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    let result = upsert_content_node(
        &pool,
        source_id,
        "notes/test.md",
        "abc123",
        Some("Test Note"),
        "Body text here.",
        None,
        Some("rust,testing"),
    )
    .await
    .expect("upsert");

    assert_eq!(result, UpsertResult::Inserted);

    let node = get_content_node(&pool, 1)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(node.relative_path, "notes/test.md");
    assert_eq!(node.content_hash, "abc123");
    assert_eq!(node.title.as_deref(), Some("Test Note"));
    assert_eq!(node.body_text, "Body text here.");
    assert_eq!(node.tags.as_deref(), Some("rust,testing"));
    assert_eq!(node.status, "pending");
}

#[tokio::test]
async fn content_node_upsert_by_hash() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    // First insert
    let r1 = upsert_content_node(
        &pool,
        source_id,
        "notes/test.md",
        "hash_v1",
        Some("V1"),
        "Version 1 content.",
        None,
        None,
    )
    .await
    .expect("upsert");
    assert_eq!(r1, UpsertResult::Inserted);

    // Upsert with different hash → Updated
    let r2 = upsert_content_node(
        &pool,
        source_id,
        "notes/test.md",
        "hash_v2",
        Some("V2"),
        "Version 2 content.",
        None,
        None,
    )
    .await
    .expect("upsert");
    assert_eq!(r2, UpsertResult::Updated);

    // Verify the content was updated
    let node = get_content_node(&pool, 1)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(node.content_hash, "hash_v2");
    assert_eq!(node.title.as_deref(), Some("V2"));
    assert_eq!(node.body_text, "Version 2 content.");
}

#[tokio::test]
async fn content_node_dedup_same_hash() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    // First insert
    upsert_content_node(
        &pool,
        source_id,
        "notes/test.md",
        "same_hash",
        Some("Title"),
        "Same content.",
        None,
        None,
    )
    .await
    .expect("upsert");

    // Same hash → Skipped
    let result = upsert_content_node(
        &pool,
        source_id,
        "notes/test.md",
        "same_hash",
        Some("Title"),
        "Same content.",
        None,
        None,
    )
    .await
    .expect("upsert");
    assert_eq!(result, UpsertResult::Skipped);
}

#[tokio::test]
async fn get_nodes_for_source_with_filter() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    upsert_content_node(&pool, source_id, "a.md", "h1", None, "A", None, None)
        .await
        .expect("upsert");
    upsert_content_node(&pool, source_id, "b.md", "h2", None, "B", None, None)
        .await
        .expect("upsert");

    // All nodes
    let all = get_nodes_for_source(&pool, source_id, None)
        .await
        .expect("get");
    assert_eq!(all.len(), 2);

    // Filter by pending
    let pending = get_nodes_for_source(&pool, source_id, Some("pending"))
        .await
        .expect("get");
    assert_eq!(pending.len(), 2);

    // Filter by processed (none)
    let processed = get_nodes_for_source(&pool, source_id, Some("processed"))
        .await
        .expect("get");
    assert!(processed.is_empty());
}

#[tokio::test]
async fn insert_draft_seed_works() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    upsert_content_node(&pool, source_id, "n.md", "h", None, "B", None, None)
        .await
        .expect("upsert");

    let seed_id = insert_draft_seed(&pool, 1, "A hook about Rust concurrency", Some("AddData"))
        .await
        .expect("insert seed");
    assert!(seed_id > 0);
}

#[tokio::test]
async fn get_pending_seeds_ordered_by_weight() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    upsert_content_node(&pool, source_id, "n.md", "h", None, "B", None, None)
        .await
        .expect("upsert");

    // Insert seeds with different weights
    let s1 = insert_draft_seed(&pool, 1, "Low weight seed", None)
        .await
        .expect("insert");
    let s2 = insert_draft_seed(&pool, 1, "High weight seed", Some("AgreeAndExpand"))
        .await
        .expect("insert");

    // Update weights: s2 gets higher weight
    sqlx::query("UPDATE draft_seeds SET engagement_weight = 0.9 WHERE id = ?")
        .bind(s2)
        .execute(&pool)
        .await
        .expect("update weight");
    sqlx::query("UPDATE draft_seeds SET engagement_weight = 0.2 WHERE id = ?")
        .bind(s1)
        .execute(&pool)
        .await
        .expect("update weight");

    let seeds = get_pending_seeds(&pool, 10).await.expect("get");
    assert_eq!(seeds.len(), 2);
    assert_eq!(seeds[0].seed_text, "High weight seed");
    assert_eq!(seeds[1].seed_text, "Low weight seed");
    assert!(seeds[0].engagement_weight > seeds[1].engagement_weight);
}

#[tokio::test]
async fn mark_seed_used_works() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    upsert_content_node(&pool, source_id, "n.md", "h", None, "B", None, None)
        .await
        .expect("upsert");

    let seed_id = insert_draft_seed(&pool, 1, "A seed", None)
        .await
        .expect("insert");

    mark_seed_used(&pool, seed_id).await.expect("mark used");

    // Should no longer appear in pending seeds
    let pending = get_pending_seeds(&pool, 10).await.expect("get");
    assert!(pending.is_empty());
}

#[tokio::test]
async fn ensure_manual_source_creates_once() {
    let pool = init_test_db().await.expect("init db");

    let id1 = ensure_manual_source(&pool).await.expect("first call");
    let id2 = ensure_manual_source(&pool).await.expect("second call");
    assert_eq!(id1, id2, "should return same source ID");

    let ctx = get_source_context(&pool, id1)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(ctx.source_type, "manual");
}

// ============================================================================
// Winning DNA storage tests
// ============================================================================

#[tokio::test]
async fn get_pending_content_nodes_returns_pending_only() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    // Insert two nodes (both start as 'pending')
    upsert_content_node(
        &pool,
        source_id,
        "a.md",
        "h1",
        Some("Note A"),
        "Body A",
        None,
        None,
    )
    .await
    .expect("upsert");
    upsert_content_node(
        &pool,
        source_id,
        "b.md",
        "h2",
        Some("Note B"),
        "Body B",
        None,
        None,
    )
    .await
    .expect("upsert");

    // Mark one as processed
    mark_node_processed(&pool, 1).await.expect("mark");

    let pending = get_pending_content_nodes(&pool, 10).await.expect("get");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].relative_path, "b.md");
}

#[tokio::test]
async fn mark_node_processed_changes_status() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");

    upsert_content_node(&pool, source_id, "a.md", "h1", None, "Body", None, None)
        .await
        .expect("upsert");

    mark_node_processed(&pool, 1).await.expect("mark");

    let node = get_content_node(&pool, 1)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(node.status, "processed");
}

#[tokio::test]
async fn insert_seed_with_weight_persists() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    upsert_content_node(&pool, source_id, "n.md", "h", None, "Body", None, None)
        .await
        .expect("upsert");

    let seed_id = insert_draft_seed_with_weight(&pool, 1, "A hook about Rust", Some("tip"), 0.75)
        .await
        .expect("insert seed");

    let row: (f64,) = sqlx::query_as("SELECT engagement_weight FROM draft_seeds WHERE id = ?")
        .bind(seed_id)
        .fetch_one(&pool)
        .await
        .expect("query");
    assert!((row.0 - 0.75).abs() < 0.001);
}

#[tokio::test]
async fn get_seeds_for_context_joins_with_nodes() {
    let pool = init_test_db().await.expect("init db");

    let source_id = insert_source_context(&pool, "local_fs", "{}")
        .await
        .expect("insert source");
    upsert_content_node(
        &pool,
        source_id,
        "rust-tips.md",
        "h1",
        Some("Rust Tips"),
        "Body text",
        None,
        None,
    )
    .await
    .expect("upsert");

    insert_draft_seed_with_weight(&pool, 1, "Hook about ownership", Some("tip"), 0.8)
        .await
        .expect("insert seed");
    insert_draft_seed_with_weight(&pool, 1, "Hook about async", Some("question"), 0.6)
        .await
        .expect("insert seed");

    let seeds = get_seeds_for_context(&pool, 10).await.expect("get");
    assert_eq!(seeds.len(), 2);
    // Ordered by weight DESC
    assert_eq!(seeds[0].seed_text, "Hook about ownership");
    assert_eq!(seeds[0].source_title.as_deref(), Some("Rust Tips"));
    assert!((seeds[0].engagement_weight - 0.8).abs() < 0.001);
    assert_eq!(seeds[1].seed_text, "Hook about async");
}

// ============================================================================
// Connection CRUD tests
// ============================================================================

#[tokio::test]
async fn migration_creates_connections_table() {
    let pool = init_test_db().await.expect("init db");

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' \
         AND name = 'connections'",
    )
    .fetch_all(&pool)
    .await
    .expect("query tables");

    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].0, "connections");
}

#[tokio::test]
async fn insert_and_get_connection() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_connection(
        &pool,
        "google_drive",
        Some("user@gmail.com"),
        Some("My Drive"),
    )
    .await
    .expect("insert");
    assert!(id > 0);

    let conn = get_connection(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(conn.connector_type, "google_drive");
    assert_eq!(conn.account_email.as_deref(), Some("user@gmail.com"));
    assert_eq!(conn.display_name.as_deref(), Some("My Drive"));
    assert_eq!(conn.status, "active");
    assert_eq!(conn.metadata_json, "{}");
}

#[tokio::test]
async fn get_connections_returns_active_only() {
    let pool = init_test_db().await.expect("init db");

    let id1 = insert_connection(&pool, "google_drive", Some("a@gmail.com"), None)
        .await
        .expect("insert");
    let id2 = insert_connection(&pool, "google_drive", Some("b@gmail.com"), None)
        .await
        .expect("insert");

    // Mark id2 as expired.
    update_connection_status(&pool, id2, "expired")
        .await
        .expect("update");

    let active = get_connections(&pool).await.expect("get");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, id1);
}

#[tokio::test]
async fn update_connection_status_works() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_connection(&pool, "google_drive", None, None)
        .await
        .expect("insert");

    update_connection_status(&pool, id, "expired")
        .await
        .expect("update");

    let conn = get_connection(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(conn.status, "expired");
}

#[tokio::test]
async fn delete_connection_works() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_connection(&pool, "google_drive", Some("user@gmail.com"), None)
        .await
        .expect("insert");

    delete_connection(&pool, id).await.expect("delete");

    let conn = get_connection(&pool, id).await.expect("get");
    assert!(conn.is_none());
}

#[tokio::test]
async fn get_connections_by_type_filters_correctly() {
    let pool = init_test_db().await.expect("init db");

    // Insert two different connector types.
    insert_connection(&pool, "google_drive", Some("a@gmail.com"), None)
        .await
        .expect("insert");
    insert_connection(&pool, "onedrive", Some("b@outlook.com"), None)
        .await
        .expect("insert");

    let gdrive = get_connections_by_type(&pool, "google_drive")
        .await
        .expect("get");
    assert_eq!(gdrive.len(), 1);
    assert_eq!(gdrive[0].account_email.as_deref(), Some("a@gmail.com"));

    let onedrive = get_connections_by_type(&pool, "onedrive")
        .await
        .expect("get");
    assert_eq!(onedrive.len(), 1);
    assert_eq!(onedrive[0].account_email.as_deref(), Some("b@outlook.com"));
}

#[tokio::test]
async fn store_and_read_encrypted_credentials() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_connection(&pool, "google_drive", Some("user@gmail.com"), None)
        .await
        .expect("insert");

    // Initially no credentials.
    let creds = read_encrypted_credentials(&pool, id).await.expect("read");
    assert!(creds.is_none());

    // Store some ciphertext.
    let ciphertext = vec![1, 2, 3, 4, 5];
    store_encrypted_credentials(&pool, id, &ciphertext)
        .await
        .expect("store");

    let creds = read_encrypted_credentials(&pool, id)
        .await
        .expect("read")
        .expect("should have creds");
    assert_eq!(creds, ciphertext);
}

#[tokio::test]
async fn read_encrypted_credentials_returns_none_for_missing() {
    let pool = init_test_db().await.expect("init db");

    let creds = read_encrypted_credentials(&pool, 99999)
        .await
        .expect("read");
    assert!(creds.is_none());
}

#[tokio::test]
async fn update_connection_metadata_works() {
    let pool = init_test_db().await.expect("init db");

    let id = insert_connection(&pool, "google_drive", None, None)
        .await
        .expect("insert");

    let metadata = r#"{"scope":"drive.readonly","linked_at":"2026-02-28T12:00:00Z"}"#;
    update_connection_metadata(&pool, id, metadata)
        .await
        .expect("update");

    let conn = get_connection(&pool, id)
        .await
        .expect("get")
        .expect("should exist");
    assert_eq!(conn.metadata_json, metadata);
}
