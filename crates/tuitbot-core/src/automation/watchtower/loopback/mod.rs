//! Loop-back metadata writing for Watchtower.
//!
//! When content from a source file is published (e.g. as a tweet),
//! this module writes the published metadata back into the originating
//! note's YAML front-matter in an idempotent, parseable format.
//!
//! The Forge sync path (`sync` submodule) enriches existing entries
//! with analytics data without creating new entries.

pub mod sync;

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::storage::DbPool;

/// Metadata about a published piece of content, written back to the source file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoopBackEntry {
    pub tweet_id: String,
    pub url: String,
    pub published_at: String,
    #[serde(rename = "type")]
    pub content_type: String,
    /// Post status: "posted", "deleted", etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Thread URL when this entry is part of a thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_url: Option<String>,
    /// Tweet IDs of child tweets in a thread (excludes the root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_tweet_ids: Option<Vec<String>>,

    // Analytics fields (Forge sync)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impressions: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retweets: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replies: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engagement_rate: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<String>,
}

/// Result of an `execute_loopback()` call.
#[derive(Debug, PartialEq, Eq)]
pub enum LoopBackResult {
    /// Metadata was written to the source file.
    Written,
    /// The tweet_id was already present in the file — no write needed.
    AlreadyPresent,
    /// The source type does not support writes (e.g. google_drive, manual).
    SourceNotWritable(String),
    /// The content node was not found in the database.
    NodeNotFound,
    /// The source file does not exist on disk.
    FileNotFound,
}

/// Execute provenance-driven loop-back: look up the source note for a content
/// node and write publishing metadata into its YAML front-matter.
///
/// Returns `LoopBackResult` indicating the outcome. DB lookup failures are
/// logged and mapped to result variants rather than propagated as errors.
pub async fn execute_loopback(
    pool: &DbPool,
    node_id: i64,
    tweet_id: &str,
    url: &str,
    content_type: &str,
) -> LoopBackResult {
    use crate::storage::watchtower::{get_content_node, get_source_context};

    let node = match get_content_node(pool, node_id).await {
        Ok(Some(n)) => n,
        Ok(None) => return LoopBackResult::NodeNotFound,
        Err(e) => {
            tracing::warn!(node_id, error = %e, "Loopback: failed to get content node");
            return LoopBackResult::NodeNotFound;
        }
    };

    let source = match get_source_context(pool, node.source_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return LoopBackResult::SourceNotWritable("source not found".into()),
        Err(e) => {
            tracing::warn!(node_id, error = %e, "Loopback: failed to get source context");
            return LoopBackResult::SourceNotWritable("db error".into());
        }
    };

    if source.source_type != "local_fs" {
        return LoopBackResult::SourceNotWritable(source.source_type);
    }

    let base_path = match serde_json::from_str::<serde_json::Value>(&source.config_json)
        .ok()
        .and_then(|v| v.get("path")?.as_str().map(String::from))
    {
        Some(p) => p,
        None => return LoopBackResult::SourceNotWritable("no path in config".into()),
    };

    let expanded = crate::storage::expand_tilde(&base_path);
    let full_path = std::path::PathBuf::from(expanded).join(&node.relative_path);

    if !full_path.exists() {
        return LoopBackResult::FileNotFound;
    }

    let entry = LoopBackEntry {
        tweet_id: tweet_id.to_string(),
        url: url.to_string(),
        published_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        content_type: content_type.to_string(),
        status: Some("posted".to_string()),
        thread_url: None,
        child_tweet_ids: None,
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };

    match write_metadata_to_file(&full_path, &entry) {
        Ok(true) => LoopBackResult::Written,
        Ok(false) => LoopBackResult::AlreadyPresent,
        Err(e) => {
            tracing::warn!(
                node_id,
                path = %full_path.display(),
                error = %e,
                "Loopback file write failed"
            );
            LoopBackResult::FileNotFound
        }
    }
}

/// Execute provenance-driven loop-back for a thread.
///
/// Like `execute_loopback`, but builds a thread-typed entry with
/// `child_tweet_ids` and `thread_url` populated.
pub async fn execute_loopback_thread(
    pool: &DbPool,
    node_id: i64,
    root_tweet_id: &str,
    url: &str,
    child_tweet_ids: Vec<String>,
) -> LoopBackResult {
    use crate::storage::watchtower::{get_content_node, get_source_context};

    let node = match get_content_node(pool, node_id).await {
        Ok(Some(n)) => n,
        Ok(None) => return LoopBackResult::NodeNotFound,
        Err(e) => {
            tracing::warn!(node_id, error = %e, "Loopback thread: failed to get content node");
            return LoopBackResult::NodeNotFound;
        }
    };

    let source = match get_source_context(pool, node.source_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return LoopBackResult::SourceNotWritable("source not found".into()),
        Err(e) => {
            tracing::warn!(node_id, error = %e, "Loopback thread: failed to get source context");
            return LoopBackResult::SourceNotWritable("db error".into());
        }
    };

    if source.source_type != "local_fs" {
        return LoopBackResult::SourceNotWritable(source.source_type);
    }

    let base_path = match serde_json::from_str::<serde_json::Value>(&source.config_json)
        .ok()
        .and_then(|v| v.get("path")?.as_str().map(String::from))
    {
        Some(p) => p,
        None => return LoopBackResult::SourceNotWritable("no path in config".into()),
    };

    let expanded = crate::storage::expand_tilde(&base_path);
    let full_path = std::path::PathBuf::from(expanded).join(&node.relative_path);

    if !full_path.exists() {
        return LoopBackResult::FileNotFound;
    }

    let entry = LoopBackEntry {
        tweet_id: root_tweet_id.to_string(),
        url: url.to_string(),
        published_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        content_type: "thread".to_string(),
        status: Some("posted".to_string()),
        thread_url: Some(url.to_string()),
        child_tweet_ids: if child_tweet_ids.is_empty() {
            None
        } else {
            Some(child_tweet_ids)
        },
        impressions: None,
        likes: None,
        retweets: None,
        replies: None,
        engagement_rate: None,
        performance_score: None,
        synced_at: None,
    };

    match write_metadata_to_file(&full_path, &entry) {
        Ok(true) => LoopBackResult::Written,
        Ok(false) => LoopBackResult::AlreadyPresent,
        Err(e) => {
            tracing::warn!(
                node_id,
                path = %full_path.display(),
                error = %e,
                "Loopback thread file write failed"
            );
            LoopBackResult::FileNotFound
        }
    }
}

/// Parsed YAML front-matter with a `tuitbot` key.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TuitbotFrontMatter {
    #[serde(default)]
    pub tuitbot: Vec<LoopBackEntry>,
    #[serde(flatten)]
    pub other: serde_yaml::Mapping,
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

    serialize_frontmatter_to_file(path, &fm, body)
}

/// Serialize `TuitbotFrontMatter` and write it back to a file with the given body.
pub(crate) fn serialize_frontmatter_to_file(
    path: &Path,
    fm: &TuitbotFrontMatter,
    body: &str,
) -> Result<bool, io::Error> {
    let yaml_out = serde_yaml::to_string(fm).map_err(io::Error::other)?;

    let mut output = String::with_capacity(yaml_out.len() + body.len() + 10);
    output.push_str("---\n");
    output.push_str(&yaml_out);
    if !yaml_out.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("---\n");
    output.push_str(body);

    std::fs::write(path, output)?;
    Ok(true)
}
