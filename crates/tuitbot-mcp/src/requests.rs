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
