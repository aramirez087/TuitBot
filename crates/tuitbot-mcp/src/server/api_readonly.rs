//! API read-only MCP server (20 tools, no mutations).
//!
//! Provides the full X API read surface plus utility and meta tools
//! for AI agents that need broad read access without mutation capability.

use std::time::Instant;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::kernel;
use crate::provider::retry::{RetryPolicy, RetryingProvider};
use crate::provider::x_api::XApiProvider;
use crate::requests::*;
use crate::state::SharedReadonlyState;
use crate::tools::response::{ToolMeta, ToolResponse};
use crate::tools::scoring;

/// API read-only MCP server (20 tools).
#[derive(Clone)]
pub struct ApiReadonlyMcpServer {
    state: SharedReadonlyState,
    tool_router: ToolRouter<Self>,
}

impl ApiReadonlyMcpServer {
    /// Create a new api-readonly-profile MCP server with the given shared state.
    pub fn new(state: SharedReadonlyState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ApiReadonlyMcpServer {
    // ── Core X API Reads (7) ────────────────────────────────────────

    /// Get a single tweet by its ID. Returns full tweet data with metrics.
    #[tool]
    async fn get_tweet_by_id(
        &self,
        Parameters(req): Parameters<TweetIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_tweet(&provider, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Look up an X user profile by username. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_username(
        &self,
        Parameters(req): Parameters<UsernameRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_user_by_username(&provider, &req.username).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Search recent tweets matching a query. Returns up to max_results tweets.
    #[tool]
    async fn x_search_tweets(
        &self,
        Parameters(req): Parameters<SearchTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(10, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::search_tweets(
            &provider,
            &req.query,
            max,
            req.since_id.as_deref(),
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get recent mentions of the authenticated user.
    #[tool]
    async fn x_get_user_mentions(
        &self,
        Parameters(req): Parameters<GetUserMentionsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_user_mentions(
            &provider,
            &self.state.authenticated_user_id,
            req.since_id.as_deref(),
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get recent tweets from a specific user by user ID.
    #[tool]
    async fn x_get_user_tweets(
        &self,
        Parameters(req): Parameters<GetUserTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(5, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_user_tweets(
            &provider,
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the authenticated user's home timeline (reverse chronological).
    #[tool]
    async fn x_get_home_timeline(
        &self,
        Parameters(req): Parameters<GetHomeTimelineRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(20).clamp(1, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_home_timeline(
            &provider,
            &self.state.authenticated_user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Look up an X user profile by user ID. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_id(
        &self,
        Parameters(req): Parameters<GetUserByIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_user_by_id(&provider, &req.user_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Extended X API Reads (7) ────────────────────────────────────

    /// Get the authenticated user's profile (username, name, metrics).
    #[tool]
    async fn x_get_me(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::utils::get_me(&provider).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get followers of a user by user ID. Returns paginated user list.
    #[tool]
    async fn x_get_followers(
        &self,
        Parameters(req): Parameters<GetFollowersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 1000);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_followers(
            &provider,
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get accounts a user is following by user ID. Returns paginated user list.
    #[tool]
    async fn x_get_following(
        &self,
        Parameters(req): Parameters<GetFollowingRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 1000);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_following(
            &provider,
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get tweets liked by a user. Returns paginated tweet list.
    #[tool]
    async fn x_get_liked_tweets(
        &self,
        Parameters(req): Parameters<GetLikedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_liked_tweets(
            &provider,
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the authenticated user's bookmarks. Returns paginated tweet list.
    #[tool]
    async fn x_get_bookmarks(
        &self,
        Parameters(req): Parameters<GetBookmarksRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_bookmarks(
            &provider,
            &self.state.authenticated_user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Look up multiple X users by their IDs (batch, 1-100). Returns user list.
    #[tool]
    async fn x_get_users_by_ids(
        &self,
        Parameters(req): Parameters<GetUsersByIdsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let ids_refs: Vec<&str> = req.user_ids.iter().map(|s| s.as_str()).collect();
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_users_by_ids(&provider, &ids_refs).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get users who liked a specific tweet. Returns paginated user list.
    #[tool]
    async fn x_get_tweet_liking_users(
        &self,
        Parameters(req): Parameters<GetTweetLikingUsersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 100);
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::read::get_tweet_liking_users(
            &provider,
            &req.tweet_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Utils (3) ───────────────────────────────────────────────────

    /// Get current Tuitbot configuration (secrets are redacted).
    #[tool]
    async fn get_config(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tools::config::get_config(&self.state.config);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Validate the current configuration and report any errors.
    #[tool]
    async fn validate_config(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tools::config::validate_config(&self.state.config);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Score a tweet for reply-worthiness using the 6-signal scoring engine.
    #[tool]
    async fn score_tweet(
        &self,
        Parameters(req): Parameters<ScoreTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = scoring::ScoreTweetInput {
            text: &req.text,
            author_username: &req.author_username,
            author_followers: req.author_followers,
            likes: req.likes,
            retweets: req.retweets,
            replies: req.replies,
            created_at: &req.created_at,
        };
        let result = scoring::score_tweet(&self.state.config, &input);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Meta (3) ────────────────────────────────────────────────────

    /// Get API read-only profile capabilities: profile name, tool families, authenticated user.
    #[tool]
    async fn get_capabilities(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = Instant::now();
        let elapsed = start.elapsed().as_millis() as u64;
        let result = ToolResponse::success(serde_json::json!({
            "profile": "api-readonly",
            "tool_families": ["read", "utils", "meta"],
            "x_client": true,
            "authenticated_user_id": self.state.authenticated_user_id,
            "db_available": false,
            "llm_available": false,
        }))
        .with_meta(ToolMeta::new(elapsed))
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Check API read-only profile health by verifying X client connectivity via get_me.
    #[tool]
    async fn health_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        if !self.state.x_available {
            let result = serde_json::json!({
                "status": "degraded",
                "x_client": false,
                "available_tools": ["get_config", "score_tweet"],
                "message": "X API tokens not configured. Run `tuitbot auth` to authenticate. Config and scoring tools are available."
            });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }
        let provider = RetryingProvider::new(
            XApiProvider::new(self.state.x_client.as_ref()),
            RetryPolicy::default(),
        );
        let result = kernel::utils::get_me(&provider).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the current operating mode and profile.
    #[tool]
    async fn get_mode(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = Instant::now();
        let mode = self.state.config.mode.to_string();
        let approval = self.state.config.effective_approval_mode();
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed).with_workflow(&mode, approval);
        let result = ToolResponse::success(serde_json::json!({
            "profile": "api-readonly",
            "mode": mode,
            "approval_mode": approval,
        }))
        .with_meta(meta)
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ApiReadonlyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot API Read-Only MCP Server — broad read-only X API surface \
                 with utility and meta tools. No mutations, no DB, no LLM."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
