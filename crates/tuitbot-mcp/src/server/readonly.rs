//! Minimal read-only MCP server (10 tools, no mutations).
//!
//! Provides the smallest useful tool surface for AI agents that only need
//! to read from X API plus a few pure-function utilities.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::kernel;
use crate::provider::retry::{RetryPolicy, RetryingProvider};
use crate::provider::x_api::XApiProvider;
use crate::requests::*;
use crate::state::SharedReadonlyState;
use crate::tools::scoring;

/// Minimal read-only MCP server (10 tools).
#[derive(Clone)]
pub struct ReadonlyMcpServer {
    state: SharedReadonlyState,
    tool_router: ToolRouter<Self>,
}

impl ReadonlyMcpServer {
    /// Create a new readonly-profile MCP server with the given shared state.
    pub fn new(state: SharedReadonlyState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ReadonlyMcpServer {
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

    // ── Utils (2) ───────────────────────────────────────────────────

    /// Get current Tuitbot configuration (secrets are redacted).
    #[tool]
    async fn get_config(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tools::config::get_config(&self.state.config);
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

    // ── Health (1) ──────────────────────────────────────────────────

    /// Check readonly profile health by verifying X client connectivity via get_me.
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
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ReadonlyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot Readonly MCP Server — minimal read-only X API surface. \
                 No mutations, no DB, no LLM."
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
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;

    use crate::kernel;
    use crate::provider::retry::{RetryPolicy, RetryingProvider};
    use crate::provider::x_api::XApiProvider;
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

    fn provider(state: &ReadonlyState) -> RetryingProvider<XApiProvider<'_>> {
        RetryingProvider::new(
            XApiProvider::new(state.x_client.as_ref()),
            RetryPolicy::default(),
        )
    }

    // ── kernel::read ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn readonly_get_tweet_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_tweet(&p, "123").await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_get_user_by_username_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_user_by_username(&p, "alice").await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_search_tweets_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::search_tweets(&p, "rust", 10, None, None).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_get_user_mentions_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_user_mentions(&p, "u1", None, None).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_get_user_tweets_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_user_tweets(&p, "u1", 10, None).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_get_home_timeline_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_home_timeline(&p, "u1", 10, None).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn readonly_get_user_by_id_returns_string() {
        let state = make_state();
        let p = provider(&state);
        let result = kernel::read::get_user_by_id(&p, "u1").await;
        assert!(!result.is_empty());
    }
}
