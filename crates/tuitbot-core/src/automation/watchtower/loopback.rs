//! Loop-back metadata writing for Watchtower.
//!
//! When content from a source file is published (e.g. as a tweet),
//! this module writes the published metadata back into the originating
//! note's YAML front-matter in an idempotent, parseable format.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Metadata about a published piece of content, written back to the source file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoopBackEntry {
    pub tweet_id: String,
    pub url: String,
    pub published_at: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

/// Parsed YAML front-matter with a `tuitbot` key.
#[derive(Debug, Default, Serialize, Deserialize)]
struct TuitbotFrontMatter {
    #[serde(default)]
    tuitbot: Vec<LoopBackEntry>,
    #[serde(flatten)]
    other: serde_yaml::Mapping,
}

/// Split a file's content into optional YAML front-matter and body.
///
/// Front-matter is delimited by `---` on its own line at the very start.
/// Returns `(Some(yaml_str), body)` if present, or `(None, full_content)`.
pub fn split_front_matter(content: &str) -> (Option<&str>, &str) {
    if !content.starts_with("---") {
        return (None, content);
    }

    // Find the closing `---` after the opening one.
    let after_open = &content[3..];
    // Skip the newline after opening ---
    let after_open = after_open
        .strip_prefix('\n')
        .unwrap_or(after_open.strip_prefix("\r\n").unwrap_or(after_open));

    if let Some(close_pos) = after_open.find("\n---") {
        let yaml = &after_open[..close_pos];
        let rest_start = close_pos + 4; // "\n---".len()
        let body = &after_open[rest_start..];
        // Strip the newline immediately after the closing ---
        let body = body
            .strip_prefix('\n')
            .unwrap_or(body.strip_prefix("\r\n").unwrap_or(body));
        (Some(yaml), body)
    } else {
        // No closing delimiter — treat entire content as body.
        (None, content)
    }
}

/// Parse existing tuitbot loop-back entries from file content.
pub fn parse_tuitbot_metadata(content: &str) -> Vec<LoopBackEntry> {
    let (yaml_str, _) = split_front_matter(content);
    let yaml_str = match yaml_str {
        Some(y) => y,
        None => return Vec::new(),
    };

    match serde_yaml::from_str::<TuitbotFrontMatter>(yaml_str) {
        Ok(fm) => fm.tuitbot,
        Err(_) => Vec::new(),
    }
}

/// Write published metadata back to a source file, idempotently.
///
/// If the `tweet_id` already exists in the file's `tuitbot` front-matter
/// array, the write is skipped. Otherwise the entry is appended.
///
/// Returns `true` if the file was modified, `false` if skipped.
pub fn write_metadata_to_file(path: &Path, entry: &LoopBackEntry) -> Result<bool, io::Error> {
    let content = std::fs::read_to_string(path)?;

    // Check if this tweet_id already exists.
    let existing = parse_tuitbot_metadata(&content);
    if existing.iter().any(|e| e.tweet_id == entry.tweet_id) {
        return Ok(false);
    }

    let (yaml_str, body) = split_front_matter(&content);

    // Parse or create front-matter.
    let mut fm: TuitbotFrontMatter = match yaml_str {
        Some(y) => serde_yaml::from_str(y).unwrap_or_default(),
        None => TuitbotFrontMatter::default(),
    };

    fm.tuitbot.push(entry.clone());

    // Serialize the front-matter.
    let yaml_out = serde_yaml::to_string(&fm).map_err(io::Error::other)?;

    // Reconstruct the file: --- + yaml + --- + body.
    let mut output = String::with_capacity(yaml_out.len() + body.len() + 10);
    output.push_str("---\n");
    output.push_str(&yaml_out);
    // serde_yaml already adds trailing newline, but ensure --- is on its own line.
    if !yaml_out.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("---\n");
    output.push_str(body);

    std::fs::write(path, output)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_entry() -> LoopBackEntry {
        LoopBackEntry {
            tweet_id: "1234567890".to_string(),
            url: "https://x.com/user/status/1234567890".to_string(),
            published_at: "2026-02-28T14:30:00Z".to_string(),
            content_type: "tweet".to_string(),
        }
    }

    #[test]
    fn split_no_front_matter() {
        let content = "Just a plain note.\n";
        let (yaml, body) = split_front_matter(content);
        assert!(yaml.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn split_with_front_matter() {
        let content = "---\ntitle: Hello\n---\nBody text here.\n";
        let (yaml, body) = split_front_matter(content);
        assert_eq!(yaml.unwrap(), "title: Hello");
        assert_eq!(body, "Body text here.\n");
    }

    #[test]
    fn split_no_closing_delimiter() {
        let content = "---\ntitle: Hello\nNo closing.\n";
        let (yaml, body) = split_front_matter(content);
        assert!(yaml.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn parse_tuitbot_entries() {
        let content = "---\ntuitbot:\n  - tweet_id: \"123\"\n    url: \"https://x.com/u/status/123\"\n    published_at: \"2026-01-01T00:00:00Z\"\n    type: tweet\n---\nBody.\n";
        let entries = parse_tuitbot_metadata(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tweet_id, "123");
    }

    #[test]
    fn parse_no_tuitbot_key() {
        let content = "---\ntitle: Hello\n---\nBody.\n";
        let entries = parse_tuitbot_metadata(content);
        assert!(entries.is_empty());
    }

    #[test]
    fn loopback_write_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "This is my note.\n").unwrap();

        let entry = sample_entry();
        let modified = write_metadata_to_file(&path, &entry).unwrap();
        assert!(modified);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("tweet_id"));
        assert!(content.contains("1234567890"));
        assert!(content.contains("This is my note."));
    }

    #[test]
    fn loopback_write_existing_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "---\ntitle: My Note\n---\nBody here.\n").unwrap();

        let entry = sample_entry();
        let modified = write_metadata_to_file(&path, &entry).unwrap();
        assert!(modified);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("title"));
        assert!(content.contains("My Note"));
        assert!(content.contains("tweet_id"));
        assert!(content.contains("Body here."));
    }

    #[test]
    fn loopback_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "My note.\n").unwrap();

        let entry = sample_entry();
        let first = write_metadata_to_file(&path, &entry).unwrap();
        assert!(first);

        let second = write_metadata_to_file(&path, &entry).unwrap();
        assert!(!second);

        // Verify only one entry exists.
        let content = fs::read_to_string(&path).unwrap();
        let entries = parse_tuitbot_metadata(&content);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn loopback_multiple_tweets() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "My note.\n").unwrap();

        let entry_a = sample_entry();
        write_metadata_to_file(&path, &entry_a).unwrap();

        let entry_b = LoopBackEntry {
            tweet_id: "9876543210".to_string(),
            url: "https://x.com/user/status/9876543210".to_string(),
            published_at: "2026-03-01T10:00:00Z".to_string(),
            content_type: "thread".to_string(),
        };
        write_metadata_to_file(&path, &entry_b).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let entries = parse_tuitbot_metadata(&content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tweet_id, "1234567890");
        assert_eq!(entries[1].tweet_id, "9876543210");
    }
}
