//! Tool router A — analytics, audit, rates, replies, targets, discovery,
//! scoring, approval queue, config/health, policy, and basic X API tools.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_router};

use crate::requests::*;
use crate::tools;
use crate::tools::workflow;

use super::AdminMcpServer;

#[tool_router(router = tools_router, vis = "pub(crate)")]
impl AdminMcpServer {
    /// Get analytics dashboard: follower trend, top topics, engagement rates, and content measurement stats.
    #[tool]
    async fn get_stats(
        &self,
        Parameters(req): Parameters<GetStatsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let days = req.days.unwrap_or(7);
        let result =
            workflow::analytics::get_stats(&self.state.pool, days, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get follower count snapshots over time.
    #[tool]
    async fn get_follower_trend(
        &self,
        Parameters(req): Parameters<GetFollowerTrendRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let limit = req.limit.unwrap_or(7);
        let result =
            workflow::analytics::get_follower_trend(&self.state.pool, limit, &self.state.config)
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get recent action log entries (searches, replies, tweets, threads, etc.).
    #[tool]
    async fn get_action_log(
        &self,
        Parameters(req): Parameters<GetActionLogRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result = workflow::actions::get_action_log(
            &self.state.pool,
            hours,
            req.action_type.as_deref(),
            &self.state.config,
        )
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
        let result =
            workflow::actions::get_action_counts(&self.state.pool, hours, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get recent mutation audit entries. Shows what writes the agent has performed, with status and timing.
    #[tool]
    async fn get_recent_mutations(
        &self,
        Parameters(req): Parameters<GetRecentMutationsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::mutation_audit::get_recent_mutations(
            &self.state.pool,
            req.limit.unwrap_or(20),
            req.tool_name.as_deref(),
            req.status.as_deref(),
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get full detail of a single mutation by correlation ID, including rollback guidance.
    #[tool]
    async fn get_mutation_detail(
        &self,
        Parameters(req): Parameters<GetMutationDetailRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::mutation_audit::get_mutation_detail(
            &self.state.pool,
            &req.correlation_id,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get current rate limit status for all action types.
    #[tool]
    async fn get_rate_limits(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::rate_limits::get_rate_limits(&self.state.pool, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get replies sent within a time window.
    #[tool]
    async fn get_recent_replies(
        &self,
        Parameters(req): Parameters<SinceHoursRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let hours = req.since_hours.unwrap_or(24);
        let result =
            workflow::replies::get_recent_replies(&self.state.pool, hours, &self.state.config)
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get count of replies sent today.
    #[tool]
    async fn get_reply_count_today(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::replies::get_reply_count_today(&self.state.pool, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// List active target accounts with engagement stats.
    #[tool]
    async fn list_target_accounts(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::targets::list_target_accounts(&self.state.pool, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// List unreplied discovered tweets above a relevance score threshold, ordered by score descending.
    #[tool]
    async fn list_unreplied_tweets(
        &self,
        Parameters(req): Parameters<ListUnrepliedTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let threshold = req.threshold.unwrap_or(0.0);
        let result = workflow::discovery::list_unreplied_tweets(
            &self.state.pool,
            threshold,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Score a tweet for reply-worthiness using the 6-signal scoring engine.
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
    /// List all pending approval queue items (posts waiting for human review).
    #[tool]
    async fn list_pending_approvals(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::approval::list_pending(&self.state.pool, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get count of pending approval items.
    #[tool]
    async fn get_pending_count(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::approval::get_pending_count(&self.state.pool, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Approve a queued post by its ID.
    #[tool]
    async fn approve_item(
        &self,
        Parameters(req): Parameters<ApprovalIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let x_available = self.state.x_client.is_some();
        let result = workflow::approval::approve_item(
            &self.state.pool,
            req.id,
            &self.state.config,
            x_available,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Reject a queued post by its ID.
    #[tool]
    async fn reject_item(
        &self,
        Parameters(req): Parameters<ApprovalIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::approval::reject_item(&self.state.pool, req.id, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Approve all pending items in the approval queue.
    #[tool]
    async fn approve_all(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let x_available = self.state.x_client.is_some();
        let result =
            workflow::approval::approve_all(&self.state.pool, &self.state.config, x_available)
                .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
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
        let result =
            workflow::health::health_check(&self.state.pool, provider, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get the current MCP mutation policy status: enforcement settings, blocked tools, rate limit usage, and operating mode.
    #[tool]
    async fn get_policy_status(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::policy_gate::get_policy_status(&self.state).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Browse high-scoring discovered tweets for manual engagement.
    #[tool]
    async fn get_discovery_feed(
        &self,
        Parameters(req): Parameters<DiscoveryFeedRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let threshold = req.min_score.unwrap_or(50.0);
        let limit = req.limit.unwrap_or(10);
        let result = workflow::discovery::list_unreplied_tweets_with_limit(
            &self.state.pool,
            threshold,
            limit,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get analytics-driven topic recommendations based on past performance.
    #[tool]
    async fn suggest_topics(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::analytics::get_top_topics(&self.state.pool, 10, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Get a single tweet by its ID. Returns full tweet data with metrics.
    #[tool]
    async fn get_tweet_by_id(
        &self,
        Parameters(req): Parameters<TweetIdRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::get_tweet_by_id(&self.state, &req.tweet_id).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Look up an X user profile by username. Returns user data with public metrics.
    #[tool]
    async fn x_get_user_by_username(
        &self,
        Parameters(req): Parameters<UsernameRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::get_user_by_username(&self.state, &req.username).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Search recent tweets matching a query. Returns up to max_results tweets.
    #[tool]
    async fn x_search_tweets(
        &self,
        Parameters(req): Parameters<SearchTweetsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let max = req.max_results.unwrap_or(10).clamp(10, 100);
        let result = workflow::x_actions::search_tweets(
            &self.state,
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
        let result = workflow::x_actions::get_user_mentions(
            &self.state,
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
        let result = workflow::x_actions::get_user_tweets(
            &self.state,
            &req.user_id,
            max,
            req.pagination_token.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Post a new tweet to X. Optionally attach media by providing media_ids from x_upload_media.
    #[tool]
    async fn x_post_tweet(
        &self,
        Parameters(req): Parameters<PostTweetTextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::x_actions::post_tweet(&self.state, &req.text, req.media_ids.as_deref()).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
    /// Reply to an existing tweet. Optionally attach media.
    #[tool]
    async fn x_reply_to_tweet(
        &self,
        Parameters(req): Parameters<ReplyToTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::x_actions::reply_to_tweet(
            &self.state,
            &req.text,
            &req.in_reply_to_id,
            req.media_ids.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}
