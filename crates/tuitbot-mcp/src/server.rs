//! MCP server definition with tool routing.
//!
//! Implements `ServerHandler` for the Tuitbot MCP server, registering all
//! tools and dispatching calls to the appropriate tool modules.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::SharedState;
use crate::tools;

/// Tuitbot MCP server.
#[derive(Clone)]
pub struct TuitbotMcpServer {
    state: SharedState,
    tool_router: ToolRouter<Self>,
}

impl TuitbotMcpServer {
    /// Create a new MCP server with the given shared state.
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

// --- Request structs for tools with parameters ---

#[derive(Debug, Deserialize, JsonSchema)]
struct GetStatsRequest {
    /// Number of days to look back (default: 7)
    days: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetFollowerTrendRequest {
    /// Number of snapshots to return (default: 7)
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetActionLogRequest {
    /// Hours to look back (default: 24)
    since_hours: Option<u32>,
    /// Filter by action type (e.g., 'reply', 'tweet', 'search')
    action_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SinceHoursRequest {
    /// Hours to look back (default: 24)
    since_hours: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListUnrepliedTweetsRequest {
    /// Minimum relevance score threshold (default: 0.0)
    threshold: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScoreTweetRequest {
    /// The tweet text content
    text: String,
    /// Author's X username
    author_username: String,
    /// Author's follower count
    author_followers: u64,
    /// Number of likes on the tweet
    likes: u64,
    /// Number of retweets
    retweets: u64,
    /// Number of replies
    replies: u64,
    /// Tweet creation timestamp (ISO 8601)
    created_at: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ApprovalIdRequest {
    /// The approval queue item ID
    id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GenerateReplyRequest {
    /// The tweet text to reply to
    tweet_text: String,
    /// Username of the tweet author
    tweet_author: String,
    /// Whether to potentially mention the product (default: false)
    mention_product: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TopicRequest {
    /// Topic (uses a random industry topic from config if not provided)
    topic: Option<String>,
}

#[tool_router]
impl TuitbotMcpServer {
    // --- Analytics ---

    /// Get analytics dashboard: follower trend, top topics, engagement rates, and content measurement stats.
    #[tool]
    async fn get_stats(
        &self,
        Parameters(req): Parameters<GetStatsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let days = req.days.unwrap_or(7);
        let result = tools::analytics::get_stats(&self.state.pool, days).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get follower count snapshots over time.
    #[tool]
    async fn get_follower_trend(
        &self,
        Parameters(req): Parameters<GetFollowerTrendRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = req.limit.unwrap_or(7);
        let result = tools::analytics::get_follower_trend(&self.state.pool, limit).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Action Log ---

    /// Get recent action log entries (searches, replies, tweets, threads, etc.).
    #[tool]
    async fn get_action_log(
        &self,
        Parameters(req): Parameters<GetActionLogRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result =
            tools::actions::get_action_log(&self.state.pool, hours, req.action_type.as_deref())
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get action counts grouped by type.
    #[tool]
    async fn get_action_counts(
        &self,
        Parameters(req): Parameters<SinceHoursRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result = tools::actions::get_action_counts(&self.state.pool, hours).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Rate Limits ---

    /// Get current rate limit status for all action types (reply, tweet, thread, search, mention_check).
    #[tool]
    async fn get_rate_limits(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::rate_limits::get_rate_limits(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Replies ---

    /// Get replies sent within a time window.
    #[tool]
    async fn get_recent_replies(
        &self,
        Parameters(req): Parameters<SinceHoursRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result = tools::replies::get_recent_replies(&self.state.pool, hours).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get count of replies sent today.
    #[tool]
    async fn get_reply_count_today(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::replies::get_reply_count_today(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Target Accounts ---

    /// List active target accounts with engagement stats.
    #[tool]
    async fn list_target_accounts(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::targets::list_target_accounts(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Discovery ---

    /// List unreplied discovered tweets above a relevance score threshold, ordered by score descending.
    #[tool]
    async fn list_unreplied_tweets(
        &self,
        Parameters(req): Parameters<ListUnrepliedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let threshold = req.threshold.unwrap_or(0.0);
        let result = tools::discovery::list_unreplied_tweets(&self.state.pool, threshold).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Scoring ---

    /// Score a tweet for reply-worthiness using the 6-signal scoring engine (keyword relevance, follower count, recency, engagement, reply count, content type).
    #[tool]
    async fn score_tweet(
        &self,
        Parameters(req): Parameters<ScoreTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = tools::scoring::ScoreTweetInput {
            text: &req.text,
            author_username: &req.author_username,
            author_followers: req.author_followers,
            likes: req.likes,
            retweets: req.retweets,
            replies: req.replies,
            created_at: &req.created_at,
        };
        let result = tools::scoring::score_tweet(&self.state.config, &input);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Approval Queue ---

    /// List all pending approval queue items (posts waiting for human review).
    #[tool]
    async fn list_pending_approvals(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::approval::list_pending(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get count of pending approval items.
    #[tool]
    async fn get_pending_count(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::approval::get_pending_count(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Approve a queued post by its ID.
    #[tool]
    async fn approve_item(
        &self,
        Parameters(req): Parameters<ApprovalIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::approval::approve_item(&self.state.pool, req.id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Reject a queued post by its ID.
    #[tool]
    async fn reject_item(
        &self,
        Parameters(req): Parameters<ApprovalIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::approval::reject_item(&self.state.pool, req.id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Approve all pending items in the approval queue.
    #[tool]
    async fn approve_all(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::approval::approve_all(&self.state.pool).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Content Generation ---

    /// Generate a reply to a tweet using the configured LLM provider. Returns the generated reply text. Requires LLM provider to be configured.
    #[tool]
    async fn generate_reply(
        &self,
        Parameters(req): Parameters<GenerateReplyRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                "Error: No LLM provider configured. Set up the [llm] section in config.toml.",
            )]));
        }
        let mention = req.mention_product.unwrap_or(false);
        let result = tools::content::generate_reply(
            &self.state,
            &self.state.config.business,
            &req.tweet_text,
            &req.tweet_author,
            mention,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Generate an original educational tweet using the configured LLM provider. Requires LLM provider to be configured.
    #[tool]
    async fn generate_tweet(
        &self,
        Parameters(req): Parameters<TopicRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                "Error: No LLM provider configured. Set up the [llm] section in config.toml.",
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .industry_topics
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result =
            tools::content::generate_tweet(&self.state, &self.state.config.business, &topic).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Generate a multi-tweet educational thread using the configured LLM provider. Returns 5-8 tweets. Requires LLM provider to be configured.
    #[tool]
    async fn generate_thread(
        &self,
        Parameters(req): Parameters<TopicRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                "Error: No LLM provider configured. Set up the [llm] section in config.toml.",
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .industry_topics
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result =
            tools::content::generate_thread(&self.state, &self.state.config.business, &topic).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Capabilities ---

    /// Get current capabilities, tier info, rate-limit remaining, and safe recommended max actions.
    /// Use this before taking actions to know what's available and how many actions are safe.
    #[tool]
    async fn get_capabilities(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let llm_available = self.state.llm_provider.is_some();
        let result = tools::capabilities::get_capabilities(
            &self.state.pool,
            &self.state.config,
            llm_available,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Configuration & Health ---

    /// Get current Tuitbot configuration (secrets are redacted).
    #[tool]
    async fn get_config(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::config::get_config(&self.state.config);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Validate the current configuration and report any errors.
    #[tool]
    async fn validate_config(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::config::validate_config(&self.state.config);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Check system health: database connectivity and LLM provider status.
    #[tool]
    async fn health_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = self.state.llm_provider.as_deref();
        let result = tools::health::health_check(&self.state.pool, provider).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for TuitbotMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot MCP Server — autonomous X growth assistant. \
                 Provides tools for analytics, content generation, approval queue management, \
                 tweet scoring, and configuration."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
