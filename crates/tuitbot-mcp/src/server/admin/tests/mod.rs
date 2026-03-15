//! Coverage tests for AdminMcpServer handlers (admin/handlers.rs, admin/tools.rs, admin/mod.rs).
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

// ── mod.rs coverage: kv_to_tuples ────────────────────────────────────

#[test]
fn kv_to_tuples_none() {
    assert!(super::kv_to_tuples(None).is_none());
}

#[test]
fn kv_to_tuples_some() {
    use crate::requests::KeyValue;
    let kv = vec![KeyValue {
        key: "a".to_string(),
        value: "b".to_string(),
    }];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(result, vec![("a".to_string(), "b".to_string())]);
}

#[test]
fn kv_to_tuples_empty_vec() {
    use crate::requests::KeyValue;
    let kv: Vec<KeyValue> = vec![];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert!(result.is_empty());
}

#[test]
fn kv_to_tuples_multiple_pairs() {
    use crate::requests::KeyValue;
    let kv = vec![
        KeyValue {
            key: "x".to_string(),
            value: "1".to_string(),
        },
        KeyValue {
            key: "y".to_string(),
            value: "2".to_string(),
        },
        KeyValue {
            key: "z".to_string(),
            value: "3".to_string(),
        },
    ];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ("x".to_string(), "1".to_string()));
    assert_eq!(result[2], ("z".to_string(), "3".to_string()));
}

#[test]
fn kv_to_tuples_special_characters() {
    use crate::requests::KeyValue;
    let kv = vec![KeyValue {
        key: "Content-Type".to_string(),
        value: "application/json".to_string(),
    }];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(
        result[0],
        ("Content-Type".to_string(), "application/json".to_string())
    );
}

// ── AdminMcpServer construction ─────────────────────────────────────

#[tokio::test]
async fn admin_server_info_has_instructions() {
    use rmcp::ServerHandler;
    let state = make_state().await;
    let server = super::AdminMcpServer::new(state);
    let info = server.get_info();
    assert!(info.instructions.is_some());
    let instructions = info.instructions.unwrap();
    assert!(
        instructions.contains("Admin"),
        "admin server instructions should mention Admin"
    );
}

#[tokio::test]
async fn admin_server_info_has_tool_capabilities() {
    use rmcp::ServerHandler;
    let state = make_state().await;
    let server = super::AdminMcpServer::new(state);
    let info = server.get_info();
    assert!(info.capabilities.tools.is_some());
}

#[tokio::test]
async fn make_state_creates_valid_state() {
    let state = make_state().await;
    assert!(state.x_client.is_some());
    assert!(state.llm_provider.is_none());
    assert_eq!(state.authenticated_user_id.as_deref(), Some("u1"));
    assert!(state.granted_scopes.is_empty());
}

#[tokio::test]
async fn admin_server_clones() {
    let state = make_state().await;
    let server = super::AdminMcpServer::new(state);
    let _clone = server.clone();
}

#[tokio::test]
async fn admin_server_instructions_mention_universal_tools() {
    use rmcp::ServerHandler;
    let state = make_state().await;
    let server = super::AdminMcpServer::new(state);
    let info = server.get_info();
    let instructions = info.instructions.unwrap();
    assert!(
        instructions.contains("x_get") || instructions.contains("universal"),
        "admin instructions should mention universal request tools"
    );
}

#[test]
fn kv_to_tuples_preserves_order() {
    use crate::requests::KeyValue;
    let kv = vec![
        KeyValue {
            key: "z".to_string(),
            value: "3".to_string(),
        },
        KeyValue {
            key: "a".to_string(),
            value: "1".to_string(),
        },
        KeyValue {
            key: "m".to_string(),
            value: "2".to_string(),
        },
    ];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(result[0].0, "z");
    assert_eq!(result[1].0, "a");
    assert_eq!(result[2].0, "m");
}

#[test]
fn kv_to_tuples_unicode_values() {
    use crate::requests::KeyValue;
    let kv = vec![KeyValue {
        key: "emoji".to_string(),
        value: "\u{1F600}".to_string(),
    }];
    let result = super::kv_to_tuples(Some(&kv)).unwrap();
    assert_eq!(result[0].1, "\u{1F600}");
}

#[tokio::test]
async fn null_x_client_returns_auth_expired() {
    let client = NullX;
    let result = client.search_tweets("test", 10, None, None).await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.get_me().await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.post_tweet("hello").await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.get_tweet("123").await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.reply_to_tweet("reply", "123").await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.get_mentions("user1", None, None).await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.get_user_tweets("user1", 10, None).await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));

    let result = client.get_user_by_username("test").await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));
}
