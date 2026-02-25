//! Media upload X API tool.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::x_api::types::{ImageFormat, MediaType};

use crate::state::SharedState;

use super::super::response::{ToolMeta, ToolResponse};
use super::not_configured_response;

/// Upload a media file for attachment to tweets.
pub async fn upload_media(state: &SharedState, file_path: &str) -> String {
    let start = Instant::now();
    let client = match state.x_client.as_ref() {
        Some(c) => c,
        None => return not_configured_response(start),
    };

    // Infer media type from file extension.
    let media_type = match infer_media_type(file_path) {
        Some(mt) => mt,
        None => {
            let elapsed = start.elapsed().as_millis() as u64;
            return ToolResponse::error(
                "unsupported_media_type",
                format!(
                    "Unsupported file extension for: {file_path}. \
                     Supported: jpg, jpeg, png, webp, gif, mp4"
                ),
                false,
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
                "file_read_error",
                format!("Failed to read file {file_path}: {e}"),
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
        }
    };

    let file_size = data.len();

    match client.upload_media(&data, media_type).await {
        Ok(media_id) => {
            let elapsed = start.elapsed().as_millis() as u64;
            #[derive(Serialize)]
            struct UploadResult {
                media_id: String,
                media_type: String,
                file_size_bytes: usize,
            }
            ToolResponse::success(UploadResult {
                media_id: media_id.0,
                media_type: media_type.mime_type().to_string(),
                file_size_bytes: file_size,
            })
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ToolResponse::error(
                "media_upload_error",
                format!("Media upload failed: {e}"),
                false,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json()
        }
    }
}

/// Infer `MediaType` from a file path extension.
fn infer_media_type(path: &str) -> Option<MediaType> {
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
