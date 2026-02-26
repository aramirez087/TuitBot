//! Shared mock providers for MCP tool tests.

use serde_json::Value;
use std::path::PathBuf;

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::error::XApiError;
use tuitbot_core::llm::{GenerationParams, LlmProvider, LlmResponse};
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;
use tuitbot_core::LlmError;

/// Canonical artifacts directory: `<repo_root>/roadmap/artifacts`.
pub fn artifacts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("roadmap/artifacts")
}

/// Returns `true` if `json` parses and contains a top-level `success` key.
pub fn validate_schema(json: &str) -> bool {
    let parsed: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return false,
    };
    parsed.get("success").is_some()
}

pub fn assert_conformant_success(json: &str, tool: &str) {
    let parsed: Value =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{tool}: invalid JSON: {e}"));
    assert!(
        parsed["success"].as_bool().unwrap_or(false),
        "{tool}: expected success=true"
    );
    assert!(parsed.get("data").is_some(), "{tool}: missing 'data' field");
    assert!(parsed.get("meta").is_some(), "{tool}: missing 'meta' field");
    assert_eq!(
        parsed["meta"]["tool_version"], "1.0",
        "{tool}: tool_version mismatch"
    );
    assert!(
        parsed["meta"]["elapsed_ms"].is_number(),
        "{tool}: elapsed_ms not a number"
    );
}

/// Assert that a JSON response is a conformant error envelope.
pub fn assert_conformant_error(json: &str, tool: &str, expected_code: &str) {
    let parsed: Value =
        serde_json::from_str(json).unwrap_or_else(|e| panic!("{tool}: invalid JSON: {e}"));
    assert!(
        !parsed["success"].as_bool().unwrap_or(true),
        "{tool}: expected success=false"
    );
    assert!(
        parsed.get("error").is_some(),
        "{tool}: missing 'error' field"
    );
    assert_eq!(
        parsed["error"]["code"].as_str().unwrap_or(""),
        expected_code,
        "{tool}: error code mismatch"
    );
    let retryable = parsed["error"]["retryable"].as_bool().unwrap_or(false);
    let code: crate::contract::ErrorCode = serde_json::from_value(parsed["error"]["code"].clone())
        .unwrap_or_else(|e| panic!("{tool}: unknown error code: {e}"));
    assert_eq!(
        retryable,
        code.is_retryable(),
        "{tool}: retryable flag mismatch for {expected_code}"
    );
}

pub struct MockProvider;

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

    async fn get_user_mentions(
        &self,
        _uid: &str,
        _since: Option<&str>,
        _pt: Option<&str>,
    ) -> Result<MentionResponse, ProviderError> {
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

    async fn get_user_tweets(
        &self,
        uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "ut1".to_string(),
                text: "User tweet".to_string(),
                author_id: uid.to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("ut1".to_string()),
                oldest_id: Some("ut1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }

    async fn get_home_timeline(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
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

    async fn get_following(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: user_id.to_string(),
            username: "iduser".to_string(),
            name: "ID User".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }

    async fn get_liked_tweets(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
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

    async fn get_bookmarks(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
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

    async fn get_users_by_ids(&self, user_ids: &[&str]) -> Result<UsersResponse, ProviderError> {
        let users = user_ids
            .iter()
            .map(|id| User {
                id: id.to_string(),
                username: format!("user_{id}"),
                name: format!("User {id}"),
                public_metrics: UserMetrics::default(),
            })
            .collect::<Vec<_>>();
        let count = users.len() as u32;
        Ok(UsersResponse {
            data: users,
            meta: UsersMeta {
                result_count: count,
                next_token: None,
            },
        })
    }

    async fn get_tweet_liking_users(
        &self,
        _tid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }
}

pub struct ErrorProvider;

#[async_trait::async_trait]
impl SocialReadProvider for ErrorProvider {
    async fn get_tweet(&self, _tid: &str) -> Result<Tweet, ProviderError> {
        Err(ProviderError::Other {
            message: "not found".to_string(),
        })
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

    async fn get_followers(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Err(ProviderError::Network {
            message: "timeout".to_string(),
        })
    }
}

pub struct MockXApiClient;

#[async_trait::async_trait]
impl XApiClient for MockXApiClient {
    async fn search_tweets(
        &self,
        _q: &str,
        _max: u32,
        _since: Option<&str>,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }

    async fn get_mentions(
        &self,
        _uid: &str,
        _since: Option<&str>,
        _pt: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        unimplemented!()
    }

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "posted_1".to_string(),
            text: text.to_string(),
        })
    }

    async fn reply_to_tweet(&self, text: &str, _reply_to: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "reply_1".to_string(),
            text: text.to_string(),
        })
    }

    async fn get_tweet(&self, _id: &str) -> Result<Tweet, XApiError> {
        unimplemented!()
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        unimplemented!()
    }

    async fn get_user_tweets(
        &self,
        _uid: &str,
        _max: u32,
        _pt: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }

    async fn get_user_by_username(&self, _u: &str) -> Result<User, XApiError> {
        unimplemented!()
    }

    async fn quote_tweet(&self, text: &str, _quoted: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "quote_1".to_string(),
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

    async fn retweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Ok(true)
    }

    async fn unretweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Ok(false)
    }

    async fn delete_tweet(&self, _tid: &str) -> Result<bool, XApiError> {
        Ok(true)
    }

    async fn unlike_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Ok(false)
    }

    async fn bookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
        Ok(true)
    }

    async fn unbookmark_tweet(&self, _uid: &str, _tid: &str) -> Result<bool, XApiError> {
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
