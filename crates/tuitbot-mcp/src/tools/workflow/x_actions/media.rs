//! Media upload X API tool with DB tracking and idempotency.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::storage::media as media_storage;
use tuitbot_core::x_api::types::{ImageFormat, MediaType};

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
            return upload_media_dry_run(state, file_path, alt_text, start).await;
        }
        None => return not_configured_response(start),
    };

    // Infer media type from file extension.
    let media_type = match infer_media_type(file_path) {
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
        return dry_run_response(file_path, &data, media_type, alt_text, &file_hash, start);
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

    // Determine upload strategy.
    let requires_chunked = media_type.requires_chunked(file_size as u64);
    let strategy = if requires_chunked {
        "chunked"
    } else {
        "simple"
    };
    let segment_count = if requires_chunked {
        file_size.div_ceil(CHUNK_SIZE)
    } else {
        1
    };
    let processing_required = matches!(media_type, MediaType::Gif | MediaType::Video);

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

    match client.upload_media(&data, media_type).await {
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
        Err(e) => {
            // Record failure.
            if let Some(tid) = tracking_id {
                let _ = media_storage::fail_media_upload(&state.pool, tid, &e.to_string()).await;
            }
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::error(
                ErrorCode::MediaUploadError,
                format!("Media upload failed: {e}"),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
    }
}

/// Dry-run for upload_media when no X client is available.
async fn upload_media_dry_run(
    _state: &SharedState,
    file_path: &str,
    alt_text: Option<&str>,
    start: Instant,
) -> String {
    let media_type = match infer_media_type(file_path) {
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
            if file_size > media_type.max_size() {
                let elapsed = start.elapsed().as_millis() as u64;
                return ToolResponse::error(
                    ErrorCode::MediaUploadError,
                    format!(
                        "File size {}B exceeds maximum {}B for {}",
                        file_size,
                        media_type.max_size(),
                        media_type.mime_type()
                    ),
                )
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
            }
            let requires_chunked = media_type.requires_chunked(file_size);
            let strategy = if requires_chunked {
                "chunked"
            } else {
                "simple"
            };
            let segment_count = if requires_chunked {
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
                processing_required: matches!(media_type, MediaType::Gif | MediaType::Video),
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
    _file_path: &str,
    data: &[u8],
    media_type: MediaType,
    alt_text: Option<&str>,
    _file_hash: &str,
    start: Instant,
) -> String {
    let file_size = data.len();
    let requires_chunked = media_type.requires_chunked(file_size as u64);
    let strategy = if requires_chunked {
        "chunked"
    } else {
        "simple"
    };
    let segment_count = if requires_chunked {
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
        processing_required: matches!(media_type, MediaType::Gif | MediaType::Video),
        alt_text: alt_text.map(|s| s.to_string()),
    })
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

/// Infer `MediaType` from a file path extension.
pub(crate) fn infer_media_type(path: &str) -> Option<MediaType> {
    let ext = path.rsplit('.').next()?.to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some(MediaType::Image(ImageFormat::Jpeg)),
        "png" => Some(MediaType::Image(ImageFormat::Png)),
        "webp" => Some(MediaType::Image(ImageFormat::Webp)),
        "gif" => Some(MediaType::Gif),
        "mp4" => Some(MediaType::Video),
        _ => None,
    }
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
    use super::*;

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
