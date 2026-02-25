//! Tests for x_actions tools.
//!
//! Split into submodules by domain: read, write, engage.

mod engage;
mod read;
mod write;

use std::sync::Arc;

use tuitbot_core::config::{Config, McpPolicyConfig};
use tuitbot_core::error::XApiError;
use tuitbot_core::storage;
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;

use crate::state::{AppState, SharedState};

use super::*;

// ── Mock X API clients ──────────────────────────────────────────────

struct MockXApiClient;

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
            data: vec![Tweet {
                id: "t1".to_string(),
                text: "Hello".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("t1".to_string()),
                oldest_id: Some("t1".to_string()),
                result_count: 1,
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
            id: "new_1".to_string(),
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
            name: "Looked Up User".to_string(),
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

    async fn retweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Ok(true)
    }

    async fn unretweet(&self, _user_id: &str, _tweet_id: &str) -> Result<bool, XApiError> {
        Ok(false)
    }

    async fn delete_tweet(&self, _tweet_id: &str) -> Result<bool, XApiError> {
        Ok(true)
    }

    async fn get_home_timeline(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "ht1".to_string(),
                text: "Home tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("ht1".to_string()),
                oldest_id: Some("ht1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }
}

/// Mock that returns errors.
struct ErrorXApiClient;

#[async_trait::async_trait]
impl XApiClient for ErrorXApiClient {
    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::RateLimited {
            retry_after: Some(30),
        })
    }

    async fn get_mentions(
        &self,
        _user_id: &str,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn post_tweet(&self, _text: &str) -> Result<PostedTweet, XApiError> {
        Err(XApiError::Forbidden {
            message: "not allowed".to_string(),
        })
    }

    async fn reply_to_tweet(
        &self,
        _text: &str,
        _in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        Err(XApiError::AccountRestricted {
            message: "suspended".to_string(),
        })
    }

    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
        Err(XApiError::ApiError {
            status: 404,
            message: "not found".to_string(),
        })
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn get_user_tweets(
        &self,
        _user_id: &str,
        _max_results: u32,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, XApiError> {
        Err(XApiError::AuthExpired)
    }
}

// ── Helper functions ────────────────────────────────────────────────

async fn make_state(x_client: Option<Box<dyn XApiClient>>, user_id: Option<String>) -> SharedState {
    let mut config = Config::default();
    config.mcp_policy.enforce_for_mutations = false;
    let pool = storage::init_test_db().await.expect("init db");
    Arc::new(AppState {
        pool,
        config,
        llm_provider: None,
        x_client,
        authenticated_user_id: user_id,
    })
}

async fn make_state_with_config(
    x_client: Option<Box<dyn XApiClient>>,
    user_id: Option<String>,
    config: Config,
) -> SharedState {
    let pool = storage::init_test_db().await.expect("init db");
    tuitbot_core::storage::rate_limits::init_mcp_rate_limit(
        &pool,
        config.mcp_policy.max_mutations_per_hour,
    )
    .await
    .expect("init mcp rate limit");
    Arc::new(AppState {
        pool,
        config,
        llm_provider: None,
        x_client,
        authenticated_user_id: user_id,
    })
}

// ── Policy config helpers ───────────────────────────────────────────

fn blocked_config() -> Config {
    let mut config = Config::default();
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        blocked_tools: vec!["post_tweet".to_string()],
        require_approval_for: Vec::new(),
        dry_run_mutations: false,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config
}

fn approval_config() -> Config {
    let mut config = Config::default();
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        require_approval_for: vec!["post_tweet".to_string()],
        blocked_tools: Vec::new(),
        dry_run_mutations: false,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config
}

fn dry_run_config() -> Config {
    let mut config = Config::default();
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        require_approval_for: Vec::new(),
        blocked_tools: Vec::new(),
        dry_run_mutations: true,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config
}

fn allowed_config() -> Config {
    let mut config = Config::default();
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        require_approval_for: Vec::new(),
        blocked_tools: Vec::new(),
        dry_run_mutations: false,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config
}

fn composer_config() -> Config {
    let mut config = Config::default();
    config.mode = tuitbot_core::config::OperatingMode::Composer;
    config.mcp_policy = McpPolicyConfig {
        enforce_for_mutations: true,
        require_approval_for: Vec::new(),
        blocked_tools: Vec::new(),
        dry_run_mutations: false,
        max_mutations_per_hour: 20,
        ..McpPolicyConfig::default()
    };
    config
}
