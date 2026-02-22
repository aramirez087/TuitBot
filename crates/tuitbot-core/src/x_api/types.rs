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

/// Request body for posting a tweet via X API v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTweetRequest {
    /// Tweet text content.
    pub text: String,
    /// Optional reply configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyTo>,
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
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("reply"));

        let req_reply = PostTweetRequest {
            text: "Nice!".to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: "999".to_string(),
            }),
        };
        let json = serde_json::to_string(&req_reply).expect("serialize");
        assert!(json.contains("in_reply_to_tweet_id"));
        assert!(json.contains("999"));
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
}
