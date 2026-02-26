//! Session 09 eval scenarios D-G extending the eval_harness pattern.
//!
//! - Scenario D: Direct kernel read flow
//! - Scenario E: Mutation with idempotency enforcement
//! - Scenario F: Rate-limited and auth error behavior
//! - Scenario G: Provider switching behavior (MockProvider vs Scraper)

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde::Serialize;
    use serde_json::Value;

    use crate::contract::ProviderError;
    use crate::kernel::{read, utils};
    use crate::provider::scraper::ScraperReadProvider;
    use crate::provider::SocialReadProvider;
    use crate::requests::ProposeItem;
    use crate::state::{AppState, SharedState};
    use crate::tools::idempotency::IdempotencyStore;
    use tuitbot_core::config::{Config, McpPolicyConfig};
    use tuitbot_core::error::XApiError;
    use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
    use tuitbot_core::storage;
    use tuitbot_core::storage::tweets::DiscoveredTweet;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;
    use tuitbot_core::LlmError;

    // ── Eval types ───────────────────────────────────────────────────

    #[derive(Debug, Serialize)]
    struct EvalResults {
        eval_name: String,
        timestamp: String,
        scenarios: Vec<ScenarioResult>,
        quality_gates: QualityGates,
    }

    #[derive(Debug, Serialize)]
    struct ScenarioResult {
        scenario: String,
        description: String,
        steps: Vec<StepResult>,
        total_latency_ms: u64,
        success: bool,
        schema_valid: bool,
    }

    #[derive(Debug, Serialize)]
    struct StepResult {
        tool_name: String,
        latency_ms: u64,
        success: bool,
        response_valid: bool,
        error_code: Option<String>,
    }

    #[derive(Debug, Serialize)]
    struct QualityGates {
        schema_validation_rate: f64,
        schema_validation_threshold: f64,
        schema_validation_pass: bool,
        unknown_error_rate: f64,
        unknown_error_threshold: f64,
        unknown_error_pass: bool,
        kernel_conformance_rate: f64,
        kernel_conformance_pass: bool,
        error_code_accuracy_rate: f64,
        error_code_accuracy_pass: bool,
        all_pass: bool,
    }

    // ── Mock provider (success) ──────────────────────────────────────

    struct MockProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for MockProvider {
        async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
            Ok(Tweet {
                id: tweet_id.to_string(),
                text: "Mock tweet".to_string(),
                author_id: "author_1".to_string(),
                created_at: "2026-02-25T00:00:00Z".to_string(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }

        async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
            Ok(User {
                id: "u1".to_string(),
                username: username.to_string(),
                name: "Mock User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
            Ok(SearchResponse {
                data: vec![Tweet {
                    id: "s1".to_string(),
                    text: "Found".to_string(),
                    author_id: "a1".to_string(),
                    created_at: String::new(),
                    public_metrics: PublicMetrics::default(),
                    conversation_id: None,
                }],
                includes: None,
                meta: SearchMeta {
                    newest_id: Some("s1".to_string()),
                    oldest_id: Some("s1".to_string()),
                    result_count: 1,
                    next_token: None,
                },
            })
        }

        async fn get_me(&self) -> Result<User, ProviderError> {
            Ok(User {
                id: "me_1".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_followers(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<UsersResponse, ProviderError> {
            Ok(UsersResponse {
                data: vec![User {
                    id: "f1".to_string(),
                    username: "follower1".to_string(),
                    name: "Follower".to_string(),
                    public_metrics: UserMetrics::default(),
                }],
                meta: UsersMeta {
                    result_count: 1,
                    next_token: None,
                },
            })
        }
    }

    // ── Error provider (rate-limited + auth) ─────────────────────────

    struct ErrorProvider;

    #[async_trait::async_trait]
    impl SocialReadProvider for ErrorProvider {
        async fn get_tweet(&self, _tid: &str) -> Result<Tweet, ProviderError> {
            Err(ProviderError::AuthExpired)
        }

        async fn get_user_by_username(&self, _u: &str) -> Result<User, ProviderError> {
            Err(ProviderError::AuthExpired)
        }

        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, ProviderError> {
            Err(ProviderError::RateLimited {
                retry_after: Some(60),
            })
        }

        async fn get_me(&self) -> Result<User, ProviderError> {
            Err(ProviderError::AuthExpired)
        }
    }

    // ── Mock XApiClient ──────────────────────────────────────────────

    struct MockXApiClient;

    #[async_trait::async_trait]
    impl XApiClient for MockXApiClient {
        async fn search_tweets(
            &self,
            _q: &str,
            _max: u32,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![],
                includes: None,
                meta: SearchMeta {
                    newest_id: None,
                    oldest_id: None,
                    result_count: 0,
                    next_token: None,
                },
            })
        }
        async fn get_mentions(
            &self,
            _uid: &str,
            _since: Option<&str>,
            _pt: Option<&str>,
        ) -> Result<MentionResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![],
                includes: None,
                meta: SearchMeta {
                    newest_id: None,
                    oldest_id: None,
                    result_count: 0,
                    next_token: None,
                },
            })
        }
        async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "posted_1".to_string(),
                text: text.to_string(),
            })
        }
        async fn reply_to_tweet(
            &self,
            text: &str,
            _reply_to: &str,
        ) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "reply_1".to_string(),
                text: text.to_string(),
            })
        }
        async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
            Ok(Tweet {
                id: tweet_id.to_string(),
                text: "Test tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: "2026-02-24T00:00:00Z".to_string(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            })
        }
        async fn get_me(&self) -> Result<User, XApiError> {
            Ok(User {
                id: "u1".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }
        async fn get_user_tweets(
            &self,
            _uid: &str,
            _max: u32,
            _pt: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: vec![],
                includes: None,
                meta: SearchMeta {
                    newest_id: None,
                    oldest_id: None,
                    result_count: 0,
                    next_token: None,
                },
            })
        }
        async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
            Ok(User {
                id: "u2".to_string(),
                username: username.to_string(),
                name: "Test".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }
        async fn quote_tweet(&self, text: &str, _quoted: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "qt_1".to_string(),
                text: text.to_string(),
            })
        }
        async fn like_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn follow_user(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(true)
        }
        async fn unfollow_user(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
            Ok(false)
        }
    }

    // ── Mock LLM ─────────────────────────────────────────────────────

    struct MockLlmProvider {
        reply_text: String,
    }

    impl MockLlmProvider {
        fn new(text: &str) -> Self {
            Self {
                reply_text: text.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlmProvider {
        fn name(&self) -> &str {
            "mock"
        }
        async fn complete(
            &self,
            _system: &str,
            _user: &str,
            _params: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                text: self.reply_text.clone(),
                usage: tuitbot_core::llm::TokenUsage {
                    input_tokens: 10,
                    output_tokens: 5,
                },
                model: "mock-model".to_string(),
            })
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    fn validate_schema(json: &str) -> bool {
        let parsed: Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return false,
        };
        parsed.get("success").is_some()
    }

    fn approval_config() -> Config {
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

    async fn make_test_state(
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

    async fn seed_discovered_tweet(state: &SharedState, id: &str, text: &str, author: &str) {
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

    fn artifacts_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/roadmap/artifacts")
    }

    // ══════════════════════════════════════════════════════════════════
    // Scenario D — Direct Kernel Read Flow
    // ══════════════════════════════════════════════════════════════════

    async fn run_scenario_d() -> ScenarioResult {
        let mut steps = Vec::new();

        // Step 1: get_tweet
        let start = std::time::Instant::now();
        let json = read::get_tweet(&MockProvider, "t42").await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "get_tweet".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        // Step 2: search_tweets
        let start = std::time::Instant::now();
        let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "search_tweets".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        // Step 3: get_followers
        let start = std::time::Instant::now();
        let json = read::get_followers(&MockProvider, "u1", 10, None).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "get_followers".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        // Step 4: get_me
        let start = std::time::Instant::now();
        let json = utils::get_me(&MockProvider).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "get_me".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        ScenarioResult {
            scenario: "D".to_string(),
            description: "Direct kernel read flow: get_tweet, search, followers, me".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ══════════════════════════════════════════════════════════════════
    // Scenario E — Mutation with Idempotency Enforcement
    // ══════════════════════════════════════════════════════════════════

    async fn run_scenario_e() -> ScenarioResult {
        let state = make_test_state(
            Some(Box::new(MockXApiClient)),
            Some(Box::new(MockLlmProvider::new("Great point!"))),
            approval_config(),
        )
        .await;
        seed_discovered_tweet(&state, "t1", "Rust async programming", "rustdev").await;

        let mut steps = Vec::new();

        // Step 1: IdempotencyStore first call → None (proceed)
        let params = r#"{"candidate_id":"t1","text":"Great point!"}"#;
        let first = state
            .idempotency
            .check_and_record("propose_and_queue_replies", params);
        steps.push(StepResult {
            tool_name: "idempotency_check_first".to_string(),
            latency_ms: 0,
            success: first.is_none(),
            response_valid: true,
            error_code: None,
        });

        // Step 2: propose_and_queue_replies → succeeds, routes to approval
        let start = std::time::Instant::now();
        let items = vec![ProposeItem {
            candidate_id: "t1".to_string(),
            pre_drafted_text: Some("Great point about Rust!".to_string()),
        }];
        let result =
            crate::tools::workflow::composite::propose_queue::execute(&state, &items, false).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: Value = serde_json::from_str(&result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "propose_and_queue_replies".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        // Step 3: IdempotencyStore same params within 30s → duplicate error
        let duplicate = state
            .idempotency
            .check_and_record("propose_and_queue_replies", params);
        let dup_valid = duplicate
            .as_ref()
            .map(|j| validate_schema(j))
            .unwrap_or(false);
        let dup_code = duplicate.as_ref().and_then(|j| {
            let p: Value = serde_json::from_str(j).unwrap_or_default();
            p["error"]["code"].as_str().map(String::from)
        });
        steps.push(StepResult {
            tool_name: "idempotency_check_duplicate".to_string(),
            latency_ms: 0,
            success: duplicate.is_some(), // success = correctly blocked
            response_valid: dup_valid,
            error_code: dup_code,
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        ScenarioResult {
            scenario: "E".to_string(),
            description: "Mutation with idempotency enforcement".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ══════════════════════════════════════════════════════════════════
    // Scenario F — Rate-Limited Error Behavior
    // ══════════════════════════════════════════════════════════════════

    async fn run_scenario_f() -> ScenarioResult {
        let mut steps = Vec::new();

        // Step 1: search_tweets rate limited
        let start = std::time::Instant::now();
        let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let code = parsed["error"]["code"].as_str().map(String::from);
        let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(false);
        let retry_after = parsed["error"]["retry_after_ms"].as_u64();

        let correct_rate_limit =
            code.as_deref() == Some("x_rate_limited") && retryable && retry_after == Some(60000);

        steps.push(StepResult {
            tool_name: "search_tweets_rate_limited".to_string(),
            latency_ms: elapsed,
            success: correct_rate_limit,
            response_valid: valid,
            error_code: code,
        });

        // Step 2: get_tweet auth expired
        let start = std::time::Instant::now();
        let json = read::get_tweet(&ErrorProvider, "t1").await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let code = parsed["error"]["code"].as_str().map(String::from);
        let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(true);

        let correct_auth = code.as_deref() == Some("x_auth_expired") && !retryable;

        steps.push(StepResult {
            tool_name: "get_tweet_auth_expired".to_string(),
            latency_ms: elapsed,
            success: correct_auth,
            response_valid: valid,
            error_code: code,
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        ScenarioResult {
            scenario: "F".to_string(),
            description: "Rate-limited and auth error behavior validation".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ══════════════════════════════════════════════════════════════════
    // Scenario G — Provider Switching Behavior
    // ══════════════════════════════════════════════════════════════════

    async fn run_scenario_g() -> ScenarioResult {
        let mut steps = Vec::new();
        let scraper = ScraperReadProvider::new();

        // Step 1: MockProvider get_tweet → success
        let start = std::time::Instant::now();
        let json = read::get_tweet(&MockProvider, "t1").await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "get_tweet_mock_provider".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
        });

        // Step 2: ScraperReadProvider get_tweet → error (stub)
        let start = std::time::Instant::now();
        let json = read::get_tweet(&scraper, "t1").await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let code = parsed["error"]["code"].as_str().map(String::from);
        // Scraper stub returns ProviderError::Other → maps to x_api_error
        let expected_scraper_error = code.as_deref() == Some("x_api_error");
        steps.push(StepResult {
            tool_name: "get_tweet_scraper_provider".to_string(),
            latency_ms: elapsed,
            success: expected_scraper_error,
            response_valid: valid,
            error_code: code,
        });

        // Step 3: ScraperReadProvider get_bookmarks → NotConfigured
        let start = std::time::Instant::now();
        let json = read::get_bookmarks(&scraper, "u1", 10, None).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&json);
        let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
        let code = parsed["error"]["code"].as_str().map(String::from);
        // Auth-gated → ProviderError::NotConfigured → x_not_configured
        let expected_not_configured = code.as_deref() == Some("x_not_configured");
        steps.push(StepResult {
            tool_name: "get_bookmarks_scraper_provider".to_string(),
            latency_ms: elapsed,
            success: expected_not_configured,
            response_valid: valid,
            error_code: code,
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        ScenarioResult {
            scenario: "G".to_string(),
            description: "Provider switching: MockProvider vs ScraperReadProvider".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ══════════════════════════════════════════════════════════════════
    // Main eval test
    // ══════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn eval_session09_all_scenarios() {
        let scenario_d = run_scenario_d().await;
        let scenario_e = run_scenario_e().await;
        let scenario_f = run_scenario_f().await;
        let scenario_g = run_scenario_g().await;

        let scenarios = vec![scenario_d, scenario_e, scenario_f, scenario_g];

        // Compute quality gates
        let total_steps: usize = scenarios.iter().map(|s| s.steps.len()).sum();
        let valid_steps: usize = scenarios
            .iter()
            .flat_map(|s| &s.steps)
            .filter(|s| s.response_valid)
            .count();
        let schema_rate = if total_steps > 0 {
            valid_steps as f64 / total_steps as f64
        } else {
            0.0
        };

        let unknown_errors: usize = scenarios
            .iter()
            .flat_map(|s| &s.steps)
            .filter(|s| {
                s.error_code
                    .as_ref()
                    .map(|c| c == "unknown" || c.is_empty())
                    .unwrap_or(false)
            })
            .count();
        let unknown_rate = if total_steps > 0 {
            unknown_errors as f64 / total_steps as f64
        } else {
            0.0
        };

        // Kernel conformance = all scenarios pass
        let conformance_passed = scenarios.iter().filter(|s| s.success).count();
        let conformance_rate = conformance_passed as f64 / scenarios.len() as f64;

        // Error code accuracy: all error steps have correct codes
        let error_steps: Vec<&StepResult> = scenarios
            .iter()
            .flat_map(|s| &s.steps)
            .filter(|s| s.error_code.is_some())
            .collect();
        let accurate_errors = error_steps.iter().filter(|s| s.success).count();
        let error_accuracy = if error_steps.is_empty() {
            1.0
        } else {
            accurate_errors as f64 / error_steps.len() as f64
        };

        let quality_gates = QualityGates {
            schema_validation_rate: schema_rate,
            schema_validation_threshold: 0.95,
            schema_validation_pass: schema_rate >= 0.95,
            unknown_error_rate: unknown_rate,
            unknown_error_threshold: 0.05,
            unknown_error_pass: unknown_rate <= 0.05,
            kernel_conformance_rate: conformance_rate,
            kernel_conformance_pass: (conformance_rate - 1.0).abs() < f64::EPSILON,
            error_code_accuracy_rate: error_accuracy,
            error_code_accuracy_pass: (error_accuracy - 1.0).abs() < f64::EPSILON,
            all_pass: schema_rate >= 0.95
                && unknown_rate <= 0.05
                && (conformance_rate - 1.0).abs() < f64::EPSILON
                && (error_accuracy - 1.0).abs() < f64::EPSILON,
        };

        let results = EvalResults {
            eval_name: "session-09-conformance-evals".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            scenarios,
            quality_gates,
        };

        // Write artifacts
        let dir = artifacts_dir();
        std::fs::create_dir_all(&dir).expect("create artifacts dir");

        let json = serde_json::to_string_pretty(&results).unwrap();
        std::fs::write(dir.join("session-09-eval-results.json"), &json)
            .expect("write eval results");

        // Write handoff markdown
        write_handoff(&results, &dir);

        // Assert gates
        assert!(
            results.quality_gates.schema_validation_pass,
            "Schema validation {:.1}% < 95%",
            results.quality_gates.schema_validation_rate * 100.0
        );
        assert!(
            results.quality_gates.unknown_error_pass,
            "Unknown error rate {:.1}% > 5%",
            results.quality_gates.unknown_error_rate * 100.0
        );
        assert!(
            results.quality_gates.kernel_conformance_pass,
            "Kernel conformance {:.1}% < 100%",
            results.quality_gates.kernel_conformance_rate * 100.0
        );
        assert!(
            results.quality_gates.error_code_accuracy_pass,
            "Error code accuracy {:.1}% < 100%",
            results.quality_gates.error_code_accuracy_rate * 100.0
        );
        assert!(results.quality_gates.all_pass, "Quality gates failed");
    }

    fn write_handoff(results: &EvalResults, dir: &std::path::PathBuf) {
        let mut md = String::from("# Session 09 — Handoff\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));

        md.push_str("## Scenarios\n\n");
        md.push_str("| Scenario | Description | Steps | Total (ms) | Success | Schema |\n");
        md.push_str("|----------|-------------|-------|------------|---------|--------|\n");
        for s in &results.scenarios {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                s.scenario,
                s.description,
                s.steps.len(),
                s.total_latency_ms,
                if s.success { "PASS" } else { "FAIL" },
                if s.schema_valid { "PASS" } else { "FAIL" },
            ));
        }

        md.push_str("\n## Quality Gates\n\n");
        md.push_str("| Gate | Rate | Threshold | Status |\n");
        md.push_str("|------|------|-----------|--------|\n");
        md.push_str(&format!(
            "| Schema validation | {:.1}% | 95% | {} |\n",
            results.quality_gates.schema_validation_rate * 100.0,
            if results.quality_gates.schema_validation_pass {
                "PASS"
            } else {
                "FAIL"
            },
        ));
        md.push_str(&format!(
            "| Unknown errors | {:.1}% | 5% | {} |\n",
            results.quality_gates.unknown_error_rate * 100.0,
            if results.quality_gates.unknown_error_pass {
                "PASS"
            } else {
                "FAIL"
            },
        ));
        md.push_str(&format!(
            "| Kernel conformance | {:.1}% | 100% | {} |\n",
            results.quality_gates.kernel_conformance_rate * 100.0,
            if results.quality_gates.kernel_conformance_pass {
                "PASS"
            } else {
                "FAIL"
            },
        ));
        md.push_str(&format!(
            "| Error code accuracy | {:.1}% | 100% | {} |\n",
            results.quality_gates.error_code_accuracy_rate * 100.0,
            if results.quality_gates.error_code_accuracy_pass {
                "PASS"
            } else {
                "FAIL"
            },
        ));
        md.push_str(&format!(
            "\n**Overall: {}**\n",
            if results.quality_gates.all_pass {
                "ALL GATES PASS"
            } else {
                "GATES FAILED"
            },
        ));

        md.push_str("\n## Session 09 Artifacts\n\n");
        md.push_str("- `session-09-conformance-results.md` — kernel tool conformance\n");
        md.push_str("- `session-09-golden-fixtures.json` — schema golden fixtures\n");
        md.push_str("- `session-09-schema-golden-report.md` — golden fixture report\n");
        md.push_str("- `session-09-eval-results.json` — eval scenario results\n");
        md.push_str("- `session-09-latency-report.md` — benchmark latency gates\n");

        md.push_str("\n## What Session 10 Must Finalize\n\n");
        md.push_str("1. Release documentation (README, CHANGELOG, API docs)\n");
        md.push_str(
            "2. Final manifest regeneration (`cargo test -p tuitbot-mcp manifest -- --ignored`)\n",
        );
        md.push_str("3. Version bump and crates.io publish preparation\n");
        md.push_str("4. End-to-end integration test with real X API sandbox (if available)\n");

        std::fs::write(dir.join("session-09-handoff.md"), &md).expect("write handoff");
    }

    // ── Individual scenario tests ────────────────────────────────────

    #[tokio::test]
    async fn scenario_d_direct_kernel_reads() {
        let result = run_scenario_d().await;
        assert!(result.success, "Scenario D failed");
        assert!(result.schema_valid, "Scenario D schema validation failed");
    }

    #[tokio::test]
    async fn scenario_e_idempotency_enforcement() {
        let result = run_scenario_e().await;
        assert!(result.success, "Scenario E failed");
        assert!(result.schema_valid, "Scenario E schema validation failed");
        // Verify the duplicate was correctly identified
        let dup_step = &result.steps[2];
        assert_eq!(
            dup_step.error_code.as_deref(),
            Some("validation_error"),
            "Duplicate should produce validation_error"
        );
    }

    #[tokio::test]
    async fn scenario_f_error_behavior() {
        let result = run_scenario_f().await;
        assert!(result.success, "Scenario F failed");
        assert!(result.schema_valid, "Scenario F schema validation failed");
    }

    #[tokio::test]
    async fn scenario_g_provider_switching() {
        let result = run_scenario_g().await;
        assert!(result.success, "Scenario G failed");
        assert!(result.schema_valid, "Scenario G schema validation failed");
        // Verify different error codes between providers
        assert_eq!(
            result.steps[1].error_code.as_deref(),
            Some("x_api_error"),
            "Scraper stub should return x_api_error"
        );
        assert_eq!(
            result.steps[2].error_code.as_deref(),
            Some("x_not_configured"),
            "Scraper auth-gated should return x_not_configured"
        );
    }
}
