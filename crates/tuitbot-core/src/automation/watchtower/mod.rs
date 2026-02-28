//! Watchtower filesystem watcher and shared ingest pipeline.
//!
//! Watches configured local directories for `.md` and `.txt` changes via
//! the `notify` crate with debouncing. Filesystem events and manual
//! `POST /api/ingest` file_hints both funnel through `ingest_file()`,
//! ensuring identical state transitions.

pub mod loopback;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify_debouncer_full::{
    new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer, FileIdMap,
};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::config::ContentSourcesConfig;
use crate::storage::watchtower as store;
use crate::storage::DbPool;

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

// ---------------------------------------------------------------------------
// Shared ingest pipeline
// ---------------------------------------------------------------------------

/// Ingest a single file into the Watchtower pipeline.
///
/// This is the shared code path used by both the filesystem watcher and
/// `POST /api/ingest` file_hints. It reads the file, parses front-matter,
/// computes a content hash, and upserts the content node in the database.
pub async fn ingest_file(
    pool: &DbPool,
    source_id: i64,
    base_path: &Path,
    relative_path: &str,
    force: bool,
) -> Result<store::UpsertResult, WatchtowerError> {
    let full_path = base_path.join(relative_path);
    let content = tokio::fs::read_to_string(&full_path).await?;

    let (fm, body) = parse_front_matter(&content);

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
        relative_path,
        &hash,
        fm.title.as_deref(),
        body,
        fm.raw_yaml.as_deref(),
        fm.tags.as_deref(),
    )
    .await?;

    Ok(result)
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
// Cooldown set
// ---------------------------------------------------------------------------

/// Tracks recently-written paths to prevent re-ingestion of our own writes.
struct CooldownSet {
    entries: HashMap<PathBuf, Instant>,
    ttl: Duration,
}

impl CooldownSet {
    fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    /// Mark a path as recently written (used by loop-back writes and tests).
    #[allow(dead_code)]
    fn mark(&mut self, path: PathBuf) {
        self.entries.insert(path, Instant::now());
    }

    /// Check if a path is in cooldown (recently written by us).
    fn is_cooling(&self, path: &Path) -> bool {
        if let Some(ts) = self.entries.get(path) {
            ts.elapsed() < self.ttl
        } else {
            false
        }
    }

    /// Remove expired entries to prevent unbounded growth.
    fn cleanup(&mut self) {
        self.entries.retain(|_, ts| ts.elapsed() < self.ttl);
    }
}

// ---------------------------------------------------------------------------
// WatchtowerLoop
// ---------------------------------------------------------------------------

/// The Watchtower filesystem watcher service.
///
/// Watches configured source directories for file changes, debounces events,
/// and ingests changed files into the database via the shared pipeline.
pub struct WatchtowerLoop {
    pool: DbPool,
    config: ContentSourcesConfig,
    debounce_duration: Duration,
    fallback_scan_interval: Duration,
    cooldown_ttl: Duration,
}

impl WatchtowerLoop {
    /// Create a new WatchtowerLoop.
    pub fn new(pool: DbPool, config: ContentSourcesConfig) -> Self {
        Self {
            pool,
            config,
            debounce_duration: Duration::from_secs(2),
            fallback_scan_interval: Duration::from_secs(300), // 5 minutes
            cooldown_ttl: Duration::from_secs(5),
        }
    }

    /// Run the watchtower loop until the cancellation token is triggered.
    pub async fn run(&self, cancel: CancellationToken) {
        let watch_sources: Vec<_> = self
            .config
            .sources
            .iter()
            .filter(|s| s.watch && s.path.is_some())
            .collect();

        if watch_sources.is_empty() {
            tracing::info!("Watchtower: no watch sources configured, exiting");
            return;
        }

        // Register source contexts in DB.
        let mut source_map: Vec<(i64, PathBuf, Vec<String>)> = Vec::new();
        for src in &watch_sources {
            let path_str = src.path.as_deref().unwrap();
            let expanded = PathBuf::from(crate::storage::expand_tilde(path_str));

            let config_json = serde_json::json!({
                "path": path_str,
                "file_patterns": src.file_patterns,
                "loop_back_enabled": src.loop_back_enabled,
            })
            .to_string();

            match store::ensure_local_fs_source(&self.pool, path_str, &config_json).await {
                Ok(source_id) => {
                    source_map.push((source_id, expanded, src.file_patterns.clone()));
                }
                Err(e) => {
                    tracing::error!(path = path_str, error = %e, "Failed to register source context");
                }
            }
        }

        if source_map.is_empty() {
            tracing::warn!("Watchtower: no sources registered, exiting");
            return;
        }

        // Initial scan of all directories.
        for (source_id, base_path, patterns) in &source_map {
            if let Err(e) = self.scan_directory(*source_id, base_path, patterns).await {
                tracing::error!(
                    path = %base_path.display(),
                    error = %e,
                    "Initial scan failed"
                );
            }
        }

        // Bridge notify's sync callback to an async-friendly tokio channel.
        let (async_tx, mut async_rx) = tokio::sync::mpsc::channel::<DebounceEventResult>(256);

        let handler = move |result: DebounceEventResult| {
            let _ = async_tx.blocking_send(result);
        };

        let debouncer_result = new_debouncer(self.debounce_duration, None, handler);
        let mut debouncer: Debouncer<notify::RecommendedWatcher, FileIdMap> = match debouncer_result
        {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(error = %e, "Failed to create filesystem watcher, falling back to polling");
                self.polling_loop(&source_map, cancel).await;
                return;
            }
        };

        // Register directories with the watcher.
        for (_, base_path, _) in &source_map {
            if let Err(e) = debouncer.watch(base_path, RecursiveMode::Recursive) {
                tracing::error!(
                    path = %base_path.display(),
                    error = %e,
                    "Failed to watch directory"
                );
            }
        }

        tracing::info!(
            sources = source_map.len(),
            "Watchtower watching for file changes"
        );

        let cooldown = Mutex::new(CooldownSet::new(self.cooldown_ttl));

        // Main event loop.
        let mut fallback_timer = tokio::time::interval(self.fallback_scan_interval);
        fallback_timer.tick().await; // Consume the immediate first tick.

        loop {
            tokio::select! {
                () = cancel.cancelled() => {
                    tracing::info!("Watchtower: cancellation received, shutting down");
                    break;
                }
                _ = fallback_timer.tick() => {
                    // Periodic fallback scan to catch any missed events.
                    for (source_id, base_path, patterns) in &source_map {
                        if let Err(e) = self.scan_directory(*source_id, base_path, patterns).await {
                            tracing::warn!(
                                path = %base_path.display(),
                                error = %e,
                                "Fallback scan failed"
                            );
                        }
                    }
                    if let Ok(mut cd) = cooldown.lock() {
                        cd.cleanup();
                    }
                }
                result = async_rx.recv() => {
                    match result {
                        Some(Ok(events)) => {
                            for event in events {
                                for path in &event.paths {
                                    self.handle_event(path, &source_map, &cooldown).await;
                                }
                            }
                        }
                        Some(Err(errs)) => {
                            for e in errs {
                                tracing::warn!(error = %e, "Watcher error");
                            }
                        }
                        None => {
                            tracing::warn!("Watcher event channel closed");
                            break;
                        }
                    }
                }
            }
        }

        // Drop the debouncer to stop watching.
        drop(debouncer);
        tracing::info!("Watchtower shut down");
    }

    /// Handle a single filesystem event for a changed path.
    async fn handle_event(
        &self,
        path: &Path,
        source_map: &[(i64, PathBuf, Vec<String>)],
        cooldown: &Mutex<CooldownSet>,
    ) {
        // Check cooldown.
        if let Ok(cd) = cooldown.lock() {
            if cd.is_cooling(path) {
                tracing::debug!(path = %path.display(), "Skipping cooldown path");
                return;
            }
        }

        // Find matching source.
        for (source_id, base_path, patterns) in source_map {
            if path.starts_with(base_path) {
                // Check pattern match.
                if !matches_patterns(path, patterns) {
                    return;
                }

                // Compute relative path.
                let rel = match path.strip_prefix(base_path) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => return,
                };

                match ingest_file(&self.pool, *source_id, base_path, &rel, false).await {
                    Ok(result) => {
                        tracing::debug!(
                            path = %rel,
                            result = ?result,
                            "Watchtower ingested file"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %rel,
                            error = %e,
                            "Watchtower ingest failed"
                        );
                    }
                }
                return;
            }
        }
    }

    /// Scan a directory for all matching files and ingest them.
    async fn scan_directory(
        &self,
        source_id: i64,
        base_path: &Path,
        patterns: &[String],
    ) -> Result<IngestSummary, WatchtowerError> {
        let mut rel_paths = Vec::new();
        Self::walk_directory(base_path, base_path, patterns, &mut rel_paths)?;

        let summary = ingest_files(&self.pool, source_id, base_path, &rel_paths, false).await;

        tracing::debug!(
            path = %base_path.display(),
            ingested = summary.ingested,
            skipped = summary.skipped,
            errors = summary.errors.len(),
            "Directory scan complete"
        );

        // Update sync cursor.
        let cursor = chrono::Utc::now().to_rfc3339();
        if let Err(e) = store::update_sync_cursor(&self.pool, source_id, &cursor).await {
            tracing::warn!(error = %e, "Failed to update sync cursor");
        }

        Ok(summary)
    }

    /// Recursively walk a directory, collecting relative paths of matching files.
    fn walk_directory(
        base: &Path,
        current: &Path,
        patterns: &[String],
        out: &mut Vec<String>,
    ) -> Result<(), WatchtowerError> {
        let entries = std::fs::read_dir(current)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();

            if file_type.is_dir() {
                // Skip hidden directories.
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }
                Self::walk_directory(base, &path, patterns, out)?;
            } else if file_type.is_file() && matches_patterns(&path, patterns) {
                if let Ok(rel) = path.strip_prefix(base) {
                    out.push(rel.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }

    /// Polling-only fallback loop when the notify watcher fails to initialize.
    async fn polling_loop(
        &self,
        source_map: &[(i64, PathBuf, Vec<String>)],
        cancel: CancellationToken,
    ) {
        let mut interval = tokio::time::interval(self.fallback_scan_interval);
        interval.tick().await; // Consume immediate tick.

        loop {
            tokio::select! {
                () = cancel.cancelled() => {
                    tracing::info!("Watchtower polling loop cancelled");
                    break;
                }
                _ = interval.tick() => {
                    for (source_id, base_path, patterns) in source_map {
                        if let Err(e) = self.scan_directory(*source_id, base_path, patterns).await {
                            tracing::warn!(
                                path = %base_path.display(),
                                error = %e,
                                "Polling scan failed"
                            );
                        }
                    }
                }
            }
        }
    }
}
