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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tuitbot_core::config::Config;
    use tuitbot_core::error::XApiError;
    use tuitbot_core::toolkit;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    use crate::state::ReadonlyState;

    // ── Minimal no-op X client ────────────────────────────────────────────
    struct NullX;

    #[async_trait::async_trait]
    impl XApiClient for NullX {
        async fn search_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_mentions(
            &self,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_me(&self) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Err(XApiError::AuthExpired)
        }
        async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
            Err(XApiError::AuthExpired)
        }
    }

    fn make_state() -> Arc<ReadonlyState> {
        Arc::new(ReadonlyState {
            config: Config::default(),
            x_client: Box::new(NullX),
            authenticated_user_id: String::new(),
            x_available: false,
        })
    }

    // ── toolkit::read ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn utility_readonly_get_tweet_error() {
        let state = make_state();
        let result = toolkit::read::get_tweet(state.x_client.as_ref(), "123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_user_by_username_error() {
        let state = make_state();
        let result = toolkit::read::get_user_by_username(state.x_client.as_ref(), "user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_user_by_id_error() {
        let state = make_state();
        let result = toolkit::read::get_user_by_id(state.x_client.as_ref(), "u1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_me_error() {
        let state = make_state();
        let result = toolkit::read::get_me(state.x_client.as_ref()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_search_tweets_empty_query() {
        let state = make_state();
        let result =
            toolkit::read::search_tweets(state.x_client.as_ref(), "", 10, None, None).await;
        // empty query is a validation error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_search_tweets_x_error() {
        let state = make_state();
        let result =
            toolkit::read::search_tweets(state.x_client.as_ref(), "rust", 10, None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_mentions_error() {
        let state = make_state();
        let result = toolkit::read::get_mentions(state.x_client.as_ref(), "u1", None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_user_tweets_error() {
        let state = make_state();
        let result = toolkit::read::get_user_tweets(state.x_client.as_ref(), "u1", 10, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn utility_readonly_get_home_timeline_error() {
        let state = make_state();
        let result =
            toolkit::read::get_home_timeline(state.x_client.as_ref(), "u1", 10, None).await;
        assert!(result.is_err());
    }

    // ── Server construction & ServerHandler ──────────────────────────────

    #[test]
    fn utility_readonly_server_construction() {
        let state = make_state();
        let _server = super::UtilityReadonlyMcpServer::new(state);
    }

    #[test]
    fn utility_readonly_server_info_has_instructions() {
        use rmcp::ServerHandler;
        let state = make_state();
        let server = super::UtilityReadonlyMcpServer::new(state);
        let info = server.get_info();
        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(
            instructions.contains("Utility Read-Only"),
            "instructions should mention Utility Read-Only"
        );
    }

    #[test]
    fn utility_readonly_server_info_has_tool_capabilities() {
        use rmcp::ServerHandler;
        let state = make_state();
        let server = super::UtilityReadonlyMcpServer::new(state);
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn utility_readonly_server_clones() {
        let state = make_state();
        let server = super::UtilityReadonlyMcpServer::new(state);
        let _clone = server.clone();
    }

    // ── Config & scoring ─────────────────────────────────────────────────

    #[test]
    fn utility_readonly_get_config() {
        let state = make_state();
        let result = crate::tools::config::get_config(&state.config);
        assert!(!result.is_empty());
    }

    #[test]
    fn utility_readonly_validate_config() {
        let state = make_state();
        let result = crate::tools::config::validate_config(&state.config);
        assert!(!result.is_empty());
    }

    #[test]
    fn utility_readonly_score_tweet() {
        let state = make_state();
        let input = crate::tools::scoring::ScoreTweetInput {
            text: "Building with Rust",
            author_username: "builder",
            author_followers: 2000,
            likes: 10,
            retweets: 3,
            replies: 1,
            created_at: "2026-01-01T00:00:00Z",
        };
        let result = crate::tools::scoring::score_tweet(&state.config, &input);
        assert!(!result.is_empty());
    }
}
