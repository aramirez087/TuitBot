//! Watchtower content source watcher and shared ingest pipeline.
//!
//! Watches configured local directories for `.md` and `.txt` changes via
//! the `notify` crate with debouncing, and polls remote content sources
//! (e.g. Google Drive) on a configurable interval.  Both local filesystem
//! events and remote polls funnel through `ingest_content()`, ensuring
//! identical state transitions.

pub mod chunker;
pub mod coordinator;
pub mod loopback;
pub mod monitor;
pub mod responder;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline_tests;

use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::storage::watchtower as store;
use crate::storage::DbPool;

pub use monitor::{CooldownSet, RemoteSource};

use crate::config::{ConnectorConfig, ContentSourcesConfig};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors specific to Watchtower operations.
#[derive(Debug, thiserror::Error)]
pub enum WatchtowerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("storage error: {0}")]
    Storage(#[from] crate::error::StorageError),

    #[error("notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("chunker error: {0}")]
    Chunker(#[from] chunker::ChunkerError),
}

// ---------------------------------------------------------------------------
// Ingest result types
// ---------------------------------------------------------------------------

/// Summary of a batch ingest operation.
#[derive(Debug, Default)]
pub struct IngestSummary {
    pub ingested: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

/// Parsed front-matter from a markdown file.
#[derive(Debug, Default)]
pub struct ParsedFrontMatter {
    pub title: Option<String>,
    pub tags: Option<String>,
    pub raw_yaml: Option<String>,
}

// ---------------------------------------------------------------------------
// Front-matter parsing
// ---------------------------------------------------------------------------

/// Parse YAML front-matter from file content.
///
/// Returns extracted metadata and the body text (content after front-matter).
pub fn parse_front_matter(content: &str) -> (ParsedFrontMatter, &str) {
    let (yaml_str, body) = loopback::split_front_matter(content);

    let yaml_str = match yaml_str {
        Some(y) => y,
        None => return (ParsedFrontMatter::default(), content),
    };

    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(yaml_str);
    match parsed {
        Ok(serde_yaml::Value::Mapping(map)) => {
            let title = map
                .get(serde_yaml::Value::String("title".to_string()))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let tags = map
                .get(serde_yaml::Value::String("tags".to_string()))
                .map(|v| match v {
                    serde_yaml::Value::Sequence(seq) => seq
                        .iter()
                        .filter_map(|item| item.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => String::new(),
                })
                .filter(|s| !s.is_empty());

            let fm = ParsedFrontMatter {
                title,
                tags,
                raw_yaml: Some(yaml_str.to_string()),
            };
            (fm, body)
        }
        _ => (
            ParsedFrontMatter {
                raw_yaml: Some(yaml_str.to_string()),
                ..Default::default()
            },
            body,
        ),
    }
}

// ---------------------------------------------------------------------------
// Pattern matching
// ---------------------------------------------------------------------------

/// Check whether a file path matches any of the given glob patterns.
///
/// Matches against the file name only (not the full path), so `*.md`
/// matches `sub/dir/note.md`.
pub fn matches_patterns(path: &Path, patterns: &[String]) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    for pattern in patterns {
        if let Ok(p) = glob::Pattern::new(pattern) {
            if p.matches(file_name) {
                return true;
            }
        }
    }
    false
}

/// Convert a relative path into a stable slash-delimited string across platforms.
pub fn relative_path_string(path: &Path) -> String {
    path.iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

// ---------------------------------------------------------------------------
// Shared ingest pipeline
// ---------------------------------------------------------------------------

/// Ingest raw text content into the Watchtower pipeline.
///
/// This is the provider-agnostic code path that both local file reads and
/// remote content fetches funnel through. It parses front-matter, computes
/// a content hash, and upserts the content node in the database.
pub async fn ingest_content(
    pool: &DbPool,
    source_id: i64,
    provider_id: &str,
    content: &str,
    force: bool,
) -> Result<store::UpsertResult, WatchtowerError> {
    let (fm, body) = parse_front_matter(content);

    let hash = if force {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.update(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_le_bytes(),
        );
        format!("{:x}", hasher.finalize())
    } else {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let result = store::upsert_content_node(
        pool,
        source_id,
        provider_id,
        &hash,
        fm.title.as_deref(),
        body,
        fm.raw_yaml.as_deref(),
        fm.tags.as_deref(),
    )
    .await?;

    Ok(result)
}

/// Ingest a single file from the local filesystem into the Watchtower pipeline.
///
/// Convenience wrapper that reads the file then delegates to `ingest_content`.
pub async fn ingest_file(
    pool: &DbPool,
    source_id: i64,
    base_path: &Path,
    relative_path: &str,
    force: bool,
) -> Result<store::UpsertResult, WatchtowerError> {
    let full_path = base_path.join(relative_path);
    let content = tokio::fs::read_to_string(&full_path).await?;
    ingest_content(pool, source_id, relative_path, &content, force).await
}

/// Ingest multiple files, collecting results into a summary.
pub async fn ingest_files(
    pool: &DbPool,
    source_id: i64,
    base_path: &Path,
    paths: &[String],
    force: bool,
) -> IngestSummary {
    let mut summary = IngestSummary::default();

    for rel_path in paths {
        match ingest_file(pool, source_id, base_path, rel_path, force).await {
            Ok(store::UpsertResult::Inserted | store::UpsertResult::Updated) => {
                summary.ingested += 1;
            }
            Ok(store::UpsertResult::Skipped) => {
                summary.skipped += 1;
            }
            Err(e) => {
                summary.errors.push(format!("{rel_path}: {e}"));
            }
        }
    }

    summary
}

// ---------------------------------------------------------------------------
// WatchtowerLoop
// ---------------------------------------------------------------------------

/// The Watchtower content source watcher service.
///
/// Watches configured source directories for file changes, debounces events,
/// and ingests changed files into the database via the shared pipeline.
pub struct WatchtowerLoop {
    pub(super) pool: DbPool,
    pub(super) config: ContentSourcesConfig,
    pub(super) connector_config: ConnectorConfig,
    pub(super) data_dir: PathBuf,
    pub(super) debounce_duration: Duration,
    pub(super) fallback_scan_interval: Duration,
    pub(super) cooldown_ttl: Duration,
}

impl WatchtowerLoop {
    /// Create a new WatchtowerLoop.
    pub fn new(
        pool: DbPool,
        config: ContentSourcesConfig,
        connector_config: ConnectorConfig,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            pool,
            config,
            connector_config,
            data_dir,
            debounce_duration: Duration::from_secs(2),
            fallback_scan_interval: Duration::from_secs(300), // 5 minutes
            cooldown_ttl: Duration::from_secs(5),
        }
    }
}
