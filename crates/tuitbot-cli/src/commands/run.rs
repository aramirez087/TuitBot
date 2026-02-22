//! Implementation of the `replyguy run` command.
//!
//! The main entry point for autonomous operation. Initializes all
//! dependencies, detects API tier, creates adapter structs, spawns
//! automation loops, and runs until a shutdown signal is received.

use std::sync::Arc;
use std::time::Duration;

use replyguy_core::automation::adapters::{
    AnalyticsStorageAdapter, ApprovalQueueAdapter, ContentSafetyAdapter, ContentStorageAdapter,
    LlmReplyAdapter, LlmThreadAdapter, LlmTweetAdapter, PostSenderAdapter, SafetyAdapter,
    ScoringAdapter, StatusQuerierAdapter, StorageAdapter, TargetStorageAdapter, TopicScorerAdapter,
    XApiMentionsAdapter, XApiPostExecutorAdapter, XApiProfileAdapter, XApiSearchAdapter,
    XApiTargetAdapter, XApiThreadPosterAdapter,
};
use replyguy_core::automation::{
    create_posting_queue, run_posting_queue_with_approval, scheduler_from_config,
    status_reporter::run_status_reporter, AnalyticsLoop, ContentLoop, DiscoveryLoop, MentionsLoop,
    Runtime, TargetLoop, TargetLoopConfig, ThreadLoop,
};
use replyguy_core::config::Config;
use replyguy_core::content::ContentGenerator;
use replyguy_core::llm::factory::create_provider;
use replyguy_core::safety::SafetyGuard;
use replyguy_core::scoring::ScoringEngine;
use replyguy_core::startup::{
    expand_tilde, format_startup_banner, load_tokens_from_file, ApiTier, TierCapabilities,
};
use replyguy_core::storage;
use replyguy_core::x_api::tier::{self, detect_tier};
use replyguy_core::x_api::{XApiClient, XApiHttpClient};

/// Execute the `replyguy run` command.
///
/// Startup sequence:
/// 1. Validate database path
/// 2. Load and verify OAuth tokens
/// 3. Detect API tier by probing the search endpoint
/// 4. Apply status_interval override
/// 5. Print startup banner
/// 6. Initialize database
/// 7. Initialize rate limits
/// 8. Create LLM provider and content generator
/// 9. Create scoring engine and safety guard
/// 10. Create posting queue and adapters
/// 11. Spawn automation loops based on tier
/// 12. Run until shutdown
pub async fn execute(config: &Config, status_interval: u64) -> anyhow::Result<()> {
    // 1. Validate database path.
    let db_path = expand_tilde(&config.storage.db_path);
    tracing::info!(path = %db_path.display(), "Database path configured");

    // 2. Load OAuth tokens.
    let tokens = load_tokens_from_file().map_err(|e| anyhow::anyhow!("{e}"))?;

    if tokens.is_expired() {
        anyhow::bail!("Authentication expired. Run `replyguy auth` to re-authenticate.");
    }
    tracing::info!(
        expires_in = %tokens.format_expiry(),
        "OAuth tokens loaded"
    );

    // 3. Determine API tier by probing the search endpoint.
    let x_client = XApiHttpClient::new(tokens.access_token.clone());
    let detected = detect_tier(&x_client)
        .await
        .map_err(|e| anyhow::anyhow!("Tier detection failed: {e}"))?;
    let tier = match detected {
        tier::ApiTier::Free => ApiTier::Free,
        tier::ApiTier::Basic => ApiTier::Basic,
        tier::ApiTier::Pro => ApiTier::Pro,
    };
    let capabilities = TierCapabilities::for_tier(tier);
    tracing::info!(tier = %tier, "{}", capabilities.format_status());

    // 4. Apply status_interval override.
    let effective_interval = if status_interval > 0 {
        status_interval
    } else {
        config.logging.status_interval_seconds
    };

    // 5. Print startup banner (always visible, even in default mode).
    let banner = format_startup_banner(tier, &capabilities, effective_interval);
    eprintln!("{banner}");

    // 6. Initialize database.
    let pool = storage::init_db(&config.storage.db_path)
        .await
        .map_err(|e| anyhow::anyhow!("Database initialization failed: {e}"))?;
    tracing::info!("Database initialized");

    // 7. Initialize rate limits.
    storage::rate_limits::init_rate_limits(&pool, &config.limits, &config.intervals)
        .await
        .map_err(|e| anyhow::anyhow!("Rate limit initialization failed: {e}"))?;
    tracing::info!("Rate limits initialized");

    // 8. Create LLM provider and content generator.
    let provider = create_provider(&config.llm)
        .map_err(|e| anyhow::anyhow!("LLM provider creation failed: {e}"))?;
    let content_gen = Arc::new(ContentGenerator::new(provider, config.business.clone()));
    tracing::info!("LLM provider and content generator initialized");

    // 9. Create scoring engine and safety guard.
    let keywords: Vec<String> = config
        .business
        .product_keywords
        .iter()
        .chain(config.business.competitor_keywords.iter())
        .cloned()
        .collect();
    let scoring_engine = Arc::new(ScoringEngine::new(config.scoring.clone(), keywords.clone()));
    let safety_guard = Arc::new(SafetyGuard::new(pool.clone()));
    tracing::info!("Scoring engine and safety guard initialized");

    // 10. Get own user ID for mentions and target loops.
    let x_client = Arc::new(x_client);
    let me = x_client
        .get_me()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get authenticated user: {e}"))?;
    let own_user_id = me.id.clone();
    tracing::info!(user = %me.username, user_id = %own_user_id, "Authenticated as");

    // 11. Create posting queue.
    let (post_tx, post_rx) = create_posting_queue();

    // 12. Create adapter structs.
    let searcher: Arc<XApiSearchAdapter> = Arc::new(XApiSearchAdapter::new(x_client.clone()));
    let mentions_fetcher: Arc<XApiMentionsAdapter> = Arc::new(XApiMentionsAdapter::new(
        x_client.clone(),
        own_user_id.clone(),
    ));
    let target_adapter: Arc<XApiTargetAdapter> = Arc::new(XApiTargetAdapter::new(x_client.clone()));
    let profile_adapter: Arc<XApiProfileAdapter> =
        Arc::new(XApiProfileAdapter::new(x_client.clone()));
    let post_executor: Arc<XApiPostExecutorAdapter> =
        Arc::new(XApiPostExecutorAdapter::new(x_client.clone()));
    let thread_poster: Arc<XApiThreadPosterAdapter> =
        Arc::new(XApiThreadPosterAdapter::new(x_client.clone()));

    let reply_gen: Arc<LlmReplyAdapter> = Arc::new(LlmReplyAdapter::new(content_gen.clone()));
    let tweet_gen: Arc<LlmTweetAdapter> = Arc::new(LlmTweetAdapter::new(content_gen.clone()));
    let thread_gen: Arc<LlmThreadAdapter> = Arc::new(LlmThreadAdapter::new(content_gen.clone()));

    let scorer: Arc<ScoringAdapter> = Arc::new(ScoringAdapter::new(scoring_engine));
    let safety: Arc<SafetyAdapter> =
        Arc::new(SafetyAdapter::new(safety_guard.clone(), pool.clone()));
    let content_safety: Arc<ContentSafetyAdapter> =
        Arc::new(ContentSafetyAdapter::new(safety_guard));

    let loop_storage: Arc<StorageAdapter> = Arc::new(StorageAdapter::new(pool.clone()));
    let content_storage: Arc<ContentStorageAdapter> =
        Arc::new(ContentStorageAdapter::new(pool.clone(), post_tx.clone()));
    let target_storage: Arc<TargetStorageAdapter> =
        Arc::new(TargetStorageAdapter::new(pool.clone()));
    let analytics_storage: Arc<AnalyticsStorageAdapter> =
        Arc::new(AnalyticsStorageAdapter::new(pool.clone()));
    let topic_scorer: Arc<TopicScorerAdapter> = Arc::new(TopicScorerAdapter::new(pool.clone()));
    let post_sender: Arc<PostSenderAdapter> = Arc::new(PostSenderAdapter::new(post_tx));
    let status_querier: Arc<StatusQuerierAdapter> =
        Arc::new(StatusQuerierAdapter::new(pool.clone()));

    // Approval queue (only if approval mode is enabled).
    let approval_queue: Option<Arc<dyn replyguy_core::automation::ApprovalQueue>> =
        if config.approval_mode {
            Some(Arc::new(ApprovalQueueAdapter::new(pool.clone())))
        } else {
            None
        };

    // 13. Create runtime and spawn tasks.
    let mut runtime = Runtime::new();
    let min_delay = Duration::from_secs(config.limits.min_action_delay_seconds);

    // Spawn posting queue consumer.
    let cancel = runtime.cancel_token();
    runtime.spawn("posting-queue", {
        let executor = post_executor as Arc<dyn replyguy_core::automation::PostExecutor>;
        async move {
            run_posting_queue_with_approval(post_rx, executor, approval_queue, min_delay, cancel)
                .await;
        }
    });

    // --- Content loop (all tiers) ---
    {
        let content_loop = ContentLoop::new(
            tweet_gen,
            content_safety.clone(),
            content_storage.clone(),
            config.business.industry_topics.clone(),
            config.intervals.content_post_window_seconds,
            false,
        )
        .with_topic_scorer(topic_scorer);

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(config.intervals.content_post_window_seconds);
        runtime.spawn("content-loop", async move {
            content_loop.run(cancel, interval).await;
        });
    }

    // --- Thread loop (all tiers) ---
    {
        let thread_loop = ThreadLoop::new(
            thread_gen,
            content_safety,
            content_storage,
            thread_poster,
            config.business.industry_topics.clone(),
            config.intervals.thread_interval_seconds,
            false,
        );

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(config.intervals.thread_interval_seconds);
        runtime.spawn("thread-loop", async move {
            thread_loop.run(cancel, interval).await;
        });
    }

    // --- Tier-gated loops (Basic/Pro only) ---
    if capabilities.discovery {
        // Discovery loop
        let discovery_loop = DiscoveryLoop::new(
            searcher,
            scorer,
            reply_gen.clone(),
            safety.clone(),
            loop_storage.clone(),
            post_sender.clone(),
            keywords,
            config.scoring.threshold as f32,
            false,
        );

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(config.intervals.discovery_search_seconds);
        runtime.spawn("discovery-loop", async move {
            discovery_loop.run(cancel, interval).await;
        });
    }

    if capabilities.mentions {
        // Mentions loop
        let mentions_loop = MentionsLoop::new(
            mentions_fetcher,
            reply_gen.clone(),
            safety.clone(),
            post_sender.clone(),
            false,
        );

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(config.intervals.mentions_check_seconds);
        let storage_clone = loop_storage.clone();
        runtime.spawn("mentions-loop", async move {
            mentions_loop.run(cancel, interval, storage_clone).await;
        });

        // Target loop
        let target_config = TargetLoopConfig {
            accounts: config.targets.accounts.clone(),
            max_target_replies_per_day: config.targets.max_target_replies_per_day,
            auto_follow: config.targets.auto_follow,
            follow_warmup_days: config.targets.follow_warmup_days,
            own_user_id,
            dry_run: false,
        };

        let target_loop = TargetLoop::new(
            target_adapter.clone(),
            target_adapter,
            reply_gen,
            safety,
            target_storage,
            post_sender,
            target_config,
        );

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(config.intervals.mentions_check_seconds);
        runtime.spawn("target-loop", async move {
            target_loop.run(cancel, interval).await;
        });

        // Analytics loop
        let analytics_loop =
            AnalyticsLoop::new(profile_adapter.clone(), profile_adapter, analytics_storage);

        let cancel = runtime.cancel_token();
        let interval = Duration::from_secs(3600); // hourly
        runtime.spawn("analytics-loop", async move {
            analytics_loop.run(cancel, interval).await;
        });
    }

    // --- Status reporter ---
    if effective_interval > 0 {
        let scheduler = scheduler_from_config(effective_interval, 0, 0);
        let cancel = runtime.cancel_token();
        runtime.spawn("status-reporter", async move {
            run_status_reporter(status_querier, scheduler, cancel).await;
        });
    }

    tracing::info!(
        tasks = runtime.task_count(),
        "All automation loops spawned, running until shutdown"
    );

    // 14. Run until shutdown signal.
    runtime.run_until_shutdown().await;

    tracing::info!("Shutdown complete.");
    Ok(())
}
