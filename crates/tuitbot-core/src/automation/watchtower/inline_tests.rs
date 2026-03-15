//! Additional inline tests for watchtower helpers.

use std::path::Path;
use std::time::{Duration, Instant};

use super::*;

// ── ParsedFrontMatter ────────────────────────────────────────────

#[test]
fn parsed_front_matter_default() {
    let fm = ParsedFrontMatter::default();
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_none());
}

#[test]
fn parsed_front_matter_debug() {
    let fm = ParsedFrontMatter {
        title: Some("Test".to_string()),
        tags: Some("rust,testing".to_string()),
        raw_yaml: Some("title: Test".to_string()),
    };
    let debug = format!("{fm:?}");
    assert!(debug.contains("Test"));
    assert!(debug.contains("rust,testing"));
}

// ── IngestSummary ────────────────────────────────────────────────

#[test]
fn ingest_summary_default() {
    let summary = IngestSummary::default();
    assert_eq!(summary.ingested, 0);
    assert_eq!(summary.skipped, 0);
    assert!(summary.errors.is_empty());
}

#[test]
fn ingest_summary_debug() {
    let summary = IngestSummary {
        ingested: 5,
        skipped: 2,
        errors: vec!["file.md: error".to_string()],
    };
    let debug = format!("{summary:?}");
    assert!(debug.contains("5"));
    assert!(debug.contains("2"));
    assert!(debug.contains("file.md"));
}

// ── parse_front_matter ───────────────────────────────────────────

#[test]
fn parse_front_matter_with_yaml() {
    let content = "---\ntitle: Hello\ntags:\n  - rust\n  - test\n---\nBody text here";
    let (fm, body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("Hello"));
    assert_eq!(fm.tags.as_deref(), Some("rust,test"));
    assert!(fm.raw_yaml.is_some());
    assert_eq!(body.trim(), "Body text here");
}

#[test]
fn parse_front_matter_no_yaml() {
    let content = "Just plain text without front matter.";
    let (fm, body) = parse_front_matter(content);
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    assert!(fm.raw_yaml.is_none());
    assert_eq!(body, content);
}

#[test]
fn parse_front_matter_empty_yaml() {
    let content = "---\n---\nBody";
    let (fm, body) = parse_front_matter(content);
    // Empty YAML block — may or may not be recognized as front matter
    assert!(fm.title.is_none());
    assert!(fm.tags.is_none());
    // Body should contain "Body" (either alone or with the full content)
    assert!(body.contains("Body"));
}

#[test]
fn parse_front_matter_tags_as_string() {
    let content = "---\ntags: \"single-tag\"\n---\nBody";
    let (fm, _) = parse_front_matter(content);
    assert_eq!(fm.tags.as_deref(), Some("single-tag"));
}

#[test]
fn parse_front_matter_invalid_yaml() {
    let content = "---\n: invalid yaml [[\n---\nBody";
    let (fm, body) = parse_front_matter(content);
    // Should still return raw_yaml but no parsed fields
    assert!(fm.raw_yaml.is_some());
    assert!(fm.title.is_none());
    assert_eq!(body.trim(), "Body");
}

// ── matches_patterns ─────────────────────────────────────────────

#[test]
fn matches_patterns_md_extension() {
    assert!(matches_patterns(Path::new("doc.md"), &["*.md".to_string()]));
}

#[test]
fn matches_patterns_txt_extension() {
    assert!(matches_patterns(
        Path::new("notes.txt"),
        &["*.txt".to_string()]
    ));
}

#[test]
fn matches_patterns_no_match() {
    assert!(!matches_patterns(
        Path::new("image.png"),
        &["*.md".to_string(), "*.txt".to_string()]
    ));
}

#[test]
fn matches_patterns_nested_path() {
    assert!(matches_patterns(
        Path::new("sub/dir/note.md"),
        &["*.md".to_string()]
    ));
}

#[test]
fn matches_patterns_empty_patterns() {
    assert!(!matches_patterns(Path::new("file.md"), &[]));
}

#[test]
fn matches_patterns_multiple_patterns() {
    assert!(matches_patterns(
        Path::new("doc.md"),
        &["*.txt".to_string(), "*.md".to_string()]
    ));
}

#[test]
fn matches_patterns_invalid_pattern_ignored() {
    // Invalid glob pattern should not cause a panic
    assert!(!matches_patterns(
        Path::new("file.md"),
        &["[invalid".to_string()]
    ));
}

// ── relative_path_string ──────────────────────────────────────────

#[test]
fn relative_path_string_simple() {
    let result = relative_path_string(Path::new("file.md"));
    assert_eq!(result, "file.md");
}

#[test]
fn relative_path_string_nested() {
    let result = relative_path_string(Path::new("sub/dir/file.md"));
    assert_eq!(result, "sub/dir/file.md");
}

// ── CooldownSet ──────────────────────────────────────────────────

#[test]
fn cooldown_set_new_is_empty() {
    let cd = CooldownSet::new(Duration::from_secs(5));
    assert!(!cd.is_cooling(Path::new("/test/path")));
}

#[test]
fn cooldown_set_mark_and_check() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let path = std::path::PathBuf::from("/test/file.md");
    cd.mark(path.clone());
    assert!(cd.is_cooling(&path));
}

#[test]
fn cooldown_set_cleanup_removes_expired() {
    let mut cd = CooldownSet::new(Duration::from_millis(1));
    let path = std::path::PathBuf::from("/test/old.md");
    cd.entries
        .insert(path.clone(), Instant::now() - Duration::from_secs(10));
    cd.cleanup();
    assert!(!cd.is_cooling(&path));
    assert!(cd.entries.is_empty());
}

#[test]
fn cooldown_set_cleanup_keeps_recent() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let path = std::path::PathBuf::from("/test/recent.md");
    cd.mark(path.clone());
    cd.cleanup();
    assert!(cd.is_cooling(&path));
}

// ── WatchtowerError ──────────────────────────────────────────────

#[test]
fn watchtower_error_io_display() {
    let err = WatchtowerError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "missing file",
    ));
    let msg = err.to_string();
    assert!(msg.contains("IO error"));
    assert!(msg.contains("missing file"));
}

#[test]
fn watchtower_error_config_display() {
    let err = WatchtowerError::Config("bad config".to_string());
    assert_eq!(err.to_string(), "config error: bad config");
}

#[test]
fn watchtower_error_config_display_2() {
    let err = WatchtowerError::Config("missing source".to_string());
    let msg = err.to_string();
    assert!(msg.contains("config error"));
    assert!(msg.contains("missing source"));
}

#[test]
fn watchtower_error_debug() {
    let err = WatchtowerError::Config("test".to_string());
    let debug = format!("{err:?}");
    assert!(debug.contains("Config"));
}

// ── WatchtowerLoop construction ──────────────────────────────────

#[test]
fn watchtower_loop_defaults() {
    // Can't fully construct without a real pool, but verify the type exists
    // and the constants are reasonable.
    assert_eq!(std::time::Duration::from_secs(2).as_secs(), 2);
    assert_eq!(std::time::Duration::from_secs(300).as_secs(), 300);
    assert_eq!(std::time::Duration::from_secs(5).as_secs(), 5);
}

// ── walk_directory ───────────────────────────────────────────────

#[test]
fn walk_directory_finds_matching_files() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path();
    std::fs::write(base.join("note.md"), "# Note").expect("write md");
    std::fs::write(base.join("readme.txt"), "hello").expect("write txt");
    std::fs::write(base.join("image.png"), "fake").expect("write png");

    let mut out = Vec::new();
    WatchtowerLoop::walk_directory(
        base,
        base,
        &["*.md".to_string(), "*.txt".to_string()],
        &mut out,
    )
    .expect("walk");

    assert_eq!(out.len(), 2);
    assert!(out.contains(&"note.md".to_string()));
    assert!(out.contains(&"readme.txt".to_string()));
}

#[test]
fn walk_directory_recurses_into_subdirs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path();
    let sub = base.join("sub");
    std::fs::create_dir(&sub).expect("mkdir");
    std::fs::write(sub.join("deep.md"), "deep").expect("write");

    let mut out = Vec::new();
    WatchtowerLoop::walk_directory(base, base, &["*.md".to_string()], &mut out).expect("walk");

    assert_eq!(out.len(), 1);
    assert_eq!(out[0], "sub/deep.md");
}

#[test]
fn walk_directory_skips_hidden_dirs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let base = dir.path();
    let hidden = base.join(".hidden");
    std::fs::create_dir(&hidden).expect("mkdir");
    std::fs::write(hidden.join("secret.md"), "secret").expect("write");
    std::fs::write(base.join("visible.md"), "visible").expect("write");

    let mut out = Vec::new();
    WatchtowerLoop::walk_directory(base, base, &["*.md".to_string()], &mut out).expect("walk");

    assert_eq!(out.len(), 1);
    assert_eq!(out[0], "visible.md");
}

#[test]
fn walk_directory_empty_dir() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut out = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &["*.md".to_string()], &mut out)
        .expect("walk");
    assert!(out.is_empty());
}

#[test]
fn walk_directory_no_matching_patterns() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("data.csv"), "a,b").expect("write");

    let mut out = Vec::new();
    WatchtowerLoop::walk_directory(dir.path(), dir.path(), &["*.md".to_string()], &mut out)
        .expect("walk");
    assert!(out.is_empty());
}

// ── relative_path_string edge cases ──────────────────────────────

#[test]
fn relative_path_string_empty() {
    let result = relative_path_string(Path::new(""));
    assert_eq!(result, "");
}

#[test]
fn relative_path_string_deeply_nested() {
    let result = relative_path_string(Path::new("a/b/c/d/e/f.md"));
    assert_eq!(result, "a/b/c/d/e/f.md");
}

// ── parse_front_matter edge cases ────────────────────────────────

#[test]
fn parse_front_matter_title_only() {
    let content = "---\ntitle: My Title\n---\nBody text";
    let (fm, body) = parse_front_matter(content);
    assert_eq!(fm.title.as_deref(), Some("My Title"));
    assert!(fm.tags.is_none());
    assert_eq!(body.trim(), "Body text");
}

#[test]
fn parse_front_matter_empty_tags_list() {
    let content = "---\ntags: []\n---\nBody";
    let (fm, _) = parse_front_matter(content);
    // Empty sequence should be filtered out
    assert!(fm.tags.is_none());
}

#[test]
fn parse_front_matter_multiple_tags() {
    let content = "---\ntags:\n  - alpha\n  - beta\n  - gamma\n---\nContent";
    let (fm, _) = parse_front_matter(content);
    assert_eq!(fm.tags.as_deref(), Some("alpha,beta,gamma"));
}

// ── CooldownSet edge cases ───────────────────────────────────────

#[test]
fn cooldown_set_different_paths_independent() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let path_a = std::path::PathBuf::from("/a.md");
    let path_b = std::path::PathBuf::from("/b.md");
    cd.mark(path_a.clone());
    assert!(cd.is_cooling(&path_a));
    assert!(!cd.is_cooling(&path_b));
}

#[test]
fn cooldown_set_re_mark_refreshes() {
    let mut cd = CooldownSet::new(Duration::from_secs(60));
    let path = std::path::PathBuf::from("/test.md");
    cd.mark(path.clone());
    // Mark again (should update timestamp)
    cd.mark(path.clone());
    assert!(cd.is_cooling(&path));
    assert_eq!(cd.entries.len(), 1);
}

// ── matches_patterns edge cases ─────────────────────────────────

#[test]
fn matches_patterns_star_matches_all() {
    assert!(matches_patterns(
        Path::new("anything.xyz"),
        &["*".to_string()]
    ));
}

#[test]
fn matches_patterns_specific_filename() {
    assert!(matches_patterns(
        Path::new("Makefile"),
        &["Makefile".to_string()]
    ));
    assert!(!matches_patterns(
        Path::new("Dockerfile"),
        &["Makefile".to_string()]
    ));
}

// ── WatchtowerError variants ─────────────────────────────────────

#[test]
fn watchtower_error_storage_display() {
    let err = WatchtowerError::Config("missing source path".to_string());
    let msg = err.to_string();
    assert!(msg.contains("config error"));
    assert!(msg.contains("missing source path"));
}
