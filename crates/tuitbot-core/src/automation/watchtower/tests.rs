//! Tests for the Watchtower filesystem watcher and ingest pipeline.

use std::path::PathBuf;

use super::*;
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

    let watchtower = WatchtowerLoop::new(pool, config);
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
            watch: true,
            file_patterns: vec!["*.md".to_string()],
            loop_back_enabled: false,
        }],
    };

    let watchtower = WatchtowerLoop::new(pool, config);
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
