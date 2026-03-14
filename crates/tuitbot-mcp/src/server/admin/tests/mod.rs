//! Coverage tests for AdminMcpServer handlers (admin/handlers.rs, admin/tools.rs, admin/mod.rs).
//!
//! Each test instantiates `AdminMcpServer` with a minimal mock state and calls
//! one handler method. The handlers are thin wrappers over workflow functions;
//! these tests verify the dispatch layer is exercised, not the business logic.

mod handlers;
mod tools;

use std::sync::Arc;

use tuitbot_core::config::Config;
use tuitbot_core::error::XApiError;
use tuitbot_core::storage;
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;

use crate::state::AppState;

// ── Minimal no-op X client ────────────────────────────────────────────

pub(super) struct NullX;

#[async_trait::async_trait]
impl XApiClient for NullX {
    async fn search_tweets(
        &self,
        _: &str,
        _: u32,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_tweet(&self, _: &str) -> Result<Tweet, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_me(&self) -> Result<User, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_user_by_username(&self, _: &str) -> Result<User, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_user_by_id(&self, _: &str) -> Result<User, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_users_by_ids(&self, _: &[&str]) -> Result<Vec<User>, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_mentions(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_user_tweets(
        &self,
        _: &str,
        _: u32,
        _: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_home_timeline(
        &self,
        _: u32,
        _: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_followers(
        &self,
        _: &str,
        _: Option<u32>,
        _: Option<&str>,
    ) -> Result<Vec<User>, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_following(
        &self,
        _: &str,
        _: Option<u32>,
        _: Option<&str>,
    ) -> Result<Vec<User>, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_liked_tweets(
        &self,
        _: &str,
        _: Option<u32>,
        _: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_bookmarks(
        &self,
        _: Option<u32>,
        _: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_tweet_liking_users(
        &self,
        _: &str,
        _: Option<u32>,
        _: Option<&str>,
    ) -> Result<Vec<User>, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn post_tweet(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<&[&str]>,
    ) -> Result<Tweet, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn reply_to_tweet(
        &self,
        _: &str,
        _: &str,
        _: Option<&[&str]>,
    ) -> Result<Tweet, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn quote_tweet(&self, _: &str, _: &str, _: Option<&[&str]>) -> Result<Tweet, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn delete_tweet(&self, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn like_tweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn unlike_tweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn retweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn unretweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn follow_user(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn unfollow_user(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn bookmark_tweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn unbookmark_tweet(&self, _: &str, _: &str) -> Result<(), XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn upload_media(
        &self,
        _: &std::path::Path,
        _: Option<&str>,
    ) -> Result<String, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn get_v2(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<&[(&str, &str)]>,
    ) -> Result<serde_json::Value, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn post_v2(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn put_v2(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, XApiError> {
        Err(XApiError::NotConfigured)
    }
    async fn delete_v2(
        &self,
        _: &str,
        _: Option<&str>,
        _: Option<&[(&str, &str)]>,
    ) -> Result<serde_json::Value, XApiError> {
        Err(XApiError::NotConfigured)
    }
}

// ── Test state factory ────────────────────────────────────────────────

pub(super) async fn make_state() -> crate::state::SharedState {
    let pool = storage::init_test_db().await.expect("init test db");
    let config = Config::default();
    storage::rate_limits::init_mcp_rate_limit(&pool, config.mcp_policy.max_mutations_per_hour)
        .await
        .expect("init rate limits");
    Arc::new(AppState {
        pool,
        config,
        llm_provider: None,
        x_client: Some(Box::new(NullX)),
        authenticated_user_id: Some("u1".to_string()),
        granted_scopes: vec![],
        idempotency: Arc::new(crate::tools::idempotency::IdempotencyStore::new()),
    })
}

// ── mod.rs coverage: kv_to_tuples ────────────────────────────────────

#[test]
fn kv_to_tuples_none() {
    assert!(super::kv_to_tuples(None).is_none());
}

#[test]
fn kv_to_tuples_some() {
    let kv = vec![KeyValue {
        key: "a".to_string(),
        value: "b".to_string(),
    }];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(result, vec![("a".to_string(), "b".to_string())]);
}
