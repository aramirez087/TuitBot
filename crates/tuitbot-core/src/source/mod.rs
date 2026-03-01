//! Content source provider abstraction.
//!
//! Defines the `ContentSourceProvider` trait that both local filesystem and
//! remote (e.g. Google Drive) sources implement. The trait covers scanning
//! for changed files and reading content — the Watchtower orchestrates
//! watching vs polling based on source type.

pub mod connector;
pub mod google_drive;
pub mod local_fs;

#[cfg(test)]
mod tests;

use async_trait::async_trait;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from content source providers.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(String),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("connection broken (id={connection_id}): {reason}")]
    ConnectionBroken { connection_id: i64, reason: String },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Metadata about a file discovered by a provider scan.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Stable identifier (relative path for local, `gdrive://<id>/<name>` for Drive).
    pub provider_id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// SHA-256 content hash.
    pub content_hash: String,
    /// RFC 3339 modification timestamp.
    pub modified_at: String,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Abstraction over content source backends.
///
/// Local sources use filesystem walking; remote sources use API polling.
/// Both produce `SourceFile` metadata and string content that feeds
/// into the shared Watchtower ingest pipeline.
#[async_trait]
pub trait ContentSourceProvider: Send + Sync {
    /// Returns the source type identifier (e.g. `"local_fs"`, `"google_drive"`).
    fn source_type(&self) -> &str;

    /// Scan for files that changed since `since_cursor`.
    ///
    /// - `since_cursor`: opaque sync cursor from the last scan (RFC 3339 timestamp
    ///   or provider-specific token). `None` means full scan.
    /// - `patterns`: glob patterns to filter files (e.g. `["*.md", "*.txt"]`).
    ///
    /// Returns metadata for each changed file. The caller is responsible for
    /// calling `read_content` on files that need ingestion.
    async fn scan_for_changes(
        &self,
        since_cursor: Option<&str>,
        patterns: &[String],
    ) -> Result<Vec<SourceFile>, SourceError>;

    /// Read the full text content of a file by its provider ID.
    async fn read_content(&self, file_id: &str) -> Result<String, SourceError>;
}
