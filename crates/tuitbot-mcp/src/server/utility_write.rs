//! Utility write MCP server — flat toolkit surface with mutations.
//!
//! Exposes stateless X API read, write, and engage tools backed directly
//! by the toolkit layer. No workflow tools, no DB, no LLM, no policy gate,
//! no composites, no approval routing. Raw toolkit calls only.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::requests::*;
use crate::state::SharedReadonlyState;
use crate::tools::scoring;

use tuitbot_core::toolkit;

use super::toolkit_response::{toolkit_error_to_result, toolkit_read_result};

/// Utility write MCP server — flat toolkit surface with mutations.
#[derive(Clone)]
pub struct UtilityWriteMcpServer {
    state: SharedReadonlyState,
    tool_router: ToolRouter<Self>,
}

impl UtilityWriteMcpServer {
    /// Create a new utility-write MCP server with the given shared state.
    pub fn new(state: SharedReadonlyState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl UtilityWriteMcpServer {
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

    // ── Extended Reads ─────────────────────────────────────────────

    /// Get followers of a user by user ID.
    #[tool]
    async fn x_get_followers(
        &self,
        Parameters(req): Parameters<GetFollowersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 1000);
        let r = toolkit::read::get_followers(
            self.state.x_client.as_ref(),
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get accounts a user is following by user ID.
    #[tool]
    async fn x_get_following(
        &self,
        Parameters(req): Parameters<GetFollowingRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 1000);
        let r = toolkit::read::get_following(
            self.state.x_client.as_ref(),
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get tweets liked by a user.
    #[tool]
    async fn x_get_liked_tweets(
        &self,
        Parameters(req): Parameters<GetLikedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let r = toolkit::read::get_liked_tweets(
            self.state.x_client.as_ref(),
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Get the authenticated user's bookmarks.
    #[tool]
    async fn x_get_bookmarks(
        &self,
        Parameters(req): Parameters<GetBookmarksRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let r = toolkit::read::get_bookmarks(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    /// Look up multiple X users by their IDs (batch, 1-100).
    #[tool]
    async fn x_get_users_by_ids(
        &self,
        Parameters(req): Parameters<GetUsersByIdsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let ids_refs: Vec<&str> = req.user_ids.iter().map(|s| s.as_str()).collect();
        let r = toolkit::read::get_users_by_ids(self.state.x_client.as_ref(), &ids_refs).await;
        toolkit_read_result(r)
    }

    /// Get users who liked a specific tweet.
    #[tool]
    async fn x_get_tweet_liking_users(
        &self,
        Parameters(req): Parameters<GetTweetLikingUsersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 100);
        let r = toolkit::read::get_tweet_liking_users(
            self.state.x_client.as_ref(),
            &req.tweet_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        toolkit_read_result(r)
    }

    // ── Write ──────────────────────────────────────────────────────

    /// Post a new tweet. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_post_tweet(
        &self,
        Parameters(req): Parameters<PostTweetTextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::write::post_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            req.media_ids.as_deref(),
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Reply to an existing tweet. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_reply_to_tweet(
        &self,
        Parameters(req): Parameters<ReplyToTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::write::reply_to_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            &req.in_reply_to_id,
            req.media_ids.as_deref(),
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Post a quote tweet. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_quote_tweet(
        &self,
        Parameters(req): Parameters<QuoteTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::write::quote_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            &req.quoted_tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Delete a tweet. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_delete_tweet(
        &self,
        Parameters(req): Parameters<DeleteTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::write::delete_tweet(self.state.x_client.as_ref(), &req.tweet_id).await;
        toolkit_error_to_result(r)
    }

    /// Post a thread. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_post_thread(
        &self,
        Parameters(req): Parameters<PostThreadMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::write::post_thread(
            self.state.x_client.as_ref(),
            &req.tweets,
            req.media_ids.as_deref(),
        )
        .await;
        toolkit_error_to_result(r)
    }

    // ── Engage ─────────────────────────────────────────────────────

    /// Like a tweet. Raw toolkit call — no policy gate or audit.
    #[tool]
    async fn x_like_tweet(
        &self,
        Parameters(req): Parameters<LikeTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::like_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Unlike a tweet. Raw toolkit call.
    #[tool]
    async fn x_unlike_tweet(
        &self,
        Parameters(req): Parameters<UnlikeTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::unlike_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Follow a user. Raw toolkit call.
    #[tool]
    async fn x_follow_user(
        &self,
        Parameters(req): Parameters<FollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::follow_user(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.target_user_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Unfollow a user. Raw toolkit call.
    #[tool]
    async fn x_unfollow_user(
        &self,
        Parameters(req): Parameters<UnfollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::unfollow_user(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.target_user_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Retweet a tweet. Raw toolkit call.
    #[tool]
    async fn x_retweet(
        &self,
        Parameters(req): Parameters<RetweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::retweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Undo a retweet. Raw toolkit call.
    #[tool]
    async fn x_unretweet(
        &self,
        Parameters(req): Parameters<UnretweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::unretweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Bookmark a tweet. Raw toolkit call.
    #[tool]
    async fn x_bookmark_tweet(
        &self,
        Parameters(req): Parameters<BookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::bookmark_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
    }

    /// Remove a bookmark. Raw toolkit call.
    #[tool]
    async fn x_unbookmark_tweet(
        &self,
        Parameters(req): Parameters<UnbookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let r = toolkit::engage::unbookmark_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        toolkit_error_to_result(r)
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
                "profile": "utility-write",
                "authenticated_user": u.username,
            })
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for UtilityWriteMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot Utility Write MCP Server — flat toolkit surface with mutations. \
                 Stateless X API reads, writes, engages, scoring, and config. \
                 Raw toolkit calls — no workflow, no policy gate, no audit, no DB, no LLM."
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

    // ── toolkit::read (reads also exposed via utility_write) ─────────────

    #[tokio::test]
    async fn utility_write_get_tweet_error() {
        let state = make_state();
        let r = toolkit::read::get_tweet(state.x_client.as_ref(), "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_user_by_username_error() {
        let state = make_state();
        let r = toolkit::read::get_user_by_username(state.x_client.as_ref(), "user").await;
        assert!(r.is_err());
    }

    // ── toolkit::write ────────────────────────────────────────────────────

    #[tokio::test]
    async fn utility_write_post_tweet_empty_text() {
        let state = make_state();
        let r = toolkit::write::post_tweet(state.x_client.as_ref(), "", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_post_tweet_x_error() {
        let state = make_state();
        let r = toolkit::write::post_tweet(state.x_client.as_ref(), "hello world", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_reply_to_tweet_empty_text() {
        let state = make_state();
        let r = toolkit::write::reply_to_tweet(state.x_client.as_ref(), "", "123", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_reply_to_tweet_x_error() {
        let state = make_state();
        let r = toolkit::write::reply_to_tweet(state.x_client.as_ref(), "reply", "123", None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_delete_tweet_error() {
        let state = make_state();
        let r = toolkit::write::delete_tweet(state.x_client.as_ref(), "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_quote_tweet_empty_text() {
        let state = make_state();
        let r = toolkit::write::quote_tweet(state.x_client.as_ref(), "", "456").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_quote_tweet_x_error() {
        let state = make_state();
        let r = toolkit::write::quote_tweet(state.x_client.as_ref(), "quoting", "456").await;
        assert!(r.is_err());
    }

    // ── toolkit::engage ───────────────────────────────────────────────────

    #[tokio::test]
    async fn utility_write_like_tweet_error() {
        let state = make_state();
        let r = toolkit::engage::like_tweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_unlike_tweet_error() {
        let state = make_state();
        let r = toolkit::engage::unlike_tweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_follow_user_error() {
        let state = make_state();
        let r = toolkit::engage::follow_user(state.x_client.as_ref(), "u1", "u2").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_unfollow_user_error() {
        let state = make_state();
        let r = toolkit::engage::unfollow_user(state.x_client.as_ref(), "u1", "u2").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_retweet_error() {
        let state = make_state();
        let r = toolkit::engage::retweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_unretweet_error() {
        let state = make_state();
        let r = toolkit::engage::unretweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_bookmark_tweet_error() {
        let state = make_state();
        let r = toolkit::engage::bookmark_tweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_unbookmark_tweet_error() {
        let state = make_state();
        let r = toolkit::engage::unbookmark_tweet(state.x_client.as_ref(), "u1", "123").await;
        assert!(r.is_err());
    }

    // ── Server construction & ServerHandler ──────────────────────────────

    #[test]
    fn utility_write_server_construction() {
        let state = make_state();
        let _server = super::UtilityWriteMcpServer::new(state);
    }

    #[test]
    fn utility_write_server_info_has_instructions() {
        use rmcp::ServerHandler;
        let state = make_state();
        let server = super::UtilityWriteMcpServer::new(state);
        let info = server.get_info();
        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(
            instructions.contains("Utility Write"),
            "instructions should mention Utility Write"
        );
    }

    #[test]
    fn utility_write_server_info_has_tool_capabilities() {
        use rmcp::ServerHandler;
        let state = make_state();
        let server = super::UtilityWriteMcpServer::new(state);
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn utility_write_server_clones() {
        let state = make_state();
        let server = super::UtilityWriteMcpServer::new(state);
        let _clone = server.clone();
    }

    // ── Config & scoring tools (pure, no X API needed) ───────────────────

    #[test]
    fn utility_write_get_config() {
        let state = make_state();
        let result = crate::tools::config::get_config(&state.config);
        assert!(!result.is_empty());
    }

    #[test]
    fn utility_write_validate_config() {
        let state = make_state();
        let result = crate::tools::config::validate_config(&state.config);
        assert!(!result.is_empty());
    }

    #[test]
    fn utility_write_score_tweet() {
        let state = make_state();
        let input = crate::tools::scoring::ScoreTweetInput {
            text: "Rust is amazing for building CLI tools",
            author_username: "rustacean",
            author_followers: 5000,
            likes: 20,
            retweets: 5,
            replies: 3,
            created_at: "2026-01-01T00:00:00Z",
        };
        let result = crate::tools::scoring::score_tweet(&state.config, &input);
        assert!(!result.is_empty());
    }

    // ── Extended read toolkit calls ──────────────────────────────────────

    #[tokio::test]
    async fn utility_write_search_tweets_error() {
        let state = make_state();
        let r = toolkit::read::search_tweets(state.x_client.as_ref(), "rust", 10, None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_mentions_error() {
        let state = make_state();
        let r = toolkit::read::get_mentions(state.x_client.as_ref(), "u1", None, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_user_tweets_error() {
        let state = make_state();
        let r = toolkit::read::get_user_tweets(state.x_client.as_ref(), "u1", 10, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_home_timeline_error() {
        let state = make_state();
        let r = toolkit::read::get_home_timeline(state.x_client.as_ref(), "u1", 20, None).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_user_by_id_error() {
        let state = make_state();
        let r = toolkit::read::get_user_by_id(state.x_client.as_ref(), "u1").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_get_me_error() {
        let state = make_state();
        let r = toolkit::read::get_me(state.x_client.as_ref()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn utility_write_post_thread_error() {
        let state = make_state();
        let tweets = vec!["tweet 1".to_string(), "tweet 2".to_string()];
        let r = toolkit::write::post_thread(state.x_client.as_ref(), &tweets, None).await;
        assert!(r.is_err());
    }
}
