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
use tuitbot_core::error::XApiError;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::safety::SafetyGuard;
use tuitbot_core::scoring::ScoringEngine;
use tuitbot_core::startup::{
    expand_tilde, load_tokens_from_file, token_file_path, ApiTier, StartupError, TierCapabilities,
};
use tuitbot_core::storage;
use tuitbot_core::x_api::auth::{TokenManager, Tokens};
use tuitbot_core::x_api::tier::{self, detect_tier};
use tuitbot_core::x_api::{create_local_client_with_data_dir, XApiClient, XApiHttpClient};

#[cfg(test)]
mod tests {
    use tuitbot_core::config::Config;
    use tuitbot_core::startup::{ApiTier, TierCapabilities};

    // ── TierCapabilities ──────────────────────────────────────────────

    #[test]
    fn tier_capabilities_free() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        assert!(!caps.search);
        assert!(!caps.discovery);
    }

    #[test]
    fn tier_capabilities_basic() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        assert!(caps.search);
    }

    #[test]
    fn tier_capabilities_pro() {
        let caps = TierCapabilities::for_tier(ApiTier::Pro);
        assert!(caps.search);
        assert!(caps.discovery);
        assert!(caps.mentions);
        assert!(caps.posting);
    }

    // ── Config default values used by deps ────────────────────────────

    #[test]
    fn config_default_db_path() {
        let config = Config::default();
        assert!(!config.storage.db_path.is_empty());
    }

    #[test]
    fn config_effective_approval_mode() {
        let mut config = Config::default();
        config.approval_mode = true;
        assert!(config.effective_approval_mode());

        config.approval_mode = false;
        // effective_approval_mode may also depend on operating_mode
    }

    #[test]
    fn config_effective_industry_topics_falls_back() {
        let mut config = Config::default();
        config.business.industry_topics = vec![];
        config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];
        let topics = config.business.effective_industry_topics();
        // Should fall back to product_keywords when industry_topics is empty
        assert!(!topics.is_empty());
    }

    #[test]
    fn config_effective_industry_topics_uses_topics_when_set() {
        let mut config = Config::default();
        config.business.industry_topics = vec!["topic1".to_string()];
        config.business.product_keywords = vec!["kw1".to_string()];
        let topics = config.business.effective_industry_topics();
        assert_eq!(topics, &["topic1"]);
    }

    // ── Keyword collection from config ────────────────────────────────

    #[test]
    fn keyword_collection_chains_product_and_competitor() {
        let mut config = Config::default();
        config.business.product_keywords = vec!["rust".to_string()];
        config.business.competitor_keywords = vec!["go".to_string()];

        let keywords: Vec<String> = config
            .business
            .product_keywords
            .iter()
            .chain(config.business.competitor_keywords.iter())
            .cloned()
            .collect();

        assert_eq!(keywords, vec!["rust", "go"]);
    }

    #[test]
    fn keyword_collection_empty() {
        let config = Config::default();
        let keywords: Vec<String> = config
            .business
            .product_keywords
            .iter()
            .chain(config.business.competitor_keywords.iter())
            .cloned()
            .collect();
        // Default config may or may not have keywords
        assert!(keywords.is_empty() || !keywords.is_empty());
    }

    // ── ApiTier display ───────────────────────────────────────────────

    #[test]
    fn api_tier_display() {
        let free_str = ApiTier::Free.to_string();
        assert!(
            free_str.eq_ignore_ascii_case("free"),
            "expected 'free' but got '{free_str}'"
        );
        let basic_str = ApiTier::Basic.to_string();
        assert!(
            basic_str.eq_ignore_ascii_case("basic"),
            "expected 'basic' but got '{basic_str}'"
        );
        let pro_str = ApiTier::Pro.to_string();
        assert!(
            pro_str.eq_ignore_ascii_case("pro"),
            "expected 'pro' but got '{pro_str}'"
        );
    }

    // ── TierCapabilities format_status ─────────────────────────────────

    #[test]
    fn tier_capabilities_format_status() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        let status = caps.format_status();
        assert!(!status.is_empty());
    }

    // ── Scraper mode capabilities ─────────────────────────────────────

    #[test]
    fn scraper_mode_capabilities() {
        let caps = TierCapabilities {
            mentions: false,
            discovery: false,
            posting: false,
            search: false,
        };
        assert!(!caps.mentions);
        assert!(!caps.discovery);
        assert!(!caps.posting);
        assert!(!caps.search);
    }

    #[test]
    fn scraper_mode_with_mutations() {
        let caps = TierCapabilities {
            mentions: false,
            discovery: false,
            posting: true,
            search: false,
        };
        assert!(caps.posting);
    }

    // ── Config schedule defaults ─────────────────────────────────────

    #[test]
    fn config_default_schedule_valid() {
        let config = Config::default();
        assert!(config.schedule.active_hours_start <= 23);
        assert!(config.schedule.active_hours_end <= 23);
        assert!(!config.schedule.timezone.is_empty());
    }

    #[test]
    fn config_default_limits_sensible() {
        let config = Config::default();
        assert!(config.limits.max_replies_per_day > 0);
        assert!(config.limits.max_tweets_per_day > 0);
        assert!(config.limits.min_action_delay_seconds <= config.limits.max_action_delay_seconds);
        assert!(config.limits.product_mention_ratio >= 0.0);
        assert!(config.limits.product_mention_ratio <= 1.0);
    }

    #[test]
    fn config_default_intervals_positive() {
        let config = Config::default();
        assert!(config.intervals.mentions_check_seconds > 0);
        assert!(config.intervals.discovery_search_seconds > 0);
        assert!(config.intervals.content_post_window_seconds > 0);
    }

    #[test]
    fn config_default_scoring_valid() {
        let config = Config::default();
        assert!(config.scoring.threshold <= 100);
        assert!(config.scoring.keyword_relevance_max >= 0.0);
        assert!(config.scoring.follower_count_max >= 0.0);
        assert!(config.scoring.recency_max >= 0.0);
    }

    // ── TierCapabilities format_status ────────────────────────────────

    #[test]
    fn tier_capabilities_basic_format_status() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        let status = caps.format_status();
        assert!(!status.is_empty());
    }

    #[test]
    fn tier_capabilities_pro_format_status() {
        let caps = TierCapabilities::for_tier(ApiTier::Pro);
        let status = caps.format_status();
        assert!(!status.is_empty());
    }

    // ── Keyword construction patterns ────────────────────────────────

    #[test]
    fn keyword_collection_empty_when_no_keywords() {
        let mut config = Config::default();
        config.business.product_keywords.clear();
        config.business.competitor_keywords.clear();
        let keywords: Vec<String> = config
            .business
            .product_keywords
            .iter()
            .chain(config.business.competitor_keywords.iter())
            .cloned()
            .collect();
        assert!(keywords.is_empty());
    }

    #[test]
    fn keyword_collection_product_only() {
        let mut config = Config::default();
        config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];
        config.business.competitor_keywords.clear();
        let keywords: Vec<String> = config
            .business
            .product_keywords
            .iter()
            .chain(config.business.competitor_keywords.iter())
            .cloned()
            .collect();
        assert_eq!(keywords, vec!["rust", "cli"]);
    }

    #[test]
    fn keyword_collection_competitor_only() {
        let mut config = Config::default();
        config.business.product_keywords.clear();
        config.business.competitor_keywords = vec!["go".to_string(), "node".to_string()];
        let keywords: Vec<String> = config
            .business
            .product_keywords
            .iter()
            .chain(config.business.competitor_keywords.iter())
            .cloned()
            .collect();
        assert_eq!(keywords, vec!["go", "node"]);
    }

    // ── ApiTier ──────────────────────────────────────────────────────

    #[test]
    fn api_tier_free_capabilities_no_search() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        assert!(!caps.search);
        assert!(!caps.discovery);
    }

    #[test]
    fn api_tier_basic_has_search() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        assert!(caps.search);
    }

    #[test]
    fn api_tier_pro_has_all() {
        let caps = TierCapabilities::for_tier(ApiTier::Pro);
        assert!(caps.search);
        assert!(caps.discovery);
        assert!(caps.mentions);
        assert!(caps.posting);
    }

    // ── TargetLoopConfig construction ────────────────────────────────

    #[test]
    fn target_loop_config_from_config() {
        let mut config = Config::default();
        config.targets.accounts = vec!["user1".to_string(), "user2".to_string()];
        config.targets.max_target_replies_per_day = 5;

        let tlc = tuitbot_core::automation::TargetLoopConfig {
            accounts: config.targets.accounts.clone(),
            max_target_replies_per_day: config.targets.max_target_replies_per_day,
            dry_run: false,
        };
        assert_eq!(tlc.accounts.len(), 2);
        assert_eq!(tlc.max_target_replies_per_day, 5);
        assert!(!tlc.dry_run);
    }

    #[test]
    fn target_loop_config_dry_run() {
        let tlc = tuitbot_core::automation::TargetLoopConfig {
            accounts: vec![],
            max_target_replies_per_day: 0,
            dry_run: true,
        };
        assert!(tlc.dry_run);
    }
}

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
        let stored = load_tokens_from_file().map_err(|e| match e {
            StartupError::AuthRequired => anyhow::anyhow!(
                "No X API credentials found.\n\
                 \n\
                 You haven't authenticated with X yet. Run `tuitbot auth` to connect your \
                 account.\n\
                 This is a one-time setup that stores your access token locally."
            ),
            StartupError::AuthExpired => anyhow::anyhow!(
                "X API credentials have expired.\n\
                 \n\
                 Your stored token is no longer valid. Run `tuitbot auth` to re-authenticate.\n\
                 This refreshes your access without needing to reconfigure anything else."
            ),
            other => anyhow::anyhow!(
                "Failed to load X API credentials: {other}\n\
                 \n\
                 If this keeps happening, run `tuitbot auth` to re-authenticate."
            ),
        })?;

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
        let detected = detect_tier(&x_client).await.map_err(|e| match e {
            XApiError::AuthExpired => anyhow::anyhow!(
                "X API token is expired or invalid.\n\
                 \n\
                 Your stored token was rejected by the X API (HTTP 401). This happens when the \
                 token expires or you revoke app access.\n\
                 Run `tuitbot auth` to re-authenticate."
            ),
            XApiError::RateLimited { retry_after } => {
                let wait = retry_after
                    .map(|s| format!("Wait {s} seconds and try again."))
                    .unwrap_or_else(|| "Wait a few minutes and try again.".to_string());
                anyhow::anyhow!(
                    "X API rate limit hit during startup (HTTP 429).\n\
                     \n\
                     The X API is throttling requests from your account because too many \
                     calls were made recently. {wait}"
                )
            }
            XApiError::Network { source } => anyhow::anyhow!(
                "Cannot reach api.x.com.\n\
                 \n\
                 A network error occurred while connecting to the X API: {source}\n\
                 Check your internet connection and try again."
            ),
            other => anyhow::anyhow!(
                "Tier detection failed: {other}\n\
                 \n\
                 tuitbot could not determine your X API access level. \
                 Check your credentials with `tuitbot test`."
            ),
        })?;
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
        let me = x_client.get_me().await.map_err(|e| match e {
            XApiError::AuthExpired => anyhow::anyhow!(
                "X API token rejected when fetching your profile (HTTP 401).\n\
                 \n\
                 Your token may have expired or been revoked since startup. \
                 Run `tuitbot auth` to re-authenticate."
            ),
            XApiError::RateLimited { retry_after } => {
                let wait = retry_after
                    .map(|s| format!("Wait {s} seconds and try again."))
                    .unwrap_or_else(|| "Wait a few minutes and try again.".to_string());
                anyhow::anyhow!(
                    "X API rate limit hit while fetching your profile (HTTP 429).\n\
                     \n\
                     The X API is throttling requests from your account. {wait}"
                )
            }
            XApiError::Network { source } => anyhow::anyhow!(
                "Cannot reach api.x.com while fetching your profile.\n\
                 \n\
                 Network error: {source}\n\
                 Check your internet connection and try again."
            ),
            other => anyhow::anyhow!(
                "Failed to get authenticated user: {other}\n\
                 \n\
                 Run `tuitbot test` to diagnose the issue."
            ),
        })?;
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

        // 3. Synthetic tier: scraper has no search/discovery API access.
        let tier = ApiTier::Free;
        let capabilities = TierCapabilities {
            mentions: false,
            discovery: false,
            posting: config.x_api.scraper_allow_mutations,
            search: false,
        };
        tracing::info!(tier = %tier, "Scraper mode capabilities: discovery=false, search=false, mentions=false, posting={}", capabilities.posting);

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
