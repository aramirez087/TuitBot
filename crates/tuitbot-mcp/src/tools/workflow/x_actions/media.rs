//! Media upload X API tool with DB tracking and idempotency.
//!
//! Media type inference, size validation, and raw upload delegate to
//! `tuitbot_core::toolkit::media`. File I/O, hashing, DB tracking,
//! idempotency, and dry-run support remain here (workflow concerns).

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::storage::media as media_storage;
use tuitbot_core::toolkit::media as toolkit_media;
use tuitbot_core::x_api::types::MediaType;

use crate::state::SharedState;

use super::not_configured_response;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Chunk size constant (matches core/x_api/media.rs).
const CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// Upload a media file for attachment to tweets.
///
/// Tracks uploads in the `media_uploads` table for idempotent re-uploads.
/// If the same file (by SHA-256 hash) was already uploaded and the media ID
/// hasn't expired, returns the cached media ID without re-uploading.
pub async fn upload_media(
    state: &SharedState,
    file_path: &str,
    alt_text: Option<&str>,
    dry_run: bool,
) -> String {
    let start = Instant::now();
    if let Some(err) = super::scraper_mutation_guard(state, start) {
        return err;
    }
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None if dry_run => {
            // In dry-run, we don't need the client — just validate.
            return upload_media_dry_run(file_path, alt_text, start).await;
        }
        None => return not_configured_response(start),
    };

    // Infer media type from file extension via toolkit.
    let media_type = match toolkit_media::infer_media_type(file_path) {
        Some(mt) => mt,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                ErrorCode::UnsupportedMediaType,
                format!(
                    "Unsupported file extension for: {file_path}. \
                     Supported: jpg, jpeg, png, webp, gif, mp4"
                ),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    // Read file bytes.
    let data = match tokio::fs::read(file_path).await {
        Ok(d) => d,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                ErrorCode::FileReadError,
                format!("Failed to read file {file_path}: {e}"),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    let file_size = data.len();
    let file_hash = media_storage::compute_file_hash(&data);

    // Dry-run: validate without uploading.
    if dry_run {
        return dry_run_response(&data, media_type, alt_text, start);
    }

    // Idempotency: check for existing upload with same hash.
    if let Ok(Some(existing)) =
        media_storage::find_ready_upload_by_hash(&state.pool, &file_hash).await
    {
        if let Some(ref mid) = existing.x_media_id {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::success(UploadResult {
                media_id: mid.clone(),
                media_type: media_type.mime_type().to_string(),
                file_size_bytes: file_size,
                upload_strategy: existing.upload_strategy.clone(),
                segment_count: existing.segment_count as usize,
                processing_required: false,
                cached: true,
                file_hash: file_hash.clone(),
                alt_text: existing.alt_text.clone(),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    }

    // Determine upload strategy via toolkit.
    let strategy = toolkit_media::upload_strategy(media_type, file_size as u64);
    let is_chunked = toolkit_media::requires_chunked(media_type, file_size as u64);
    let segment_count = if is_chunked {
        file_size.div_ceil(CHUNK_SIZE)
    } else {
        1
    };
    let processing_required = toolkit_media::requires_processing(media_type);

    // Track in DB.
    let tracking_id = media_storage::insert_media_upload(
        &state.pool,
        &file_hash,
        file_path,
        file_size as i64,
        media_type.mime_type(),
        strategy,
        segment_count as i64,
    )
    .await
    .ok();

    // Upload via toolkit (includes size validation).
    match toolkit_media::upload_media(client.as_ref(), &data, media_type).await {
        Ok(media_id) => {
            // Record success.
            if let Some(tid) = tracking_id {
                let _ =
                    media_storage::finalize_media_upload(&state.pool, tid, &media_id.0, alt_text)
                        .await;
            }
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(UploadResult {
                media_id: media_id.0,
                media_type: media_type.mime_type().to_string(),
                file_size_bytes: file_size,
                upload_strategy: strategy.to_string(),
                segment_count,
                processing_required,
                cached: false,
                file_hash,
                alt_text: alt_text.map(|s| s.to_string()),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(ref e) => {
            // Record failure.
            if let Some(tid) = tracking_id {
                let _ = media_storage::fail_media_upload(&state.pool, tid, &e.to_string()).await;
            }
            super::toolkit_error_response(e, start)
        }
    }
}

/// Dry-run for upload_media when no X client is available.
async fn upload_media_dry_run(file_path: &str, alt_text: Option<&str>, start: Instant) -> String {
    let media_type = match toolkit_media::infer_media_type(file_path) {
        Some(mt) => mt,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                ErrorCode::UnsupportedMediaType,
                format!(
                    "Unsupported file extension for: {file_path}. \
                     Supported: jpg, jpeg, png, webp, gif, mp4"
                ),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    // Try to read the file to validate it exists and check size.
    match tokio::fs::metadata(file_path).await {
        Ok(meta) => {
            let file_size = meta.len();
            if let Err(ref e) = toolkit_media::validate_media_size(file_size, media_type) {
                return super::toolkit_error_response(e, start);
            }
            let strategy = toolkit_media::upload_strategy(media_type, file_size);
            let is_chunked = toolkit_media::requires_chunked(media_type, file_size);
            let segment_count = if is_chunked {
                (file_size as usize).div_ceil(CHUNK_SIZE)
            } else {
                1
            };
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::success(DryRunUploadResult {
                dry_run: true,
                valid: true,
                media_type: media_type.mime_type().to_string(),
                file_size_bytes: file_size as usize,
                upload_strategy: strategy.to_string(),
                segment_count,
                processing_required: toolkit_media::requires_processing(media_type),
                alt_text: alt_text.map(|s| s.to_string()),
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::error(
                ErrorCode::FileReadError,
                format!("Cannot access file {file_path}: {e}"),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
    }
}

/// Build a dry-run response for an already-read file.
fn dry_run_response(
    data: &[u8],
    media_type: MediaType,
    alt_text: Option<&str>,
    start: Instant,
) -> String {
    let file_size = data.len();
    let strategy = toolkit_media::upload_strategy(media_type, file_size as u64);
    let is_chunked = toolkit_media::requires_chunked(media_type, file_size as u64);
    let segment_count = if is_chunked {
        file_size.div_ceil(CHUNK_SIZE)
    } else {
        1
    };
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::success(DryRunUploadResult {
        dry_run: true,
        valid: true,
        media_type: media_type.mime_type().to_string(),
        file_size_bytes: file_size,
        upload_strategy: strategy.to_string(),
        segment_count,
        processing_required: toolkit_media::requires_processing(media_type),
        alt_text: alt_text.map(|s| s.to_string()),
    })
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

#[derive(Serialize)]
struct UploadResult {
    media_id: String,
    media_type: String,
    file_size_bytes: usize,
    upload_strategy: String,
    segment_count: usize,
    processing_required: bool,
    cached: bool,
    file_hash: String,
    alt_text: Option<String>,
}

#[derive(Serialize)]
struct DryRunUploadResult {
    dry_run: bool,
    valid: bool,
    media_type: String,
    file_size_bytes: usize,
    upload_strategy: String,
    segment_count: usize,
    processing_required: bool,
    alt_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use tuitbot_core::toolkit::media::infer_media_type;
    use tuitbot_core::x_api::types::{ImageFormat, MediaType};

    #[test]
    fn infer_jpeg() {
        assert_eq!(
            infer_media_type("photo.jpg"),
            Some(MediaType::Image(ImageFormat::Jpeg))
        );
        assert_eq!(
            infer_media_type("photo.JPEG"),
            Some(MediaType::Image(ImageFormat::Jpeg))
        );
    }

    #[test]
    fn infer_png() {
        assert_eq!(
            infer_media_type("image.png"),
            Some(MediaType::Image(ImageFormat::Png))
        );
    }

    #[test]
    fn infer_gif() {
        assert_eq!(infer_media_type("anim.gif"), Some(MediaType::Gif));
    }

    #[test]
    fn infer_mp4() {
        assert_eq!(infer_media_type("video.mp4"), Some(MediaType::Video));
    }

    #[test]
    fn infer_unsupported() {
        assert_eq!(infer_media_type("file.bmp"), None);
        assert_eq!(infer_media_type("noext"), None);
    }
}
