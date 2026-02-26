//! Admin-profile MCP server (all tools including universal requests).
//!
//! Superset of the write profile. Adds `x_get`, `x_post`, `x_put`, `x_delete`
//! for arbitrary X API endpoint access. Only available when explicitly configured.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::requests::*;
use crate::state::SharedState;
use crate::tools;
use crate::tools::response::{ToolMeta, ToolResponse};
use crate::tools::workflow;

/// Admin-profile MCP server (all tools including universal requests).
#[derive(Clone)]
pub struct AdminMcpServer {
    state: SharedState,
    tool_router: ToolRouter<Self>,
}

impl AdminMcpServer {
    /// Create a new admin-profile MCP server with the given shared state.
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl AdminMcpServer {
    // --- Analytics ---

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

    // --- Action Log ---

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

    // --- Mutation Audit ---

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

    // --- Rate Limits ---

    /// Get current rate limit status for all action types (reply, tweet, thread, search, mention_check).
    #[tool]
    async fn get_rate_limits(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::rate_limits::get_rate_limits(&self.state.pool, &self.state.config).await;
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

    // --- Target Accounts ---

    /// List active target accounts with engagement stats.
    #[tool]
    async fn list_target_accounts(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::targets::list_target_accounts(&self.state.pool, &self.state.config).await;
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
        let result = workflow::discovery::list_unreplied_tweets(
            &self.state.pool,
            threshold,
            &self.state.config,
        )
        .await;
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
        let result =
            workflow::approval::approve_item(&self.state.pool, req.id, &self.state.config).await;
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
        let result = workflow::approval::approve_all(&self.state.pool, &self.state.config).await;
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
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let mention = req.mention_product.unwrap_or(false);
        let result = workflow::content::generate_reply(
            &self.state,
            &self.state.config.business,
            &req.tweet_text,
            &req.tweet_author,
            mention,
            &self.state.config,
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
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .effective_industry_topics()
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result = workflow::content::generate_tweet(
            &self.state,
            &self.state.config.business,
            &topic,
            &self.state.config,
        )
        .await;
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
                ToolResponse::llm_not_configured().to_json(),
            )]));
        }
        let topic = req.topic.unwrap_or_else(|| {
            self.state
                .config
                .business
                .effective_industry_topics()
                .first()
                .cloned()
                .unwrap_or_else(|| "general industry trends".to_string())
        });
        let result = workflow::content::generate_thread(
            &self.state,
            &self.state.config.business,
            &topic,
            &self.state.config,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Capabilities ---

    /// Get current capabilities, tier info, scope analysis, endpoint group availability,
    /// rate-limit remaining, and actionable guidance. Use this before taking actions to know
    /// what's available, which scopes are missing, and how many actions are safe.
    #[tool]
    async fn get_capabilities(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let llm_available = self.state.llm_provider.is_some();
        let x_available = self.state.x_client.is_some();
        let result = workflow::capabilities::get_capabilities(
            &self.state.pool,
            &self.state.config,
            llm_available,
            x_available,
            self.state.authenticated_user_id.as_deref(),
            &self.state.granted_scopes,
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
        let result =
            workflow::health::health_check(&self.state.pool, provider, &self.state.config).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Composer Mode ---

    /// Get the current operating mode (autopilot or composer) and effective approval mode.
    #[tool]
    async fn get_mode(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = std::time::Instant::now();
        let mode = self.state.config.mode.to_string();
        let approval = self.state.config.effective_approval_mode();
        let elapsed = start.elapsed().as_millis() as u64;
        let meta = ToolMeta::new(elapsed).with_workflow(&mode, approval);
        let result = ToolResponse::success(serde_json::json!({
            "mode": mode,
            "approval_mode": approval,
        }))
        .with_meta(meta)
        .to_json();
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Get the current MCP mutation policy status: enforcement settings, blocked tools, rate limit usage, and operating mode.
    #[tool]
    async fn get_policy_status(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::policy_gate::get_policy_status(&self.state).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Create a new draft or scheduled tweet/thread. In composer mode, this is the primary way to queue content.
    #[tool]
    async fn compose_tweet(
        &self,
        Parameters(req): Parameters<ComposeTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let start = std::time::Instant::now();
        let params = serde_json::json!({
            "content": req.content,
            "content_type": req.content_type,
            "scheduled_for": req.scheduled_for,
        })
        .to_string();
        match workflow::policy_gate::check_policy(&self.state, "compose_tweet", &params, start)
            .await
        {
            workflow::policy_gate::GateResult::EarlyReturn(r) => {
                return Ok(CallToolResult::success(vec![Content::text(r)]));
            }
            workflow::policy_gate::GateResult::Proceed => {}
        }
        let content_type = req.content_type.as_deref().unwrap_or("tweet");
        let config = &self.state.config;
        let result = if let Some(scheduled_for) = &req.scheduled_for {
            match tuitbot_core::storage::scheduled_content::insert(
                &self.state.pool,
                content_type,
                &req.content,
                Some(scheduled_for),
            )
            .await
            {
                Ok(id) => {
                    let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                        &self.state.pool,
                        "compose_tweet",
                        &self.state.config.mcp_policy.rate_limits,
                    )
                    .await;
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::success(serde_json::json!({
                        "scheduled_item_id": id,
                        "content_type": content_type,
                        "scheduled_for": scheduled_for,
                    }))
                    .with_meta(meta)
                    .to_json()
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::db_error(format!("Error scheduling content: {e}"))
                        .with_meta(meta)
                        .to_json()
                }
            }
        } else {
            match tuitbot_core::storage::scheduled_content::insert_draft(
                &self.state.pool,
                content_type,
                &req.content,
                "mcp",
            )
            .await
            {
                Ok(id) => {
                    let _ = tuitbot_core::mcp_policy::McpPolicyEvaluator::record_mutation(
                        &self.state.pool,
                        "compose_tweet",
                        &self.state.config.mcp_policy.rate_limits,
                    )
                    .await;
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::success(serde_json::json!({
                        "draft_id": id,
                        "content_type": content_type,
                    }))
                    .with_meta(meta)
                    .to_json()
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let meta = ToolMeta::new(elapsed)
                        .with_workflow(config.mode.to_string(), config.effective_approval_mode());
                    ToolResponse::db_error(format!("Error creating draft: {e}"))
                        .with_meta(meta)
                        .to_json()
                }
            }
        };
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

    // --- Direct X API ---

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

    /// Post a new tweet to X. Returns the posted tweet data. Optionally attach media by providing media_ids from x_upload_media.
    #[tool]
    async fn x_post_tweet(
        &self,
        Parameters(req): Parameters<PostTweetTextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result =
            workflow::x_actions::post_tweet(&self.state, &req.text, req.media_ids.as_deref()).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Reply to an existing tweet. Returns the posted reply data. Optionally attach media.
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

    /// Post a quote tweet referencing another tweet. Returns the posted tweet data. Optionally attach media.
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

    /// Post a thread (ordered sequence of tweets). Validates all tweets before posting. Returns all posted tweet IDs.
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

    /// Upload a media file (image/gif/video) for attaching to tweets. Returns a media_id with upload metadata. Set dry_run=true to validate without uploading.
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

    /// Validate a tweet without posting. Checks length, media IDs, and policy. Returns what would be posted.
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

    /// Validate a thread without posting. Checks all lengths, media per tweet, policy, and reply chain plan. Returns deterministic validation result.
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

    /// Look up multiple X users by their IDs (batch, 1-100). Returns user list.
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

    // --- Context Intelligence ---

    /// Get a rich context profile for an author: prior interactions, response rates, topic affinity, and risk signals.
    #[tool]
    async fn get_author_context(
        &self,
        Parameters(req): Parameters<GetAuthorContextRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = workflow::context::get_author_context(&self.state, &req.identifier).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Recommend an engagement action (reply/skip/observe) for a tweet, with confidence score, contributing factors, and policy considerations.
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

    /// Get topics ranked by performance with "double_down/reduce/maintain/experiment" recommendations over a lookback window.
    #[tool]
    async fn topic_performance_snapshot(
        &self,
        Parameters(req): Parameters<TopicPerformanceSnapshotRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let days = req.lookback_days.unwrap_or(30);
        let result = workflow::context::topic_performance_snapshot(&self.state.pool, days).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // --- Telemetry ---

    /// Get time-windowed MCP tool execution metrics: call counts, success rates, latency percentiles, per tool.
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

    // --- Composite Tools ---

    /// Search X for tweets, score them, persist to DB, and return ranked reply opportunities. Read-only (no posts made).
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

    /// Generate reply drafts for previously discovered tweet candidates. Read-only. Requires LLM provider.
    #[tool]
    async fn draft_replies_for_candidates(
        &self,
        Parameters(req): Parameters<DraftRepliesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                ToolResponse::llm_not_configured().to_json(),
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

    /// Safety-check replies and either queue them for approval or execute them directly. MUTATION — policy-gated.
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

    /// Generate a structured thread with hook analysis and performance estimate. Read-only. Requires LLM provider.
    #[tool]
    async fn generate_thread_plan(
        &self,
        Parameters(req): Parameters<GenerateThreadPlanRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if self.state.llm_provider.is_none() {
            return Ok(CallToolResult::success(vec![Content::text(
                ToolResponse::llm_not_configured().to_json(),
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

    // --- Universal X API Request Tools ---

    /// Send a GET request to any authorized X API endpoint. Supports auto-pagination with next_token. Host is restricted to api.x.com, upload.x.com, upload.twitter.com.
    #[tool]
    async fn x_get(
        &self,
        Parameters(req): Parameters<XGetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = kv_to_tuples(req.query.as_deref());
        let headers = kv_to_tuples(req.headers.as_deref());
        let result = workflow::x_actions::x_request::x_get(
            &self.state,
            &req.path,
            req.host.as_deref(),
            query.as_deref(),
            headers.as_deref(),
            req.auto_paginate,
            req.max_pages,
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Send a POST request to any authorized X API endpoint. Host is restricted to api.x.com, upload.x.com, upload.twitter.com. MUTATION — admin-only, host-constrained.
    #[tool]
    async fn x_post(
        &self,
        Parameters(req): Parameters<XPostRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = kv_to_tuples(req.query.as_deref());
        let headers = kv_to_tuples(req.headers.as_deref());
        let result = workflow::x_actions::x_request::x_post(
            &self.state,
            &req.path,
            req.host.as_deref(),
            query.as_deref(),
            req.body.as_deref(),
            headers.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Send a PUT request to any authorized X API endpoint. Host is restricted to api.x.com, upload.x.com, upload.twitter.com. MUTATION — admin-only, host-constrained.
    #[tool]
    async fn x_put(
        &self,
        Parameters(req): Parameters<XPutRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = kv_to_tuples(req.query.as_deref());
        let headers = kv_to_tuples(req.headers.as_deref());
        let result = workflow::x_actions::x_request::x_put(
            &self.state,
            &req.path,
            req.host.as_deref(),
            query.as_deref(),
            req.body.as_deref(),
            headers.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Send a DELETE request to any authorized X API endpoint. Host is restricted to api.x.com, upload.x.com, upload.twitter.com. MUTATION — admin-only, host-constrained.
    #[tool]
    async fn x_delete(
        &self,
        Parameters(req): Parameters<XDeleteRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = kv_to_tuples(req.query.as_deref());
        let headers = kv_to_tuples(req.headers.as_deref());
        let result = workflow::x_actions::x_request::x_delete(
            &self.state,
            &req.path,
            req.host.as_deref(),
            query.as_deref(),
            headers.as_deref(),
        )
        .await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }
}

/// Convert `Option<&[KeyValue]>` to `Option<Vec<(String, String)>>`.
fn kv_to_tuples(kv: Option<&[crate::requests::KeyValue]>) -> Option<Vec<(String, String)>> {
    kv.map(|pairs| {
        pairs
            .iter()
            .map(|kv| (kv.key.clone(), kv.value.clone()))
            .collect()
    })
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AdminMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Tuitbot Admin MCP Server — full X growth assistant with universal request tools. \
                 Includes all write-profile tools plus x_get/x_post/x_put/x_delete for \
                 arbitrary X API endpoint access."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
