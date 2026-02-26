//! Lightweight API-profile MCP server (~24 tools, no DB/LLM).
//!
//! Provides generic X client tools for AI agents that need to interact
//! with X API without the full TuitBot workflow stack.

use std::time::Instant;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::kernel;
use crate::provider::x_api::XApiProvider;
use crate::requests::*;
use crate::state::SharedApiState;
use crate::tools::response::{ToolMeta, ToolResponse};
use crate::tools::scoring;

/// Lightweight API-profile MCP server.
#[derive(Clone)]
pub struct ApiMcpServer {
    state: SharedApiState,
    tool_router: ToolRouter<Self>,
}

impl ApiMcpServer {
    /// Create a new API-profile MCP server with the given shared state.
    pub fn new(state: SharedApiState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ApiMcpServer {
    // ── Read (14) ───────────────────────────────────────────────────

    /// Get a single tweet by its ID. Returns full tweet data with metrics.
    #[tool]
    async fn get_tweet_by_id(
        &self,
        Parameters(req): Parameters<TweetIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::read::get_tweet(&provider, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Look up an X user profile by username. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_username(
        &self,
        Parameters(req): Parameters<UsernameRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::read::get_home_timeline(
            &provider,
            &self.state.authenticated_user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the authenticated user's profile (username, name, metrics).
    #[tool]
    async fn x_get_me(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::read::get_following(
            &provider,
            &req.user_id,
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::read::get_user_by_id(&provider, &req.user_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get tweets liked by a user. Returns paginated tweet list.
    #[tool]
    async fn x_get_liked_tweets(
        &self,
        Parameters(req): Parameters<GetLikedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
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
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::read::get_tweet_liking_users(
            &provider,
            &req.tweet_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Write (5) ───────────────────────────────────────────────────

    /// Post a new tweet to X. Returns the posted tweet data. Optionally attach media by providing media_ids from x_upload_media.
    #[tool]
    async fn x_post_tweet(
        &self,
        Parameters(req): Parameters<PostTweetTextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::write::post_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Reply to an existing tweet. Returns the posted reply data. Optionally attach media.
    #[tool]
    async fn x_reply_to_tweet(
        &self,
        Parameters(req): Parameters<ReplyToTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::write::reply_to_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            &req.in_reply_to_id,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Post a quote tweet referencing another tweet. Returns the posted tweet data.
    #[tool]
    async fn x_quote_tweet(
        &self,
        Parameters(req): Parameters<QuoteTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::write::quote_tweet(
            self.state.x_client.as_ref(),
            &req.text,
            &req.quoted_tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Delete a tweet by its ID.
    #[tool]
    async fn x_delete_tweet(
        &self,
        Parameters(req): Parameters<DeleteTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::write::delete_tweet(self.state.x_client.as_ref(), &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Post a thread (ordered sequence of tweets). Validates all tweets before posting. Returns all posted tweet IDs.
    #[tool]
    async fn x_post_thread(
        &self,
        Parameters(req): Parameters<PostThreadMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::write::post_thread(
            self.state.x_client.as_ref(),
            &req.tweets,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Engage (8) ──────────────────────────────────────────────────

    /// Like a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_like_tweet(
        &self,
        Parameters(req): Parameters<LikeTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::like_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Follow an X user by user ID.
    #[tool]
    async fn x_follow_user(
        &self,
        Parameters(req): Parameters<FollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::follow_user(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.target_user_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Unfollow an X user by user ID.
    #[tool]
    async fn x_unfollow_user(
        &self,
        Parameters(req): Parameters<UnfollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::unfollow_user(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.target_user_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Retweet a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_retweet(
        &self,
        Parameters(req): Parameters<RetweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::retweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Undo a retweet on behalf of the authenticated user.
    #[tool]
    async fn x_unretweet(
        &self,
        Parameters(req): Parameters<UnretweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::unretweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Unlike a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_unlike_tweet(
        &self,
        Parameters(req): Parameters<UnlikeTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::unlike_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Bookmark a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_bookmark_tweet(
        &self,
        Parameters(req): Parameters<BookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::bookmark_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Remove a bookmark on behalf of the authenticated user.
    #[tool]
    async fn x_unbookmark_tweet(
        &self,
        Parameters(req): Parameters<UnbookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = kernel::engage::unbookmark_tweet(
            self.state.x_client.as_ref(),
            &self.state.authenticated_user_id,
            &req.tweet_id,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Media (1) ───────────────────────────────────────────────────

    /// Upload a media file (image/gif/video) for attaching to tweets. Returns a media_id.
    #[tool]
    async fn x_upload_media(
        &self,
        Parameters(req): Parameters<UploadMediaMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            kernel::media::upload_media(self.state.x_client.as_ref(), &req.file_path).await;
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

    // ── Meta (2) ────────────────────────────────────────────────────

    /// Get API profile capabilities: profile name, tool families, authenticated user.
    #[tool]
    async fn get_capabilities(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = Instant::now();
        let elapsed = start.elapsed().as_millis() as u64;
        let result = ToolResponse::success(serde_json::json!({
            "profile": "api",
            "tool_families": ["read", "write", "engage", "media", "utils", "meta"],
            "x_client": true,
            "authenticated_user_id": self.state.authenticated_user_id,
            "db_available": false,
            "llm_available": false,
        }))
        .with_meta(ToolMeta::new(elapsed))
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Check API profile health by verifying X client connectivity via get_me.
    #[tool]
    async fn health_check(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let provider = XApiProvider::new(self.state.x_client.as_ref());
        let result = kernel::utils::get_me(&provider).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // ── Mode (1) ────────────────────────────────────────────────────

    /// Get the current operating mode and profile.
    #[tool]
    async fn get_mode(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = Instant::now();
        let mode = self.state.config.mode.to_string();
        let approval = self.state.config.effective_approval_mode();
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed).with_mode(&mode, approval);
        let result = ToolResponse::success(serde_json::json!({
            "profile": "api",
            "mode": mode,
            "approval_mode": approval,
        }))
        .with_meta(meta)
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ApiMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot API MCP Server — lightweight X API client. \
                 Provides tools for reading, writing, and engaging on X \
                 without the full TuitBot workflow stack (no DB, no LLM)."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
