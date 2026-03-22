//! Write request types: mutations, content generation, uploads.

use schemars::JsonSchema;
use serde::Deserialize;

// --- Content Generation ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateReplyRequest {
    /// The tweet text to reply to
    pub tweet_text: String,
    /// Username of the tweet author
    pub tweet_author: String,
    /// Whether to potentially mention the product (default: false)
    pub mention_product: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopicRequest {
    /// Topic (uses a random industry topic from config if not provided)
    pub topic: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ComposeTweetRequest {
    /// The text content of the tweet or thread (JSON array for thread).
    pub content: String,
    /// Content type: "tweet" or "thread" (default: "tweet").
    pub content_type: Option<String>,
    /// Optional ISO-8601 datetime for scheduling. If omitted, creates a draft.
    pub scheduled_for: Option<String>,
}

// --- Direct X API (mutations) ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostTweetTextRequest {
    /// The tweet text content (max 280 characters).
    pub text: String,
    /// Optional media IDs to attach (from x_upload_media).
    pub media_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplyToTweetRequest {
    /// The reply text content.
    pub text: String,
    /// The tweet ID to reply to.
    pub in_reply_to_id: String,
    /// Optional media IDs to attach (from x_upload_media).
    pub media_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QuoteTweetRequest {
    /// The quote tweet text content.
    pub text: String,
    /// The tweet ID to quote.
    pub quoted_tweet_id: String,
    /// Optional media IDs to attach (from x_upload_media).
    pub media_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LikeTweetMcpRequest {
    /// The tweet ID to like.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FollowUserMcpRequest {
    /// The user ID to follow.
    pub target_user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnfollowUserMcpRequest {
    /// The user ID to unfollow.
    pub target_user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnlikeTweetMcpRequest {
    /// The tweet ID to unlike.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BookmarkTweetMcpRequest {
    /// The tweet ID to bookmark.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnbookmarkTweetMcpRequest {
    /// The tweet ID to unbookmark.
    pub tweet_id: String,
}

// --- Retweet / Delete / Thread / Media / Timeline / Usage ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RetweetMcpRequest {
    /// The tweet ID to retweet.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnretweetMcpRequest {
    /// The tweet ID to unretweet.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteTweetMcpRequest {
    /// The tweet ID to delete.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostThreadMcpRequest {
    /// Ordered list of tweet texts forming the thread.
    pub tweets: Vec<String>,
    /// Optional media IDs per tweet (outer index matches tweet index).
    pub media_ids: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UploadMediaMcpRequest {
    /// Local file path of the media to upload.
    pub file_path: String,
    /// Optional alt text for accessibility.
    pub alt_text: Option<String>,
    /// If true, validate the file without uploading (default: false).
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostTweetDryRunRequest {
    /// The tweet text content (max 280 characters).
    pub text: String,
    /// Optional media IDs to attach (from x_upload_media).
    pub media_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostThreadDryRunRequest {
    /// Ordered list of tweet texts forming the thread.
    pub tweets: Vec<String>,
    /// Optional media IDs per tweet (outer index matches tweet index).
    pub media_ids: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetXUsageRequest {
    /// Number of days to look back (default: 7).
    pub days: Option<u32>,
}

// --- Telemetry (mutations/audits) ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMcpToolMetricsRequest {
    /// Hours to look back (default: 24)
    pub since_hours: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMcpErrorBreakdownRequest {
    /// Hours to look back (default: 24)
    pub since_hours: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Content Generation ---

    #[test]
    fn generate_reply_request_deser() {
        let json = r#"{"tweet_text": "hello", "tweet_author": "alice", "mention_product": true}"#;
        let req: GenerateReplyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_text, "hello");
        assert_eq!(req.tweet_author, "alice");
        assert_eq!(req.mention_product, Some(true));
    }

    #[test]
    fn topic_request_deser() {
        let json = r#"{"topic": "AI"}"#;
        let req: TopicRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.topic.as_deref(), Some("AI"));
    }

    #[test]
    fn topic_request_deser_empty() {
        let json = r#"{}"#;
        let req: TopicRequest = serde_json::from_str(json).unwrap();
        assert!(req.topic.is_none());
    }

    #[test]
    fn compose_tweet_request_deser() {
        let json = r#"{"content": "hello world", "content_type": "tweet", "scheduled_for": "2026-03-22T10:00:00Z"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "hello world");
        assert_eq!(req.content_type.as_deref(), Some("tweet"));
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-03-22T10:00:00Z"));
    }

    // --- Direct X API (mutations) ---

    #[test]
    fn post_tweet_text_request_deser() {
        let json = r#"{"text": "hello", "media_ids": ["123", "456"]}"#;
        let req: PostTweetTextRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "hello");
        assert_eq!(req.media_ids.as_ref().map(|v| v.len()), Some(2));
    }

    #[test]
    fn reply_to_tweet_request_deser() {
        let json = r#"{"text": "reply", "in_reply_to_id": "123"}"#;
        let req: ReplyToTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "reply");
        assert_eq!(req.in_reply_to_id, "123");
    }

    #[test]
    fn quote_tweet_request_deser() {
        let json = r#"{"text": "quote", "quoted_tweet_id": "456"}"#;
        let req: QuoteTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "quote");
        assert_eq!(req.quoted_tweet_id, "456");
    }

    #[test]
    fn like_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "789"}"#;
        let req: LikeTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "789");
    }

    #[test]
    fn follow_user_mcp_request_deser() {
        let json = r#"{"target_user_id": "user123"}"#;
        let req: FollowUserMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.target_user_id, "user123");
    }

    #[test]
    fn unfollow_user_mcp_request_deser() {
        let json = r#"{"target_user_id": "user456"}"#;
        let req: UnfollowUserMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.target_user_id, "user456");
    }

    #[test]
    fn unlike_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "111"}"#;
        let req: UnlikeTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "111");
    }

    #[test]
    fn bookmark_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "222"}"#;
        let req: BookmarkTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "222");
    }

    #[test]
    fn unbookmark_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "333"}"#;
        let req: UnbookmarkTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "333");
    }

    // --- Retweet / Delete / Thread / Media ---

    #[test]
    fn retweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "444"}"#;
        let req: RetweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "444");
    }

    #[test]
    fn unretweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "555"}"#;
        let req: UnretweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "555");
    }

    #[test]
    fn delete_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "666"}"#;
        let req: DeleteTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "666");
    }

    #[test]
    fn post_thread_mcp_request_deser() {
        let json = r#"{"tweets": ["first", "second"], "media_ids": [["m1"], ["m2"]]}"#;
        let req: PostThreadMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweets.len(), 2);
        assert!(req.media_ids.is_some());
    }

    #[test]
    fn upload_media_mcp_request_deser() {
        let json = r#"{"file_path": "/tmp/image.jpg", "alt_text": "photo", "dry_run": true}"#;
        let req: UploadMediaMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.file_path, "/tmp/image.jpg");
        assert_eq!(req.alt_text.as_deref(), Some("photo"));
        assert!(req.dry_run);
    }

    #[test]
    fn post_tweet_dry_run_request_deser() {
        let json = r#"{"text": "test"}"#;
        let req: PostTweetDryRunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "test");
    }

    #[test]
    fn post_thread_dry_run_request_deser() {
        let json = r#"{"tweets": ["a", "b", "c"]}"#;
        let req: PostThreadDryRunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweets.len(), 3);
    }

    #[test]
    fn get_x_usage_request_deser() {
        let json = r#"{"days": 14}"#;
        let req: GetXUsageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.days, Some(14));
    }

    // --- Telemetry ---

    #[test]
    fn get_mcp_tool_metrics_request_deser() {
        let json = r#"{"since_hours": 48}"#;
        let req: GetMcpToolMetricsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.since_hours, Some(48));
    }

    #[test]
    fn get_mcp_error_breakdown_request_deser() {
        let json = r#"{"since_hours": 72}"#;
        let req: GetMcpErrorBreakdownRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.since_hours, Some(72));
    }
}
