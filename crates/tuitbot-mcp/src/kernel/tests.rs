//! Kernel tests using a mock [`SocialReadProvider`].
//!
//! Proves the kernel read tools work through the provider boundary
//! without any `XApiClient`, `AppState`, or database dependency.

use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;
use tuitbot_core::x_api::types::*;

use super::read;

// ── Mock provider (success) ─────────────────────────────────────────

struct MockProvider;

#[async_trait::async_trait]
impl SocialReadProvider for MockProvider {
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError> {
        Ok(Tweet {
            id: tweet_id.to_string(),
            text: "Mock tweet content".to_string(),
            author_id: "mock_author".to_string(),
            created_at: "2026-02-25T00:00:00Z".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: "mock_user_id".to_string(),
            username: username.to_string(),
            name: "Mock User".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![Tweet {
                id: "search_1".to_string(),
                text: "Found tweet".to_string(),
                author_id: "a1".to_string(),
                created_at: String::new(),
                public_metrics: PublicMetrics::default(),
                conversation_id: None,
            }],
            includes: None,
            meta: SearchMeta {
                newest_id: Some("search_1".to_string()),
                oldest_id: Some("search_1".to_string()),
                result_count: 1,
                next_token: None,
            },
        })
    }
}

// ── Mock provider (errors) ──────────────────────────────────────────

struct ErrorProvider;

#[async_trait::async_trait]
impl SocialReadProvider for ErrorProvider {
    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, ProviderError> {
        Err(ProviderError::Other {
            message: "not found".to_string(),
        })
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<User, ProviderError> {
        Err(ProviderError::AuthExpired)
    }

    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Err(ProviderError::RateLimited {
            retry_after: Some(60),
        })
    }
}

// ── Success path tests ──────────────────────────────────────────────

#[tokio::test]
async fn get_tweet_success() {
    let json = read::get_tweet(&MockProvider, "t42").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "t42");
    assert_eq!(parsed["data"]["text"], "Mock tweet content");
    assert!(parsed["meta"]["elapsed_ms"].is_number());
}

#[tokio::test]
async fn get_user_by_username_success() {
    let json = read::get_user_by_username(&MockProvider, "alice").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["username"], "alice");
    assert_eq!(parsed["data"]["id"], "mock_user_id");
}

#[tokio::test]
async fn search_tweets_success() {
    let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["meta"]["result_count"], 1);
    assert_eq!(parsed["data"]["data"][0]["text"], "Found tweet");
}

#[tokio::test]
async fn search_tweets_with_pagination() {
    let json = read::search_tweets(&MockProvider, "rust", 10, Some("s1"), Some("next")).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], true);
}

// ── Error path tests ────────────────────────────────────────────────

#[tokio::test]
async fn get_tweet_error() {
    let json = read::get_tweet(&ErrorProvider, "missing").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_api_error");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn get_user_by_username_auth_expired() {
    let json = read::get_user_by_username(&ErrorProvider, "nobody").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn search_tweets_rate_limited() {
    let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert_eq!(parsed["error"]["retryable"], true);
    assert!(parsed["error"]["message"].as_str().unwrap().contains("60s"));
}

// ── Envelope structure tests ────────────────────────────────────────

#[tokio::test]
async fn response_always_has_meta() {
    let json = read::get_tweet(&MockProvider, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["meta"].is_object());
    assert_eq!(parsed["meta"]["tool_version"], "1.0");
}

#[tokio::test]
async fn error_response_has_meta() {
    let json = read::get_tweet(&ErrorProvider, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["meta"].is_object());
    assert_eq!(parsed["meta"]["tool_version"], "1.0");
}
