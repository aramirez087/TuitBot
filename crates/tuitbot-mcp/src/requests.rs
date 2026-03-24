//! Request structs for MCP tool parameters.
//!
//! Extracted from `server.rs` to keep the tool router focused on routing
//! and to share request types across primitive and composite tools.

use schemars::JsonSchema;
use serde::Deserialize;

// --- Analytics ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetStatsRequest {
    /// Number of days to look back (default: 7)
    pub days: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFollowerTrendRequest {
    /// Number of snapshots to return (default: 7)
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAnalyticsSummaryRequest {
    /// Placeholder (no parameters needed)
    #[serde(default)]
    pub _unused: Option<()>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetActionLogRequest {
    /// Hours to look back (default: 24)
    pub since_hours: Option<u32>,
    /// Filter by action type (e.g., 'reply', 'tweet', 'search')
    pub action_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SinceHoursRequest {
    /// Hours to look back (default: 24)
    pub since_hours: Option<u32>,
}

// --- Mutation Audit ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRecentMutationsRequest {
    /// Max entries to return (default: 20, max: 100)
    pub limit: Option<u32>,
    /// Filter by tool name (e.g., "post_tweet", "like_tweet")
    pub tool_name: Option<String>,
    /// Filter by status: "pending", "success", "failure", "duplicate"
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMutationDetailRequest {
    /// The correlation ID of the mutation to look up
    pub correlation_id: String,
}

// --- Discovery ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListUnrepliedTweetsRequest {
    /// Minimum relevance score threshold (default: 0.0)
    pub threshold: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiscoveryFeedRequest {
    /// Minimum relevance score (default: 50.0)
    pub min_score: Option<f64>,
    /// Maximum number of tweets to return (default: 10)
    pub limit: Option<u32>,
}

// --- Scoring ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScoreTweetRequest {
    /// The tweet text content
    pub text: String,
    /// Author's X username
    pub author_username: String,
    /// Author's follower count
    pub author_followers: u64,
    /// Number of likes on the tweet
    pub likes: u64,
    /// Number of retweets
    pub retweets: u64,
    /// Number of replies
    pub replies: u64,
    /// Tweet creation timestamp (ISO 8601)
    pub created_at: String,
}

// --- Approval ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApprovalIdRequest {
    /// The approval queue item ID
    pub id: i64,
}

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

// --- Direct X API ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TweetIdRequest {
    /// The tweet ID to look up.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UsernameRequest {
    /// The X username (without @) to look up.
    pub username: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchTweetsRequest {
    /// Search query string.
    pub query: String,
    /// Maximum number of results (10-100, default: 10).
    pub max_results: Option<u32>,
    /// Only return tweets newer than this tweet ID.
    pub since_id: Option<String>,
    /// Pagination token for fetching the next page of results.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserMentionsRequest {
    /// Only return mentions newer than this tweet ID.
    pub since_id: Option<String>,
    /// Pagination token for fetching the next page of results.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserTweetsRequest {
    /// The user ID whose tweets to fetch.
    pub user_id: String,
    /// Maximum number of results (5-100, default: 10).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page of results.
    pub pagination_token: Option<String>,
}

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

// --- New Direct X API (Session 04) ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnlikeTweetMcpRequest {
    /// The tweet ID to unlike.
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFollowersRequest {
    /// The user ID whose followers to fetch.
    pub user_id: String,
    /// Maximum number of results (1-1000, default: 100).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFollowingRequest {
    /// The user ID whose following list to fetch.
    pub user_id: String,
    /// Maximum number of results (1-1000, default: 100).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUserByIdRequest {
    /// The user ID to look up.
    pub user_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLikedTweetsRequest {
    /// The user ID whose liked tweets to fetch.
    pub user_id: String,
    /// Maximum number of results (1-100, default: 10).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBookmarksRequest {
    /// Maximum number of results (1-100, default: 10).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetUsersByIdsRequest {
    /// List of user IDs to look up (1-100).
    pub user_ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTweetLikingUsersRequest {
    /// The tweet ID to get liking users for.
    pub tweet_id: String,
    /// Maximum number of results (1-100, default: 100).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
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
pub struct GetHomeTimelineRequest {
    /// Maximum number of results (1-100, default: 20).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetXUsageRequest {
    /// Number of days to look back (default: 7).
    pub days: Option<u32>,
}

// --- Context Intelligence ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAuthorContextRequest {
    /// Author username (with or without @) or author ID.
    pub identifier: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecommendEngagementRequest {
    /// Username of the tweet author.
    pub author_username: String,
    /// The tweet text to evaluate for engagement.
    pub tweet_text: String,
    /// Optional campaign objective (e.g., "grow developer audience").
    pub campaign_objective: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopicPerformanceSnapshotRequest {
    /// Number of days to look back (default: 30).
    pub lookback_days: Option<u32>,
}

// --- Telemetry ---

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

// --- Composite Tools ---

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindReplyOpportunitiesRequest {
    /// Search query (defaults to product keywords joined with OR).
    pub query: Option<String>,
    /// Minimum score to include (defaults to scoring threshold from config).
    pub min_score: Option<f64>,
    /// Maximum number of results (default: 10).
    pub limit: Option<u32>,
    /// Only return tweets newer than this tweet ID.
    pub since_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DraftRepliesRequest {
    /// Tweet IDs of previously discovered candidates.
    pub candidate_ids: Vec<String>,
    /// Override the reply archetype (e.g., "agree_and_expand", "ask_question").
    pub archetype: Option<String>,
    /// Whether to potentially mention the product (default: false).
    pub mention_product: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProposeAndQueueRepliesRequest {
    /// Items to propose as replies.
    pub items: Vec<ProposeItem>,
    /// Whether to potentially mention the product in auto-generated replies (default: false).
    pub mention_product: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProposeItem {
    /// The tweet ID to reply to.
    pub candidate_id: String,
    /// Pre-drafted reply text. If omitted, generates one via LLM.
    pub pre_drafted_text: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateThreadPlanRequest {
    /// Topic for the thread.
    pub topic: String,
    /// Objective for the thread (e.g., "establish expertise", "drive traffic").
    pub objective: Option<String>,
    /// Target audience description.
    pub target_audience: Option<String>,
    /// Thread structure override (e.g., "transformation", "framework", "mistakes", "analysis").
    pub structure: Option<String>,
}

// --- Universal X API Request Tools ---

/// Key-value pair for query parameters and headers.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct KeyValue {
    /// Parameter key.
    pub key: String,
    /// Parameter value.
    pub value: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XGetRequest {
    /// API path (e.g., "/2/tweets/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
    /// Auto-paginate by following next_token (default: false). Only for GET.
    #[serde(default)]
    pub auto_paginate: bool,
    /// Maximum pages to fetch when auto_paginate is true (default: 10, max: 10).
    pub max_pages: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XPostRequest {
    /// API path (e.g., "/2/tweets"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// JSON request body as a string.
    pub body: Option<String>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XPutRequest {
    /// API path (e.g., "/2/lists/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// JSON request body as a string.
    pub body: Option<String>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct XDeleteRequest {
    /// API path (e.g., "/2/tweets/123"). Must start with "/".
    pub path: String,
    /// Target host (default: "api.x.com"). Only api.x.com, upload.x.com, upload.twitter.com, ads-api.x.com allowed.
    pub host: Option<String>,
    /// Query parameters as key-value pairs.
    pub query: Option<Vec<KeyValue>>,
    /// Extra headers as key-value pairs. Authorization/Host/Cookie are blocked.
    pub headers: Option<Vec<KeyValue>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Analytics ---

    #[test]
    fn get_stats_request_deser() {
        let json = r#"{"days": 14}"#;
        let req: GetStatsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.days, Some(14));
    }

    #[test]
    fn get_stats_request_deser_empty() {
        let json = r#"{}"#;
        let req: GetStatsRequest = serde_json::from_str(json).unwrap();
        assert!(req.days.is_none());
    }

    #[test]
    fn get_follower_trend_request_deser() {
        let json = r#"{"limit": 30}"#;
        let req: GetFollowerTrendRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, Some(30));
    }

    #[test]
    fn get_action_log_request_deser() {
        let json = r#"{"since_hours": 48, "action_type": "reply"}"#;
        let req: GetActionLogRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.since_hours, Some(48));
        assert_eq!(req.action_type.as_deref(), Some("reply"));
    }

    #[test]
    fn since_hours_request_deser() {
        let json = r#"{}"#;
        let req: SinceHoursRequest = serde_json::from_str(json).unwrap();
        assert!(req.since_hours.is_none());
    }

    // --- Mutation Audit ---

    #[test]
    fn get_recent_mutations_request_deser() {
        let json = r#"{"limit": 50, "tool_name": "post_tweet", "status": "success"}"#;
        let req: GetRecentMutationsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, Some(50));
        assert_eq!(req.tool_name.as_deref(), Some("post_tweet"));
        assert_eq!(req.status.as_deref(), Some("success"));
    }

    #[test]
    fn get_mutation_detail_request_deser() {
        let json = r#"{"correlation_id": "abc-123"}"#;
        let req: GetMutationDetailRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.correlation_id, "abc-123");
    }

    // --- Discovery ---

    #[test]
    fn list_unreplied_tweets_request_deser() {
        let json = r#"{"threshold": 0.5}"#;
        let req: ListUnrepliedTweetsRequest = serde_json::from_str(json).unwrap();
        assert!((req.threshold.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn discovery_feed_request_deser() {
        let json = r#"{"min_score": 75.0, "limit": 5}"#;
        let req: DiscoveryFeedRequest = serde_json::from_str(json).unwrap();
        assert!((req.min_score.unwrap() - 75.0).abs() < f64::EPSILON);
        assert_eq!(req.limit, Some(5));
    }

    // --- Scoring ---

    #[test]
    fn score_tweet_request_deser() {
        let json = r#"{
            "text": "Great tweet",
            "author_username": "dev",
            "author_followers": 1000,
            "likes": 50,
            "retweets": 10,
            "replies": 5,
            "created_at": "2026-01-01T00:00:00Z"
        }"#;
        let req: ScoreTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "Great tweet");
        assert_eq!(req.author_followers, 1000);
        assert_eq!(req.likes, 50);
    }

    // --- Approval ---

    #[test]
    fn approval_id_request_deser() {
        let json = r#"{"id": 42}"#;
        let req: ApprovalIdRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, 42);
    }

    // --- Content Generation ---

    #[test]
    fn generate_reply_request_deser() {
        let json = r#"{"tweet_text": "Hello", "tweet_author": "alice", "mention_product": true}"#;
        let req: GenerateReplyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_text, "Hello");
        assert_eq!(req.tweet_author, "alice");
        assert_eq!(req.mention_product, Some(true));
    }

    #[test]
    fn topic_request_deser() {
        let json = r#"{"topic": "rust"}"#;
        let req: TopicRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.topic.as_deref(), Some("rust"));
    }

    #[test]
    fn topic_request_empty() {
        let json = r#"{}"#;
        let req: TopicRequest = serde_json::from_str(json).unwrap();
        assert!(req.topic.is_none());
    }

    #[test]
    fn compose_tweet_request_deser() {
        let json = r#"{"content": "Hello world", "content_type": "tweet", "scheduled_for": null}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "Hello world");
        assert_eq!(req.content_type.as_deref(), Some("tweet"));
    }

    // --- Direct X API ---

    #[test]
    fn tweet_id_request_deser() {
        let json = r#"{"tweet_id": "12345"}"#;
        let req: TweetIdRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "12345");
    }

    #[test]
    fn username_request_deser() {
        let json = r#"{"username": "alice"}"#;
        let req: UsernameRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
    }

    #[test]
    fn search_tweets_request_deser() {
        let json = r#"{"query": "rust lang", "max_results": 50}"#;
        let req: SearchTweetsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query, "rust lang");
        assert_eq!(req.max_results, Some(50));
        assert!(req.since_id.is_none());
        assert!(req.pagination_token.is_none());
    }

    #[test]
    fn get_user_mentions_request_deser() {
        let json = r#"{"since_id": "999"}"#;
        let req: GetUserMentionsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.since_id.as_deref(), Some("999"));
    }

    #[test]
    fn get_user_tweets_request_deser() {
        let json = r#"{"user_id": "u1", "max_results": 20}"#;
        let req: GetUserTweetsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
        assert_eq!(req.max_results, Some(20));
    }

    #[test]
    fn post_tweet_text_request_deser() {
        let json = r#"{"text": "Hello!", "media_ids": ["m1", "m2"]}"#;
        let req: PostTweetTextRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "Hello!");
        assert_eq!(req.media_ids.unwrap().len(), 2);
    }

    #[test]
    fn reply_to_tweet_request_deser() {
        let json = r#"{"text": "Nice!", "in_reply_to_id": "123"}"#;
        let req: ReplyToTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "Nice!");
        assert_eq!(req.in_reply_to_id, "123");
        assert!(req.media_ids.is_none());
    }

    #[test]
    fn quote_tweet_request_deser() {
        let json = r#"{"text": "Great thread!", "quoted_tweet_id": "456"}"#;
        let req: QuoteTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.quoted_tweet_id, "456");
    }

    #[test]
    fn like_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "t1"}"#;
        let req: LikeTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "t1");
    }

    #[test]
    fn follow_user_mcp_request_deser() {
        let json = r#"{"target_user_id": "u1"}"#;
        let req: FollowUserMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.target_user_id, "u1");
    }

    #[test]
    fn unfollow_user_mcp_request_deser() {
        let json = r#"{"target_user_id": "u2"}"#;
        let req: UnfollowUserMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.target_user_id, "u2");
    }

    #[test]
    fn unlike_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "t2"}"#;
        let req: UnlikeTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "t2");
    }

    #[test]
    fn get_followers_request_deser() {
        let json = r#"{"user_id": "u1", "max_results": 100}"#;
        let req: GetFollowersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
    }

    #[test]
    fn get_following_request_deser() {
        let json = r#"{"user_id": "u1"}"#;
        let req: GetFollowingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
        assert!(req.max_results.is_none());
    }

    #[test]
    fn get_user_by_id_request_deser() {
        let json = r#"{"user_id": "123"}"#;
        let req: GetUserByIdRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "123");
    }

    #[test]
    fn get_liked_tweets_request_deser() {
        let json = r#"{"user_id": "u1", "max_results": 50}"#;
        let req: GetLikedTweetsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
        assert_eq!(req.max_results, Some(50));
    }

    #[test]
    fn get_bookmarks_request_deser() {
        let json = r#"{"max_results": 10}"#;
        let req: GetBookmarksRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(10));
    }

    #[test]
    fn bookmark_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "b1"}"#;
        let req: BookmarkTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "b1");
    }

    #[test]
    fn unbookmark_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "ub1"}"#;
        let req: UnbookmarkTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "ub1");
    }

    #[test]
    fn get_users_by_ids_request_deser() {
        let json = r#"{"user_ids": ["u1", "u2", "u3"]}"#;
        let req: GetUsersByIdsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_ids.len(), 3);
    }

    #[test]
    fn get_tweet_liking_users_request_deser() {
        let json = r#"{"tweet_id": "t1", "max_results": 50}"#;
        let req: GetTweetLikingUsersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "t1");
        assert_eq!(req.max_results, Some(50));
    }

    #[test]
    fn retweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "rt1"}"#;
        let req: RetweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "rt1");
    }

    #[test]
    fn unretweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "urt1"}"#;
        let req: UnretweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "urt1");
    }

    #[test]
    fn delete_tweet_mcp_request_deser() {
        let json = r#"{"tweet_id": "del1"}"#;
        let req: DeleteTweetMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweet_id, "del1");
    }

    #[test]
    fn post_thread_mcp_request_deser() {
        let json = r#"{"tweets": ["first", "second"], "media_ids": [["m1"], []]}"#;
        let req: PostThreadMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweets.len(), 2);
        let media = req.media_ids.unwrap();
        assert_eq!(media[0], vec!["m1"]);
        assert!(media[1].is_empty());
    }

    #[test]
    fn upload_media_mcp_request_deser() {
        let json = r#"{"file_path": "/tmp/img.png", "alt_text": "photo", "dry_run": true}"#;
        let req: UploadMediaMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.file_path, "/tmp/img.png");
        assert_eq!(req.alt_text.as_deref(), Some("photo"));
        assert!(req.dry_run);
    }

    #[test]
    fn upload_media_mcp_request_dry_run_default() {
        let json = r#"{"file_path": "/tmp/img.png"}"#;
        let req: UploadMediaMcpRequest = serde_json::from_str(json).unwrap();
        assert!(!req.dry_run);
    }

    #[test]
    fn post_tweet_dry_run_request_deser() {
        let json = r#"{"text": "test tweet"}"#;
        let req: PostTweetDryRunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "test tweet");
        assert!(req.media_ids.is_none());
    }

    #[test]
    fn post_thread_dry_run_request_deser() {
        let json = r#"{"tweets": ["a", "b", "c"]}"#;
        let req: PostThreadDryRunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.tweets.len(), 3);
    }

    #[test]
    fn get_home_timeline_request_deser() {
        let json = r#"{"max_results": 50}"#;
        let req: GetHomeTimelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(50));
    }

    #[test]
    fn get_x_usage_request_deser() {
        let json = r#"{"days": 30}"#;
        let req: GetXUsageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.days, Some(30));
    }

    // --- Context Intelligence ---

    #[test]
    fn get_author_context_request_deser() {
        let json = r#"{"identifier": "@alice"}"#;
        let req: GetAuthorContextRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.identifier, "@alice");
    }

    #[test]
    fn recommend_engagement_request_deser() {
        let json = r#"{"author_username": "dev", "tweet_text": "New feature!", "campaign_objective": "grow audience"}"#;
        let req: RecommendEngagementRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.author_username, "dev");
        assert_eq!(req.campaign_objective.as_deref(), Some("grow audience"));
    }

    #[test]
    fn topic_performance_snapshot_request_deser() {
        let json = r#"{"lookback_days": 60}"#;
        let req: TopicPerformanceSnapshotRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.lookback_days, Some(60));
    }

    // --- Telemetry ---

    #[test]
    fn get_mcp_tool_metrics_request_deser() {
        let json = r#"{"since_hours": 12}"#;
        let req: GetMcpToolMetricsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.since_hours, Some(12));
    }

    #[test]
    fn get_mcp_error_breakdown_request_deser() {
        let json = r#"{}"#;
        let req: GetMcpErrorBreakdownRequest = serde_json::from_str(json).unwrap();
        assert!(req.since_hours.is_none());
    }

    // --- Composite Tools ---

    #[test]
    fn find_reply_opportunities_request_deser() {
        let json = r#"{"query": "rust", "min_score": 60.0, "limit": 5}"#;
        let req: FindReplyOpportunitiesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query.as_deref(), Some("rust"));
        assert_eq!(req.limit, Some(5));
    }

    #[test]
    fn draft_replies_request_deser() {
        let json = r#"{"candidate_ids": ["c1", "c2"], "archetype": "agree_and_expand", "mention_product": false}"#;
        let req: DraftRepliesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.candidate_ids.len(), 2);
        assert_eq!(req.archetype.as_deref(), Some("agree_and_expand"));
    }

    #[test]
    fn propose_and_queue_replies_request_deser() {
        let json = r#"{"items": [{"candidate_id": "c1", "pre_drafted_text": "draft"}], "mention_product": true}"#;
        let req: ProposeAndQueueRepliesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.items.len(), 1);
        assert_eq!(req.items[0].candidate_id, "c1");
        assert_eq!(req.items[0].pre_drafted_text.as_deref(), Some("draft"));
    }

    #[test]
    fn propose_item_without_pre_drafted_text() {
        let json = r#"{"candidate_id": "c2"}"#;
        let item: ProposeItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.candidate_id, "c2");
        assert!(item.pre_drafted_text.is_none());
    }

    #[test]
    fn generate_thread_plan_request_deser() {
        let json = r#"{"topic": "testing", "objective": "educate", "structure": "framework"}"#;
        let req: GenerateThreadPlanRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.topic, "testing");
        assert_eq!(req.objective.as_deref(), Some("educate"));
        assert_eq!(req.structure.as_deref(), Some("framework"));
        assert!(req.target_audience.is_none());
    }

    // --- Universal X API Request ---

    #[test]
    fn key_value_deser() {
        let json = r#"{"key": "k", "value": "v"}"#;
        let kv: KeyValue = serde_json::from_str(json).unwrap();
        assert_eq!(kv.key, "k");
        assert_eq!(kv.value, "v");
    }

    #[test]
    fn x_get_request_deser() {
        let json = r#"{"path": "/2/tweets/123", "auto_paginate": true, "max_pages": 5}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets/123");
        assert!(req.auto_paginate);
        assert_eq!(req.max_pages, Some(5));
        assert!(req.host.is_none());
    }

    #[test]
    fn x_get_request_auto_paginate_default() {
        let json = r#"{"path": "/2/tweets"}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert!(!req.auto_paginate);
    }

    #[test]
    fn x_post_request_deser() {
        let json = r#"{"path": "/2/tweets", "body": "{\"text\":\"hi\"}"}"#;
        let req: XPostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets");
        assert!(req.body.is_some());
    }

    #[test]
    fn x_put_request_deser() {
        let json = r#"{"path": "/2/lists/123", "host": "api.x.com"}"#;
        let req: XPutRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/lists/123");
        assert_eq!(req.host.as_deref(), Some("api.x.com"));
    }

    #[test]
    fn x_delete_request_deser() {
        let json = r#"{"path": "/2/tweets/123"}"#;
        let req: XDeleteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets/123");
    }

    // --- Debug impls ---

    #[test]
    fn key_value_debug() {
        let kv = KeyValue {
            key: "k".to_string(),
            value: "v".to_string(),
        };
        let debug = format!("{kv:?}");
        assert!(debug.contains("k"));
        assert!(debug.contains("v"));
    }

    #[test]
    fn key_value_clone() {
        let kv = KeyValue {
            key: "k".to_string(),
            value: "v".to_string(),
        };
        let kv2 = kv.clone();
        assert_eq!(kv2.key, "k");
        assert_eq!(kv2.value, "v");
    }

    #[test]
    fn propose_item_debug_and_clone() {
        let item = ProposeItem {
            candidate_id: "c1".to_string(),
            pre_drafted_text: Some("draft".to_string()),
        };
        let debug = format!("{item:?}");
        assert!(debug.contains("c1"));
        let clone = item.clone();
        assert_eq!(clone.candidate_id, "c1");
    }

    // --- Schema generation ---

    #[test]
    fn get_stats_request_schema() {
        let schema = schemars::schema_for!(GetStatsRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("days"));
    }

    #[test]
    fn score_tweet_request_schema() {
        let schema = schemars::schema_for!(ScoreTweetRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("author_username"));
        assert!(json.contains("author_followers"));
    }

    #[test]
    fn key_value_schema() {
        let schema = schemars::schema_for!(KeyValue);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn x_get_request_schema() {
        let schema = schemars::schema_for!(XGetRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("auto_paginate"));
    }

    #[test]
    fn compose_tweet_request_schema() {
        let schema = schemars::schema_for!(ComposeTweetRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("content"));
        assert!(json.contains("content_type"));
    }

    #[test]
    fn generate_reply_request_schema() {
        let schema = schemars::schema_for!(GenerateReplyRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("tweet_text"));
        assert!(json.contains("mention_product"));
    }

    #[test]
    fn find_reply_opportunities_request_schema() {
        let schema = schemars::schema_for!(FindReplyOpportunitiesRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("query"));
        assert!(json.contains("min_score"));
    }

    #[test]
    fn propose_and_queue_replies_request_schema() {
        let schema = schemars::schema_for!(ProposeAndQueueRepliesRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("items"));
    }

    #[test]
    fn generate_thread_plan_request_schema() {
        let schema = schemars::schema_for!(GenerateThreadPlanRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("topic"));
        assert!(json.contains("objective"));
    }

    #[test]
    fn x_post_request_schema() {
        let schema = schemars::schema_for!(XPostRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        assert!(json.contains("body"));
    }

    #[test]
    fn x_put_request_schema() {
        let schema = schemars::schema_for!(XPutRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
    }

    #[test]
    fn x_delete_request_schema() {
        let schema = schemars::schema_for!(XDeleteRequest);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
    }

    // --- Debug formatting of all request types ---

    #[test]
    fn all_request_types_debug() {
        let _ = format!("{:?}", GetStatsRequest { days: Some(7) });
        let _ = format!("{:?}", GetFollowerTrendRequest { limit: Some(30) });
        let _ = format!(
            "{:?}",
            GetActionLogRequest {
                since_hours: Some(24),
                action_type: None,
            }
        );
        let _ = format!(
            "{:?}",
            SinceHoursRequest {
                since_hours: Some(48),
            }
        );
        let _ = format!(
            "{:?}",
            GetRecentMutationsRequest {
                limit: Some(20),
                tool_name: None,
                status: None,
            }
        );
        let _ = format!(
            "{:?}",
            GetMutationDetailRequest {
                correlation_id: "abc".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            ListUnrepliedTweetsRequest {
                threshold: Some(0.5),
            }
        );
        let _ = format!(
            "{:?}",
            DiscoveryFeedRequest {
                min_score: Some(50.0),
                limit: Some(10),
            }
        );
        let _ = format!("{:?}", ApprovalIdRequest { id: 1 });
        let _ = format!(
            "{:?}",
            TopicRequest {
                topic: Some("rust".to_string()),
            }
        );
        let _ = format!(
            "{:?}",
            TweetIdRequest {
                tweet_id: "123".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            UsernameRequest {
                username: "alice".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            RetweetMcpRequest {
                tweet_id: "rt1".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            UnretweetMcpRequest {
                tweet_id: "urt1".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            DeleteTweetMcpRequest {
                tweet_id: "del1".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            PostThreadMcpRequest {
                tweets: vec!["a".to_string()],
                media_ids: None,
            }
        );
        let _ = format!(
            "{:?}",
            UploadMediaMcpRequest {
                file_path: "/tmp/img.png".to_string(),
                alt_text: None,
                dry_run: false,
            }
        );
        let _ = format!(
            "{:?}",
            GetHomeTimelineRequest {
                max_results: Some(20),
                pagination_token: None,
            }
        );
        let _ = format!("{:?}", GetXUsageRequest { days: Some(7) });
        let _ = format!(
            "{:?}",
            GetAuthorContextRequest {
                identifier: "@alice".to_string(),
            }
        );
        let _ = format!(
            "{:?}",
            RecommendEngagementRequest {
                author_username: "dev".to_string(),
                tweet_text: "hello".to_string(),
                campaign_objective: None,
            }
        );
        let _ = format!(
            "{:?}",
            TopicPerformanceSnapshotRequest {
                lookback_days: Some(30),
            }
        );
        let _ = format!(
            "{:?}",
            GetMcpToolMetricsRequest {
                since_hours: Some(24),
            }
        );
        let _ = format!("{:?}", GetMcpErrorBreakdownRequest { since_hours: None });
        let _ = format!(
            "{:?}",
            DraftRepliesRequest {
                candidate_ids: vec!["c1".to_string()],
                archetype: None,
                mention_product: None,
            }
        );
    }

    // --- X request types with all fields ---

    #[test]
    fn x_get_request_with_query_and_headers() {
        let json = r#"{"path": "/2/tweets", "host": "api.x.com", "query": [{"key": "q", "value": "rust"}], "headers": [{"key": "Accept", "value": "application/json"}], "auto_paginate": false}"#;
        let req: XGetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.query.as_ref().unwrap().len(), 1);
        assert_eq!(req.headers.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn x_post_request_with_all_fields() {
        let json = r#"{"path": "/2/tweets", "host": "api.x.com", "query": [{"key": "a", "value": "1"}], "body": "{}", "headers": [{"key": "X-Custom", "value": "val"}]}"#;
        let req: XPostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.path, "/2/tweets");
        assert!(req.query.is_some());
        assert!(req.body.is_some());
        assert!(req.headers.is_some());
    }

    #[test]
    fn x_put_request_with_all_fields() {
        let json =
            r#"{"path": "/2/lists/1", "body": "{\"name\":\"test\"}", "query": [], "headers": []}"#;
        let req: XPutRequest = serde_json::from_str(json).unwrap();
        assert!(req.query.unwrap().is_empty());
        assert!(req.headers.unwrap().is_empty());
    }

    #[test]
    fn x_delete_request_with_all_fields() {
        let json =
            r#"{"path": "/2/tweets/99", "host": "api.x.com", "query": null, "headers": null}"#;
        let req: XDeleteRequest = serde_json::from_str(json).unwrap();
        assert!(req.query.is_none());
        assert!(req.headers.is_none());
    }

    // --- Edge case: compose tweet with scheduled_for ---

    #[test]
    fn compose_tweet_request_with_schedule() {
        let json = r#"{"content": "Hi", "content_type": "thread", "scheduled_for": "2026-12-01T12:00:00Z"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content_type.as_deref(), Some("thread"));
        assert_eq!(req.scheduled_for.as_deref(), Some("2026-12-01T12:00:00Z"));
    }

    #[test]
    fn compose_tweet_request_minimal() {
        let json = r#"{"content": "Just text"}"#;
        let req: ComposeTweetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "Just text");
        assert!(req.content_type.is_none());
        assert!(req.scheduled_for.is_none());
    }

    // --- Search tweets request with all optional fields ---

    #[test]
    fn search_tweets_request_all_fields() {
        let json =
            r#"{"query": "q", "max_results": 100, "since_id": "999", "pagination_token": "tok"}"#;
        let req: SearchTweetsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(100));
        assert_eq!(req.since_id.as_deref(), Some("999"));
        assert_eq!(req.pagination_token.as_deref(), Some("tok"));
    }

    // --- Post tweet request with no media ---

    #[test]
    fn post_tweet_text_request_no_media() {
        let json = r#"{"text": "Hello"}"#;
        let req: PostTweetTextRequest = serde_json::from_str(json).unwrap();
        assert!(req.media_ids.is_none());
    }

    // --- Followers/following with pagination ---

    #[test]
    fn get_followers_request_with_pagination() {
        let json = r#"{"user_id": "u1", "max_results": 500, "pagination_token": "next"}"#;
        let req: GetFollowersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(500));
        assert_eq!(req.pagination_token.as_deref(), Some("next"));
    }

    #[test]
    fn get_following_request_with_pagination() {
        let json = r#"{"user_id": "u1", "max_results": 200, "pagination_token": "next2"}"#;
        let req: GetFollowingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(200));
        assert_eq!(req.pagination_token.as_deref(), Some("next2"));
    }

    // --- Generate reply without mention_product ---

    #[test]
    fn generate_reply_request_no_mention() {
        let json = r#"{"tweet_text": "Hello", "tweet_author": "bob"}"#;
        let req: GenerateReplyRequest = serde_json::from_str(json).unwrap();
        assert!(req.mention_product.is_none());
    }
}
