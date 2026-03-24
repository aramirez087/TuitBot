//! Tweet and search types for X API v2.

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
    pub users: Vec<super::user_types::User>,
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
    /// Optional media attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<super::api_types::MediaPayload>,
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

/// Wrapper for single-tweet responses with expansion support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleTweetResponse {
    /// The tweet data.
    pub data: Tweet,
    /// Expanded objects.
    #[serde(default)]
    pub includes: Option<Includes>,
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
