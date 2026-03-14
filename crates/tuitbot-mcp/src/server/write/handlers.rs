//! Tool router B — X API engagement/write/media, context intelligence,
//! telemetry, and composite tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_router};

use crate::requests::*;
use crate::tools::workflow;

use super::WriteMcpServer;

#[tool_router(router = handlers_router, vis = "pub(crate)")]
impl WriteMcpServer {
    /// Post a quote tweet referencing another tweet. Optionally attach media.
    #[tool]
    async fn x_quote_tweet(
        &self,
        Parameters(req): Parameters<QuoteTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::quote_tweet(
            &self.state,
            &req.text,
            &req.quoted_tweet_id,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Like a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_like_tweet(
        &self,
        Parameters(req): Parameters<LikeTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::like_tweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Follow an X user by user ID.
    #[tool]
    async fn x_follow_user(
        &self,
        Parameters(req): Parameters<FollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::follow_user(&self.state, &req.target_user_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Unfollow an X user by user ID.
    #[tool]
    async fn x_unfollow_user(
        &self,
        Parameters(req): Parameters<UnfollowUserMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::unfollow_user(&self.state, &req.target_user_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Retweet a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_retweet(
        &self,
        Parameters(req): Parameters<RetweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::retweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Undo a retweet on behalf of the authenticated user.
    #[tool]
    async fn x_unretweet(
        &self,
        Parameters(req): Parameters<UnretweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::unretweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Delete a tweet by its ID. Always requires policy approval.
    #[tool]
    async fn x_delete_tweet(
        &self,
        Parameters(req): Parameters<DeleteTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::delete_tweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Post a thread (ordered sequence of tweets). Validates all tweets before posting.
    #[tool]
    async fn x_post_thread(
        &self,
        Parameters(req): Parameters<PostThreadMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::x_actions::post_thread(&self.state, &req.tweets, req.media_ids.as_deref())
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Upload a media file (image/gif/video) for attaching to tweets. Set dry_run=true to validate without uploading.
    #[tool]
    async fn x_upload_media(
        &self,
        Parameters(req): Parameters<UploadMediaMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::upload_media(
            &self.state,
            &req.file_path,
            req.alt_text.as_deref(),
            req.dry_run,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Validate a tweet without posting. Checks length, media IDs, and policy.
    #[tool]
    async fn x_post_tweet_dry_run(
        &self,
        Parameters(req): Parameters<PostTweetDryRunRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::post_tweet_dry_run(
            &self.state,
            &req.text,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Validate a thread without posting. Checks all lengths, media per tweet, policy, and reply chain plan.
    #[tool]
    async fn x_post_thread_dry_run(
        &self,
        Parameters(req): Parameters<PostThreadDryRunRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::post_thread_dry_run(
            &self.state,
            &req.tweets,
            req.media_ids.as_deref(),
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
        let result = workflow::x_actions::get_home_timeline(
            &self.state,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get followers of a user by user ID. Returns paginated user list.
    #[tool]
    async fn x_get_followers(
        &self,
        Parameters(req): Parameters<GetFollowersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 1000);
        let result = workflow::x_actions::get_followers(
            &self.state,
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
        let result = workflow::x_actions::get_following(
            &self.state,
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
        let result = workflow::x_actions::get_user_by_id(&self.state, &req.user_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get tweets liked by a user. Returns paginated tweet list.
    #[tool]
    async fn x_get_liked_tweets(
        &self,
        Parameters(req): Parameters<GetLikedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(1, 100);
        let result = workflow::x_actions::get_liked_tweets(
            &self.state,
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
        let result =
            workflow::x_actions::get_bookmarks(&self.state, max, req.pagination_token.as_deref())
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Look up multiple X users by their IDs (batch, 1-100).
    #[tool]
    async fn x_get_users_by_ids(
        &self,
        Parameters(req): Parameters<GetUsersByIdsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let ids_refs: Vec<&str> = req.user_ids.iter().map(|s| s.as_str()).collect();
        let result = workflow::x_actions::get_users_by_ids(&self.state, &ids_refs).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get users who liked a specific tweet. Returns paginated user list.
    #[tool]
    async fn x_get_tweet_liking_users(
        &self,
        Parameters(req): Parameters<GetTweetLikingUsersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(100).clamp(1, 100);
        let result = workflow::x_actions::get_tweet_liking_users(
            &self.state,
            &req.tweet_id,
            max,
            req.pagination_token.as_deref(),
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
        let result = workflow::x_actions::unlike_tweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Bookmark a tweet on behalf of the authenticated user.
    #[tool]
    async fn x_bookmark_tweet(
        &self,
        Parameters(req): Parameters<BookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::bookmark_tweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Remove a bookmark on behalf of the authenticated user.
    #[tool]
    async fn x_unbookmark_tweet(
        &self,
        Parameters(req): Parameters<UnbookmarkTweetMcpRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::unbookmark_tweet(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get X API usage statistics: costs, call counts, and endpoint breakdown.
    #[tool]
    async fn get_x_usage(
        &self,
        Parameters(req): Parameters<GetXUsageRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let days = req.days.unwrap_or(7);
        let result = workflow::x_actions::get_x_usage(&self.state, days).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get a rich context profile for an author: prior interactions, response rates, topic affinity, and risk signals.
    #[tool]
    async fn get_author_context(
        &self,
        Parameters(req): Parameters<GetAuthorContextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::context::get_author_context(&self.state, &req.identifier).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Recommend an engagement action (reply/skip/observe) for a tweet, with confidence score and policy considerations.
    #[tool]
    async fn recommend_engagement_action(
        &self,
        Parameters(req): Parameters<RecommendEngagementRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::context::recommend_engagement(
            &self.state,
            &req.author_username,
            &req.tweet_text,
            req.campaign_objective.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get topics ranked by performance with strategy recommendations over a lookback window.
    #[tool]
    async fn topic_performance_snapshot(
        &self,
        Parameters(req): Parameters<TopicPerformanceSnapshotRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let days = req.lookback_days.unwrap_or(30);
        let result = workflow::context::topic_performance_snapshot(&self.state.pool, days).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get time-windowed MCP tool execution metrics: call counts, success rates, latency percentiles.
    #[tool]
    async fn get_mcp_tool_metrics(
        &self,
        Parameters(req): Parameters<GetMcpToolMetricsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result = workflow::telemetry::get_mcp_tool_metrics(&self.state.pool, hours).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get MCP tool error distribution grouped by tool and error code in a time window.
    #[tool]
    async fn get_mcp_error_breakdown(
        &self,
        Parameters(req): Parameters<GetMcpErrorBreakdownRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result = workflow::telemetry::get_mcp_error_breakdown(&self.state.pool, hours).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Search X for tweets, score them, persist to DB, and return ranked reply opportunities. Read-only.
    #[tool]
    async fn find_reply_opportunities(
        &self,
        Parameters(req): Parameters<FindReplyOpportunitiesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::composite::find_opportunities::execute(
            &self.state,
            req.query.as_deref(),
            req.min_score,
            req.limit,
            req.since_id.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Generate reply drafts for previously discovered tweet candidates. Requires LLM provider.
    #[tool]
    async fn draft_replies_for_candidates(
        &self,
        Parameters(req): Parameters<DraftRepliesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                crate::tools::response::ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let mention = req.mention_product.unwrap_or(false);
        let result = workflow::composite::draft_replies::execute(
            &self.state,
            &req.candidate_ids,
            req.archetype.as_deref(),
            mention,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Safety-check replies and either queue for approval or execute directly. MUTATION — policy-gated.
    #[tool]
    async fn propose_and_queue_replies(
        &self,
        Parameters(req): Parameters<ProposeAndQueueRepliesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mention = req.mention_product.unwrap_or(false);
        let result =
            workflow::composite::propose_queue::execute(&self.state, &req.items, mention).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Generate a structured thread with hook analysis and performance estimate. Requires LLM provider.
    #[tool]
    async fn generate_thread_plan(
        &self,
        Parameters(req): Parameters<GenerateThreadPlanRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                crate::tools::response::ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let result = workflow::composite::thread_plan::execute(
            &self.state,
            &req.topic,
            req.objective.as_deref(),
            req.target_audience.as_deref(),
            req.structure.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}
