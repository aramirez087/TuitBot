//! Monitoring logic: directory scanning, remote polling, event handling.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use crate::source::ContentSourceProvider;
use crate::storage::watchtower as store;

use super::{
    ingest_content, ingest_file, ingest_files, matches_patterns, relative_path_string,
    IngestSummary, WatchtowerError, WatchtowerLoop,
};

// ---------------------------------------------------------------------------
// Cooldown tracking for loopback writes
// ---------------------------------------------------------------------------

/// Tracks recently-written paths to prevent re-ingestion of our own writes.
pub struct CooldownSet {
    pub entries: std::collections::HashMap<PathBuf, Instant>,
    ttl: Duration,
}

impl CooldownSet {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            ttl,
        }
    }

    /// Mark a path as recently written (used by loop-back writes and tests).
    #[allow(dead_code)]
    pub fn mark(&mut self, path: PathBuf) {
        self.entries.insert(path, Instant::now());
    }

    /// Check if a path is in cooldown (recently written by us).
    pub fn is_cooling(&self, path: &Path) -> bool {
        if let Some(ts) = self.entries.get(path) {
            ts.elapsed() < self.ttl
        } else {
            false
        }
    }

    /// Remove expired entries to prevent unbounded growth.
    pub fn cleanup(&mut self) {
        self.entries.retain(|_, ts| ts.elapsed() < self.ttl);
    }
}

// ---------------------------------------------------------------------------
// Remote source type
// ---------------------------------------------------------------------------

/// A registered remote source: (db_source_id, provider, file_patterns, poll_interval).
pub type RemoteSource = (i64, Box<dyn ContentSourceProvider>, Vec<String>, Duration);

// ---------------------------------------------------------------------------
// WatchtowerLoop monitoring methods
// ---------------------------------------------------------------------------

impl WatchtowerLoop {
    /// Handle a single filesystem event for a changed path.
    pub async fn handle_event(
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
                    Ok(r) => relative_path_string(r),
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
    pub async fn scan_directory(
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
    pub fn walk_directory(
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
                    out.push(relative_path_string(rel));
                }
            }
        }
        Ok(())
    }

    /// Poll all remote sources for changes, ingest new/updated content.
    pub async fn poll_remote_sources(&self, remote_sources: &[RemoteSource]) {
        for (source_id, provider, patterns, _interval) in remote_sources {
            let _ = store::update_source_status(&self.pool, *source_id, "syncing", None).await;

            let cursor = match store::get_source_context(&self.pool, *source_id).await {
                Ok(Some(ctx)) => ctx.sync_cursor,
                Ok(None) => None,
                Err(e) => {
                    tracing::warn!(source_id, error = %e, "Failed to get source context");
                    continue;
                }
            };

            match provider.scan_for_changes(cursor.as_deref(), patterns).await {
                Ok(files) => {
                    let mut ingested = 0u32;
                    let mut skipped = 0u32;
                    for file in &files {
                        match provider.read_content(&file.provider_id).await {
                            Ok(content) => {
                                match ingest_content(
                                    &self.pool,
                                    *source_id,
                                    &file.provider_id,
                                    &content,
                                    false,
                                )
                                .await
                                {
                                    Ok(
                                        store::UpsertResult::Inserted
                                        | store::UpsertResult::Updated,
                                    ) => {
                                        ingested += 1;
                                    }
                                    Ok(store::UpsertResult::Skipped) => {
                                        skipped += 1;
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            provider_id = %file.provider_id,
                                            error = %e,
                                            "Remote ingest failed"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    provider_id = %file.provider_id,
                                    error = %e,
                                    "Failed to read remote content"
                                );
                            }
                        }
                    }

                    tracing::debug!(
                        source_type = provider.source_type(),
                        ingested,
                        skipped,
                        total = files.len(),
                        "Remote poll complete"
                    );

                    // Update sync cursor and mark active.
                    let new_cursor = chrono::Utc::now().to_rfc3339();
                    if let Err(e) =
                        store::update_sync_cursor(&self.pool, *source_id, &new_cursor).await
                    {
                        tracing::warn!(error = %e, "Failed to update remote sync cursor");
                    }
                    let _ =
                        store::update_source_status(&self.pool, *source_id, "active", None).await;
                }
                Err(crate::source::SourceError::ConnectionBroken {
                    connection_id,
                    ref reason,
                }) => {
                    tracing::warn!(
                        source_id,
                        connection_id,
                        reason = %reason,
                        "Connection broken -- marking source as error"
                    );
                    let _ =
                        store::update_source_status(&self.pool, *source_id, "error", Some(reason))
                            .await;
                    let _ =
                        store::update_connection_status(&self.pool, connection_id, "expired").await;
                }
                Err(e) => {
                    tracing::warn!(
                        source_type = provider.source_type(),
                        error = %e,
                        "Remote scan failed"
                    );
                    let _ = store::update_source_status(
                        &self.pool,
                        *source_id,
                        "error",
                        Some(&e.to_string()),
                    )
                    .await;
                }
            }
        }
    }

    /// Loop for when only remote sources are configured (no local watchers).
    pub async fn remote_only_loop(&self, remote_map: &[RemoteSource], cancel: CancellationToken) {
        let interval_dur = remote_map
            .iter()
            .map(|(_, _, _, d)| *d)
            .min()
            .unwrap_or(self.fallback_scan_interval);
        let mut interval = tokio::time::interval(interval_dur);
        interval.tick().await;

        loop {
            tokio::select! {
                () = cancel.cancelled() => {
                    tracing::info!("Watchtower remote-only loop cancelled");
                    break;
                }
                _ = interval.tick() => {
                    self.poll_remote_sources(remote_map).await;
                    self.chunk_pending().await;
                }
            }
        }
    }

    /// Polling-only fallback loop when the notify watcher fails to initialize.
    pub async fn polling_loop(
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
