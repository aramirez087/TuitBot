//! Coordinator: orchestrates the main watchtower run loop.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::storage::watchtower as store;

use super::{CooldownSet, RemoteSource, WatchtowerLoop};

impl WatchtowerLoop {
    /// Run the watchtower loop until the cancellation token is triggered.
    ///
    /// Registers both local filesystem and remote sources, then runs:
    /// - `notify` watcher + fallback polling for local sources
    /// - interval-based polling for remote sources (e.g. Google Drive)
    pub async fn run(&self, cancel: CancellationToken) {
        // Split config into local (watchable) and remote (pollable) sources.
        // Uses `is_enabled()` which respects both `enabled` and legacy `watch`.
        let local_sources: Vec<_> = self
            .config
            .sources
            .iter()
            .filter(|s| s.source_type == "local_fs" && s.is_enabled() && s.path.is_some())
            .collect();

        let remote_sources: Vec<_> = self
            .config
            .sources
            .iter()
            .filter(|s| s.source_type == "google_drive" && s.is_enabled() && s.folder_id.is_some())
            .collect();

        if local_sources.is_empty() && remote_sources.is_empty() {
            tracing::info!("Watchtower: no watch sources configured, exiting");
            return;
        }

        // Register local source contexts in DB.
        let mut source_map: Vec<(i64, PathBuf, Vec<String>)> = Vec::new();
        for src in &local_sources {
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

        // Register remote source contexts and build provider instances.
        let mut remote_map: Vec<RemoteSource> = Vec::new();
        for src in &remote_sources {
            let folder_id = src.folder_id.as_deref().unwrap();
            let config_json = serde_json::json!({
                "folder_id": folder_id,
                "file_patterns": src.file_patterns,
                "service_account_key": src.service_account_key,
                "connection_id": src.connection_id,
            })
            .to_string();

            match store::ensure_google_drive_source(&self.pool, folder_id, &config_json).await {
                Ok(source_id) => {
                    let interval = Duration::from_secs(src.poll_interval_seconds.unwrap_or(300));

                    // connection_id takes precedence over service_account_key.
                    let provider: Box<dyn crate::source::ContentSourceProvider> =
                        if let Some(connection_id) = src.connection_id {
                            match self.build_connection_provider(folder_id, connection_id) {
                                Ok(p) => Box::new(p),
                                Err(reason) => {
                                    tracing::warn!(
                                        folder_id,
                                        connection_id,
                                        reason = %reason,
                                        "Skipping connection-based source"
                                    );
                                    continue;
                                }
                            }
                        } else if src.service_account_key.is_some() {
                            let key_path = src.service_account_key.clone().unwrap_or_default();
                            Box::new(crate::source::google_drive::GoogleDriveProvider::new(
                                folder_id.to_string(),
                                key_path,
                            ))
                        } else {
                            tracing::warn!(
                            folder_id,
                            "Skipping Google Drive source: no connection_id or service_account_key"
                        );
                            continue;
                        };

                    remote_map.push((source_id, provider, src.file_patterns.clone(), interval));
                }
                Err(e) => {
                    tracing::error!(
                        folder_id = folder_id,
                        error = %e,
                        "Failed to register Google Drive source"
                    );
                }
            }
        }

        if source_map.is_empty() && remote_map.is_empty() {
            tracing::warn!("Watchtower: no sources registered, exiting");
            return;
        }

        // Initial scan of all local directories (all enabled sources, regardless of change_detection).
        for (source_id, base_path, patterns) in &source_map {
            let _ = store::update_source_status(&self.pool, *source_id, "syncing", None).await;
            match self.scan_directory(*source_id, base_path, patterns).await {
                Ok(_) => {
                    let _ =
                        store::update_source_status(&self.pool, *source_id, "active", None).await;
                }
                Err(e) => {
                    tracing::error!(
                        path = %base_path.display(),
                        error = %e,
                        "Initial scan failed"
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

        // Chunk any nodes created during initial local scan.
        self.chunk_pending().await;

        // Initial poll of remote sources.
        if !remote_map.is_empty() {
            self.poll_remote_sources(&remote_map).await;
            self.chunk_pending().await;
        }

        // Partition local sources: those with ongoing monitoring vs scan-only.
        // Scan-only sources (`change_detection = "none"`) already did their initial
        // scan above and don't participate in the event loop.
        let watch_source_map: Vec<_> = source_map
            .iter()
            .zip(local_sources.iter())
            .filter(|(_, src)| !src.is_scan_only())
            .map(|(entry, _)| entry.clone())
            .collect();

        // Further split: sources that need notify vs poll-only.
        let notify_source_map: Vec<_> = source_map
            .iter()
            .zip(local_sources.iter())
            .filter(|(_, src)| src.effective_change_detection() == "auto")
            .map(|(entry, _)| entry.clone())
            .collect();

        // If no sources need ongoing monitoring, only run remote polling.
        if watch_source_map.is_empty() {
            if remote_map.is_empty() {
                tracing::info!(
                    "Watchtower: all local sources are scan-only and no remote sources, exiting"
                );
                return;
            }
            self.remote_only_loop(&remote_map, cancel).await;
            return;
        }

        // Bridge notify's sync callback to an async-friendly tokio channel.
        let (async_tx, mut async_rx) =
            tokio::sync::mpsc::channel::<notify_debouncer_full::DebounceEventResult>(256);

        let handler = move |result: notify_debouncer_full::DebounceEventResult| {
            let _ = async_tx.blocking_send(result);
        };

        let debouncer_result =
            notify_debouncer_full::new_debouncer(self.debounce_duration, None, handler);
        let mut debouncer: notify_debouncer_full::Debouncer<
            notify::RecommendedWatcher,
            notify_debouncer_full::RecommendedCache,
        > = match debouncer_result {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(error = %e, "Failed to create filesystem watcher, falling back to polling");
                self.polling_loop(&watch_source_map, cancel).await;
                return;
            }
        };

        // Register directories with the notify watcher (only "auto" sources, not poll-only).
        for (_, base_path, _) in &notify_source_map {
            if let Err(e) = debouncer.watch(
                base_path,
                notify_debouncer_full::notify::RecursiveMode::Recursive,
            ) {
                tracing::error!(
                    path = %base_path.display(),
                    error = %e,
                    "Failed to watch directory"
                );
            }
        }

        tracing::info!(
            local_sources = source_map.len(),
            watching = notify_source_map.len(),
            polling = watch_source_map.len() - notify_source_map.len(),
            remote_sources = remote_map.len(),
            "Watchtower watching for changes"
        );

        let cooldown = Mutex::new(CooldownSet::new(self.cooldown_ttl));

        // Main event loop.
        let mut fallback_timer = tokio::time::interval(self.fallback_scan_interval);
        fallback_timer.tick().await; // Consume the immediate first tick.

        // Remote poll interval (use minimum configured or fallback default).
        let remote_interval = remote_map
            .iter()
            .map(|(_, _, _, d)| *d)
            .min()
            .unwrap_or(self.fallback_scan_interval);
        let mut remote_timer = tokio::time::interval(remote_interval);
        remote_timer.tick().await; // Consume the immediate first tick.

        loop {
            tokio::select! {
                () = cancel.cancelled() => {
                    tracing::info!("Watchtower: cancellation received, shutting down");
                    break;
                }
                _ = fallback_timer.tick() => {
                    // Periodic fallback scan for all local sources with ongoing monitoring
                    // (both "auto" and "poll" change_detection modes).
                    for (source_id, base_path, patterns) in &watch_source_map {
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
                    self.chunk_pending().await;
                }
                _ = remote_timer.tick(), if !remote_map.is_empty() => {
                    self.poll_remote_sources(&remote_map).await;
                    self.chunk_pending().await;
                }
                result = async_rx.recv() => {
                    match result {
                        Some(Ok(events)) => {
                            for event in events {
                                for path in &event.paths {
                                    self.handle_event(path, &source_map, &cooldown).await;
                                }
                            }
                            self.chunk_pending().await;
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
}
