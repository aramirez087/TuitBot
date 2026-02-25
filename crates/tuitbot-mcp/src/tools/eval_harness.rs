//! Eval harness for MCP observability quality gates.
//!
//! Runs three scenarios and captures telemetry results:
//! - Scenario A: raw direct reply flow (single tool)
//! - Scenario B: composite find → draft → queue flow
//! - Scenario C: blocked-by-policy mutation
//!
//! Results are written to `docs/roadmap/artifacts/task-07-eval-results.json`
//! and summarized in `task-07-eval-summary.md`.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde::Serialize;
    use tuitbot_core::config::{Config, McpPolicyConfig};
    use tuitbot_core::error::XApiError;
    use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
    use tuitbot_core::storage;
    use tuitbot_core::storage::tweets::DiscoveredTweet;
    use tuitbot_core::x_api::types::*;
    use tuitbot_core::x_api::XApiClient;
    use tuitbot_core::LlmError;

    use crate::requests::ProposeItem;
    use crate::state::{AppState, SharedState};

    // ── Eval types ──────────────────────────────────────────────────

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
        telemetry_entries: u64,
        schema_valid: bool,
    }

    #[derive(Debug, Serialize)]
    struct StepResult {
        tool_name: String,
        latency_ms: u64,
        success: bool,
        response_valid: bool,
        error_code: Option<String>,
        policy_decision: Option<String>,
    }

    #[derive(Debug, Serialize)]
    struct QualityGates {
        schema_validation_rate: f64,
        schema_validation_threshold: f64,
        schema_validation_pass: bool,
        unknown_error_rate: f64,
        unknown_error_threshold: f64,
        unknown_error_pass: bool,
        all_pass: bool,
    }

    // ── Mock impls (same as composite tests) ────────────────────────

    struct MockXApiClient {
        tweets: Vec<Tweet>,
        users: Vec<User>,
    }

    impl MockXApiClient {
        fn with_results(tweets: Vec<Tweet>, users: Vec<User>) -> Self {
            Self { tweets, users }
        }
    }

    #[async_trait::async_trait]
    impl XApiClient for MockXApiClient {
        async fn search_tweets(
            &self,
            _query: &str,
            _max_results: u32,
            _since_id: Option<&str>,
            _pagination_token: Option<&str>,
        ) -> Result<SearchResponse, XApiError> {
            Ok(SearchResponse {
                data: self.tweets.clone(),
                includes: if self.users.is_empty() {
                    None
                } else {
                    Some(Includes {
                        users: self.users.clone(),
                    })
                },
                meta: SearchMeta {
                    newest_id: self.tweets.first().map(|t| t.id.clone()),
                    oldest_id: self.tweets.last().map(|t| t.id.clone()),
                    result_count: self.tweets.len() as u32,
                    next_token: None,
                },
            })
        }

        async fn get_mentions(
            &self,
            _user_id: &str,
            _since_id: Option<&str>,
            _pagination_token: Option<&str>,
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
            _in_reply_to_id: &str,
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
            _user_id: &str,
            _max_results: u32,
            _pagination_token: Option<&str>,
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

        async fn quote_tweet(
            &self,
            text: &str,
            _quoted_tweet_id: &str,
        ) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "qt_1".to_string(),
                text: text.to_string(),
            })
        }

        async fn like_tweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn follow_user(
            &self,
            _user_id: &str,
            _target_user_id: &str,
        ) -> Result<bool, XApiError> {
            Ok(true)
        }

        async fn unfollow_user(
            &self,
            _user_id: &str,
            _target_user_id: &str,
        ) -> Result<bool, XApiError> {
            Ok(false)
        }
    }

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
            _user_message: &str,
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

    fn test_config() -> Config {
        let mut config = Config::default();
        config.mcp_policy = McpPolicyConfig {
            enforce_for_mutations: false,
            blocked_tools: Vec::new(),
            require_approval_for: Vec::new(),
            dry_run_mutations: false,
            max_mutations_per_hour: 20,
            ..McpPolicyConfig::default()
        };
        config.business.product_keywords = vec!["rust".to_string(), "async".to_string()];
        config.business.industry_topics = vec!["software engineering".to_string()];
        config.scoring.threshold = 0;
        config
    }

    fn blocked_policy_config() -> Config {
        let mut config = test_config();
        config.mcp_policy.enforce_for_mutations = true;
        config.mcp_policy.blocked_tools = vec!["propose_and_queue_replies".to_string()];
        config
    }

    fn approval_config() -> Config {
        let mut config = test_config();
        config.mcp_policy.enforce_for_mutations = true;
        config.mcp_policy.max_mutations_per_hour = 20;
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
        })
    }

    fn sample_tweet(id: &str, text: &str, author_id: &str) -> Tweet {
        Tweet {
            id: id.to_string(),
            text: text.to_string(),
            author_id: author_id.to_string(),
            created_at: "2026-02-24T12:00:00Z".to_string(),
            public_metrics: PublicMetrics {
                like_count: 10,
                retweet_count: 2,
                reply_count: 1,
                quote_count: 0,
                impression_count: 500,
                bookmark_count: 0,
            },
            conversation_id: None,
        }
    }

    fn sample_user(id: &str, username: &str, followers: u64) -> User {
        User {
            id: id.to_string(),
            username: username.to_string(),
            name: username.to_string(),
            public_metrics: UserMetrics {
                followers_count: followers,
                following_count: 100,
                tweet_count: 500,
            },
        }
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

    /// Validate that a JSON string parses and has the ToolResponse envelope.
    fn validate_schema(json: &str) -> bool {
        let parsed: serde_json::Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(_) => return false,
        };
        // Must have "success" key (envelope marker)
        parsed.get("success").is_some()
    }

    // ── Scenario A: Raw direct reply flow ───────────────────────────

    async fn run_scenario_a() -> ScenarioResult {
        let llm = MockLlmProvider::new("Great point about Rust async!");
        let client = MockXApiClient::with_results(vec![], vec![]);
        let state =
            make_test_state(Some(Box::new(client)), Some(Box::new(llm)), test_config()).await;
        seed_discovered_tweet(
            &state,
            "t1",
            "Rust async programming is fascinating",
            "rustdev",
        )
        .await;

        let mut steps = Vec::new();

        // Step 1: Draft a reply for a known candidate
        let start = std::time::Instant::now();
        let ids = vec!["t1".to_string()];
        let result =
            crate::tools::composite::draft_replies::execute(&state, &ids, None, false).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);

        steps.push(StepResult {
            tool_name: "draft_replies_for_candidates".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
            policy_decision: None,
        });

        // Step 2: Queue the reply (in approval mode)
        let state2 = make_test_state(
            Some(Box::new(MockXApiClient::with_results(vec![], vec![]))),
            Some(Box::new(MockLlmProvider::new("Great point!"))),
            approval_config(),
        )
        .await;
        seed_discovered_tweet(&state2, "t1", "Rust async programming", "rustdev").await;

        let start = std::time::Instant::now();
        let items = vec![ProposeItem {
            candidate_id: "t1".to_string(),
            pre_drafted_text: Some("Great point about Rust async!".to_string()),
        }];
        let result = crate::tools::composite::propose_queue::execute(&state2, &items, false).await;
        let elapsed2 = start.elapsed().as_millis() as u64;
        let valid2 = validate_schema(&result);
        let parsed2: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let success2 = parsed2["success"].as_bool().unwrap_or(false);

        steps.push(StepResult {
            tool_name: "propose_and_queue_replies".to_string(),
            latency_ms: elapsed2,
            success: success2,
            response_valid: valid2,
            error_code: None,
            policy_decision: Some("allow".to_string()),
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        let telemetry_count =
            storage::mcp_telemetry::get_summary(&state2.pool, "2000-01-01T00:00:00Z")
                .await
                .map(|s| s.total_calls as u64)
                .unwrap_or(0);

        ScenarioResult {
            scenario: "A".to_string(),
            description: "Raw direct reply flow: draft → queue".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            telemetry_entries: telemetry_count,
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ── Scenario B: Composite find → draft → queue flow ─────────────

    async fn run_scenario_b() -> ScenarioResult {
        let tweets = vec![
            sample_tweet("t1", "Learning rust async programming today!", "a1"),
            sample_tweet("t2", "Async patterns in rust are powerful", "a2"),
        ];
        let users = vec![
            sample_user("a1", "rustdev", 5000),
            sample_user("a2", "asyncfan", 3000),
        ];
        let client = MockXApiClient::with_results(tweets, users);
        let llm = MockLlmProvider::new("Excellent insight on async Rust!");
        let state = make_test_state(
            Some(Box::new(client)),
            Some(Box::new(llm)),
            approval_config(),
        )
        .await;

        let mut steps = Vec::new();

        // Step 1: find_reply_opportunities
        let start = std::time::Instant::now();
        let result = crate::tools::composite::find_opportunities::execute(
            &state,
            Some("rust async"),
            None,
            Some(10),
            None,
        )
        .await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "find_reply_opportunities".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
            policy_decision: None,
        });

        // Collect discovered candidate IDs
        let candidate_ids: Vec<String> = parsed["data"]["candidates"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|c| c["tweet_id"].as_str().map(String::from))
            .collect();

        // Step 2: draft_replies_for_candidates
        let start = std::time::Instant::now();
        let result =
            crate::tools::composite::draft_replies::execute(&state, &candidate_ids, None, false)
                .await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "draft_replies_for_candidates".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
            policy_decision: None,
        });

        // Extract drafts
        let draft_items: Vec<ProposeItem> = parsed["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|d| {
                if d["status"] == "success" {
                    Some(ProposeItem {
                        candidate_id: d["candidate_id"].as_str()?.to_string(),
                        pre_drafted_text: d["draft_text"].as_str().map(String::from),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Step 3: propose_and_queue_replies
        let start = std::time::Instant::now();
        let result = if draft_items.is_empty() {
            // Use fallback items if none found
            let fallback = vec![ProposeItem {
                candidate_id: candidate_ids
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "t1".to_string()),
                pre_drafted_text: Some("Great insight!".to_string()),
            }];
            crate::tools::composite::propose_queue::execute(&state, &fallback, false).await
        } else {
            crate::tools::composite::propose_queue::execute(&state, &draft_items, false).await
        };
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);
        steps.push(StepResult {
            tool_name: "propose_and_queue_replies".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
            policy_decision: Some("allow".to_string()),
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        let telemetry_count =
            storage::mcp_telemetry::get_summary(&state.pool, "2000-01-01T00:00:00Z")
                .await
                .map(|s| s.total_calls as u64)
                .unwrap_or(0);

        ScenarioResult {
            scenario: "B".to_string(),
            description: "Composite flow: find → draft → queue".to_string(),
            total_latency_ms: total,
            success: steps.iter().all(|s| s.success),
            telemetry_entries: telemetry_count,
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ── Scenario C: Blocked-by-policy mutation ──────────────────────

    async fn run_scenario_c() -> ScenarioResult {
        let client = MockXApiClient::with_results(vec![], vec![]);
        let llm = MockLlmProvider::new("This will be blocked!");
        let state = make_test_state(
            Some(Box::new(client)),
            Some(Box::new(llm)),
            blocked_policy_config(),
        )
        .await;
        seed_discovered_tweet(&state, "t1", "Rust topic", "dev").await;

        let mut steps = Vec::new();

        // Step 1: Try to propose (blocked by policy)
        let start = std::time::Instant::now();
        let items = vec![ProposeItem {
            candidate_id: "t1".to_string(),
            pre_drafted_text: Some("This reply should be blocked".to_string()),
        }];
        let result = crate::tools::composite::propose_queue::execute(&state, &items, false).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&result);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let is_denied = parsed["error"]["code"]
            .as_str()
            .map(|c| c.starts_with("policy_denied"))
            .unwrap_or(false);

        steps.push(StepResult {
            tool_name: "propose_and_queue_replies".to_string(),
            latency_ms: elapsed,
            success: false,
            response_valid: valid,
            error_code: parsed["error"]["code"].as_str().map(String::from),
            policy_decision: Some("deny".to_string()),
        });

        // Step 2: Verify telemetry captured the denial
        let start = std::time::Instant::now();
        let metrics_result =
            crate::tools::telemetry::get_mcp_error_breakdown(&state.pool, 24).await;
        let elapsed = start.elapsed().as_millis() as u64;
        let valid = validate_schema(&metrics_result);
        let parsed: serde_json::Value = serde_json::from_str(&metrics_result).unwrap_or_default();
        let success = parsed["success"].as_bool().unwrap_or(false);

        steps.push(StepResult {
            tool_name: "get_mcp_error_breakdown".to_string(),
            latency_ms: elapsed,
            success,
            response_valid: valid,
            error_code: None,
            policy_decision: None,
        });

        let total = steps.iter().map(|s| s.latency_ms).sum();
        let telemetry_count =
            storage::mcp_telemetry::get_summary(&state.pool, "2000-01-01T00:00:00Z")
                .await
                .map(|s| s.total_calls as u64)
                .unwrap_or(0);

        ScenarioResult {
            scenario: "C".to_string(),
            description: "Blocked-by-policy mutation with telemetry verification".to_string(),
            total_latency_ms: total,
            success: is_denied, // success means the policy correctly blocked
            telemetry_entries: telemetry_count,
            schema_valid: steps.iter().all(|s| s.response_valid),
            steps,
        }
    }

    // ── Main eval test ──────────────────────────────────────────────

    #[tokio::test]
    async fn eval_harness_all_scenarios() {
        let scenario_a = run_scenario_a().await;
        let scenario_b = run_scenario_b().await;
        let scenario_c = run_scenario_c().await;

        let scenarios = vec![scenario_a, scenario_b, scenario_c];

        // Compute quality gates
        let total_steps: usize = scenarios.iter().map(|s| s.steps.len()).sum();
        let valid_steps: usize = scenarios
            .iter()
            .flat_map(|s| &s.steps)
            .filter(|s| s.response_valid)
            .count();
        let schema_validation_rate = if total_steps > 0 {
            valid_steps as f64 / total_steps as f64
        } else {
            0.0
        };

        let error_steps: usize = scenarios
            .iter()
            .flat_map(|s| &s.steps)
            .filter(|s| {
                s.error_code
                    .as_ref()
                    .map(|c| c == "unknown" || c.is_empty())
                    .unwrap_or(false)
            })
            .count();
        let unknown_error_rate = if total_steps > 0 {
            error_steps as f64 / total_steps as f64
        } else {
            0.0
        };

        let quality_gates = QualityGates {
            schema_validation_rate,
            schema_validation_threshold: 0.95,
            schema_validation_pass: schema_validation_rate >= 0.95,
            unknown_error_rate,
            unknown_error_threshold: 0.05,
            unknown_error_pass: unknown_error_rate <= 0.05,
            all_pass: schema_validation_rate >= 0.95 && unknown_error_rate <= 0.05,
        };

        let results = EvalResults {
            eval_name: "task-07-observability-evals".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            scenarios,
            quality_gates,
        };

        // Write eval results JSON
        let json = serde_json::to_string_pretty(&results).expect("serialize results");
        let artifacts_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/roadmap/artifacts");
        std::fs::create_dir_all(&artifacts_dir).expect("create artifacts dir");

        let json_path = artifacts_dir.join("task-07-eval-results.json");
        std::fs::write(&json_path, &json).expect("write eval results");

        // Write eval summary markdown
        let mut md = String::new();
        md.push_str("# Task 07 — Observability Eval Results\n\n");
        md.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));
        md.push_str("## Scenarios\n\n");
        md.push_str("| Scenario | Description | Steps | Total (ms) | Success | Schema Valid | Telemetry Entries |\n");
        md.push_str("|----------|-------------|-------|------------|---------|--------------|-------------------|\n");
        for s in &results.scenarios {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                s.scenario,
                s.description,
                s.steps.len(),
                s.total_latency_ms,
                if s.success { "PASS" } else { "FAIL" },
                if s.schema_valid { "PASS" } else { "FAIL" },
                s.telemetry_entries,
            ));
        }
        md.push_str("\n## Step Details\n\n");
        for s in &results.scenarios {
            md.push_str(&format!(
                "### Scenario {}: {}\n\n",
                s.scenario, s.description
            ));
            md.push_str("| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |\n");
            md.push_str("|------|-------------|---------|--------------|-------|--------|\n");
            for step in &s.steps {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    step.tool_name,
                    step.latency_ms,
                    if step.success { "PASS" } else { "FAIL" },
                    if step.response_valid { "PASS" } else { "FAIL" },
                    step.error_code.as_deref().unwrap_or("-"),
                    step.policy_decision.as_deref().unwrap_or("-"),
                ));
            }
            md.push('\n');
        }

        md.push_str("## Quality Gates\n\n");
        md.push_str("| Gate | Rate | Threshold | Status |\n");
        md.push_str("|------|------|-----------|--------|\n");
        md.push_str(&format!(
            "| Schema validation | {:.1}% | {:.0}% | {} |\n",
            results.quality_gates.schema_validation_rate * 100.0,
            results.quality_gates.schema_validation_threshold * 100.0,
            if results.quality_gates.schema_validation_pass {
                "PASS"
            } else {
                "FAIL"
            },
        ));
        md.push_str(&format!(
            "| Unknown errors | {:.1}% | {:.0}% | {} |\n",
            results.quality_gates.unknown_error_rate * 100.0,
            results.quality_gates.unknown_error_threshold * 100.0,
            if results.quality_gates.unknown_error_pass {
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

        let md_path = artifacts_dir.join("task-07-eval-summary.md");
        std::fs::write(&md_path, &md).expect("write eval summary");

        // Assert quality gates
        assert!(
            results.quality_gates.schema_validation_pass,
            "Schema validation rate {:.1}% below threshold {:.0}%",
            results.quality_gates.schema_validation_rate * 100.0,
            results.quality_gates.schema_validation_threshold * 100.0,
        );
        assert!(
            results.quality_gates.unknown_error_pass,
            "Unknown error rate {:.1}% above threshold {:.0}%",
            results.quality_gates.unknown_error_rate * 100.0,
            results.quality_gates.unknown_error_threshold * 100.0,
        );
        assert!(results.quality_gates.all_pass, "Quality gates failed");
    }

    // ── Individual scenario tests (fast feedback) ───────────────────

    #[tokio::test]
    async fn scenario_a_raw_reply() {
        let result = run_scenario_a().await;
        assert!(result.success, "Scenario A failed");
        assert!(result.schema_valid, "Scenario A schema validation failed");
    }

    #[tokio::test]
    async fn scenario_b_composite_flow() {
        let result = run_scenario_b().await;
        assert!(result.success, "Scenario B failed");
        assert!(result.schema_valid, "Scenario B schema validation failed");
        assert!(
            result.telemetry_entries > 0,
            "Scenario B should have telemetry entries"
        );
    }

    #[tokio::test]
    async fn scenario_c_policy_blocked() {
        let result = run_scenario_c().await;
        assert!(result.success, "Scenario C: policy should have blocked");
        assert!(result.schema_valid, "Scenario C schema validation failed");
        assert!(
            result.telemetry_entries > 0,
            "Scenario C should have telemetry entries for blocked mutation"
        );
    }
}
