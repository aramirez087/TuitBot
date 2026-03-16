//! X API v1.1 media upload implementation.
//!
//! Supports simple upload for small images and chunked upload for
//! large images, GIFs, and videos. Chunked upload follows the
//! INIT → APPEND → FINALIZE → STATUS polling state machine.

use crate::error::XApiError;
use crate::x_api::types::{MediaId, MediaType};

/// Chunk size for chunked uploads (5 MB).
const CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// Maximum time to wait for media processing (300 seconds).
const MAX_PROCESSING_WAIT_SECS: u64 = 300;

/// Response from the media upload endpoint.
#[derive(Debug, serde::Deserialize)]
struct MediaUploadResponse {
    media_id_string: String,
    #[serde(default)]
    processing_info: Option<ProcessingInfo>,
}

/// Processing status for async media (video, GIF).
#[derive(Debug, serde::Deserialize)]
struct ProcessingInfo {
    state: String,
    #[serde(default)]
    check_after_secs: Option<u64>,
    #[serde(default)]
    error: Option<ProcessingError>,
}

/// Error detail from media processing.
#[derive(Debug, serde::Deserialize)]
struct ProcessingError {
    #[serde(default)]
    message: Option<String>,
}

/// Upload media to X API, routing to simple or chunked upload based on type and size.
pub async fn upload_media(
    client: &reqwest::Client,
    upload_base_url: &str,
    access_token: &str,
    data: &[u8],
    media_type: MediaType,
) -> Result<MediaId, XApiError> {
    let size = data.len() as u64;

    // Validate size
    if size > media_type.max_size() {
        return Err(XApiError::MediaUploadError {
            message: format!(
                "file size {}B exceeds maximum {}B for {}",
                size,
                media_type.max_size(),
                media_type.mime_type()
            ),
        });
    }

    if media_type.requires_chunked(size) {
        chunked_upload(client, upload_base_url, access_token, data, media_type).await
    } else {
        simple_upload(client, upload_base_url, access_token, data, media_type).await
    }
}

/// Simple upload for small images (< 5MB).
async fn simple_upload(
    client: &reqwest::Client,
    upload_base_url: &str,
    access_token: &str,
    data: &[u8],
    media_type: MediaType,
) -> Result<MediaId, XApiError> {
    let url = format!("{}/media/upload.json", upload_base_url);

    let part = reqwest::multipart::Part::bytes(data.to_vec())
        .mime_str(media_type.mime_type())
        .map_err(|e| XApiError::MediaUploadError {
            message: format!("failed to set MIME type: {e}"),
        })?;

    let form = reqwest::multipart::Form::new()
        .text("media_category", media_type.media_category().to_string())
        .part("media_data_part", part);

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| XApiError::Network { source: e })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(XApiError::MediaUploadError {
            message: format!("upload failed (HTTP {status}): {body}"),
        });
    }

    let resp: MediaUploadResponse =
        response
            .json()
            .await
            .map_err(|e| XApiError::MediaUploadError {
                message: format!("failed to parse upload response: {e}"),
            })?;

    Ok(MediaId(resp.media_id_string))
}

/// Chunked upload for large images, GIFs, and videos.
///
/// Follows the state machine: INIT → APPEND (5MB chunks) → FINALIZE → STATUS poll.
async fn chunked_upload(
    client: &reqwest::Client,
    upload_base_url: &str,
    access_token: &str,
    data: &[u8],
    media_type: MediaType,
) -> Result<MediaId, XApiError> {
    let url = format!("{}/media/upload.json", upload_base_url);
    let total_bytes = data.len();

    // INIT
    let init_form = reqwest::multipart::Form::new()
        .text("command", "INIT")
        .text("total_bytes", total_bytes.to_string())
        .text("media_type", media_type.mime_type().to_string())
        .text("media_category", media_type.media_category().to_string());

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(init_form)
        .send()
        .await
        .map_err(|e| XApiError::Network { source: e })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(XApiError::MediaUploadError {
            message: format!("INIT failed (HTTP {status}): {body}"),
        });
    }

    let init_resp: MediaUploadResponse =
        response
            .json()
            .await
            .map_err(|e| XApiError::MediaUploadError {
                message: format!("failed to parse INIT response: {e}"),
            })?;

    let media_id = &init_resp.media_id_string;

    // APPEND — send data in chunks
    for (segment_index, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
        let part = reqwest::multipart::Part::bytes(chunk.to_vec());

        let append_form = reqwest::multipart::Form::new()
            .text("command", "APPEND")
            .text("media_id", media_id.clone())
            .text("segment_index", segment_index.to_string())
            .part("media_data", part);

        let response = client
            .post(&url)
            .bearer_auth(access_token)
            .multipart(append_form)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(XApiError::MediaUploadError {
                message: format!("APPEND segment {segment_index} failed (HTTP {status}): {body}"),
            });
        }
    }

    // FINALIZE
    let finalize_form = reqwest::multipart::Form::new()
        .text("command", "FINALIZE")
        .text("media_id", media_id.clone());

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(finalize_form)
        .send()
        .await
        .map_err(|e| XApiError::Network { source: e })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(XApiError::MediaUploadError {
            message: format!("FINALIZE failed (HTTP {status}): {body}"),
        });
    }

    let finalize_resp: MediaUploadResponse =
        response
            .json()
            .await
            .map_err(|e| XApiError::MediaUploadError {
                message: format!("failed to parse FINALIZE response: {e}"),
            })?;

    // STATUS polling — only needed if processing_info is present
    if let Some(info) = finalize_resp.processing_info {
        poll_processing_status(client, &url, access_token, media_id, info).await?;
    }

    Ok(MediaId(media_id.clone()))
}

/// Poll the STATUS endpoint with exponential backoff until processing completes.
async fn poll_processing_status(
    client: &reqwest::Client,
    url: &str,
    access_token: &str,
    media_id: &str,
    initial_info: ProcessingInfo,
) -> Result<(), XApiError> {
    let mut total_waited: u64 = 0;
    let mut wait_secs = initial_info.check_after_secs.unwrap_or(5);

    if initial_info.state == "succeeded" {
        return Ok(());
    }

    if initial_info.state == "failed" {
        let msg = initial_info
            .error
            .and_then(|e| e.message)
            .unwrap_or_else(|| "unknown processing error".to_string());
        return Err(XApiError::MediaUploadError { message: msg });
    }

    loop {
        if total_waited >= MAX_PROCESSING_WAIT_SECS {
            return Err(XApiError::MediaProcessingTimeout {
                seconds: total_waited,
            });
        }

        tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
        total_waited += wait_secs;

        let response = client
            .get(url)
            .bearer_auth(access_token)
            .query(&[("command", "STATUS"), ("media_id", media_id)])
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(XApiError::MediaUploadError {
                message: format!("STATUS check failed (HTTP {status}): {body}"),
            });
        }

        let resp: MediaUploadResponse =
            response
                .json()
                .await
                .map_err(|e| XApiError::MediaUploadError {
                    message: format!("failed to parse STATUS response: {e}"),
                })?;

        match resp.processing_info {
            Some(info) if info.state == "succeeded" => return Ok(()),
            Some(info) if info.state == "failed" => {
                let msg = info
                    .error
                    .and_then(|e| e.message)
                    .unwrap_or_else(|| "unknown processing error".to_string());
                return Err(XApiError::MediaUploadError { message: msg });
            }
            Some(info) => {
                wait_secs = info.check_after_secs.unwrap_or(wait_secs * 2).min(30);
            }
            None => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn simple_upload_success() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "media_id_string": "123456"
            })))
            .mount(&server)
            .await;

        let data = vec![0u8; 1024]; // 1KB image
        let result = upload_media(
            &client,
            &server.uri(),
            "test-token",
            &data,
            MediaType::Image(crate::x_api::types::ImageFormat::Jpeg),
        )
        .await;

        let media_id = result.expect("upload should succeed");
        assert_eq!(media_id.0, "123456");
    }

    #[tokio::test]
    async fn size_validation_rejects_oversized_file() {
        let client = reqwest::Client::new();
        let data = vec![0u8; 6 * 1024 * 1024]; // 6MB > 5MB limit for images

        let result = upload_media(
            &client,
            "http://unused",
            "test-token",
            &data,
            MediaType::Image(crate::x_api::types::ImageFormat::Jpeg),
        )
        .await;

        match result {
            Err(XApiError::MediaUploadError { message }) => {
                assert!(message.contains("exceeds maximum"));
            }
            other => panic!("expected MediaUploadError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn chunked_upload_init_append_finalize() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        // The upload endpoint handles INIT, APPEND, and FINALIZE via multipart
        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "media_id_string": "chunked_789"
            })))
            .expect(3) // INIT + APPEND + FINALIZE
            .mount(&server)
            .await;

        let data = vec![0u8; 1024]; // Small data but GIF forces chunked
        let result =
            upload_media(&client, &server.uri(), "test-token", &data, MediaType::Gif).await;

        let media_id = result.expect("chunked upload should succeed");
        assert_eq!(media_id.0, "chunked_789");
    }

    #[tokio::test]
    async fn upload_error_response() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(413).set_body_string("Request Entity Too Large"))
            .mount(&server)
            .await;

        let data = vec![0u8; 1024];
        let result = upload_media(
            &client,
            &server.uri(),
            "test-token",
            &data,
            MediaType::Image(crate::x_api::types::ImageFormat::Png),
        )
        .await;

        assert!(matches!(result, Err(XApiError::MediaUploadError { .. })));
    }

    #[tokio::test]
    async fn tweet_with_media_request_body() {
        use crate::x_api::types::{MediaPayload, PostTweetRequest};

        let req = PostTweetRequest {
            text: "Hello with media".to_string(),
            reply: None,
            media: Some(MediaPayload {
                media_ids: vec!["111".to_string(), "222".to_string()],
            }),
            quote_tweet_id: None,
        };

        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("media_ids"));
        assert!(json.contains("111"));
        assert!(json.contains("222"));

        // Verify round-trip
        let parsed: PostTweetRequest = serde_json::from_str(&json).expect("deserialize");
        let media = parsed.media.expect("media should be present");
        assert_eq!(media.media_ids.len(), 2);
    }

    // ── MediaUploadResponse deserialization ──────────────────────

    #[test]
    fn media_upload_response_basic() {
        let json = r#"{"media_id_string": "12345"}"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.media_id_string, "12345");
        assert!(resp.processing_info.is_none());
    }

    #[test]
    fn media_upload_response_with_processing_info() {
        let json = r#"{
            "media_id_string": "67890",
            "processing_info": {
                "state": "pending",
                "check_after_secs": 5
            }
        }"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.media_id_string, "67890");
        let info = resp.processing_info.unwrap();
        assert_eq!(info.state, "pending");
        assert_eq!(info.check_after_secs, Some(5));
    }

    #[test]
    fn media_upload_response_processing_succeeded() {
        let json = r#"{
            "media_id_string": "111",
            "processing_info": {
                "state": "succeeded"
            }
        }"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.processing_info.unwrap().state, "succeeded");
    }

    #[test]
    fn media_upload_response_processing_failed_with_error() {
        let json = r#"{
            "media_id_string": "222",
            "processing_info": {
                "state": "failed",
                "error": {
                    "message": "InvalidMedia"
                }
            }
        }"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        let info = resp.processing_info.unwrap();
        assert_eq!(info.state, "failed");
        assert_eq!(info.error.unwrap().message.as_deref(), Some("InvalidMedia"));
    }

    // ── Size validation tests ───────────────────────────────────

    #[tokio::test]
    async fn size_validation_gif_under_limit() {
        // GIF limit is 15MB; 1KB should be fine (but will fail at network level)
        let client = reqwest::Client::new();
        let data = vec![0u8; 1024];
        // GIF requires chunked upload so this will try to POST;
        // without a server it will fail with network error, not size error
        let result =
            upload_media(&client, "http://127.0.0.1:1", "tok", &data, MediaType::Gif).await;
        assert!(
            !matches!(
                result,
                Err(XApiError::MediaUploadError { ref message }) if message.contains("exceeds maximum")
            ),
            "should not reject 1KB GIF for size"
        );
    }

    #[tokio::test]
    async fn size_validation_video_over_limit() {
        let client = reqwest::Client::new();
        // Video limit is 512MB; create a fake > 512MB reference
        // Actually we can't allocate that much, but we can test the boundary
        // with a smaller type. Image limit is 5MB.
        let data = vec![0u8; 6 * 1024 * 1024]; // 6MB > 5MB
        let result = upload_media(
            &client,
            "http://unused",
            "tok",
            &data,
            MediaType::Image(crate::x_api::types::ImageFormat::Png),
        )
        .await;
        assert!(matches!(result, Err(XApiError::MediaUploadError { .. })));
    }

    // ── Chunked upload with mock: multi-segment ─────────────────

    #[tokio::test]
    async fn chunked_upload_video_multiple_segments() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        // 6MB video → 2 segments (5MB + 1MB)
        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "media_id_string": "video_99"
            })))
            .expect(4) // INIT + 2x APPEND + FINALIZE
            .mount(&server)
            .await;

        let data = vec![0u8; 6 * 1024 * 1024]; // 6MB
        let result = upload_media(
            &client,
            &server.uri(),
            "test-token",
            &data,
            MediaType::Video,
        )
        .await;

        let media_id = result.expect("chunked upload should succeed");
        assert_eq!(media_id.0, "video_99");
    }

    // ── Processing status: immediate success ────────────────────

    #[tokio::test]
    async fn chunked_upload_with_processing_succeeded() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        // INIT + APPEND → normal response
        // FINALIZE → response with processing_info.state = "succeeded"
        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "media_id_string": "proc_ok",
                "processing_info": {
                    "state": "succeeded"
                }
            })))
            .mount(&server)
            .await;

        let data = vec![0u8; 1024];
        let result =
            upload_media(&client, &server.uri(), "test-token", &data, MediaType::Gif).await;

        let media_id = result.expect("should succeed");
        assert_eq!(media_id.0, "proc_ok");
    }

    // ── Simple upload HTTP error ────────────────────────────────

    #[tokio::test]
    async fn simple_upload_400_error() {
        let server = MockServer::start().await;
        let client = reqwest::Client::new();

        Mock::given(method("POST"))
            .and(path("/media/upload.json"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request: invalid media"))
            .mount(&server)
            .await;

        let data = vec![0u8; 1024];
        let result = upload_media(
            &client,
            &server.uri(),
            "test-token",
            &data,
            MediaType::Image(crate::x_api::types::ImageFormat::Jpeg),
        )
        .await;

        match result {
            Err(XApiError::MediaUploadError { message }) => {
                assert!(message.contains("400"));
            }
            other => panic!("expected MediaUploadError, got: {other:?}"),
        }
    }

    // ── PostTweetRequest without media ──────────────────────────

    #[test]
    fn post_tweet_request_no_media() {
        use crate::x_api::types::PostTweetRequest;

        let req = PostTweetRequest {
            text: "Just text".to_string(),
            reply: None,
            media: None,
            quote_tweet_id: None,
        };

        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("Just text"));

        let parsed: PostTweetRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.media.is_none());
    }

    // ── ProcessingInfo deserialization ─────────────────────────────

    #[test]
    fn processing_info_pending() {
        let json = r#"{"state": "pending", "check_after_secs": 10}"#;
        let info: ProcessingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.state, "pending");
        assert_eq!(info.check_after_secs, Some(10));
        assert!(info.error.is_none());
    }

    #[test]
    fn processing_info_in_progress() {
        let json = r#"{"state": "in_progress", "check_after_secs": 5}"#;
        let info: ProcessingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.state, "in_progress");
    }

    #[test]
    fn processing_info_succeeded_no_extras() {
        let json = r#"{"state": "succeeded"}"#;
        let info: ProcessingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.state, "succeeded");
        assert!(info.check_after_secs.is_none());
        assert!(info.error.is_none());
    }

    #[test]
    fn processing_info_failed_with_message() {
        let json = r#"{
            "state": "failed",
            "error": {"message": "InvalidMedia: unsupported format"}
        }"#;
        let info: ProcessingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.state, "failed");
        let err = info.error.unwrap();
        assert_eq!(
            err.message.as_deref(),
            Some("InvalidMedia: unsupported format")
        );
    }

    #[test]
    fn processing_info_failed_no_message() {
        let json = r#"{"state": "failed", "error": {}}"#;
        let info: ProcessingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.state, "failed");
        assert!(info.error.unwrap().message.is_none());
    }

    // ── ProcessingError deserialization ────────────────────────────

    #[test]
    fn processing_error_empty() {
        let json = r#"{}"#;
        let err: ProcessingError = serde_json::from_str(json).unwrap();
        assert!(err.message.is_none());
    }

    #[test]
    fn processing_error_with_message() {
        let json = r#"{"message": "file too large"}"#;
        let err: ProcessingError = serde_json::from_str(json).unwrap();
        assert_eq!(err.message.as_deref(), Some("file too large"));
    }

    // ── MediaUploadResponse edge cases ────────────────────────────

    #[test]
    fn media_upload_response_with_unknown_state() {
        let json = r#"{
            "media_id_string": "999",
            "processing_info": {
                "state": "unknown_state",
                "check_after_secs": 3
            }
        }"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.media_id_string, "999");
        let info = resp.processing_info.unwrap();
        assert_eq!(info.state, "unknown_state");
    }

    #[test]
    fn media_upload_response_large_media_id() {
        let json = r#"{"media_id_string": "1234567890123456789"}"#;
        let resp: MediaUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.media_id_string, "1234567890123456789");
    }

    // ── Constants ─────────────────────────────────────────────────

    #[test]
    fn chunk_size_is_5mb() {
        assert_eq!(CHUNK_SIZE, 5 * 1024 * 1024);
    }

    #[test]
    fn max_processing_wait_is_300s() {
        assert_eq!(MAX_PROCESSING_WAIT_SECS, 300);
    }

    // ── MediaType tests ───────────────────────────────────────────

    #[test]
    fn media_type_requires_chunked() {
        use crate::x_api::types::ImageFormat;
        // Images < 5MB should not require chunked
        assert!(!MediaType::Image(ImageFormat::Jpeg).requires_chunked(1024));
        // GIF always requires chunked
        assert!(MediaType::Gif.requires_chunked(1024));
        // Video always requires chunked
        assert!(MediaType::Video.requires_chunked(1024));
    }

    #[test]
    fn media_type_max_size() {
        use crate::x_api::types::ImageFormat;
        assert!(MediaType::Image(ImageFormat::Jpeg).max_size() > 0);
        assert!(MediaType::Gif.max_size() > 0);
        assert!(MediaType::Video.max_size() > MediaType::Gif.max_size());
    }

    #[test]
    fn media_type_mime_type() {
        use crate::x_api::types::ImageFormat;
        assert_eq!(
            MediaType::Image(ImageFormat::Jpeg).mime_type(),
            "image/jpeg"
        );
        assert_eq!(MediaType::Image(ImageFormat::Png).mime_type(), "image/png");
        assert_eq!(
            MediaType::Image(ImageFormat::Webp).mime_type(),
            "image/webp"
        );
        assert_eq!(MediaType::Gif.mime_type(), "image/gif");
        assert_eq!(MediaType::Video.mime_type(), "video/mp4");
    }

    #[test]
    fn media_type_media_category() {
        use crate::x_api::types::ImageFormat;
        assert_eq!(
            MediaType::Image(ImageFormat::Jpeg).media_category(),
            "tweet_image"
        );
        assert_eq!(MediaType::Gif.media_category(), "tweet_gif");
        assert_eq!(MediaType::Video.media_category(), "tweet_video");
    }

    // ── PostTweetRequest with quote_tweet_id ──────────────────────

    #[test]
    fn post_tweet_request_with_quote() {
        use crate::x_api::types::PostTweetRequest;

        let req = PostTweetRequest {
            text: "Check this out".to_string(),
            reply: None,
            media: None,
            quote_tweet_id: Some("999".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("quote_tweet_id"));
        assert!(json.contains("999"));
    }

    #[test]
    fn post_tweet_request_with_reply() {
        use crate::x_api::types::{PostTweetRequest, ReplyTo};

        let req = PostTweetRequest {
            text: "Great point!".to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: "456".to_string(),
            }),
            media: None,
            quote_tweet_id: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("in_reply_to_tweet_id"));
        assert!(json.contains("456"));
    }
}
