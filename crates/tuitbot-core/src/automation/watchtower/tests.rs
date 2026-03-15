//! Tests for the Watchtower filesystem watcher and ingest pipeline.

use std::path::PathBuf;

use super::*;
use crate::automation::watchtower::chunker::{self, extract_fragments};
use crate::storage::init_test_db;
use crate::storage::watchtower as store;

// ---------------------------------------------------------------------------
// Pattern matching
// ---------------------------------------------------------------------------

#[test]
fn matches_patterns_md_and_txt() {
    let patterns = vec!["*.md".to_string(), "*.txt".to_string()];
    assert!(matches_patterns(Path::new("note.md"), &patterns));
    assert!(matches_patterns(Path::new("readme.txt"), &patterns));
}

#[test]
fn matches_patterns_rejects_jpg() {
    let patterns = vec!["*.md".to_string(), "*.txt".to_string()];
    assert!(!matches_patterns(Path::new("photo.jpg"), &patterns));
}

#[test]
fn matches_patterns_nested_path() {
    let patterns = vec!["*.md".to_string()];
    assert!(matches_patterns(Path::new("sub/dir/note.md"), &patterns));
}

#[test]
fn matches_patterns_empty_patterns() {
    let patterns: Vec<String> = Vec::new();
    assert!(!matches_patterns(Path::new("note.md"), &patterns));
}

// ---------------------------------------------------------------------------
// Front-matter parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_front_matter_extracts_yaml() {
    let content = "---\ntitle: Test Note\ntags:\n  - rust\n  - watchtower\n---\nBody text here.\n";
    let (fm, body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("Test Note"));
    assert_eq!(fm.tags.as_deref(), Some("rust,watchtower"));
    assert_eq!(body, "Body text here.\n");
}

#[test]
fn parse_front_matter_no_yaml() {
    let content = "Just plain text without front-matter.\n";
    let (fm, body) = parse_front_matter(content);
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_none());
    assert_eq!(body, content);
}

#[test]
fn parse_front_matter_title_only() {
    let content = "---\ntitle: Hello World\n---\nContent.\n";
    let (fm, body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("Hello World"));
    assert!(fm.tags.is_none());
    assert_eq!(body, "Content.\n");
}

#[test]
fn parse_front_matter_tags_as_string() {
    let content = "---\ntags: \"rust, testing\"\n---\nBody.\n";
    let (fm, _body) = parse_front_matter(content);
    assert_eq!(fm.tags.as_deref(), Some("rust, testing"));
}

// ---------------------------------------------------------------------------
// Ingest pipeline
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_file_creates_content_node() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    std::fs::write(dir.path().join("note.md"), "Hello from the watchtower.\n").unwrap();

    let result = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();

    assert_eq!(result, store::UpsertResult::Inserted);

    let nodes = store::get_nodes_for_source(&pool, source_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].relative_path, "note.md");
    assert!(nodes[0].body_text.contains("Hello from the watchtower"));
}

#[tokio::test]
async fn ingest_file_with_front_matter() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    let content = "---\ntitle: My Great Note\ntags:\n  - idea\n  - draft\n---\nNote body.\n";
    std::fs::write(dir.path().join("idea.md"), content).unwrap();

    let result = ingest_file(&pool, source_id, dir.path(), "idea.md", false)
        .await
        .unwrap();

    assert_eq!(result, store::UpsertResult::Inserted);

    let nodes = store::get_nodes_for_source(&pool, source_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].title.as_deref(), Some("My Great Note"));
    assert_eq!(nodes[0].tags.as_deref(), Some("idea,draft"));
    assert!(nodes[0].front_matter_json.is_some());
}

#[tokio::test]
async fn ingest_file_dedup_by_hash() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    std::fs::write(dir.path().join("note.md"), "Static content.\n").unwrap();

    let first = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();
    assert_eq!(first, store::UpsertResult::Inserted);

    let second = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();
    assert_eq!(second, store::UpsertResult::Skipped);
}

#[tokio::test]
async fn ingest_file_updates_on_change() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    std::fs::write(dir.path().join("note.md"), "Version 1.\n").unwrap();

    let first = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();
    assert_eq!(first, store::UpsertResult::Inserted);

    std::fs::write(dir.path().join("note.md"), "Version 2 with changes.\n").unwrap();

    let second = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();
    assert_eq!(second, store::UpsertResult::Updated);
}

#[tokio::test]
async fn ingest_file_force_bypasses_hash() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    std::fs::write(dir.path().join("note.md"), "Forced content.\n").unwrap();

    let first = ingest_file(&pool, source_id, dir.path(), "note.md", false)
        .await
        .unwrap();
    assert_eq!(first, store::UpsertResult::Inserted);

    // Force re-ingest should update even though content is the same.
    let second = ingest_file(&pool, source_id, dir.path(), "note.md", true)
        .await
        .unwrap();
    assert_eq!(second, store::UpsertResult::Updated);
}

// ---------------------------------------------------------------------------
// Batch ingest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn batch_ingest_summary() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    std::fs::write(dir.path().join("a.md"), "File A.\n").unwrap();
    std::fs::write(dir.path().join("b.md"), "File B.\n").unwrap();

    // First batch: 2 new files.
    let paths = vec!["a.md".to_string(), "b.md".to_string()];
    let summary = ingest_files(&pool, source_id, dir.path(), &paths, false).await;
    assert_eq!(summary.ingested, 2);
    assert_eq!(summary.skipped, 0);

    // Second batch: same files + missing file.
    let paths2 = vec![
        "a.md".to_string(),
        "b.md".to_string(),
        "missing.md".to_string(),
    ];
    let summary2 = ingest_files(&pool, source_id, dir.path(), &paths2, false).await;
    assert_eq!(summary2.skipped, 2);
    assert_eq!(summary2.errors.len(), 1);
}

// ---------------------------------------------------------------------------
// Cooldown
// ---------------------------------------------------------------------------

#[test]
fn cooldown_prevents_reingest() {
    let mut cd = CooldownSet::new(Duration::from_secs(5));
    let path = PathBuf::from("/tmp/test.md");
    cd.mark(path.clone());
    assert!(cd.is_cooling(&path));
}

#[test]
fn cooldown_allows_unknown_path() {
    let cd = CooldownSet::new(Duration::from_secs(5));
    assert!(!cd.is_cooling(Path::new("/tmp/other.md")));
}

#[test]
fn cooldown_cleanup_removes_old() {
    let mut cd = CooldownSet::new(Duration::from_millis(0));
    let path = PathBuf::from("/tmp/test.md");
    cd.mark(path.clone());
    // With 0ms TTL, it's immediately expired.
    std::thread::sleep(Duration::from_millis(1));
    assert!(!cd.is_cooling(&path));
    cd.cleanup();
    assert!(cd.entries.is_empty());
}

// ---------------------------------------------------------------------------
// Directory walking
// ---------------------------------------------------------------------------

#[test]
fn walk_directory_finds_matching_files() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();

    std::fs::write(dir.path().join("root.md"), "root").unwrap();
    std::fs::write(sub.join("nested.md"), "nested").unwrap();
    std::fs::write(dir.path().join("image.jpg"), "binary").unwrap();

    let patterns = vec!["*.md".to_string()];
    let mut paths = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &patterns, &mut paths).unwrap();

    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"root.md".to_string()));
    assert!(paths.contains(&"subdir/nested.md".to_string()));
}

#[test]
fn walk_directory_skips_hidden_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let hidden = dir.path().join(".hidden");
    std::fs::create_dir(&hidden).unwrap();
    std::fs::write(hidden.join("secret.md"), "hidden").unwrap();
    std::fs::write(dir.path().join("visible.md"), "visible").unwrap();

    let patterns = vec!["*.md".to_string()];
    let mut paths = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &patterns, &mut paths).unwrap();

    assert_eq!(paths.len(), 1);
    assert!(paths.contains(&"visible.md".to_string()));
}

// ---------------------------------------------------------------------------
// Storage helpers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ensure_local_fs_source_creates_and_reuses() {
    let pool = init_test_db().await.expect("init db");

    let id1 =
        store::ensure_local_fs_source(&pool, "/home/user/notes", "{\"path\":\"/home/user/notes\"}")
            .await
            .unwrap();

    let id2 =
        store::ensure_local_fs_source(&pool, "/home/user/notes", "{\"path\":\"/home/user/notes\"}")
            .await
            .unwrap();

    assert_eq!(id1, id2);
}

#[tokio::test]
async fn find_source_by_path_returns_none_for_missing() {
    let pool = init_test_db().await.expect("init db");

    let result = store::find_source_by_path(&pool, "/nonexistent")
        .await
        .unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// Watcher cancellation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn watcher_respects_cancellation() {
    let pool = init_test_db().await.expect("init db");
    let config = ContentSourcesConfig {
        sources: Vec::new(), // No sources = immediate exit.
    };

    let watchtower = WatchtowerLoop::new(pool, config, Default::default(), std::env::temp_dir());
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        watchtower.run(cancel_clone).await;
    });

    // With no sources, the loop exits immediately.
    let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
    assert!(
        result.is_ok(),
        "Watcher should exit when no sources configured"
    );
}

#[tokio::test]
async fn watcher_cancels_with_sources() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    let config = ContentSourcesConfig {
        sources: vec![crate::config::ContentSourceEntry {
            source_type: "local_fs".to_string(),
            path: Some(dir.path().to_string_lossy().to_string()),
            folder_id: None,
            service_account_key: None,
            connection_id: None,
            watch: true,
            file_patterns: vec!["*.md".to_string()],
            loop_back_enabled: false,
            poll_interval_seconds: None,
            enabled: None,
            change_detection: "auto".to_string(),
        }],
    };

    let watchtower = WatchtowerLoop::new(pool, config, Default::default(), std::env::temp_dir());
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        watchtower.run(cancel_clone).await;
    });

    // Give the watcher time to start.
    tokio::time::sleep(Duration::from_millis(200)).await;

    cancel.cancel();

    let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
    assert!(
        result.is_ok(),
        "Watcher should exit within timeout after cancellation"
    );
}

// ---------------------------------------------------------------------------
// Connection-based source registration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn watchtower_skips_source_without_auth() {
    let pool = init_test_db().await.expect("init db");

    // A Google Drive source with neither service_account_key nor connection_id
    // should be skipped during registration.
    let config = ContentSourcesConfig {
        sources: vec![crate::config::ContentSourceEntry {
            source_type: "google_drive".to_string(),
            path: None,
            folder_id: Some("folder_no_auth".to_string()),
            service_account_key: None,
            connection_id: None,
            watch: true,
            file_patterns: vec!["*.md".to_string()],
            loop_back_enabled: false,
            poll_interval_seconds: Some(300),
            enabled: None,
            change_detection: "auto".to_string(),
        }],
    };

    let watchtower = WatchtowerLoop::new(
        pool.clone(),
        config,
        Default::default(),
        std::env::temp_dir(),
    );
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        watchtower.run(cancel_clone).await;
    });

    // The watchtower should register no remote sources and exit quickly.
    tokio::time::sleep(Duration::from_millis(200)).await;
    cancel.cancel();

    let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
    assert!(
        result.is_ok(),
        "Watcher should handle source without auth gracefully"
    );
}

#[tokio::test]
async fn watchtower_handles_broken_connection() {
    // Test that a ConnectionBroken error is properly handled by
    // updating both the source status and connection status.
    let pool = init_test_db().await.expect("init db");

    // Create a connection and mark it as active.
    let conn_id = store::insert_connection(&pool, "google_drive", Some("test@example.com"), None)
        .await
        .unwrap();

    // Register a source context for the connection.
    let src_id = store::insert_source_context(
        &pool,
        "google_drive",
        &serde_json::json!({
            "folder_id": "folder_broken",
            "connection_id": conn_id,
        })
        .to_string(),
    )
    .await
    .unwrap();

    // Simulate what poll_remote_sources does on ConnectionBroken.
    let reason = "token revoked: Token has been revoked";
    store::update_source_status(&pool, src_id, "error", Some(reason))
        .await
        .unwrap();
    store::update_connection_status(&pool, conn_id, "expired")
        .await
        .unwrap();

    // Verify both statuses were updated.
    let ctx = store::get_source_context(&pool, src_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ctx.status, "error");
    assert_eq!(ctx.error_message.as_deref(), Some(reason));

    let conn = store::get_connection(&pool, conn_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(conn.status, "expired");
}

#[tokio::test]
async fn watchtower_preserves_cursor_across_restart() {
    let pool = init_test_db().await.expect("init db");

    // Register a source and set a cursor.
    let src_id = store::ensure_google_drive_source(
        &pool,
        "folder_cursor",
        r#"{"folder_id":"folder_cursor"}"#,
    )
    .await
    .unwrap();

    let cursor = "2026-02-28T15:30:00Z";
    store::update_sync_cursor(&pool, src_id, cursor)
        .await
        .unwrap();

    // Simulate restart by reading the source context (which the
    // Watchtower does at the start of poll_remote_sources).
    let ctx = store::get_source_context(&pool, src_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ctx.sync_cursor.as_deref(), Some(cursor));
}

#[tokio::test]
async fn watchtower_mixed_legacy_and_connection_sources() {
    let pool = init_test_db().await.expect("init db");
    let dir = tempfile::tempdir().unwrap();

    // Create config with both a local source and a Google Drive source
    // (the Drive source has no auth, so it will be skipped, but the
    // local source should still work).
    let config = ContentSourcesConfig {
        sources: vec![
            crate::config::ContentSourceEntry {
                source_type: "local_fs".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                folder_id: None,
                service_account_key: None,
                connection_id: None,
                watch: true,
                file_patterns: vec!["*.md".to_string()],
                loop_back_enabled: false,
                poll_interval_seconds: None,
                enabled: None,
                change_detection: "auto".to_string(),
            },
            crate::config::ContentSourceEntry {
                source_type: "google_drive".to_string(),
                path: None,
                folder_id: Some("folder_mixed".to_string()),
                service_account_key: None,
                connection_id: None, // No auth = skipped
                watch: true,
                file_patterns: vec!["*.md".to_string()],
                loop_back_enabled: false,
                poll_interval_seconds: Some(300),
                enabled: None,
                change_detection: "auto".to_string(),
            },
        ],
    };

    // Write a test file for the local source.
    std::fs::write(dir.path().join("note.md"), "Test content.\n").unwrap();

    let watchtower = WatchtowerLoop::new(
        pool.clone(),
        config,
        Default::default(),
        std::env::temp_dir(),
    );
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        watchtower.run(cancel_clone).await;
    });

    // Give it time to do the initial scan.
    tokio::time::sleep(Duration::from_millis(500)).await;
    cancel.cancel();

    let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
    assert!(result.is_ok(), "Mixed source config should not crash");

    // Verify the local source was ingested.
    let contexts = store::get_source_contexts(&pool).await.unwrap();
    assert!(
        !contexts.is_empty(),
        "At least the local source should be registered"
    );
}

// ---------------------------------------------------------------------------
// Fragment extraction (pure function tests)
// ---------------------------------------------------------------------------

#[test]
fn extract_fragments_with_headings() {
    let body = "\
Some intro text.

## Market Analysis

The market is shifting.

### Competitor Landscape

Our main competitors are...

## Product Roadmap

Next quarter plans.
";
    let frags = extract_fragments(body);
    assert_eq!(frags.len(), 4);

    assert_eq!(frags[0].heading_path, "");
    assert!(frags[0].text.contains("Some intro text"));
    assert_eq!(frags[0].index, 0);

    assert_eq!(frags[1].heading_path, "## Market Analysis");
    assert!(frags[1].text.contains("market is shifting"));
    assert_eq!(frags[1].index, 1);

    assert_eq!(
        frags[2].heading_path,
        "## Market Analysis/### Competitor Landscape"
    );
    assert!(frags[2].text.contains("main competitors"));
    assert_eq!(frags[2].index, 2);

    assert_eq!(frags[3].heading_path, "## Product Roadmap");
    assert!(frags[3].text.contains("Next quarter"));
    assert_eq!(frags[3].index, 3);
}

#[test]
fn extract_fragments_no_headings() {
    let body = "Just plain text\nwith multiple lines.\n";
    let frags = extract_fragments(body);
    assert_eq!(frags.len(), 1);
    assert_eq!(frags[0].heading_path, "");
    assert!(frags[0].text.contains("Just plain text"));
}

#[test]
fn extract_fragments_nested_headings_with_reset() {
    let body = "\
# Title

Intro.

## Section A

Content A.

### Subsection

Deep content.

## Section B

Back to level 2.
";
    let frags = extract_fragments(body);
    assert_eq!(frags.len(), 4);

    assert_eq!(frags[0].heading_path, "# Title");
    assert!(frags[0].text.contains("Intro"));
    assert_eq!(frags[1].heading_path, "# Title/## Section A");
    assert_eq!(frags[2].heading_path, "# Title/## Section A/### Subsection");
    assert_eq!(frags[3].heading_path, "# Title/## Section B");
}

#[test]
fn extract_fragments_empty_body() {
    let frags = extract_fragments("");
    assert!(frags.is_empty());

    let frags = extract_fragments("   \n\n  \n");
    assert!(frags.is_empty());
}

#[test]
fn extract_fragments_consecutive_headings_no_body() {
    let body = "## First\n## Second\nSome content.\n";
    let frags = extract_fragments(body);
    // First heading has no body text → skipped.
    assert_eq!(frags.len(), 1);
    assert_eq!(frags[0].heading_path, "## Second");
    assert!(frags[0].text.contains("Some content"));
}

#[test]
fn extract_fragments_preserves_content() {
    let body = "## Code Example\n\n```rust\nfn main() {\n    # This is not a heading\n    println!(\"hello\");\n}\n```\n\nEnd of section.\n";
    let frags = extract_fragments(body);
    assert_eq!(frags.len(), 1);
    // Code block content including the `# ` line should be preserved.
    assert!(frags[0].text.contains("# This is not a heading"));
    assert!(frags[0].text.contains("```rust"));
}

#[test]
fn extract_fragments_heading_inside_code_block_ignored() {
    let body =
        "Intro text.\n\n```\n# Not a heading\n## Also not\n```\n\n## Real Heading\n\nContent.\n";
    let frags = extract_fragments(body);
    assert_eq!(frags.len(), 2);
    assert_eq!(frags[0].heading_path, "");
    assert!(frags[0].text.contains("# Not a heading"));
    assert_eq!(frags[1].heading_path, "## Real Heading");
}

// ---------------------------------------------------------------------------
// chunk_node DB integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn chunk_node_creates_chunks() {
    let pool = init_test_db().await.expect("init db");

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    let body = "## Intro\n\nHello world.\n\n## Details\n\nMore info.\n";
    let content = format!("---\ntitle: Test\n---\n{body}");

    ingest_content(&pool, source_id, "test.md", &content, false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, source_id, Some("pending"))
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];

    let ids = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);

    let chunks = store::get_chunks_for_node(&pool, &node.account_id, node.id)
        .await
        .unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].heading_path, "## Intro");
    assert_eq!(chunks[1].heading_path, "## Details");

    // Verify node status transitioned to chunked.
    let updated_nodes = store::get_nodes_for_source(&pool, source_id, Some("chunked"))
        .await
        .unwrap();
    assert_eq!(updated_nodes.len(), 1);
}

#[tokio::test]
async fn chunk_node_stale_on_update_preserves_unchanged() {
    let pool = init_test_db().await.expect("init db");

    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    // Initial ingest and chunk.
    let content_v1 =
        "---\ntitle: V1\n---\n## Intro\n\nOriginal intro.\n\n## Details\n\nOriginal details.\n";
    ingest_content(&pool, source_id, "note.md", content_v1, false)
        .await
        .unwrap();

    let nodes = store::get_nodes_for_source(&pool, source_id, Some("pending"))
        .await
        .unwrap();
    let node = &nodes[0];

    let ids_v1 = chunker::chunk_node(&pool, &node.account_id, node.id, &node.body_text)
        .await
        .unwrap();
    assert_eq!(ids_v1.len(), 2);

    // Update content — change one section, keep the other.
    let content_v2 =
        "---\ntitle: V2\n---\n## Intro\n\nOriginal intro.\n\n## Details\n\nUpdated details.\n";
    ingest_content(&pool, source_id, "note.md", content_v2, false)
        .await
        .unwrap();

    let nodes_v2 = store::get_nodes_for_source(&pool, source_id, Some("pending"))
        .await
        .unwrap();
    let node_v2 = &nodes_v2[0];

    let ids_v2 = chunker::chunk_node(&pool, &node_v2.account_id, node_v2.id, &node_v2.body_text)
        .await
        .unwrap();
    assert_eq!(ids_v2.len(), 2);

    // The "Intro" chunk should be preserved (same ID) since content didn't change.
    assert_eq!(ids_v1[0], ids_v2[0], "Unchanged chunk should keep same ID");
    // The "Details" chunk should be new (different ID) since content changed.
    assert_ne!(ids_v1[1], ids_v2[1], "Changed chunk should get new ID");

    // Active chunks should only be the v2 ones.
    let active = store::get_chunks_for_node(&pool, &node_v2.account_id, node_v2.id)
        .await
        .unwrap();
    assert_eq!(active.len(), 2);
}

// ---------------------------------------------------------------------------
// Pure function coverage
// ---------------------------------------------------------------------------

#[test]
fn relative_path_string_simple() {
    let p = Path::new("subdir/file.md");
    assert_eq!(relative_path_string(p), "subdir/file.md");
}

#[test]
fn relative_path_string_single_component() {
    let p = Path::new("file.txt");
    assert_eq!(relative_path_string(p), "file.txt");
}

#[test]
fn relative_path_string_deep_nesting() {
    let p = Path::new("a/b/c/d.md");
    assert_eq!(relative_path_string(p), "a/b/c/d.md");
}

#[test]
fn ingest_summary_default() {
    let s = IngestSummary::default();
    assert_eq!(s.ingested, 0);
    assert_eq!(s.skipped, 0);
    assert!(s.errors.is_empty());
}

#[test]
fn parsed_front_matter_default() {
    let fm = ParsedFrontMatter::default();
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_none());
}

#[test]
fn matches_patterns_with_star_glob() {
    let patterns = vec!["*.rs".to_string()];
    assert!(matches_patterns(Path::new("lib.rs"), &patterns));
    assert!(!matches_patterns(Path::new("lib.py"), &patterns));
}

#[test]
fn matches_patterns_no_filename() {
    // A path that is just a directory won't have a file_name the same way
    let patterns = vec!["*.md".to_string()];
    // Path::new(".") doesn't match *.md
    assert!(!matches_patterns(Path::new("."), &patterns));
}

#[test]
fn parse_front_matter_invalid_yaml() {
    let content = "---\n: invalid: yaml: [[\n---\nBody.\n";
    let (fm, body) = parse_front_matter(content);
    // Invalid YAML: title/tags should be None, but raw_yaml should be present
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_some());
    assert_eq!(body, "Body.\n");
}

#[test]
fn parse_front_matter_yaml_not_mapping() {
    // YAML that parses but is not a mapping (e.g., a scalar)
    let content = "---\njust a string\n---\nBody text.\n";
    let (fm, body) = parse_front_matter(content);
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_some());
    assert_eq!(body, "Body text.\n");
}

#[test]
fn watchtower_error_display() {
    let e = WatchtowerError::Config("bad path".to_string());
    assert_eq!(e.to_string(), "config error: bad path");
}

#[test]
fn ingest_summary_debug() {
    let s = IngestSummary {
        ingested: 3,
        skipped: 1,
        errors: vec!["fail".to_string()],
    };
    let debug = format!("{s:?}");
    assert!(debug.contains("3"));
    assert!(debug.contains("fail"));
}

// ---------------------------------------------------------------------------
// Additional coverage: walk_directory edge cases
// ---------------------------------------------------------------------------

#[test]
fn walk_directory_multiple_patterns() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.md"), "md").unwrap();
    std::fs::write(dir.path().join("readme.txt"), "txt").unwrap();
    std::fs::write(dir.path().join("image.png"), "png").unwrap();

    let patterns = vec!["*.md".to_string(), "*.txt".to_string()];
    let mut paths = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &patterns, &mut paths).unwrap();

    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"note.md".to_string()));
    assert!(paths.contains(&"readme.txt".to_string()));
}

#[test]
fn walk_directory_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let patterns = vec!["*.md".to_string()];
    let mut paths = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &patterns, &mut paths).unwrap();
    assert!(paths.is_empty());
}

#[test]
fn walk_directory_deeply_nested() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("deep.md"), "deep").unwrap();

    let patterns = vec!["*.md".to_string()];
    let mut paths = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &patterns, &mut paths).unwrap();

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], "a/b/c/deep.md");
}

#[test]
fn walk_directory_nonexistent_returns_error() {
    let patterns = vec!["*.md".to_string()];
    let mut paths = Vec::new();
    let result = WatchtowerLoop::walk_directory(
        Path::new("/nonexistent_watchtower_dir_xyz"),
        Path::new("/nonexistent_watchtower_dir_xyz"),
        &patterns,
        &mut paths,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Additional coverage: CooldownSet edge cases
// ---------------------------------------------------------------------------

#[test]
fn cooldown_multiple_paths() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let p1 = PathBuf::from(std::env::temp_dir().join("cooldown_a.md"));
    let p2 = PathBuf::from(std::env::temp_dir().join("cooldown_b.md"));

    cd.mark(p1.clone());
    cd.mark(p2.clone());

    assert!(cd.is_cooling(&p1));
    assert!(cd.is_cooling(&p2));
    assert!(!cd.is_cooling(&std::env::temp_dir().join("cooldown_c.md")));
}

#[test]
fn cooldown_cleanup_retains_fresh_entries() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let p = PathBuf::from(std::env::temp_dir().join("fresh.md"));
    cd.mark(p.clone());
    cd.cleanup();
    // With 60s TTL, entry should still be present after cleanup
    assert!(cd.is_cooling(&p));
    assert_eq!(cd.entries.len(), 1);
}

// ---------------------------------------------------------------------------
// Additional coverage: matches_patterns edge cases
// ---------------------------------------------------------------------------

#[test]
fn matches_patterns_multiple_extensions() {
    let patterns = vec!["*.md".to_string(), "*.txt".to_string(), "*.rst".to_string()];
    assert!(matches_patterns(Path::new("doc.rst"), &patterns));
    assert!(!matches_patterns(Path::new("doc.html"), &patterns));
}

#[test]
fn matches_patterns_invalid_glob() {
    // An invalid glob pattern should be silently skipped
    let patterns = vec!["[invalid".to_string(), "*.md".to_string()];
    assert!(matches_patterns(Path::new("note.md"), &patterns));
}

#[test]
fn matches_patterns_exact_filename() {
    let patterns = vec!["README.md".to_string()];
    assert!(matches_patterns(Path::new("README.md"), &patterns));
    assert!(!matches_patterns(Path::new("readme.md"), &patterns));
}

// ---------------------------------------------------------------------------
// Additional coverage: relative_path_string edge cases
// ---------------------------------------------------------------------------

#[test]
fn relative_path_string_empty() {
    let p = Path::new("");
    // Empty path should produce empty string
    assert_eq!(relative_path_string(p), "");
}

// ---------------------------------------------------------------------------
// Additional coverage: parse_front_matter edge cases
// ---------------------------------------------------------------------------

#[test]
fn parse_front_matter_empty_tags_sequence() {
    let content = "---\ntitle: Test\ntags: []\n---\nBody.\n";
    let (fm, _body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("Test"));
    // Empty sequence should produce no tags
    assert!(fm.tags.is_none());
}

#[test]
fn parse_front_matter_non_string_tag_values() {
    let content = "---\ntags:\n  - 42\n  - true\n---\nBody.\n";
    let (fm, _body) = parse_front_matter(content);
    // Non-string values in tags sequence are filtered out by filter_map(as_str)
    assert!(fm.tags.is_none());
}

#[test]
fn parse_front_matter_extra_fields_ignored() {
    let content = "---\ntitle: Note\nauthor: Alice\ndate: 2026-01-01\n---\nBody.\n";
    let (fm, body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("Note"));
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_some());
    assert_eq!(body, "Body.\n");
}

// ---------------------------------------------------------------------------
// Additional coverage: WatchtowerError variants
// ---------------------------------------------------------------------------

#[test]
fn watchtower_error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
    let e = WatchtowerError::from(io_err);
    assert!(e.to_string().contains("IO error"));
}

#[test]
fn watchtower_error_debug() {
    let e = WatchtowerError::Config("test config error".to_string());
    let debug = format!("{e:?}");
    assert!(debug.contains("Config"));
    assert!(debug.contains("test config error"));
}

// ---------------------------------------------------------------------------
// Additional coverage: ingest_content hash behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ingest_content_with_front_matter_stores_metadata() {
    let pool = init_test_db().await.expect("init db");
    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    let content = "---\ntitle: Test Title\ntags:\n  - alpha\n  - beta\n---\nContent body here.\n";
    let result = ingest_content(&pool, source_id, "test.md", content, false)
        .await
        .unwrap();
    assert_eq!(result, store::UpsertResult::Inserted);

    let nodes = store::get_nodes_for_source(&pool, source_id, None)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].title.as_deref(), Some("Test Title"));
    assert_eq!(nodes[0].tags.as_deref(), Some("alpha,beta"));
}

#[tokio::test]
async fn ingest_content_force_always_updates() {
    let pool = init_test_db().await.expect("init db");
    let source_id = store::insert_source_context(&pool, "local_fs", "{}")
        .await
        .unwrap();

    let content = "Same content.\n";
    let r1 = ingest_content(&pool, source_id, "forced.md", content, false)
        .await
        .unwrap();
    assert_eq!(r1, store::UpsertResult::Inserted);

    // Force flag should cause update even with same content
    let r2 = ingest_content(&pool, source_id, "forced.md", content, true)
        .await
        .unwrap();
    assert_eq!(r2, store::UpsertResult::Updated);
}
