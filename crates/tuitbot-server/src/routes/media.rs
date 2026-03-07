//! Media upload and serving endpoints.

use std::sync::Arc;

use axum::extract::{Multipart, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::storage::media;

use crate::account::{require_mutate, AccountContext};
use crate::error::ApiError;
use crate::state::AppState;

/// `POST /api/media/upload` — upload a media file.
///
/// Accepts multipart form data with a `file` field.
/// Returns `{ id, path, media_type, size }`.
pub async fn upload(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    require_mutate(&ctx)?;
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("invalid multipart data: {e}")))?
        .ok_or_else(|| ApiError::BadRequest("no file field in request".to_string()))?;

    let filename = field.file_name().unwrap_or("upload.bin").to_string();
    let content_type = field.content_type().map(|s| s.to_string());

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(format!("failed to read file data: {e}")))?;

    let media_type =
        media::detect_media_type(&filename, content_type.as_deref()).ok_or_else(|| {
            ApiError::BadRequest(
                "unsupported media type; accepted: jpeg, png, webp, gif, mp4".to_string(),
            )
        })?;

    // Validate size.
    if data.len() as u64 > media_type.max_size() {
        return Err(ApiError::BadRequest(format!(
            "file size {}B exceeds maximum {}B for {}",
            data.len(),
            media_type.max_size(),
            media_type.mime_type()
        )));
    }

    let local = media::store_media(&state.data_dir, &data, &filename, media_type)
        .await
        .map_err(|e| ApiError::Internal(format!("failed to store media: {e}")))?;

    // Trigger cleanup in background if media folder exceeds threshold.
    let data_dir = state.data_dir.clone();
    let db = state.db.clone();
    tokio::spawn(async move {
        if let Err(e) = media::cleanup_if_over_threshold(&data_dir, &db).await {
            tracing::warn!(error = %e, "Media cleanup failed");
        }
    });

    Ok(Json(json!({
        "path": local.path,
        "media_type": media_type.mime_type(),
        "size": local.size,
    })))
}

/// Query params for serving media files.
#[derive(Deserialize)]
pub struct MediaFileQuery {
    /// Path to the media file.
    pub path: String,
}

/// `GET /api/media/file?path=...` — serve a local media file.
pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    _ctx: AccountContext,
    Query(params): Query<MediaFileQuery>,
) -> Result<Response, ApiError> {
    // Path traversal protection.
    if !media::is_safe_media_path(&params.path, &state.data_dir) {
        return Err(ApiError::BadRequest("invalid media path".to_string()));
    }

    let data = media::read_media(&params.path)
        .await
        .map_err(|e| ApiError::NotFound(format!("media file not found: {e}")))?;

    let content_type = media::detect_media_type(&params.path, None)
        .map(|mt| mt.mime_type())
        .unwrap_or("application/octet-stream");

    Ok(([(header::CONTENT_TYPE, content_type)], data).into_response())
}
