//! Query/read-only request types: analytics, mutation audit, discovery,
//! scoring, approval, context intelligence, and telemetry.

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

// --- Direct X API (queries) ---

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetHomeTimelineRequest {
    /// Maximum number of results (1-100, default: 20).
    pub max_results: Option<u32>,
    /// Pagination token for fetching the next page.
    pub pagination_token: Option<String>,
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

    // --- Direct X API (queries) ---

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
    fn search_tweets_request_all_fields() {
        let json =
            r#"{"query": "q", "max_results": 100, "since_id": "999", "pagination_token": "tok"}"#;
        let req: SearchTweetsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(100));
        assert_eq!(req.since_id.as_deref(), Some("999"));
        assert_eq!(req.pagination_token.as_deref(), Some("tok"));
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
    fn get_followers_request_deser() {
        let json = r#"{"user_id": "u1", "max_results": 100}"#;
        let req: GetFollowersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
    }

    #[test]
    fn get_followers_request_with_pagination() {
        let json = r#"{"user_id": "u1", "max_results": 500, "pagination_token": "next"}"#;
        let req: GetFollowersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(500));
        assert_eq!(req.pagination_token.as_deref(), Some("next"));
    }

    #[test]
    fn get_following_request_deser() {
        let json = r#"{"user_id": "u1"}"#;
        let req: GetFollowingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u1");
        assert!(req.max_results.is_none());
    }

    #[test]
    fn get_following_request_with_pagination() {
        let json = r#"{"user_id": "u1", "max_results": 200, "pagination_token": "next2"}"#;
        let req: GetFollowingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(200));
        assert_eq!(req.pagination_token.as_deref(), Some("next2"));
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
    fn get_home_timeline_request_deser() {
        let json = r#"{"max_results": 50}"#;
        let req: GetHomeTimelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_results, Some(50));
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

    // --- Debug formatting ---

    #[test]
    fn query_request_types_debug() {
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
            GetHomeTimelineRequest {
                max_results: Some(20),
                pagination_token: None,
            }
        );
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
    }
}
