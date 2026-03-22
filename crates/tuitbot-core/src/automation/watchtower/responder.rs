//! Responder logic: loopback deduplication and chunking.

use crate::storage::accounts::DEFAULT_ACCOUNT_ID;
use crate::storage::DbPool;

use super::WatchtowerLoop;

impl WatchtowerLoop {
    /// Build a GoogleDriveProvider backed by a linked-account connection.
    ///
    /// Loads the connector encryption key and constructs the connector
    /// from config. Returns an error string if setup fails (caller
    /// logs and skips the source).
    pub fn build_connection_provider(
        &self,
        folder_id: &str,
        connection_id: i64,
    ) -> Result<crate::source::google_drive::GoogleDriveProvider, String> {
        let key = crate::source::connector::crypto::ensure_connector_key(&self.data_dir)
            .map_err(|e| format!("connector key error: {e}"))?;

        let connector = crate::source::connector::google_drive::GoogleDriveConnector::new(
            &self.connector_config.google_drive,
        )
        .map_err(|e| format!("connector config error: {e}"))?;

        Ok(
            crate::source::google_drive::GoogleDriveProvider::from_connection(
                folder_id.to_string(),
                connection_id,
                self.pool.clone(),
                key,
                connector,
            ),
        )
    }

    /// Process pending content nodes: extract fragments and persist as chunks.
    pub async fn chunk_pending(&self) {
        let chunked =
            super::chunker::chunk_pending_nodes(&self.pool, DEFAULT_ACCOUNT_ID, 100).await;
        if chunked > 0 {
            tracing::debug!(chunked, "Watchtower chunked pending nodes");
        }
    }

    /// Perform a one-shot full rescan of a single local source.
    ///
    /// Used by the reindex API. Sets status to `"syncing"` before the scan
    /// and `"active"` (or `"error"`) afterward.
    pub async fn reindex_local_source(
        pool: &DbPool,
        source_id: i64,
        base_path: &std::path::Path,
        patterns: &[String],
    ) -> Result<super::IngestSummary, super::WatchtowerError> {
        crate::storage::watchtower::update_source_status(pool, source_id, "syncing", None).await?;

        let mut rel_paths = Vec::new();
        WatchtowerLoop::walk_directory_static(base_path, base_path, patterns, &mut rel_paths)?;

        let summary = super::ingest_files(pool, source_id, base_path, &rel_paths, true).await;

        let cursor = chrono::Utc::now().to_rfc3339();
        let _ = crate::storage::watchtower::update_sync_cursor(pool, source_id, &cursor).await;

        if summary.errors.is_empty() {
            let _ =
                crate::storage::watchtower::update_source_status(pool, source_id, "active", None)
                    .await;
        } else {
            let msg = format!("{} errors during reindex", summary.errors.len());
            let _ = crate::storage::watchtower::update_source_status(
                pool,
                source_id,
                "error",
                Some(&msg),
            )
            .await;
        }

        Ok(summary)
    }
}

// Static helper for reindex (doesn't need &self).
impl WatchtowerLoop {
    pub(super) fn walk_directory_static(
        base: &std::path::Path,
        current: &std::path::Path,
        patterns: &[String],
        out: &mut Vec<String>,
    ) -> Result<(), super::WatchtowerError> {
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
                Self::walk_directory_static(base, &path, patterns, out)?;
            } else if file_type.is_file() && super::matches_patterns(&path, patterns) {
                if let Ok(rel) = path.strip_prefix(base) {
                    out.push(super::relative_path_string(rel));
                }
            }
        }
        Ok(())
    }
}
