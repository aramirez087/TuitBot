//! X API v2 request and response types.
//!
//! All types derive Serde traits and match the X API v2 JSON field names.
//! Tweet IDs are strings because X API v2 returns them as strings and
//! some IDs exceed `i64` range.

use serde::{Deserialize, Serialize};

/// A tweet returned by the X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    /// Unique tweet ID (string format per X API v2).
    pub id: String,
    /// Full text content of the tweet.
    pub text: String,
    /// ID of the user who posted the tweet.
    pub author_id: String,
    /// ISO-8601 timestamp when the tweet was created.
    #[serde(default)]
    pub created_at: String,
    /// Engagement metrics for the tweet.
    #[serde(default)]
    pub public_metrics: PublicMetrics,
    /// Conversation thread ID (matches the root tweet's ID).
    #[serde(default)]
    pub conversation_id: Option<String>,
}

/// Public engagement metrics for a tweet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublicMetrics {
    /// Number of retweets.
    #[serde(default)]
    pub retweet_count: u64,
    /// Number of replies.
    #[serde(default)]
    pub reply_count: u64,
    /// Number of likes.
    #[serde(default)]
    pub like_count: u64,
    /// Number of quote tweets.
    #[serde(default)]
    pub quote_count: u64,
    /// Number of impressions (may be absent depending on tweet type).
    #[serde(default)]
    pub impression_count: u64,
    /// Number of bookmarks (may be absent depending on tweet type).
    #[serde(default)]
    pub bookmark_count: u64,
}

/// An X API user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub id: String,
    /// @username handle (without the @).
    pub username: String,
    /// Display name.
    pub name: String,
    /// Profile image URL.
    #[serde(default)]
    pub profile_image_url: Option<String>,
    /// Bio / profile description.
    #[serde(default)]
    pub description: Option<String>,
    /// Self-reported location string.
    #[serde(default)]
    pub location: Option<String>,
    /// Profile URL (often a t.co shortened link).
    #[serde(default)]
    pub url: Option<String>,
    /// User engagement metrics.
    #[serde(default)]
    pub public_metrics: UserMetrics,
}

/// Public metrics for a user profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetrics {
    /// Number of followers.
    #[serde(default)]
    pub followers_count: u64,
    /// Number of accounts being followed.
    #[serde(default)]
    pub following_count: u64,
    /// Total number of tweets posted.
    #[serde(default)]
    pub tweet_count: u64,
}

/// Response from the X API v2 tweet search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// List of matching tweets.
    #[serde(default)]
    pub data: Vec<Tweet>,
    /// Expanded objects referenced by tweets (users, etc.).
    #[serde(default)]
    pub includes: Option<Includes>,
    /// Pagination and result metadata.
    pub meta: SearchMeta,
}

/// Expanded objects included in search/mention responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Includes {
    /// User objects referenced by `author_id` in tweets.
    #[serde(default)]
    pub users: Vec<User>,
}

/// Metadata from a search or mention response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMeta {
    /// ID of the newest tweet in the result set.
    #[serde(default)]
    pub newest_id: Option<String>,
    /// ID of the oldest tweet in the result set.
    #[serde(default)]
    pub oldest_id: Option<String>,
    /// Number of tweets returned in this response.
    #[serde(default)]
    pub result_count: u32,
    /// Pagination token for fetching the next page.
    #[serde(default)]
    pub next_token: Option<String>,
}

/// Response from the X API v2 mentions endpoint.
///
/// Structurally identical to `SearchResponse`.
pub type MentionResponse = SearchResponse;

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

/// Request body for posting a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTweetRequest {
    /// Tweet text content.
    pub text: String,
    /// Optional reply configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyTo>,
    /// Optional media attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaPayload>,
    /// Optional tweet ID to quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_tweet_id: Option<String>,
}

/// Specifies which tweet this is a reply to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyTo {
    /// The ID of the tweet being replied to.
    pub in_reply_to_tweet_id: String,
}

/// Response from posting a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTweetResponse {
    /// The posted tweet data.
    pub data: PostedTweet,
}

/// A tweet that was successfully posted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostedTweet {
    /// The new tweet's ID.
    pub id: String,
    /// The tweet text as stored by X.
    pub text: String,
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
    pub headers: std::collections::HashMap<String, String>,
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

/// Wrapper for single-tweet responses with expansion support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleTweetResponse {
    /// The tweet data.
    pub data: Tweet,
    /// Expanded objects.
    #[serde(default)]
    pub includes: Option<Includes>,
}

/// Wrapper for user/me responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    /// The user data.
    pub data: User,
}

/// Response from endpoints returning a list of users (followers, following, batch lookup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    /// List of users.
    #[serde(default)]
    pub data: Vec<User>,
    /// Pagination and result metadata.
    pub meta: UsersMeta,
}

/// Metadata from a users list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersMeta {
    /// Number of users returned in this response.
    #[serde(default)]
    pub result_count: u32,
    /// Pagination token for fetching the next page.
    #[serde(default)]
    pub next_token: Option<String>,
}

/// Request body for liking a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeTweetRequest {
    /// The tweet ID to like.
    pub tweet_id: String,
}

/// Request body for following a user via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUserRequest {
    /// The target user ID to follow.
    pub target_user_id: String,
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

/// Response from deleting a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTweetResponse {
    /// The deletion result data.
    pub data: DeleteTweetData,
}

/// Data from a tweet deletion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTweetData {
    /// Whether the tweet was successfully deleted.
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_tweet() {
        let json = r#"{
            "id": "1234567890",
            "text": "Hello world",
            "author_id": "987654321",
            "created_at": "2026-02-21T12:00:00.000Z",
            "public_metrics": {
                "retweet_count": 5,
                "reply_count": 2,
                "like_count": 10,
                "quote_count": 1,
                "impression_count": 500,
                "bookmark_count": 3
            },
            "conversation_id": "1234567890"
        }"#;

        let tweet: Tweet = serde_json::from_str(json).expect("deserialize tweet");
        assert_eq!(tweet.id, "1234567890");
        assert_eq!(tweet.text, "Hello world");
        assert_eq!(tweet.public_metrics.like_count, 10);
        assert_eq!(tweet.conversation_id, Some("1234567890".to_string()));
    }

    #[test]
    fn deserialize_tweet_missing_optional_fields() {
        let json = r#"{
            "id": "123",
            "text": "Hello",
            "author_id": "456"
        }"#;

        let tweet: Tweet = serde_json::from_str(json).expect("deserialize");
        assert_eq!(tweet.public_metrics.like_count, 0);
        assert!(tweet.conversation_id.is_none());
        assert!(tweet.created_at.is_empty());
    }

    #[test]
    fn deserialize_search_response() {
        let json = r#"{
            "data": [
                {
                    "id": "1",
                    "text": "Tweet 1",
                    "author_id": "a1"
                }
            ],
            "includes": {
                "users": [
                    {
                        "id": "a1",
                        "username": "user1",
                        "name": "User One",
                        "public_metrics": {
                            "followers_count": 100,
                            "following_count": 50,
                            "tweet_count": 200
                        }
                    }
                ]
            },
            "meta": {
                "newest_id": "1",
                "oldest_id": "1",
                "result_count": 1,
                "next_token": "abc123"
            }
        }"#;

        let resp: SearchResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "1");
        let users = resp.includes.expect("includes");
        assert_eq!(users.users[0].username, "user1");
        assert_eq!(users.users[0].public_metrics.followers_count, 100);
        assert_eq!(resp.meta.result_count, 1);
        assert_eq!(resp.meta.next_token, Some("abc123".to_string()));
    }

    #[test]
    fn deserialize_search_response_empty() {
        let json = r#"{
            "meta": {
                "result_count": 0
            }
        }"#;

        let resp: SearchResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.data.is_empty());
        assert!(resp.includes.is_none());
        assert_eq!(resp.meta.result_count, 0);
    }

    #[test]
    fn serialize_post_tweet_request() {
        let req = PostTweetRequest {
            text: "Hello!".to_string(),
            reply: None,
            media: None,
            quote_tweet_id: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("reply"));
        assert!(!json.contains("media"));
        assert!(!json.contains("quote_tweet_id"));

        let req_reply = PostTweetRequest {
            text: "Nice!".to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: "999".to_string(),
            }),
            media: None,
            quote_tweet_id: None,
        };
        let json = serde_json::to_string(&req_reply).expect("serialize");
        assert!(json.contains("in_reply_to_tweet_id"));
        assert!(json.contains("999"));
    }

    #[test]
    fn serialize_post_tweet_request_with_quote() {
        let req = PostTweetRequest {
            text: "Great thread!".to_string(),
            reply: None,
            media: None,
            quote_tweet_id: Some("qt_123".to_string()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("quote_tweet_id"));
        assert!(json.contains("qt_123"));
        assert!(!json.contains("reply"));
    }

    #[test]
    fn serialize_post_tweet_request_with_media() {
        let req = PostTweetRequest {
            text: "Check this out!".to_string(),
            reply: None,
            media: Some(MediaPayload {
                media_ids: vec!["12345".to_string(), "67890".to_string()],
            }),
            quote_tweet_id: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("media_ids"));
        assert!(json.contains("12345"));
        assert!(json.contains("67890"));
    }

    #[test]
    fn media_type_properties() {
        let jpeg = MediaType::Image(ImageFormat::Jpeg);
        assert_eq!(jpeg.mime_type(), "image/jpeg");
        assert_eq!(jpeg.max_size(), 5 * 1024 * 1024);
        assert_eq!(jpeg.media_category(), "tweet_image");
        assert!(!jpeg.requires_chunked(1024));

        let gif = MediaType::Gif;
        assert_eq!(gif.mime_type(), "image/gif");
        assert_eq!(gif.max_size(), 15 * 1024 * 1024);
        assert!(gif.requires_chunked(1024));

        let video = MediaType::Video;
        assert_eq!(video.mime_type(), "video/mp4");
        assert_eq!(video.max_size(), 512 * 1024 * 1024);
        assert!(video.requires_chunked(1024));
    }

    #[test]
    fn deserialize_post_tweet_response() {
        let json = r#"{
            "data": {
                "id": "111",
                "text": "My tweet"
            }
        }"#;

        let resp: PostTweetResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.data.id, "111");
        assert_eq!(resp.data.text, "My tweet");
    }

    #[test]
    fn deserialize_error_response() {
        let json = r#"{
            "detail": "Too Many Requests",
            "title": "Too Many Requests",
            "type": "about:blank",
            "status": 429
        }"#;

        let err: XApiErrorResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(err.detail, Some("Too Many Requests".to_string()));
        assert_eq!(err.status, Some(429));
    }

    #[test]
    fn deserialize_users_response() {
        let json = r#"{
            "data": [
                {
                    "id": "u1",
                    "username": "alice",
                    "name": "Alice",
                    "public_metrics": {
                        "followers_count": 500,
                        "following_count": 200,
                        "tweet_count": 1000
                    }
                },
                {
                    "id": "u2",
                    "username": "bob",
                    "name": "Bob"
                }
            ],
            "meta": {
                "result_count": 2,
                "next_token": "page2"
            }
        }"#;

        let resp: UsersResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].username, "alice");
        assert_eq!(resp.data[0].public_metrics.followers_count, 500);
        assert_eq!(resp.data[1].username, "bob");
        assert_eq!(resp.meta.result_count, 2);
        assert_eq!(resp.meta.next_token, Some("page2".to_string()));
    }

    #[test]
    fn deserialize_users_response_empty() {
        let json = r#"{
            "meta": {
                "result_count": 0
            }
        }"#;

        let resp: UsersResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.data.is_empty());
        assert_eq!(resp.meta.result_count, 0);
        assert!(resp.meta.next_token.is_none());
    }

    #[test]
    fn action_result_data_bookmarked_alias() {
        let json = r#"{"bookmarked": true}"#;
        let data: ActionResultData = serde_json::from_str(json).expect("deserialize");
        assert!(data.result);
    }

    #[test]
    fn deserialize_user_response() {
        let json = r#"{
            "data": {
                "id": "123",
                "username": "testuser",
                "name": "Test User",
                "public_metrics": {
                    "followers_count": 1000,
                    "following_count": 500,
                    "tweet_count": 5000
                }
            }
        }"#;

        let resp: UserResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.data.username, "testuser");
        assert_eq!(resp.data.public_metrics.followers_count, 1000);
    }

    #[test]
    fn deserialize_user_with_profile_fields() {
        let json = r#"{
            "data": {
                "id": "123",
                "username": "testuser",
                "name": "Test User",
                "description": "Building cool stuff | Rust enthusiast",
                "location": "San Francisco, CA",
                "url": "https://t.co/abc123",
                "profile_image_url": "https://pbs.twimg.com/profile_images/123/photo.jpg",
                "public_metrics": {
                    "followers_count": 1000,
                    "following_count": 500,
                    "tweet_count": 5000
                }
            }
        }"#;
        let resp: UserResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(
            resp.data.description.as_deref(),
            Some("Building cool stuff | Rust enthusiast")
        );
        assert_eq!(resp.data.location.as_deref(), Some("San Francisco, CA"));
        assert_eq!(resp.data.url.as_deref(), Some("https://t.co/abc123"));
    }

    #[test]
    fn deserialize_user_without_profile_fields() {
        let json = r#"{"data":{"id":"1","username":"u","name":"N"}}"#;
        let resp: UserResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.data.description.is_none());
        assert!(resp.data.location.is_none());
        assert!(resp.data.url.is_none());
    }

    // --- Serde roundtrip coverage for remaining types ---

    #[test]
    fn tweet_serde_roundtrip() {
        let json = r#"{"id":"99","text":"roundtrip","author_id":"7","created_at":"2026-01-01T00:00:00Z","public_metrics":{"retweet_count":1,"reply_count":2,"like_count":3,"quote_count":4,"impression_count":5,"bookmark_count":6},"conversation_id":"99"}"#;
        let tweet: Tweet = serde_json::from_str(json).unwrap();
        assert_eq!(tweet.id, "99");
        assert_eq!(tweet.public_metrics.bookmark_count, 6);
        let back = serde_json::to_string(&tweet).unwrap();
        let re: Tweet = serde_json::from_str(&back).unwrap();
        assert_eq!(re.text, "roundtrip");
    }

    #[test]
    fn public_metrics_default() {
        let pm = PublicMetrics::default();
        assert_eq!(pm.retweet_count, 0);
        assert_eq!(pm.reply_count, 0);
        assert_eq!(pm.like_count, 0);
        assert_eq!(pm.quote_count, 0);
        assert_eq!(pm.impression_count, 0);
        assert_eq!(pm.bookmark_count, 0);
    }

    #[test]
    fn user_metrics_default() {
        let um = UserMetrics::default();
        assert_eq!(um.followers_count, 0);
        assert_eq!(um.following_count, 0);
        assert_eq!(um.tweet_count, 0);
    }

    #[test]
    fn user_metrics_serde_roundtrip() {
        let um = UserMetrics {
            followers_count: 100,
            following_count: 50,
            tweet_count: 200,
        };
        let json = serde_json::to_string(&um).unwrap();
        let back: UserMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(back.followers_count, 100);
        assert_eq!(back.following_count, 50);
        assert_eq!(back.tweet_count, 200);
    }

    #[test]
    fn includes_default() {
        let inc = Includes::default();
        assert!(inc.users.is_empty());
    }

    #[test]
    fn includes_serde_roundtrip() {
        let inc = Includes {
            users: vec![User {
                id: "1".into(),
                username: "u".into(),
                name: "N".into(),
                profile_image_url: None,
                description: None,
                location: None,
                url: None,
                public_metrics: UserMetrics::default(),
            }],
        };
        let json = serde_json::to_string(&inc).unwrap();
        let back: Includes = serde_json::from_str(&json).unwrap();
        assert_eq!(back.users.len(), 1);
    }

    #[test]
    fn user_serde_roundtrip() {
        let user = User {
            id: "42".into(),
            username: "alice".into(),
            name: "Alice".into(),
            profile_image_url: Some("https://img.example.com/a.jpg".into()),
            description: Some("dev".into()),
            location: Some("NYC".into()),
            url: Some("https://example.com".into()),
            public_metrics: UserMetrics {
                followers_count: 10,
                following_count: 5,
                tweet_count: 30,
            },
        };
        let json = serde_json::to_string(&user).unwrap();
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "42");
        assert_eq!(back.description.as_deref(), Some("dev"));
    }

    #[test]
    fn search_meta_serde_roundtrip() {
        let meta = SearchMeta {
            newest_id: Some("n1".into()),
            oldest_id: Some("o1".into()),
            result_count: 5,
            next_token: Some("tok".into()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: SearchMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result_count, 5);
        assert_eq!(back.next_token.as_deref(), Some("tok"));
    }

    #[test]
    fn media_payload_serde_roundtrip() {
        let mp = MediaPayload {
            media_ids: vec!["a".into(), "b".into()],
        };
        let json = serde_json::to_string(&mp).unwrap();
        let back: MediaPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.media_ids.len(), 2);
    }

    #[test]
    fn reply_to_serde_roundtrip() {
        let rt = ReplyTo {
            in_reply_to_tweet_id: "999".into(),
        };
        let json = serde_json::to_string(&rt).unwrap();
        let back: ReplyTo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.in_reply_to_tweet_id, "999");
    }

    #[test]
    fn posted_tweet_serde_roundtrip() {
        let pt = PostedTweet {
            id: "100".into(),
            text: "posted".into(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let back: PostedTweet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "100");
    }

    #[test]
    fn post_tweet_response_serde_roundtrip() {
        let resp = PostTweetResponse {
            data: PostedTweet {
                id: "200".into(),
                text: "hello".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: PostTweetResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.data.id, "200");
    }

    #[test]
    fn raw_api_response_serde_roundtrip() {
        let raw = RawApiResponse {
            status: 200,
            headers: std::collections::HashMap::from([("x-rate".into(), "100".into())]),
            body: r#"{"ok":true}"#.into(),
            rate_limit: None,
        };
        let json = serde_json::to_string(&raw).unwrap();
        let back: RawApiResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, 200);
        assert!(back.body.contains("ok"));
        // rate_limit is skipped by serde
        assert!(back.rate_limit.is_none());
    }

    #[test]
    fn x_api_error_response_serde_roundtrip() {
        let err = XApiErrorResponse {
            detail: Some("bad".into()),
            title: Some("Error".into()),
            error_type: Some("about:blank".into()),
            status: Some(400),
        };
        let json = serde_json::to_string(&err).unwrap();
        let back: XApiErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, Some(400));
        assert_eq!(back.detail.as_deref(), Some("bad"));
    }

    #[test]
    fn x_api_error_response_empty() {
        let json = r#"{}"#;
        let err: XApiErrorResponse = serde_json::from_str(json).unwrap();
        assert!(err.detail.is_none());
        assert!(err.title.is_none());
        assert!(err.status.is_none());
    }

    #[test]
    fn single_tweet_response_serde_roundtrip() {
        let json = r#"{"data":{"id":"1","text":"hi","author_id":"2"}}"#;
        let resp: SingleTweetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.id, "1");
        assert!(resp.includes.is_none());
        let back = serde_json::to_string(&resp).unwrap();
        assert!(back.contains("\"id\":\"1\""));
    }

    #[test]
    fn users_meta_serde_roundtrip() {
        let meta = UsersMeta {
            result_count: 3,
            next_token: Some("nt".into()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: UsersMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result_count, 3);
    }

    #[test]
    fn like_tweet_request_serde_roundtrip() {
        let req = LikeTweetRequest {
            tweet_id: "t1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: LikeTweetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tweet_id, "t1");
    }

    #[test]
    fn follow_user_request_serde_roundtrip() {
        let req = FollowUserRequest {
            target_user_id: "u1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: FollowUserRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.target_user_id, "u1");
    }

    #[test]
    fn action_result_response_serde_roundtrip() {
        let resp = ActionResultResponse {
            data: ActionResultData { result: true },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: ActionResultResponse = serde_json::from_str(&json).unwrap();
        assert!(back.data.result);
    }

    #[test]
    fn action_result_data_liked_alias() {
        let json = r#"{"liked": true}"#;
        let data: ActionResultData = serde_json::from_str(json).unwrap();
        assert!(data.result);
    }

    #[test]
    fn action_result_data_following_alias() {
        let json = r#"{"following": false}"#;
        let data: ActionResultData = serde_json::from_str(json).unwrap();
        assert!(!data.result);
    }

    #[test]
    fn action_result_data_retweeted_alias() {
        let json = r#"{"retweeted": true}"#;
        let data: ActionResultData = serde_json::from_str(json).unwrap();
        assert!(data.result);
    }

    #[test]
    fn bookmark_tweet_request_serde_roundtrip() {
        let req = BookmarkTweetRequest {
            tweet_id: "b1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: BookmarkTweetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tweet_id, "b1");
    }

    #[test]
    fn retweet_request_serde_roundtrip() {
        let req = RetweetRequest {
            tweet_id: "rt1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: RetweetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tweet_id, "rt1");
    }

    #[test]
    fn delete_tweet_response_serde_roundtrip() {
        let resp = DeleteTweetResponse {
            data: DeleteTweetData { deleted: true },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: DeleteTweetResponse = serde_json::from_str(&json).unwrap();
        assert!(back.data.deleted);
    }

    #[test]
    fn delete_tweet_data_false() {
        let json = r#"{"deleted": false}"#;
        let data: DeleteTweetData = serde_json::from_str(json).unwrap();
        assert!(!data.deleted);
    }

    #[test]
    fn media_type_png_properties() {
        let png = MediaType::Image(ImageFormat::Png);
        assert_eq!(png.mime_type(), "image/png");
        assert_eq!(png.max_size(), 5 * 1024 * 1024);
        assert_eq!(png.media_category(), "tweet_image");
        assert!(!png.requires_chunked(1024));
        assert!(png.requires_chunked(6 * 1024 * 1024));
    }

    #[test]
    fn media_type_webp_properties() {
        let webp = MediaType::Image(ImageFormat::Webp);
        assert_eq!(webp.mime_type(), "image/webp");
        assert_eq!(webp.media_category(), "tweet_image");
    }

    #[test]
    fn media_type_gif_category() {
        assert_eq!(MediaType::Gif.media_category(), "tweet_gif");
    }

    #[test]
    fn media_type_video_category() {
        assert_eq!(MediaType::Video.media_category(), "tweet_video");
    }

    #[test]
    fn media_id_clone_eq() {
        let id = MediaId("mid1".into());
        let id2 = id.clone();
        assert_eq!(id, id2);
        assert_eq!(id.0, "mid1");
    }

    #[test]
    fn mention_response_is_search_response() {
        // MentionResponse is a type alias for SearchResponse
        let json = r#"{"data":[],"meta":{"result_count":0}}"#;
        let resp: MentionResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_empty());
        assert_eq!(resp.meta.result_count, 0);
    }

    #[test]
    fn image_format_eq() {
        assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
        assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
        assert_ne!(ImageFormat::Png, ImageFormat::Webp);
    }

    #[test]
    fn rate_limit_info_construction() {
        let info = RateLimitInfo {
            remaining: Some(100),
            reset_at: Some(1700000000),
        };
        assert_eq!(info.remaining, Some(100));
        assert_eq!(info.reset_at, Some(1700000000));
    }

    #[test]
    fn post_tweet_request_full_roundtrip() {
        let req = PostTweetRequest {
            text: "full test".into(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: "123".into(),
            }),
            media: Some(MediaPayload {
                media_ids: vec!["m1".into()],
            }),
            quote_tweet_id: Some("qt1".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PostTweetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.text, "full test");
        assert_eq!(back.reply.unwrap().in_reply_to_tweet_id, "123");
        assert_eq!(back.media.unwrap().media_ids.len(), 1);
        assert_eq!(back.quote_tweet_id.as_deref(), Some("qt1"));
    }

    #[test]
    fn user_response_serde_roundtrip() {
        let resp = UserResponse {
            data: User {
                id: "u1".into(),
                username: "test".into(),
                name: "Test".into(),
                profile_image_url: None,
                description: None,
                location: None,
                url: None,
                public_metrics: UserMetrics::default(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: UserResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.data.username, "test");
    }

    #[test]
    fn users_response_serde_roundtrip() {
        let resp = UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: UsersResponse = serde_json::from_str(&json).unwrap();
        assert!(back.data.is_empty());
        assert_eq!(back.meta.result_count, 0);
    }
}
