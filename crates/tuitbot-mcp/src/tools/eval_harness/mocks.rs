//! Configurable mock providers for the eval harness.
//!
//! These are kept local because `MockXApiClient` here has `tweets`/`users`
//! fields for configurable search results, unlike the shared one.

use std::sync::Arc;

use tuitbot_core::config::{Config, McpPolicyConfig};
use tuitbot_core::error::XApiError;
use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
use tuitbot_core::storage;
use tuitbot_core::storage::tweets::DiscoveredTweet;
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;
use tuitbot_core::LlmError;

use crate::state::{AppState, SharedState};

pub struct MockXApiClient {
    pub tweets: Vec<Tweet>,
    pub users: Vec<User>,
}

impl MockXApiClient {
    pub fn with_results(tweets: Vec<Tweet>, users: Vec<User>) -> Self {
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

pub struct MockLlmProvider {
    pub reply_text: String,
}

impl MockLlmProvider {
    pub fn new(text: &str) -> Self {
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

pub fn test_config() -> Config {
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

pub fn blocked_policy_config() -> Config {
    let mut config = test_config();
    config.mcp_policy.enforce_for_mutations = true;
    config.mcp_policy.blocked_tools = vec!["propose_and_queue_replies".to_string()];
    config
}

pub fn approval_config() -> Config {
    let mut config = test_config();
    config.mcp_policy.enforce_for_mutations = true;
    config.mcp_policy.max_mutations_per_hour = 20;
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
        idempotency: Arc::new(crate::tools::idempotency::IdempotencyStore::new()),
    })
}

pub fn sample_tweet(id: &str, text: &str, author_id: &str) -> Tweet {
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

pub fn sample_user(id: &str, username: &str, followers: u64) -> User {
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

pub fn validate_schema(json: &str) -> bool {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return false,
    };
    parsed.get("success").is_some()
}
