use std::sync::Arc;

use crate::state::{AppState, SharedState};
use crate::tools::idempotency::IdempotencyStore;
use tuitbot_core::config::{Config, McpPolicyConfig};
use tuitbot_core::llm::LlmProvider;
use tuitbot_core::storage;
use tuitbot_core::storage::tweets::DiscoveredTweet;
use tuitbot_core::x_api::XApiClient;

pub use crate::tools::test_mocks::validate_schema;

pub fn approval_config() -> Config {
    let mut config = Config::default();
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        blocked_tools: Vec::new(),
        require_approval_for: Vec::new(),
        dry_run_mutations: false,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config.business.product_keywords = vec!["rust".to_string()];
    config.business.industry_topics = vec!["software".to_string()];
    config.scoring.threshold = 0;
    config.approval_mode = true;
    config
}

pub async fn make_test_state(
    x_client: Option<Box<dyn XApiClient>>,
    llm_provider: Option<Box<dyn LlmProvider>>,
    config: Config,
) -> SharedState {
    let pool = storage::init_test_db().await.expect("init db");
    storage::rate_limits::init_mcp_rate_limit(&pool, config.mcp_policy.max_mutations_per_hour)
        .await
        .expect("init rate limit");
    Arc::new(AppState {
        pool,
        config,
        llm_provider,
        x_client,
        authenticated_user_id: Some("u1".to_string()),
        idempotency: Arc::new(IdempotencyStore::new()),
    })
}

pub async fn seed_discovered_tweet(state: &SharedState, id: &str, text: &str, author: &str) {
    let tweet = DiscoveredTweet {
        id: id.to_string(),
        author_id: "a1".to_string(),
        author_username: author.to_string(),
        content: text.to_string(),
        like_count: 10,
        retweet_count: 2,
        reply_count: 1,
        impression_count: Some(500),
        relevance_score: Some(75.0),
        matched_keyword: Some("rust".to_string()),
        discovered_at: "2026-02-24T12:00:00Z".to_string(),
        replied_to: 0,
    };
    storage::tweets::insert_discovered_tweet(&state.pool, &tweet)
        .await
        .expect("seed tweet");
}
