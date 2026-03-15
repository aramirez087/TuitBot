//! Coverage tests for WriteMcpServer (write/handlers.rs, write/tools.rs, write/mod.rs).
//!
//! Tests exercise the workflow layer that the handlers delegate to, using a
//! minimal AppState backed by an in-memory test DB and a no-op X client.
//! Handler methods are private (proc-macro generated), so tests call the
//! same workflow functions the handlers call — this covers the dispatch path
//! and the business logic in one shot.

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
//
// Only the 8 required (non-default) trait methods need to be implemented.
// All optional methods already have default impls in XApiClient that return
// ApiError — those defaults are sufficient for these smoke tests.

pub(super) struct NullX;

#[async_trait::async_trait]
impl XApiClient for NullX {
    async fn search_tweets(
        &self,
        _query: &str,
        _max_results: u32,
        _since_id: Option<&str>,
        _pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        Err(XApiError::AuthExpired)
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
        Err(XApiError::AuthExpired)
    }

    async fn reply_to_tweet(
        &self,
        _text: &str,
        _in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        Err(XApiError::AuthExpired)
    }

    async fn get_tweet(&self, _tweet_id: &str) -> Result<Tweet, XApiError> {
        Err(XApiError::AuthExpired)
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
