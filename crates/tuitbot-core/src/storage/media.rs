//! Local media file management and upload tracking.
//!
//! Stores uploaded media files on disk under `{data_dir}/media/` and
//! provides read/cleanup helpers. Also tracks media uploads in SQLite
//! for idempotent re-uploads and agent observability.

use std::path::{Path, PathBuf};

use crate::error::StorageError;
use crate::x_api::types::{ImageFormat, MediaType};

use super::DbPool;

/// A locally stored media file.
#[derive(Debug, Clone)]
pub struct LocalMedia {
    /// Absolute path to the stored file.
    pub path: String,
    /// Detected media type.
    pub media_type: MediaType,
    /// File size in bytes.
    pub size: u64,
}

/// Store media data to disk under `{data_dir}/media/{uuid}.{ext}`.
///
/// Creates the media directory if it doesn't exist.
pub async fn store_media(
    data_dir: &Path,
    data: &[u8],
    _filename: &str,
    media_type: MediaType,
) -> Result<LocalMedia, StorageError> {
    let media_dir = data_dir.join("media");
    tokio::fs::create_dir_all(&media_dir)
        .await
        .map_err(|e| StorageError::Query {
            source: sqlx::Error::Io(e),
        })?;

    let ext = extension_for_type(media_type);
    let uuid = uuid_v4();
    let file_name = format!("{uuid}.{ext}");
    let file_path = media_dir.join(&file_name);

    tokio::fs::write(&file_path, data)
        .await
        .map_err(|e| StorageError::Query {
            source: sqlx::Error::Io(e),
        })?;

    Ok(LocalMedia {
        path: file_path.to_string_lossy().to_string(),
        media_type,
        size: data.len() as u64,
    })
}

/// Read media data from a local file path.
pub async fn read_media(path: &str) -> Result<Vec<u8>, StorageError> {
    tokio::fs::read(path)
        .await
        .map_err(|e| StorageError::Query {
            source: sqlx::Error::Io(e),
        })
}

/// Delete local media files. Errors are logged but not propagated.
pub async fn cleanup_media(paths: &[String]) {
    for path in paths {
        if let Err(e) = tokio::fs::remove_file(path).await {
            tracing::warn!(path = %path, error = %e, "Failed to clean up media file");
        }
    }
}

/// Detect media type from filename extension or content type string.
pub fn detect_media_type(filename: &str, content_type: Option<&str>) -> Option<MediaType> {
    // Try content type first.
    if let Some(ct) = content_type {
        match ct {
            "image/jpeg" => return Some(MediaType::Image(ImageFormat::Jpeg)),
            "image/png" => return Some(MediaType::Image(ImageFormat::Png)),
            "image/webp" => return Some(MediaType::Image(ImageFormat::Webp)),
            "image/gif" => return Some(MediaType::Gif),
            "video/mp4" => return Some(MediaType::Video),
            _ => {}
        }
    }

    // Fall back to extension.
    let lower = filename.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some(MediaType::Image(ImageFormat::Jpeg))
    } else if lower.ends_with(".png") {
        Some(MediaType::Image(ImageFormat::Png))
    } else if lower.ends_with(".webp") {
        Some(MediaType::Image(ImageFormat::Webp))
    } else if lower.ends_with(".gif") {
        Some(MediaType::Gif)
    } else if lower.ends_with(".mp4") {
        Some(MediaType::Video)
    } else {
        None
    }
}

/// Get file extension for a media type.
fn extension_for_type(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Image(ImageFormat::Jpeg) => "jpg",
        MediaType::Image(ImageFormat::Png) => "png",
        MediaType::Image(ImageFormat::Webp) => "webp",
        MediaType::Gif => "gif",
        MediaType::Video => "mp4",
    }
}

/// Generate a simple UUID v4-like string using rand.
fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

// ── DB-backed upload tracking ──────────────────────────────────────

/// A tracked media upload record from the database.
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct MediaUploadRecord {
    pub id: i64,
    pub file_hash: String,
    pub file_name: String,
    pub file_size_bytes: i64,
    pub media_type: String,
    pub upload_strategy: String,
    pub segment_count: i64,
    pub x_media_id: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub alt_text: Option<String>,
    pub created_at: String,
    pub finalized_at: Option<String>,
    pub expires_at: Option<String>,
}

/// Compute SHA-256 hash of file content for idempotency checks.
pub fn compute_file_hash(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(data);
    format!("{hash:x}")
}

/// Find a non-expired, ready upload with the same file hash (idempotent re-upload).
pub async fn find_ready_upload_by_hash(
    pool: &DbPool,
    file_hash: &str,
) -> Result<Option<MediaUploadRecord>, StorageError> {
    let row: Option<MediaUploadRecord> = sqlx::query_as(
        "SELECT id, file_hash, file_name, file_size_bytes, media_type, \
                upload_strategy, segment_count, x_media_id, status, \
                error_message, alt_text, created_at, finalized_at, expires_at \
         FROM media_uploads \
         WHERE file_hash = ? \
           AND status = 'ready' \
           AND (expires_at IS NULL OR expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) \
         ORDER BY created_at DESC \
         LIMIT 1",
    )
    .bind(file_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(row)
}

/// Insert a new media upload tracking record. Returns the row ID.
pub async fn insert_media_upload(
    pool: &DbPool,
    file_hash: &str,
    file_name: &str,
    file_size_bytes: i64,
    media_type: &str,
    upload_strategy: &str,
    segment_count: i64,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO media_uploads (file_hash, file_name, file_size_bytes, media_type, upload_strategy, segment_count, status) \
         VALUES (?, ?, ?, ?, ?, ?, 'uploading')",
    )
    .bind(file_hash)
    .bind(file_name)
    .bind(file_size_bytes)
    .bind(media_type)
    .bind(upload_strategy)
    .bind(segment_count)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(result.last_insert_rowid())
}

/// Mark a media upload as ready with its X API media ID.
pub async fn finalize_media_upload(
    pool: &DbPool,
    id: i64,
    x_media_id: &str,
    alt_text: Option<&str>,
) -> Result<(), StorageError> {
    // X media IDs expire 24 hours after upload.
    sqlx::query(
        "UPDATE media_uploads \
         SET x_media_id = ?, status = 'ready', alt_text = ?, \
             finalized_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), \
             expires_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '+24 hours') \
         WHERE id = ?",
    )
    .bind(x_media_id)
    .bind(alt_text)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Mark a media upload as failed.
pub async fn fail_media_upload(
    pool: &DbPool,
    id: i64,
    error_message: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE media_uploads SET status = 'failed', error_message = ? WHERE id = ?")
        .bind(error_message)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Default size threshold (200 MB) before media cleanup kicks in.
const CLEANUP_THRESHOLD_BYTES: u64 = 200 * 1024 * 1024;

/// Run media cleanup if the media folder exceeds the size threshold.
///
/// Collects all media paths referenced by pending scheduled content and
/// approval queue items, then deletes the oldest unreferenced files until
/// total size drops below the threshold.
pub async fn cleanup_if_over_threshold(
    data_dir: &Path,
    pool: &DbPool,
) -> Result<u64, StorageError> {
    let media_dir = data_dir.join("media");
    if !media_dir.exists() {
        return Ok(0);
    }

    // Scan all files and compute total size.
    let mut files = scan_media_files(&media_dir).await?;
    let total_size: u64 = files.iter().map(|f| f.size).sum();

    if total_size <= CLEANUP_THRESHOLD_BYTES {
        return Ok(0);
    }

    // Collect referenced paths from approval_queue and scheduled_content.
    let referenced = collect_referenced_paths(pool).await?;

    // Filter to unreferenced files, sort oldest first.
    files.retain(|f| !referenced.contains(&f.path));
    files.sort_by_key(|f| f.modified);

    // Delete oldest unreferenced files until under threshold.
    let mut current_size = total_size;
    let mut deleted = 0u64;
    for file in &files {
        if current_size <= CLEANUP_THRESHOLD_BYTES {
            break;
        }
        if tokio::fs::remove_file(&file.path).await.is_ok() {
            current_size = current_size.saturating_sub(file.size);
            deleted += 1;
        }
    }

    if deleted > 0 {
        tracing::info!(
            deleted_files = deleted,
            freed_bytes = total_size.saturating_sub(current_size),
            remaining_bytes = current_size,
            "Media cleanup completed"
        );
    }

    Ok(deleted)
}

struct MediaFile {
    path: String,
    size: u64,
    modified: std::time::SystemTime,
}

async fn scan_media_files(media_dir: &Path) -> Result<Vec<MediaFile>, StorageError> {
    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(media_dir)
        .await
        .map_err(|e| StorageError::Query {
            source: sqlx::Error::Io(e),
        })?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| StorageError::Query {
            source: sqlx::Error::Io(e),
        })?
    {
        let meta = match entry.metadata().await {
            Ok(m) if m.is_file() => m,
            _ => continue,
        };
        files.push(MediaFile {
            path: entry.path().to_string_lossy().to_string(),
            size: meta.len(),
            modified: meta.modified().unwrap_or(std::time::UNIX_EPOCH),
        });
    }
    Ok(files)
}

/// Collect all media file paths referenced by active approval queue items
/// and pending scheduled content.
async fn collect_referenced_paths(
    pool: &DbPool,
) -> Result<std::collections::HashSet<String>, StorageError> {
    let mut paths = std::collections::HashSet::new();

    // 1. approval_queue: media_paths is a JSON array of strings.
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT COALESCE(media_paths, '[]') FROM approval_queue WHERE status = 'pending'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    for (json_str,) in &rows {
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(json_str) {
            paths.extend(arr);
        }
    }

    // 2. scheduled_content: content may contain blocks with media_paths.
    let content_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT content_type, content FROM scheduled_content WHERE status = 'scheduled'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    for (content_type, content) in &content_rows {
        if content_type == "thread" {
            if let Some(blocks) = crate::content::deserialize_blocks_from_content(content) {
                for block in blocks {
                    paths.extend(block.media_paths);
                }
            }
        }
    }

    Ok(paths)
}

/// Validate that a file path is under the expected media directory (path traversal protection).
pub fn is_safe_media_path(path: &str, data_dir: &Path) -> bool {
    let media_dir = data_dir.join("media");
    match PathBuf::from(path).canonicalize() {
        Ok(canonical) => canonical.starts_with(&media_dir),
        // If the file doesn't exist yet, check prefix.
        Err(_) => Path::new(path).starts_with(&media_dir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_media_type_from_content_type() {
        assert_eq!(
            detect_media_type("photo.bin", Some("image/jpeg")),
            Some(MediaType::Image(ImageFormat::Jpeg))
        );
        assert_eq!(
            detect_media_type("x", Some("image/gif")),
            Some(MediaType::Gif)
        );
        assert_eq!(
            detect_media_type("x", Some("video/mp4")),
            Some(MediaType::Video)
        );
    }

    #[test]
    fn detect_media_type_from_extension() {
        assert_eq!(
            detect_media_type("photo.jpg", None),
            Some(MediaType::Image(ImageFormat::Jpeg))
        );
        assert_eq!(
            detect_media_type("photo.JPEG", None),
            Some(MediaType::Image(ImageFormat::Jpeg))
        );
        assert_eq!(
            detect_media_type("image.png", None),
            Some(MediaType::Image(ImageFormat::Png))
        );
        assert_eq!(
            detect_media_type("pic.webp", None),
            Some(MediaType::Image(ImageFormat::Webp))
        );
        assert_eq!(detect_media_type("ani.gif", None), Some(MediaType::Gif));
        assert_eq!(detect_media_type("clip.mp4", None), Some(MediaType::Video));
        assert_eq!(detect_media_type("file.txt", None), None);
    }

    #[tokio::test]
    async fn store_and_read_media() {
        let dir = tempfile::tempdir().expect("temp dir");
        let data = b"fake image data";

        let media = store_media(
            dir.path(),
            data,
            "test.jpg",
            MediaType::Image(ImageFormat::Jpeg),
        )
        .await
        .expect("store");

        assert!(media.path.ends_with(".jpg"));
        assert_eq!(media.size, data.len() as u64);

        let read_back = read_media(&media.path).await.expect("read");
        assert_eq!(read_back, data);
    }

    #[tokio::test]
    async fn cleanup_removes_files() {
        let dir = tempfile::tempdir().expect("temp dir");
        let data = b"temp media";

        let media = store_media(
            dir.path(),
            data,
            "temp.png",
            MediaType::Image(ImageFormat::Png),
        )
        .await
        .expect("store");

        assert!(Path::new(&media.path).exists());
        cleanup_media(&[media.path.clone()]).await;
        assert!(!Path::new(&media.path).exists());
    }

    #[test]
    fn compute_file_hash_deterministic() {
        let data = b"hello world";
        let h1 = compute_file_hash(data);
        let h2 = compute_file_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[tokio::test]
    async fn insert_and_find_media_upload() {
        let pool = crate::storage::init_test_db().await.expect("db");
        let hash = compute_file_hash(b"test data");

        // No record yet.
        let found = find_ready_upload_by_hash(&pool, &hash).await.expect("find");
        assert!(found.is_none());

        // Insert and finalize.
        let id = insert_media_upload(&pool, &hash, "test.jpg", 1024, "image/jpeg", "simple", 1)
            .await
            .expect("insert");
        assert!(id > 0);

        finalize_media_upload(&pool, id, "x_media_123", None)
            .await
            .expect("finalize");

        // Now findable.
        let found = find_ready_upload_by_hash(&pool, &hash)
            .await
            .expect("find")
            .expect("should exist");
        assert_eq!(found.x_media_id.as_deref(), Some("x_media_123"));
        assert_eq!(found.status, "ready");
    }

    #[tokio::test]
    async fn fail_media_upload_records_error() {
        let pool = crate::storage::init_test_db().await.expect("db");
        let hash = compute_file_hash(b"bad data");

        let id = insert_media_upload(&pool, &hash, "fail.mp4", 999, "video/mp4", "chunked", 3)
            .await
            .expect("insert");

        fail_media_upload(&pool, id, "upload timed out")
            .await
            .expect("fail");

        // Should NOT be found as ready.
        let found = find_ready_upload_by_hash(&pool, &hash).await.expect("find");
        assert!(found.is_none());
    }

    // ── detect_media_type additional coverage ────────────────────

    #[test]
    fn detect_media_type_content_type_png() {
        assert_eq!(
            detect_media_type("x", Some("image/png")),
            Some(MediaType::Image(ImageFormat::Png))
        );
    }

    #[test]
    fn detect_media_type_content_type_webp() {
        assert_eq!(
            detect_media_type("x", Some("image/webp")),
            Some(MediaType::Image(ImageFormat::Webp))
        );
    }

    #[test]
    fn detect_media_type_unknown_content_type_falls_back_to_extension() {
        assert_eq!(
            detect_media_type("image.png", Some("application/octet-stream")),
            Some(MediaType::Image(ImageFormat::Png))
        );
    }

    #[test]
    fn detect_media_type_no_extension_no_content_type() {
        assert_eq!(detect_media_type("no_extension", None), None);
    }

    #[test]
    fn detect_media_type_case_insensitive_extension() {
        assert_eq!(
            detect_media_type("IMAGE.PNG", None),
            Some(MediaType::Image(ImageFormat::Png))
        );
        assert_eq!(detect_media_type("video.MP4", None), Some(MediaType::Video));
        assert_eq!(detect_media_type("anim.GIF", None), Some(MediaType::Gif));
        assert_eq!(
            detect_media_type("photo.WebP", None),
            Some(MediaType::Image(ImageFormat::Webp))
        );
    }

    // ── extension_for_type coverage ─────────────────────────────

    #[test]
    fn extension_for_type_all_variants() {
        assert_eq!(
            extension_for_type(MediaType::Image(ImageFormat::Jpeg)),
            "jpg"
        );
        assert_eq!(
            extension_for_type(MediaType::Image(ImageFormat::Png)),
            "png"
        );
        assert_eq!(
            extension_for_type(MediaType::Image(ImageFormat::Webp)),
            "webp"
        );
        assert_eq!(extension_for_type(MediaType::Gif), "gif");
        assert_eq!(extension_for_type(MediaType::Video), "mp4");
    }

    // ── compute_file_hash edge cases ────────────────────────────

    #[test]
    fn compute_file_hash_empty() {
        let hash = compute_file_hash(b"");
        assert_eq!(hash.len(), 64);
        // SHA-256 of empty is well-known
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn compute_file_hash_different_data_different_hash() {
        let h1 = compute_file_hash(b"data1");
        let h2 = compute_file_hash(b"data2");
        assert_ne!(h1, h2);
    }

    // ── store_media edge cases ──────────────────────────────────

    #[tokio::test]
    async fn store_media_all_types() {
        let dir = tempfile::tempdir().expect("temp dir");

        let types = vec![
            (MediaType::Image(ImageFormat::Jpeg), "jpg"),
            (MediaType::Image(ImageFormat::Png), "png"),
            (MediaType::Image(ImageFormat::Webp), "webp"),
            (MediaType::Gif, "gif"),
            (MediaType::Video, "mp4"),
        ];

        for (media_type, expected_ext) in types {
            let media = store_media(dir.path(), b"data", "test", media_type)
                .await
                .expect("store");
            assert!(
                media.path.ends_with(&format!(".{expected_ext}")),
                "expected extension .{expected_ext}, got path: {}",
                media.path
            );
            assert_eq!(media.media_type, media_type);
        }
    }

    #[tokio::test]
    async fn store_media_empty_data() {
        let dir = tempfile::tempdir().expect("temp dir");
        let media = store_media(
            dir.path(),
            b"",
            "empty.png",
            MediaType::Image(ImageFormat::Png),
        )
        .await
        .expect("store");
        assert_eq!(media.size, 0);
    }

    #[tokio::test]
    async fn read_media_nonexistent_returns_error() {
        let result = read_media("/nonexistent/path/file.jpg").await;
        assert!(result.is_err());
    }

    // ── cleanup_media edge cases ────────────────────────────────

    #[tokio::test]
    async fn cleanup_media_nonexistent_paths_no_panic() {
        // Should not panic when files don't exist
        cleanup_media(&[
            "/nonexistent/a.jpg".to_string(),
            "/nonexistent/b.png".to_string(),
        ])
        .await;
    }

    // ── is_safe_media_path ──────────────────────────────────────

    #[test]
    fn is_safe_media_path_nonexistent_under_media() {
        let dir = tempfile::tempdir().expect("temp dir");
        let media_dir = dir.path().join("media");
        std::fs::create_dir_all(&media_dir).expect("mkdir");
        // Non-existent file under media dir: canonicalize fails, falls back to prefix check
        let file_path = media_dir.join("future.jpg");
        assert!(is_safe_media_path(file_path.to_str().unwrap(), dir.path()));
    }

    #[test]
    fn is_safe_media_path_outside_media_dir_nonexistent() {
        let dir = tempfile::tempdir().expect("temp dir");
        // A path clearly outside the data_dir (non-existent, prefix check)
        let bad_path = format!("{}/not_media/file.jpg", dir.path().display());
        assert!(!is_safe_media_path(&bad_path, dir.path()));
    }

    // ── finalize and find round-trip ────────────────────────────

    #[tokio::test]
    async fn finalize_with_alt_text() {
        let pool = crate::storage::init_test_db().await.expect("db");
        let hash = compute_file_hash(b"alt text test");

        let id = insert_media_upload(&pool, &hash, "photo.jpg", 2048, "image/jpeg", "simple", 1)
            .await
            .expect("insert");

        finalize_media_upload(&pool, id, "x_media_456", Some("A beautiful sunset"))
            .await
            .expect("finalize");

        let found = find_ready_upload_by_hash(&pool, &hash)
            .await
            .expect("find")
            .expect("should exist");
        assert_eq!(found.alt_text.as_deref(), Some("A beautiful sunset"));
        assert_eq!(found.x_media_id.as_deref(), Some("x_media_456"));
    }

    #[tokio::test]
    async fn multiple_uploads_same_hash_finds_one() {
        let pool = crate::storage::init_test_db().await.expect("db");
        let hash = compute_file_hash(b"duplicate data");

        let id1 = insert_media_upload(&pool, &hash, "a.jpg", 100, "image/jpeg", "simple", 1)
            .await
            .expect("insert 1");
        finalize_media_upload(&pool, id1, "media_1", None)
            .await
            .expect("finalize 1");

        let id2 = insert_media_upload(&pool, &hash, "b.jpg", 100, "image/jpeg", "simple", 1)
            .await
            .expect("insert 2");
        finalize_media_upload(&pool, id2, "media_2", None)
            .await
            .expect("finalize 2");

        let found = find_ready_upload_by_hash(&pool, &hash)
            .await
            .expect("find")
            .expect("should exist");
        // Should find one of the ready uploads (ORDER BY created_at DESC LIMIT 1)
        let media_id = found.x_media_id.as_deref().unwrap();
        assert!(
            media_id == "media_1" || media_id == "media_2",
            "should find one of the finalized uploads"
        );
        assert_eq!(found.status, "ready");
    }
}
