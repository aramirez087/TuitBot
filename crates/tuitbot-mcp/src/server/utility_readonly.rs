//! Utility read-only MCP server — flat toolkit surface.
//!
//! Exposes stateless X API read tools backed directly by the toolkit layer.
//! No workflow tools, no DB, no LLM, no policy gate, no composites.
//! Deterministic mapping: each spec endpoint → exactly one utility tool.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::requests::*;
use crate::state::SharedReadonlyState;
use crate::tools::scoring;

use tuitbot_core::toolkit;

use super::toolkit_response::{toolkit_error_to_result, toolkit_read_result};

/// Utility read-only MCP server — flat toolkit surface.
#[derive(Clone)]
pub struct UtilityReadonlyMcpServer {
    state: SharedReadonlyState,
    tool_router: ToolRouter<Self>,
}

impl UtilityReadonlyMcpServer {
    /// Create a new utility-readonly MCP server with the given shared state.
    pub fn new(state: SharedReadonlyState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl UtilityReadonlyMcpServer {
    // ── Core X API Reads ───────────────────────────────────────────

    /// Get a single tweet by its ID. Returns full tweet data with metrics.
    #[tool]
    async fn get_tweet_by_id(
        &self,
        Parameters(req): Parameters<TweetIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::read::get_tweet(self.state.x_client.as_ref(), &req.tweet_id).await;
        toolkit_read_result(r)
    }

    /// Look up an X user profile by username. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_username(
        &self,
        Parameters(req): Parameters<UsernameRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r =
            toolkit::read::get_user_by_username(self.state.x_client.as_ref(), &req.username).await;
        toolkit_read_result(r)
    }

    /// Search recent tweets matching a query. Returns up to max_results tweets.
    #[tool]
    async fn x_search_tweets(
        &self,
        Parameters(req): Parameters<SearchTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(10, 100);
        let r = toolkit::read::search_tweets(
            self.state.x_client.as_ref(),
            &req.query,
            max,
            req.since_id.as_deref(),
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get recent mentions of the authenticated user.
    #[tool]
    async fn x_get_user_mentions(
        &self,
        Parameters(req): Parameters<GetUserMentionsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::read::get_mentions(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            req.since_id.as_deref(),
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get recent tweets from a specific user by user ID.
    #[tool]
    async fn x_get_user_tweets(
        &self,
        Parameters(req): Parameters<GetUserTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(5, 100);
        let r = toolkit::read::get_user_tweets(
            self.state.x_client.as_ref(),
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get the authenticated user's home timeline (reverse chronological).
    #[tool]
    async fn x_get_home_timeline(
        &self,
        Parameters(req): Parameters<GetHomeTimelineRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(20).clamp(1, 100);
        let r = toolkit::read::get_home_timeline(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Look up an X user profile by user ID. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_id(
        &self,
        Parameters(req): Parameters<GetUserByIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::read::get_user_by_id(self.state.x_client.as_ref(), &req.user_id).await;
        toolkit_read_result(r)
    }

    // ── Utils ──────────────────────────────────────────────────────

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

    // ── Health ─────────────────────────────────────────────────────

    /// Check utility profile health by verifying X client connectivity via get_me.
    #[tool]
    async fn health_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::read::get_me(self.state.x_client.as_ref()).await;
        toolkit_error_to_result(r.map(|u| {
            serde_json::json!({
                "status": "ok",
                "profile": "utility-readonly",
                "authenticated_user": u.username,
            })
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for UtilityReadonlyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot Utility Read-Only MCP Server — flat toolkit surface. \
                 Stateless X API reads, scoring, and config. \
                 No workflow tools, no DB, no LLM, no policy gate."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
