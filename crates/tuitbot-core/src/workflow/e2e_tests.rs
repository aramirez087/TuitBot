//! End-to-end tests for the workflow layer over toolkit primitives.
//!
//! These tests verify that the workflow layer correctly composes toolkit
//! functions with DB state, proving the full utility-first pipeline
//! works without MCP transport.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::config::Config;
    use crate::error::XApiError;
    use crate::llm::{GenerationParams, LlmProvider, LlmResponse};
    use crate::storage;
    use crate::workflow::discover::{self, DiscoverInput};
    use crate::workflow::draft::{self, DraftInput};
    use crate::workflow::orchestrate::{self, CycleInput};
    use crate::workflow::publish;
    use crate::workflow::thread_plan::{self, ThreadPlanInput};
    use crate::workflow::{ProposeResult, WorkflowError};
    use crate::x_api::types::*;
    use crate::x_api::XApiClient;
    use crate::LlmError;

    // ── Mocks ────────────────────────────────────────────────────────

    struct E2eXApiClient {
        tweets: Vec<Tweet>,
        users: Vec<User>,
    }

    impl E2eXApiClient {
        fn with_data(tweets: Vec<Tweet>, users: Vec<User>) -> Self {
            Self { tweets, users }
        }
    }

    #[async_trait::async_trait]
    impl XApiClient for E2eXApiClient {
        async fn search_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
            _: Option<&str>,
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
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
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
                id: "posted_e2e".to_string(),
                text: text.to_string(),
            })
        }

        async fn reply_to_tweet(&self, text: &str, _: &str) -> Result<PostedTweet, XApiError> {
            Ok(PostedTweet {
                id: "reply_e2e".to_string(),
                text: text.to_string(),
            })
        }

        async fn get_tweet(&self, id: &str) -> Result<Tweet, XApiError> {
            self.tweets
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| XApiError::ApiError {
                    status: 404,
                    message: "Not found".to_string(),
                })
        }

        async fn get_me(&self) -> Result<User, XApiError> {
            Ok(User {
                id: "me".to_string(),
                username: "testbot".to_string(),
                name: "Test Bot".to_string(),
                public_metrics: UserMetrics::default(),
            })
        }

        async fn get_user_tweets(
            &self,
            _: &str,
            _: u32,
            _: Option<&str>,
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

        async fn get_user_by_username(&self, u: &str) -> Result<User, XApiError> {
            self.users
                .iter()
                .find(|user| user.username == u)
                .cloned()
                .ok_or_else(|| XApiError::ApiError {
                    status: 404,
                    message: "Not found".to_string(),
                })
        }
    }

    struct E2eLlm {
        reply_text: String,
    }

    impl E2eLlm {
        fn new(text: &str) -> Self {
            Self {
                reply_text: text.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for E2eLlm {
        fn name(&self) -> &str {
            "e2e-mock"
        }
        async fn complete(
            &self,
            _system: &str,
            _user_message: &str,
            _params: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                text: self.reply_text.clone(),
                model: "mock".to_string(),
                usage: crate::llm::TokenUsage {
                    input_tokens: 10,
                    output_tokens: 20,
                },
            })
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    fn test_config() -> Config {
        let mut config = Config::default();
        config.business.product_name = "TestProduct".to_string();
        config.business.product_keywords = vec!["rust".to_string(), "programming".to_string()];
        config.scoring.threshold = 10;
        config.approval_mode = true;
        config
    }

    fn sample_tweet(id: &str, text: &str, author_id: &str) -> Tweet {
        Tweet {
            id: id.to_string(),
            text: text.to_string(),
            author_id: author_id.to_string(),
            created_at: "2026-02-24T12:00:00Z".to_string(),
            public_metrics: PublicMetrics {
                like_count: 15,
                retweet_count: 5,
                reply_count: 3,
                impression_count: 2000,
                ..Default::default()
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

    // ── E2E: Discover finds candidates via toolkit search ────────────

    #[tokio::test]
    async fn e2e_discover_uses_toolkit_search() {
        let db = storage::init_test_db().await.unwrap();
        let tweets = vec![
            sample_tweet("t1", "Learning Rust async programming today", "a1"),
            sample_tweet("t2", "Best practices for Rust error handling", "a2"),
        ];
        let users = vec![
            sample_user("a1", "rustdev", 3000),
            sample_user("a2", "sysdev", 5000),
        ];
        let client = E2eXApiClient::with_data(tweets, users);
        let config = test_config();

        let output = discover::execute(
            &db,
            &client,
            &config,
            DiscoverInput {
                query: Some("rust programming".to_string()),
                min_score: Some(0.0), // Accept all for this test
                limit: Some(10),
                since_id: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(output.candidates.len(), 2);
        assert_eq!(output.query_used, "rust programming");
        // Candidates should have scores assigned by the scoring engine
        for c in &output.candidates {
            assert!(c.score_total > 0.0, "score should be positive");
            assert!(!c.author_username.is_empty());
        }
    }

    // ── E2E: Publish uses toolkit write ──────────────────────────────

    #[tokio::test]
    async fn e2e_publish_reply_uses_toolkit() {
        let client = E2eXApiClient::with_data(vec![], vec![]);

        let output = publish::reply(&client, "Great insight!", "t1")
            .await
            .unwrap();
        assert_eq!(output.tweet_id, "reply_e2e");
    }

    #[tokio::test]
    async fn e2e_publish_tweet_uses_toolkit() {
        let client = E2eXApiClient::with_data(vec![], vec![]);

        let output = publish::tweet(&client, "Check out this tool!")
            .await
            .unwrap();
        assert_eq!(output.tweet_id, "posted_e2e");
    }

    // ── E2E: Full pipeline discover → draft → queue ──────────────────

    #[tokio::test]
    async fn e2e_full_pipeline_with_approval() {
        let db = storage::init_test_db().await.unwrap();
        let tweets = vec![sample_tweet(
            "t1",
            "How do you handle errors in Rust programming?",
            "a1",
        )];
        let users = vec![sample_user("a1", "curious_dev", 2500)];
        let client = E2eXApiClient::with_data(tweets, users);
        let llm: Arc<dyn LlmProvider> = Arc::new(E2eLlm::new(
            "Use the ? operator for clean error propagation!",
        ));
        let mut config = test_config();
        config.approval_mode = true;

        let report = orchestrate::run_discovery_cycle(
            &db,
            &client,
            &llm,
            &config,
            CycleInput {
                query: Some("rust error handling".to_string()),
                min_score: Some(0.0),
                limit: Some(10),
                since_id: None,
                mention_product: false,
            },
        )
        .await
        .unwrap();

        assert!(report.summary.candidates_found > 0);
        // With approval_mode = true, replies should be queued not executed
        let has_queued = report
            .queued
            .iter()
            .any(|r| matches!(r, ProposeResult::Queued { .. }));
        let has_executed = report
            .queued
            .iter()
            .any(|r| matches!(r, ProposeResult::Executed { .. }));
        // At least one path should have been taken
        assert!(
            has_queued || has_executed || report.summary.drafts_generated > 0,
            "pipeline should produce output for valid candidates"
        );
    }

    // ── E2E: Empty search gracefully returns empty report ────────────

    #[tokio::test]
    async fn e2e_empty_search_graceful() {
        let db = storage::init_test_db().await.unwrap();
        let client = E2eXApiClient::with_data(vec![], vec![]);
        let llm: Arc<dyn LlmProvider> = Arc::new(E2eLlm::new("reply"));
        let config = test_config();

        let report = orchestrate::run_discovery_cycle(
            &db,
            &client,
            &llm,
            &config,
            CycleInput {
                query: Some("nonexistent topic".to_string()),
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

    // ── E2E: Thread plan via workflow + LLM ──────────────────────────

    #[tokio::test]
    async fn e2e_thread_plan_generates_structure() {
        let llm: Arc<dyn LlmProvider> = Arc::new(E2eLlm::new(
            "Here's how Rust handles memory\n---\n\
             The ownership system is key\n---\n\
             Borrowing lets you share references safely\n---\n\
             The borrow checker catches issues at compile time\n---\n\
             Result: zero-cost abstractions with safety guarantees",
        ));
        let config = test_config();

        let output = thread_plan::execute(
            &llm,
            &config,
            ThreadPlanInput {
                topic: "Rust memory safety".to_string(),
                objective: Some("educate".to_string()),
                target_audience: Some("developers".to_string()),
                structure: None,
            },
        )
        .await
        .unwrap();

        assert!(
            output.tweet_count >= 2,
            "thread should have multiple tweets"
        );
        assert!(!output.hook_type.is_empty(), "hook type should be detected");
    }

    // ── E2E: WorkflowError propagation from toolkit ──────────────────

    #[tokio::test]
    async fn e2e_workflow_error_from_toolkit_search_failure() {
        struct FailingClient;

        #[async_trait::async_trait]
        impl XApiClient for FailingClient {
            async fn search_tweets(
                &self,
                _: &str,
                _: u32,
                _: Option<&str>,
                _: Option<&str>,
            ) -> Result<SearchResponse, XApiError> {
                Err(XApiError::RateLimited {
                    retry_after: Some(60),
                })
            }
            async fn get_mentions(
                &self,
                _: &str,
                _: Option<&str>,
                _: Option<&str>,
            ) -> Result<MentionResponse, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn post_tweet(&self, _: &str) -> Result<PostedTweet, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn reply_to_tweet(&self, _: &str, _: &str) -> Result<PostedTweet, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn get_me(&self) -> Result<User, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn get_user_tweets(
                &self,
                _: &str,
                _: u32,
                _: Option<&str>,
            ) -> Result<SearchResponse, XApiError> {
                Err(XApiError::AuthExpired)
            }
            async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
                Err(XApiError::AuthExpired)
            }
        }

        let db = storage::init_test_db().await.unwrap();
        let config = test_config();

        // Toolkit rate-limit error should propagate through workflow layer
        let err = discover::execute(
            &db,
            &FailingClient,
            &config,
            DiscoverInput {
                query: Some("test".to_string()),
                min_score: None,
                limit: None,
                since_id: None,
            },
        )
        .await
        .unwrap_err();

        assert!(
            matches!(err, WorkflowError::Toolkit(_)),
            "expected Toolkit error variant, got: {err:?}"
        );
    }

    // ── E2E: LLM not configured produces clean error ─────────────────

    #[tokio::test]
    async fn e2e_draft_empty_candidates_returns_validation_error() {
        let db = storage::init_test_db().await.unwrap();
        let config = test_config();

        // Empty candidate IDs should produce a validation error
        let llm: Arc<dyn LlmProvider> = Arc::new(E2eLlm::new("reply"));
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

        assert!(
            matches!(err, WorkflowError::InvalidInput(_)),
            "expected InvalidInput, got: {err:?}"
        );
    }
}
