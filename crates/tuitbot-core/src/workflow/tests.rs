//! Integration tests for workflow step composition.

use std::sync::Arc;

use crate::config::{Config, McpPolicyConfig};
use crate::error::XApiError;
use crate::llm::{GenerationParams, LlmProvider, LlmResponse};
use crate::storage;
use crate::storage::tweets::DiscoveredTweet;
use crate::x_api::types::*;
use crate::x_api::XApiClient;
use crate::LlmError;

use super::*;

// ── Mock Helpers ─────────────────────────────────────────────────────

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
            usage: crate::llm::TokenUsage {
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
    config.scoring.threshold = 0; // low threshold for test tweets to pass
    config
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

async fn seed_discovered_tweet(db: &storage::DbPool, id: &str, text: &str, author: &str) {
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
    storage::tweets::insert_discovered_tweet(db, &tweet)
        .await
        .expect("seed tweet");
}

// ── Discover step tests ──────────────────────────────────────────────

mod discover_tests {
    use super::*;

    #[tokio::test]
    async fn happy_path_search_score_rank() {
        let db = storage::init_test_db().await.unwrap();
        let tweets = vec![
            sample_tweet("t1", "Learning rust async programming today", "a1"),
            sample_tweet("t2", "Just had coffee", "a2"),
        ];
        let users = vec![
            sample_user("a1", "rustdev", 5000),
            sample_user("a2", "coffeelover", 200),
        ];
        let client = MockXApiClient::with_results(tweets, users);
        let config = test_config();

        let output = discover::execute(
            &db,
            &client,
            &config,
            DiscoverInput {
                query: Some("rust".to_string()),
                min_score: None,
                limit: Some(10),
                since_id: None,
            },
        )
        .await
        .unwrap();

        assert!(!output.candidates.is_empty());
        assert_eq!(output.query_used, "rust");
    }

    #[tokio::test]
    async fn empty_results() {
        let db = storage::init_test_db().await.unwrap();
        let client = MockXApiClient::empty();
        let config = test_config();

        let output = discover::execute(
            &db,
            &client,
            &config,
            DiscoverInput {
                query: Some("rust".to_string()),
                min_score: None,
                limit: None,
                since_id: None,
            },
        )
        .await
        .unwrap();

        assert!(output.candidates.is_empty());
    }

    #[tokio::test]
    async fn default_query_from_keywords() {
        let db = storage::init_test_db().await.unwrap();
        let tweets = vec![sample_tweet(
            "t1",
            "rust async is amazing for async tasks",
            "a1",
        )];
        let users = vec![sample_user("a1", "dev", 1000)];
        let client = MockXApiClient::with_results(tweets, users);
        let config = test_config();

        let output = discover::execute(
            &db,
            &client,
            &config,
            DiscoverInput {
                query: None, // should use product_keywords
                min_score: None,
                limit: None,
                since_id: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(output.query_used, "rust OR async");
    }

    #[tokio::test]
    async fn no_query_no_keywords_errors() {
        let db = storage::init_test_db().await.unwrap();
        let client = MockXApiClient::empty();
        let mut config = test_config();
        config.business.product_keywords = vec![];

        let err = discover::execute(
            &db,
            &client,
            &config,
            DiscoverInput {
                query: None,
                min_score: None,
                limit: None,
                since_id: None,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, WorkflowError::InvalidInput(_)));
    }
}

// ── Draft step tests ─────────────────────────────────────────────────

mod draft_tests {
    use super::*;

    #[tokio::test]
    async fn happy_path_generate_drafts() {
        let db = storage::init_test_db().await.unwrap();
        seed_discovered_tweet(
            &db,
            "t1",
            "Rust is great for systems programming",
            "rustdev",
        )
        .await;

        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Great point about Rust!"));
        let config = test_config();

        let results = draft::execute(
            &db,
            &llm,
            &config,
            DraftInput {
                candidate_ids: vec!["t1".to_string()],
                archetype: None,
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        match &results[0] {
            DraftResult::Success { draft_text, .. } => {
                assert_eq!(draft_text, "Great point about Rust!");
            }
            DraftResult::Error { error_message, .. } => {
                panic!("Expected success, got error: {error_message}");
            }
        }
    }

    #[tokio::test]
    async fn candidate_not_found() {
        let db = storage::init_test_db().await.unwrap();
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Reply"));
        let config = test_config();

        let results = draft::execute(
            &db,
            &llm,
            &config,
            DraftInput {
                candidate_ids: vec!["nonexistent".to_string()],
                archetype: None,
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], DraftResult::Error { error_code, .. } if error_code == "not_found")
        );
    }

    #[tokio::test]
    async fn empty_input_errors() {
        let db = storage::init_test_db().await.unwrap();
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Reply"));
        let config = test_config();

        let err = draft::execute(
            &db,
            &llm,
            &config,
            DraftInput {
                candidate_ids: vec![],
                archetype: None,
                mention_product: false,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, WorkflowError::InvalidInput(_)));
    }
}

// ── Queue step tests ─────────────────────────────────────────────────

mod queue_tests {
    use super::*;

    #[tokio::test]
    async fn queues_in_approval_mode() {
        let db = storage::init_test_db().await.unwrap();
        seed_discovered_tweet(&db, "t1", "Rust topic", "dev").await;

        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Great insight!"));
        let client = MockXApiClient::empty();
        let mut config = test_config();
        config.approval_mode = true;

        let results = queue::execute(
            &db,
            Some(&client as &dyn XApiClient),
            Some(&llm),
            &config,
            QueueInput {
                items: vec![QueueItem {
                    candidate_id: "t1".to_string(),
                    pre_drafted_text: Some("This is my reply!".to_string()),
                }],
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], ProposeResult::Queued { .. }));
    }

    #[tokio::test]
    async fn executes_in_autopilot_mode() {
        let db = storage::init_test_db().await.unwrap();
        seed_discovered_tweet(&db, "t1", "Rust topic", "dev").await;

        let client = MockXApiClient::empty(); // reply_to_tweet returns "reply_1"
        let mut config = test_config();
        config.approval_mode = false;

        let results = queue::execute(
            &db,
            Some(&client as &dyn XApiClient),
            None,
            &config,
            QueueInput {
                items: vec![QueueItem {
                    candidate_id: "t1".to_string(),
                    pre_drafted_text: Some("Direct reply!".to_string()),
                }],
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        match &results[0] {
            ProposeResult::Executed { reply_tweet_id, .. } => {
                assert_eq!(reply_tweet_id, "reply_1");
            }
            other => panic!("Expected Executed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn tweet_not_found_blocked() {
        let db = storage::init_test_db().await.unwrap();
        let config = test_config();

        let results = queue::execute(
            &db,
            None,
            None,
            &config,
            QueueInput {
                items: vec![QueueItem {
                    candidate_id: "nonexistent".to_string(),
                    pre_drafted_text: Some("reply".to_string()),
                }],
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], ProposeResult::Blocked { .. }));
    }

    #[tokio::test]
    async fn empty_items_errors() {
        let db = storage::init_test_db().await.unwrap();
        let config = test_config();

        let err = queue::execute(
            &db,
            None,
            None,
            &config,
            QueueInput {
                items: vec![],
                mention_product: false,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(err, WorkflowError::InvalidInput(_)));
    }
}

// ── Thread plan step tests ───────────────────────────────────────────

mod thread_plan_tests {
    use super::*;

    fn valid_thread_text() -> &'static str {
        "Most people think async is hard\n---\n\
         But the reality is simpler than you think\n---\n\
         Step one: understand the event loop\n---\n\
         Step two: learn about futures and polling\n---\n\
         Step three: build something real and iterate"
    }

    #[tokio::test]
    async fn happy_path_generates_thread() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(valid_thread_text()));
        let config = test_config();

        let output = thread_plan::execute(
            &llm,
            &config,
            ThreadPlanInput {
                topic: "software engineering".to_string(),
                objective: Some("establish expertise".to_string()),
                target_audience: Some("developers".to_string()),
                structure: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(output.tweet_count, 5);
        assert_eq!(output.estimated_performance, "high");
        assert_eq!(output.hook_type, "contrarian"); // "Most people..."
    }

    #[tokio::test]
    async fn novel_topic_medium_performance() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(valid_thread_text()));
        let config = test_config();

        let output = thread_plan::execute(
            &llm,
            &config,
            ThreadPlanInput {
                topic: "cooking recipes".to_string(),
                objective: None,
                target_audience: None,
                structure: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(output.estimated_performance, "medium");
        assert_eq!(output.topic_relevance, "novel_topic");
    }
}

// ── Orchestrator tests ───────────────────────────────────────────────

mod orchestrate_tests {
    use super::*;

    #[tokio::test]
    async fn full_cycle_discover_draft_queue() {
        let db = storage::init_test_db().await.unwrap();
        let tweets = vec![sample_tweet(
            "t1",
            "Learning rust async programming today",
            "a1",
        )];
        let users = vec![sample_user("a1", "rustdev", 5000)];
        let client = MockXApiClient::with_results(tweets, users);
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Great point about Rust!"));
        let mut config = test_config();
        config.approval_mode = true;

        let report = orchestrate::run_discovery_cycle(
            &db,
            &client,
            &llm,
            &config,
            CycleInput {
                query: Some("rust".to_string()),
                min_score: None,
                limit: Some(10),
                since_id: None,
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert!(report.summary.candidates_found > 0);
        // Drafts should have been generated for actionable candidates
        assert!(report.summary.drafts_generated > 0 || report.summary.drafts_failed > 0);
    }

    #[tokio::test]
    async fn empty_search_returns_empty_report() {
        let db = storage::init_test_db().await.unwrap();
        let client = MockXApiClient::empty();
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new("Reply"));
        let config = test_config();

        let report = orchestrate::run_discovery_cycle(
            &db,
            &client,
            &llm,
            &config,
            CycleInput {
                query: Some("rust".to_string()),
                min_score: None,
                limit: None,
                since_id: None,
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert_eq!(report.summary.candidates_found, 0);
        assert!(report.drafts.is_empty());
        assert!(report.queued.is_empty());
    }
}

// ── Publish step tests ───────────────────────────────────────────────

mod publish_tests {
    use super::*;

    #[tokio::test]
    async fn publish_reply_through_toolkit() {
        let client = MockXApiClient::empty();

        let output = publish::reply(&client, "Great point!", "t1").await.unwrap();

        assert_eq!(output.tweet_id, "reply_1");
        assert_eq!(output.text, "Great point!");
    }

    #[tokio::test]
    async fn publish_tweet_through_toolkit() {
        let client = MockXApiClient::empty();

        let output = publish::tweet(&client, "Hello world!").await.unwrap();

        assert_eq!(output.tweet_id, "posted_1");
        assert_eq!(output.text, "Hello world!");
    }
}

// ── Error propagation tests ──────────────────────────────────────────

mod error_tests {
    use super::*;

    #[test]
    fn workflow_error_from_toolkit() {
        let toolkit_err = crate::toolkit::ToolkitError::InvalidInput {
            message: "bad input".to_string(),
        };
        let workflow_err: WorkflowError = toolkit_err.into();
        assert!(matches!(workflow_err, WorkflowError::Toolkit(_)));
    }

    #[test]
    fn workflow_error_from_llm() {
        let llm_err = LlmError::NotConfigured;
        let workflow_err: WorkflowError = llm_err.into();
        assert!(matches!(workflow_err, WorkflowError::Llm(_)));
    }

    #[test]
    fn workflow_error_display() {
        let err = WorkflowError::InvalidInput("test error".to_string());
        assert_eq!(err.to_string(), "invalid input: test error");
    }
}
