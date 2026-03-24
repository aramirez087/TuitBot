//! Media, request, and API response types for X API v2.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported image formats for media upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
}

/// Media type for upload to X API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Image(ImageFormat),
    Gif,
    Video,
}

impl MediaType {
    /// Returns the MIME type string for this media type.
    pub fn mime_type(&self) -> &'static str {
        match self {
            MediaType::Image(ImageFormat::Jpeg) => "image/jpeg",
            MediaType::Image(ImageFormat::Png) => "image/png",
            MediaType::Image(ImageFormat::Webp) => "image/webp",
            MediaType::Gif => "image/gif",
            MediaType::Video => "video/mp4",
        }
    }

    /// Returns the maximum file size in bytes allowed by X API.
    pub fn max_size(&self) -> u64 {
        match self {
            MediaType::Image(_) => 5 * 1024 * 1024, // 5 MB
            MediaType::Gif => 15 * 1024 * 1024,     // 15 MB
            MediaType::Video => 512 * 1024 * 1024,  // 512 MB
        }
    }

    /// Returns the media_category string for X API upload.
    pub fn media_category(&self) -> &'static str {
        match self {
            MediaType::Image(_) => "tweet_image",
            MediaType::Gif => "tweet_gif",
            MediaType::Video => "tweet_video",
        }
    }

    /// Whether this media type requires chunked upload for the given size.
    pub fn requires_chunked(&self, size: u64) -> bool {
        match self {
            MediaType::Image(_) => size > 5 * 1024 * 1024,
            MediaType::Gif | MediaType::Video => true,
        }
    }
}

/// A media ID returned by the X API upload endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaId(pub String);

/// Media attachment payload for tweet requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPayload {
    /// List of media IDs to attach to the tweet.
    pub media_ids: Vec<String>,
}

/// Parsed rate limit information from X API response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Remaining requests in the current rate limit window.
    pub remaining: Option<u64>,
    /// UTC epoch second when the rate limit window resets.
    pub reset_at: Option<u64>,
}

/// Raw HTTP response from the X API, returned by [`XApiClient::raw_request`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawApiResponse {
    /// HTTP status code.
    pub status: u16,
    /// Selected response headers (rate limit headers, content-type).
    pub headers: HashMap<String, String>,
    /// Response body as a string.
    pub body: String,
    /// Parsed rate limit information from response headers.
    #[serde(skip)]
    pub rate_limit: Option<RateLimitInfo>,
}

/// X API error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XApiErrorResponse {
    /// Error detail message.
    #[serde(default)]
    pub detail: Option<String>,
    /// Error title.
    #[serde(default)]
    pub title: Option<String>,
    /// Error type identifier.
    #[serde(default, rename = "type")]
    pub error_type: Option<String>,
    /// HTTP status code.
    #[serde(default)]
    pub status: Option<u16>,
}

/// Request body for liking a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeTweetRequest {
    /// The tweet ID to like.
    pub tweet_id: String,
}

/// Request body for bookmarking a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkTweetRequest {
    /// The tweet ID to bookmark.
    pub tweet_id: String,
}

/// Request body for retweeting a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetweetRequest {
    /// The tweet ID to retweet.
    pub tweet_id: String,
}

/// Response from action endpoints (like, follow, unfollow) via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResultResponse {
    /// The action result data.
    pub data: ActionResultData,
}

/// Data from an action endpoint (like, follow, unfollow, retweet, bookmark).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResultData {
    /// Whether the action was successful (liked/following/retweeted/bookmarked).
    #[serde(
        alias = "liked",
        alias = "following",
        alias = "retweeted",
        alias = "bookmarked"
    )]
    pub result: bool,
}
