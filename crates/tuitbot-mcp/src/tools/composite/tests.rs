//! Tests for composite MCP tools.

use std::sync::Arc;

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

// ── Test Helpers ──────────────────────────────────────────────────────

struct MockXApiClient {
    tweets: Vec<Tweet>,
    users: Vec<User>,
}

impl MockXApiClient {
    fn with_results(tweets: Vec<Tweet>, users: Vec<User>) -> Self {
        Self { tweets, users }
    }

    fn empty() -> Self {
        Self {
            tweets: vec![],
            users: vec![],
        }
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

    async fn follow_user(&self, _user_id: &str, _target_user_id: &str) -> Result<bool, XApiError> {
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

struct FailingLlmProvider;

#[async_trait::async_trait]
impl LlmProvider for FailingLlmProvider {
    fn name(&self) -> &str {
        "failing"
    }

    async fn complete(
        &self,
        _system: &str,
        _user_message: &str,
        _params: &GenerationParams,
    ) -> Result<LlmResponse, LlmError> {
        Err(LlmError::GenerationFailed("mock LLM failure".to_string()))
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        Err(LlmError::GenerationFailed("mock failure".to_string()))
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
    config.scoring.threshold = 0; // low threshold for test tweets to pass
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

// ── find_reply_opportunities ──────────────────────────────────────────

mod find_opportunities {
    use super::*;
    use crate::tools::composite::find_opportunities;

    #[tokio::test]
    async fn happy_path_search_score_rank() {
        let tweets = vec![
            sample_tweet("t1", "Learning rust async programming today", "a1"),
            sample_tweet("t2", "Just had coffee", "a2"),
        ];
        let users = vec![
            sample_user("a1", "rustdev", 5000),
            sample_user("a2", "coffeelover", 200),
        ];
        let client = MockXApiClient::with_results(tweets, users);
        let state = make_test_state(Some(Box::new(client)), None, test_config()).await;

        let result = find_opportunities::execute(&state, Some("rust"), None, Some(10), None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        assert!(parsed["data"]["candidates"].is_array());
        assert!(parsed["data"]["total_found"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn x_not_configured() {
        let state = make_test_state(None, None, test_config()).await;
        let result = find_opportunities::execute(&state, Some("rust"), None, None, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "x_not_configured");
    }

    #[tokio::test]
    async fn empty_results() {
        let client = MockXApiClient::empty();
        let state = make_test_state(Some(Box::new(client)), None, test_config()).await;

        let result = find_opportunities::execute(&state, Some("rust"), None, None, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["total_searched"], 0);
        assert_eq!(parsed["data"]["candidates"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn default_query_from_keywords() {
        let tweets = vec![sample_tweet(
            "t1",
            "rust async is amazing for async tasks",
            "a1",
        )];
        let users = vec![sample_user("a1", "dev", 1000)];
        let client = MockXApiClient::with_results(tweets, users);
        let state = make_test_state(Some(Box::new(client)), None, test_config()).await;

        // No query — should use product_keywords
        let result = find_opportunities::execute(&state, None, None, None, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["query"], "rust OR async");
    }
}

// ── draft_replies_for_candidates ──────────────────────────────────────

mod draft_replies {
    use super::*;
    use crate::tools::composite::draft_replies;

    #[tokio::test]
    async fn happy_path_generate_drafts() {
        let llm = MockLlmProvider::new("Great point about Rust!");
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;
        seed_discovered_tweet(
            &state,
            "t1",
            "Rust is great for systems programming",
            "rustdev",
        )
        .await;

        let ids = vec!["t1".to_string()];
        let result = draft_replies::execute(&state, &ids, None, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0]["status"], "success");
        assert_eq!(data[0]["draft_text"], "Great point about Rust!");
        assert!(data[0]["char_count"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn candidate_not_found() {
        let llm = MockLlmProvider::new("Reply");
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;

        let ids = vec!["nonexistent".to_string()];
        let result = draft_replies::execute(&state, &ids, None, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data[0]["status"], "error");
        assert_eq!(data[0]["error_code"], "not_found");
    }

    #[tokio::test]
    async fn llm_error_for_one_continues() {
        let llm = FailingLlmProvider;
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;
        seed_discovered_tweet(&state, "t1", "Rust topic", "dev1").await;
        seed_discovered_tweet(&state, "t2", "Async topic", "dev2").await;

        let ids = vec!["t1".to_string(), "t2".to_string()];
        let result = draft_replies::execute(&state, &ids, None, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        // Both should be errors since the LLM always fails
        assert_eq!(data.len(), 2);
        assert_eq!(data[0]["status"], "error");
        assert_eq!(data[0]["error_code"], "llm_error");
        assert_eq!(data[1]["status"], "error");
    }

    #[tokio::test]
    async fn archetype_override() {
        let llm = MockLlmProvider::new("I have a question about this");
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;
        seed_discovered_tweet(&state, "t1", "Rust topic", "dev").await;

        let ids = vec!["t1".to_string()];
        let result = draft_replies::execute(&state, &ids, Some("ask_question"), false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data[0]["archetype"], "AskQuestion");
    }

    #[tokio::test]
    async fn empty_input_errors() {
        let state = make_test_state(None, None, test_config()).await;

        let ids: Vec<String> = vec![];
        let result = draft_replies::execute(&state, &ids, None, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "invalid_input");
    }
}

// ── propose_and_queue_replies ─────────────────────────────────────────

mod propose_queue {
    use super::*;
    use crate::tools::composite::propose_queue;

    #[tokio::test]
    async fn queues_in_approval_mode() {
        let llm = MockLlmProvider::new("Great insight!");
        let client = MockXApiClient::empty();
        let state = make_test_state(
            Some(Box::new(client)),
            Some(Box::new(llm)),
            approval_config(),
        )
        .await;
        seed_discovered_tweet(&state, "t1", "Rust topic", "dev").await;

        let items = vec![ProposeItem {
            candidate_id: "t1".to_string(),
            pre_drafted_text: Some("This is my reply!".to_string()),
        }];
        let result = propose_queue::execute(&state, &items, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data[0]["status"], "queued");
        assert!(data[0]["approval_queue_id"].is_number());
    }

    #[tokio::test]
    async fn executes_in_autopilot_mode() {
        let llm = MockLlmProvider::new("Great!");
        let client = MockXApiClient::empty(); // reply_to_tweet returns "reply_1"
        let mut config = test_config();
        config.approval_mode = false;
        let state = make_test_state(Some(Box::new(client)), Some(Box::new(llm)), config).await;
        seed_discovered_tweet(&state, "t1", "Rust topic", "dev").await;

        let items = vec![ProposeItem {
            candidate_id: "t1".to_string(),
            pre_drafted_text: Some("Direct reply!".to_string()),
        }];
        let result = propose_queue::execute(&state, &items, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data[0]["status"], "executed");
        assert_eq!(data[0]["reply_tweet_id"], "reply_1");
    }

    #[tokio::test]
    async fn tweet_not_found_blocked() {
        let state = make_test_state(None, None, test_config()).await;

        let items = vec![ProposeItem {
            candidate_id: "nonexistent".to_string(),
            pre_drafted_text: Some("reply".to_string()),
        }];
        let result = propose_queue::execute(&state, &items, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        let data = parsed["data"].as_array().unwrap();
        assert_eq!(data[0]["status"], "blocked");
    }

    #[tokio::test]
    async fn empty_items_errors() {
        let state = make_test_state(None, None, test_config()).await;

        let items: Vec<ProposeItem> = vec![];
        let result = propose_queue::execute(&state, &items, false).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "invalid_input");
    }
}

// ── generate_thread_plan ──────────────────────────────────────────────

mod thread_plan {
    use super::*;
    use crate::tools::composite::thread_plan;

    /// Build a valid 5-tweet thread mock output.
    fn valid_thread_text() -> &'static str {
        "Most people think async is hard\n---\n\
         But the reality is simpler than you think\n---\n\
         Step one: understand the event loop\n---\n\
         Step two: learn about futures and polling\n---\n\
         Step three: build something real and iterate"
    }

    #[tokio::test]
    async fn happy_path_generates_thread() {
        let llm = MockLlmProvider::new(valid_thread_text());
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;

        let result = thread_plan::execute(
            &state,
            "software engineering",
            Some("establish expertise"),
            Some("developers"),
            None,
        )
        .await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        assert!(parsed["data"]["thread_tweets"].is_array());
        assert_eq!(parsed["data"]["tweet_count"], 5);
        assert!(parsed["data"]["hook_analysis"]["type"].is_string());
        assert_eq!(parsed["data"]["estimated_performance"], "high");
    }

    #[tokio::test]
    async fn llm_not_configured() {
        let state = make_test_state(None, None, test_config()).await;

        let result = thread_plan::execute(&state, "topic", None, None, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"]["code"], "llm_not_configured");
    }

    #[tokio::test]
    async fn structure_override() {
        let llm = MockLlmProvider::new(valid_thread_text());
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;

        let result =
            thread_plan::execute(&state, "topic", None, None, Some("transformation")).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["structure_used"], "transformation");
    }

    #[tokio::test]
    async fn novel_topic_medium_performance() {
        let llm = MockLlmProvider::new(valid_thread_text());
        let state = make_test_state(None, Some(Box::new(llm)), test_config()).await;

        // "cooking" doesn't match any industry_topics
        let result = thread_plan::execute(&state, "cooking recipes", None, None, None).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["estimated_performance"], "medium");
        assert_eq!(parsed["data"]["topic_relevance"], "novel_topic");
    }
}
