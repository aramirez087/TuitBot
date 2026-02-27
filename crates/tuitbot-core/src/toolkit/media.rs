//! Stateless media operations over `&dyn XApiClient`.
//!
//! Media type inference, size validation, and raw upload.
//! File I/O, hashing, DB tracking, and idempotency belong in the workflow layer.

use super::ToolkitError;
use crate::x_api::types::{ImageFormat, MediaId, MediaType};
use crate::x_api::XApiClient;

/// Upload media bytes to X API.
///
/// The caller is responsible for reading the file and inferring the media type.
/// This function only handles the raw upload via the client trait.
pub async fn upload_media(
    client: &dyn XApiClient,
    data: &[u8],
    media_type: MediaType,
) -> Result<MediaId, ToolkitError> {
    validate_media_size(data.len() as u64, media_type)?;
    Ok(client.upload_media(data, media_type).await?)
}

/// Infer `MediaType` from a file path extension.
///
/// Supports: jpg, jpeg, png, webp, gif, mp4.
pub fn infer_media_type(path: &str) -> Option<MediaType> {
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

/// Validate media data size against X API limits for the given type.
pub fn validate_media_size(size: u64, media_type: MediaType) -> Result<(), ToolkitError> {
    let max = media_type.max_size();
    if size > max {
        return Err(ToolkitError::MediaTooLarge {
            size,
            max,
            media_type: media_type.mime_type().to_string(),
        });
    }
    Ok(())
}

/// Whether this media type requires processing after upload (GIF/video).
pub fn requires_processing(media_type: MediaType) -> bool {
    matches!(media_type, MediaType::Gif | MediaType::Video)
}

/// Whether this media type requires chunked upload for the given size.
pub fn requires_chunked(media_type: MediaType, size: u64) -> bool {
    media_type.requires_chunked(size)
}

/// Determine the upload strategy string for a given media type and size.
pub fn upload_strategy(media_type: MediaType, size: u64) -> &'static str {
    if media_type.requires_chunked(size) {
        "chunked"
    } else {
        "simple"
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
    fn infer_webp() {
        assert_eq!(
            infer_media_type("pic.webp"),
            Some(MediaType::Image(ImageFormat::Webp))
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

    #[test]
    fn validate_size_ok() {
        let mt = MediaType::Image(ImageFormat::Jpeg);
        assert!(validate_media_size(1024, mt).is_ok());
    }

    #[test]
    fn validate_size_too_large() {
        let mt = MediaType::Image(ImageFormat::Jpeg);
        let e = validate_media_size(10 * 1024 * 1024, mt).unwrap_err();
        assert!(matches!(e, ToolkitError::MediaTooLarge { .. }));
    }

    #[test]
    fn validate_video_size_ok() {
        assert!(validate_media_size(100 * 1024 * 1024, MediaType::Video).is_ok());
    }

    #[test]
    fn validate_video_size_too_large() {
        let e = validate_media_size(600 * 1024 * 1024, MediaType::Video).unwrap_err();
        assert!(matches!(e, ToolkitError::MediaTooLarge { .. }));
    }

    #[test]
    fn processing_required_for_gif_and_video() {
        assert!(requires_processing(MediaType::Gif));
        assert!(requires_processing(MediaType::Video));
        assert!(!requires_processing(MediaType::Image(ImageFormat::Jpeg)));
    }

    #[test]
    fn upload_strategy_simple_for_small_image() {
        assert_eq!(
            upload_strategy(MediaType::Image(ImageFormat::Jpeg), 1024),
            "simple"
        );
    }

    #[test]
    fn upload_strategy_chunked_for_gif() {
        assert_eq!(upload_strategy(MediaType::Gif, 1024), "chunked");
    }
}
