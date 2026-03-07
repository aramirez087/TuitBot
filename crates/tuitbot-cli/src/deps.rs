//! Shared runtime dependencies for `tuitbot run` and `tuitbot tick`.
//!
//! Encapsulates the ~125 lines of initialization (DB, tokens, tier
//! detection, adapters, posting queue, schedule) into a reusable struct.

use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;

use tuitbot_core::automation::adapters::{
    AnalyticsStorageAdapter, ApprovalQueueAdapter, ContentSafetyAdapter, ContentStorageAdapter,
    LlmReplyAdapter, LlmThreadAdapter, LlmTweetAdapter, PostSenderAdapter, SafetyAdapter,
    ScoringAdapter, StatusQuerierAdapter, StorageAdapter, TargetStorageAdapter, TopicScorerAdapter,
    XApiMentionsAdapter, XApiPostExecutorAdapter, XApiProfileAdapter, XApiSearchAdapter,
    XApiTargetAdapter, XApiThreadPosterAdapter,
};
use tuitbot_core::automation::schedule::ActiveSchedule;
use tuitbot_core::automation::{create_posting_queue, ApprovalQueue, PostAction, TargetLoopConfig};
use tuitbot_core::config::Config;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::safety::SafetyGuard;
use tuitbot_core::scoring::ScoringEngine;
use tuitbot_core::startup::{
    expand_tilde, load_tokens_from_file, token_file_path, ApiTier, TierCapabilities,
};
use tuitbot_core::storage;
use tuitbot_core::x_api::auth::{TokenManager, Tokens};
use tuitbot_core::x_api::tier::{self, detect_tier};
use tuitbot_core::x_api::{create_local_client_with_data_dir, XApiClient, XApiHttpClient};

/// All shared dependencies needed by the automation loops.
pub struct RuntimeDeps {
    pub pool: sqlx::SqlitePool,
    pub tier: ApiTier,
    pub capabilities: TierCapabilities,

    // X API adapters
    pub searcher: Arc<XApiSearchAdapter>,
    pub mentions_fetcher: Arc<XApiMentionsAdapter>,
    pub target_adapter: Arc<XApiTargetAdapter>,
    pub profile_adapter: Arc<XApiProfileAdapter>,
    pub post_executor: Arc<XApiPostExecutorAdapter>,
    pub thread_poster: Arc<XApiThreadPosterAdapter>,

    // Dynamic client (official or local mode)
    pub dyn_client: Arc<dyn XApiClient>,

    // LLM adapters
    pub reply_gen: Arc<LlmReplyAdapter>,
    pub tweet_gen: Arc<LlmTweetAdapter>,
    pub thread_gen: Arc<LlmThreadAdapter>,

    // Scoring / safety
    pub scorer: Arc<ScoringAdapter>,
    pub safety: Arc<SafetyAdapter>,
    pub content_safety: Arc<ContentSafetyAdapter>,

    // Storage adapters
    pub loop_storage: Arc<StorageAdapter>,
    pub content_storage: Arc<ContentStorageAdapter>,
    pub target_storage: Arc<TargetStorageAdapter>,
    pub analytics_storage: Arc<AnalyticsStorageAdapter>,
    pub topic_scorer: Arc<TopicScorerAdapter>,
    pub post_sender: Arc<PostSenderAdapter>,
    pub status_querier: Arc<StatusQuerierAdapter>,

    // Schedule
    pub active_schedule: Option<Arc<ActiveSchedule>>,

    // Posting queue
    pub post_rx: Option<mpsc::Receiver<PostAction>>,

    // Approval
    pub approval_queue: Option<Arc<dyn ApprovalQueue>>,

    // Token refresh (None in scraper mode)
    pub token_manager: Option<Arc<TokenManager>>,
    pub x_client: Option<Arc<XApiHttpClient>>,

    // Config slices needed by loops
    pub keywords: Vec<String>,
    pub target_loop_config: TargetLoopConfig,
}

impl RuntimeDeps {
    /// Initialize all shared dependencies from config.
    ///
    /// This encapsulates DB init, token loading, tier detection,
    /// adapter creation, and posting queue setup.
    ///
    /// In scraper mode (`provider_backend = "scraper"`), skips OAuth token
    /// loading, tier detection, and `get_me()`. Creates a `LocalModeXClient`
    /// instead of `XApiHttpClient`.
    pub async fn init(config: &Config, dry_run: bool) -> anyhow::Result<Self> {
        if config.x_api.provider_backend == "scraper" {
            return Self::init_scraper_mode(config, dry_run).await;
        }

        Self::init_official_mode(config, dry_run).await
    }

    /// Initialize in official X API mode (existing behavior).
    async fn init_official_mode(config: &Config, dry_run: bool) -> anyhow::Result<Self> {
        // 1. Validate database path.
        let db_path = expand_tilde(&config.storage.db_path);
        tracing::info!(path = %db_path.display(), "Database path configured");

        // 2. Load OAuth tokens and create token manager.
        let stored = load_tokens_from_file().map_err(|e| anyhow::anyhow!("{e}"))?;

        let auth_tokens = Tokens {
            access_token: stored.access_token.clone(),
            refresh_token: stored.refresh_token.clone().unwrap_or_default(),
            expires_at: stored.expires_at.unwrap_or_else(Utc::now),
            scopes: stored.scopes.clone(),
        };

        let token_manager = Arc::new(TokenManager::new(
            auth_tokens,
            config.x_api.client_id.clone(),
            token_file_path(),
        ));

        // Attempt refresh if token is expired or near expiry, instead of bailing.
        if let Err(e) = token_manager.refresh_if_needed().await {
            if stored.is_expired() {
                anyhow::bail!(
                    "Authentication expired and refresh failed ({e}). Run `tuitbot auth` to re-authenticate."
                );
            }
            // Token not yet expired but refresh failed — log and continue.
            tracing::warn!(error = %e, "Token refresh attempt failed, continuing with current token");
        }

        let current_token = token_manager
            .tokens_lock()
            .read()
            .await
            .access_token
            .clone();
        tracing::info!(
            expires_in = %stored.format_expiry(),
            "OAuth tokens loaded"
        );

        // 3. Determine API tier by probing the search endpoint.
        let x_client = XApiHttpClient::new(current_token);
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

        // 4. Initialize database.
        let pool = storage::init_db(&config.storage.db_path)
            .await
            .map_err(|e| anyhow::anyhow!("Database initialization failed: {e}"))?;
        tracing::info!("Database initialized");

        // 4b. Inject DB pool into X API client for usage tracking.
        x_client.set_pool(pool.clone()).await;

        // 5. Initialize rate limits.
        storage::rate_limits::init_rate_limits(&pool, &config.limits, &config.intervals)
            .await
            .map_err(|e| anyhow::anyhow!("Rate limit initialization failed: {e}"))?;
        tracing::info!("Rate limits initialized");

        // 5b. Persist detected tier for MCP tools.
        storage::cursors::set_cursor(&pool, "api_tier", &tier.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to persist API tier: {e}"))?;

        // 6. Create LLM provider and content generator.
        let provider = create_provider(&config.llm)
            .map_err(|e| anyhow::anyhow!("LLM provider creation failed: {e}"))?;
        let content_gen = Arc::new(ContentGenerator::new(provider, config.business.clone()));
        tracing::info!("LLM provider and content generator initialized");

        // 7. Create scoring engine and safety guard.
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

        // 8. Get own user ID.
        let x_client = Arc::new(x_client);
        let me = x_client
            .get_me()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get authenticated user: {e}"))?;
        let own_user_id = me.id.clone();
        tracing::info!(user = %me.username, user_id = %own_user_id, "Authenticated as");

        // 9. Create posting queue.
        let (post_tx, post_rx) = create_posting_queue();

        // 10. Create adapter structs.
        // Cast to trait object once for all adapters (AD-06).
        let dyn_client: Arc<dyn XApiClient> = x_client.clone() as Arc<dyn XApiClient>;

        let deps = Self::build_adapters(
            pool,
            tier,
            capabilities,
            dyn_client,
            own_user_id,
            content_gen,
            scoring_engine,
            safety_guard,
            post_tx,
            post_rx,
            config,
            dry_run,
            Some(token_manager),
            Some(x_client.clone()),
            keywords,
        );

        Ok(deps)
    }

    /// Initialize in scraper mode — no OAuth tokens, no tier detection.
    async fn init_scraper_mode(config: &Config, dry_run: bool) -> anyhow::Result<Self> {
        tracing::info!("Starting in Local No-Key Mode (scraper backend)");

        // 1. Validate database path.
        let db_path = expand_tilde(&config.storage.db_path);
        tracing::info!(path = %db_path.display(), "Database path configured");

        // 2. Create LocalModeXClient (with cookie-auth if session exists).
        let data_dir = db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let dyn_client = create_local_client_with_data_dir(&config.x_api, Some(data_dir))
            .await
            .expect("scraper backend should produce a local client");
        tracing::info!(
            allow_mutations = config.x_api.scraper_allow_mutations,
            "Local mode X client created"
        );

        // 3. Synthetic tier: Basic capabilities minus mentions.
        let tier = ApiTier::Basic;
        let capabilities = TierCapabilities {
            mentions: false,
            discovery: true,
            posting: config.x_api.scraper_allow_mutations,
            search: true,
        };
        tracing::info!(tier = %tier, "Scraper mode capabilities: discovery=true, mentions=false, posting={}", capabilities.posting);

        // 4. Initialize database.
        let pool = storage::init_db(&config.storage.db_path)
            .await
            .map_err(|e| anyhow::anyhow!("Database initialization failed: {e}"))?;
        tracing::info!("Database initialized");

        // 5. Initialize rate limits.
        storage::rate_limits::init_rate_limits(&pool, &config.limits, &config.intervals)
            .await
            .map_err(|e| anyhow::anyhow!("Rate limit initialization failed: {e}"))?;
        tracing::info!("Rate limits initialized");

        // 5b. Persist tier for MCP tools.
        storage::cursors::set_cursor(&pool, "api_tier", &tier.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to persist API tier: {e}"))?;

        // 6. Create LLM provider and content generator.
        let provider = create_provider(&config.llm)
            .map_err(|e| anyhow::anyhow!("LLM provider creation failed: {e}"))?;
        let content_gen = Arc::new(ContentGenerator::new(provider, config.business.clone()));
        tracing::info!("LLM provider and content generator initialized");

        // 7. Create scoring engine and safety guard.
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

        // 8. No get_me() — use empty user ID (mentions loop won't run).
        let own_user_id = String::new();

        // 9. Create posting queue.
        let (post_tx, post_rx) = create_posting_queue();

        // 10. Create adapter structs.
        let deps = Self::build_adapters(
            pool,
            tier,
            capabilities,
            dyn_client,
            own_user_id,
            content_gen,
            scoring_engine,
            safety_guard,
            post_tx,
            post_rx,
            config,
            dry_run,
            None, // No token manager in scraper mode
            None, // No XApiHttpClient in scraper mode
            keywords,
        );

        Ok(deps)
    }

    /// Build all adapter structs from shared dependencies.
    ///
    /// Common to both official and scraper init paths.
    #[allow(clippy::too_many_arguments)]
    fn build_adapters(
        pool: sqlx::SqlitePool,
        tier: ApiTier,
        capabilities: TierCapabilities,
        dyn_client: Arc<dyn XApiClient>,
        own_user_id: String,
        content_gen: Arc<ContentGenerator>,
        scoring_engine: Arc<ScoringEngine>,
        safety_guard: Arc<SafetyGuard>,
        post_tx: mpsc::Sender<PostAction>,
        post_rx: mpsc::Receiver<PostAction>,
        config: &Config,
        dry_run: bool,
        token_manager: Option<Arc<TokenManager>>,
        x_client: Option<Arc<XApiHttpClient>>,
        keywords: Vec<String>,
    ) -> Self {
        let searcher: Arc<XApiSearchAdapter> = Arc::new(XApiSearchAdapter::new(dyn_client.clone()));
        let mentions_fetcher: Arc<XApiMentionsAdapter> =
            Arc::new(XApiMentionsAdapter::new(dyn_client.clone(), own_user_id));
        let target_adapter: Arc<XApiTargetAdapter> =
            Arc::new(XApiTargetAdapter::new(dyn_client.clone()));
        let profile_adapter: Arc<XApiProfileAdapter> =
            Arc::new(XApiProfileAdapter::new(dyn_client.clone()));
        let post_executor: Arc<XApiPostExecutorAdapter> =
            Arc::new(XApiPostExecutorAdapter::new(dyn_client.clone()));
        let thread_poster: Arc<XApiThreadPosterAdapter> =
            Arc::new(XApiThreadPosterAdapter::new(dyn_client.clone()));

        let reply_gen: Arc<LlmReplyAdapter> =
            Arc::new(LlmReplyAdapter::new(content_gen.clone(), pool.clone()));
        let tweet_gen: Arc<LlmTweetAdapter> =
            Arc::new(LlmTweetAdapter::new(content_gen.clone(), pool.clone()));
        let thread_gen: Arc<LlmThreadAdapter> =
            Arc::new(LlmThreadAdapter::new(content_gen.clone(), pool.clone()));

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

        // Approval queue (enabled if approval_mode is set or in composer mode).
        let approval_queue: Option<Arc<dyn ApprovalQueue>> = if config.effective_approval_mode() {
            Some(Arc::new(ApprovalQueueAdapter::new(pool.clone())))
        } else {
            None
        };

        // Parse active hours schedule.
        let active_schedule: Option<Arc<ActiveSchedule>> =
            ActiveSchedule::from_config(&config.schedule).map(|s| {
                tracing::info!(
                    timezone = %config.schedule.timezone,
                    hours = format!("{}-{}", config.schedule.active_hours_start, config.schedule.active_hours_end),
                    "Active hours schedule configured"
                );
                Arc::new(s)
            });

        // Target loop config.
        let target_loop_config = TargetLoopConfig {
            accounts: config.targets.accounts.clone(),
            max_target_replies_per_day: config.targets.max_target_replies_per_day,
            dry_run,
        };

        Self {
            pool,
            tier,
            capabilities,
            searcher,
            mentions_fetcher,
            target_adapter,
            profile_adapter,
            post_executor,
            thread_poster,
            dyn_client,
            reply_gen,
            tweet_gen,
            thread_gen,
            scorer,
            safety,
            content_safety,
            loop_storage,
            content_storage,
            target_storage,
            analytics_storage,
            topic_scorer,
            post_sender,
            status_querier,
            active_schedule,
            post_rx: Some(post_rx),
            approval_queue,
            token_manager,
            x_client,
            keywords,
            target_loop_config,
        }
    }
}
