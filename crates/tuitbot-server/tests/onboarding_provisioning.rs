//! Integration tests for onboarding provisioning: X profile population,
//! token migration, idempotent completion, and account-context correctness.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::storage;
use tuitbot_core::storage::accounts::{self, DEFAULT_ACCOUNT_ID};
use tuitbot_core::storage::DbPool;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::AccountWsEvent;

const TEST_TOKEN: &str = "test-token-abc123";

/// A valid minimal config body that passes `Config::validate()`.
fn valid_config_body() -> serde_json::Value {
    serde_json::json!({
        "x_api": {
            "client_id": "test-client-id"
        },
        "business": {
            "product_name": "TestBot",
            "product_keywords": ["rust", "testing"],
            "product_description": "A test bot",
            "industry_topics": ["testing"]
        }
    })
}

/// Create a test router with an isolated temp directory, returning the DB pool
/// so tests can verify account state.
async fn test_router_with_pool(dir: &std::path::Path) -> (axum::Router, DbPool) {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path: dir.join("config.toml"),
        data_dir: dir.to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: tokio::sync::RwLock::new(None),
        passphrase_hash_mtime: tokio::sync::RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(std::collections::HashMap::new()),
        content_generators: Mutex::new(std::collections::HashMap::new()),
        runtimes: Mutex::new(std::collections::HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: tokio::sync::RwLock::new(None),
        content_sources: tokio::sync::RwLock::new(Default::default()),
        connector_config: Default::default(),
        deployment_mode: Default::default(),
        pending_oauth: Mutex::new(std::collections::HashMap::new()),
        token_managers: Mutex::new(std::collections::HashMap::new()),
        x_client_id: String::new(),
    });

    (tuitbot_server::build_router(state), pool)
}

/// Send a POST with JSON body (no auth) and return status, body, headers.
async fn post_json_noauth(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value, axum::http::HeaderMap) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json, headers)
}

// ============================================================
// X profile provisioning
// ============================================================

#[tokio::test]
async fn init_with_x_profile_populates_account() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, pool) = test_router_with_pool(dir.path()).await;

    let mut body = valid_config_body();
    body["x_profile"] = serde_json::json!({
        "x_user_id": "12345",
        "x_username": "testuser",
        "x_display_name": "Test User",
        "x_avatar_url": "https://pbs.twimg.com/profile/test.jpg"
    });

    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");

    // Verify the default account has X profile data.
    let account = accounts::get_account(&pool, DEFAULT_ACCOUNT_ID)
        .await
        .expect("db query")
        .expect("default account should exist");

    assert_eq!(account.x_user_id.as_deref(), Some("12345"));
    assert_eq!(account.x_username.as_deref(), Some("testuser"));
    assert_eq!(account.x_display_name.as_deref(), Some("Test User"));
    assert_eq!(
        account.x_avatar_url.as_deref(),
        Some("https://pbs.twimg.com/profile/test.jpg")
    );
}

#[tokio::test]
async fn init_without_x_profile_leaves_account_empty() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, pool) = test_router_with_pool(dir.path()).await;

    let body = valid_config_body();

    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");

    // Verify the default account has no X profile data.
    let account = accounts::get_account(&pool, DEFAULT_ACCOUNT_ID)
        .await
        .expect("db query")
        .expect("default account should exist");

    assert!(account.x_user_id.is_none());
    assert!(account.x_username.is_none());
    assert!(account.x_display_name.is_none());
    assert!(account.x_avatar_url.is_none());
}

#[tokio::test]
async fn init_with_x_profile_and_claim() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, pool) = test_router_with_pool(dir.path()).await;

    let mut body = valid_config_body();
    body["x_profile"] = serde_json::json!({
        "x_user_id": "99999",
        "x_username": "claimeduser",
        "x_display_name": "Claimed User"
    });
    body["claim"] = serde_json::json!({ "passphrase": "alpha bravo charlie delta" });

    let (status, json, headers) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");
    assert!(json["csrf_token"].is_string());

    // Session cookie should be set.
    let cookie = headers
        .get("set-cookie")
        .expect("expected Set-Cookie header")
        .to_str()
        .unwrap();
    assert!(cookie.contains("tuitbot_session="));

    // Verify X profile was written.
    let account = accounts::get_account(&pool, DEFAULT_ACCOUNT_ID)
        .await
        .expect("db query")
        .expect("default account should exist");

    assert_eq!(account.x_user_id.as_deref(), Some("99999"));
    assert_eq!(account.x_username.as_deref(), Some("claimeduser"));
    assert_eq!(account.x_display_name.as_deref(), Some("Claimed User"));
    // x_avatar_url not provided — should be None.
    assert!(account.x_avatar_url.is_none());
}

// ============================================================
// Token migration
// ============================================================

#[tokio::test]
async fn init_migrates_onboarding_tokens() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Create a fake onboarding_tokens.json.
    let onboarding_path = dir.path().join("onboarding_tokens.json");
    std::fs::write(&onboarding_path, r#"{"access_token":"tok123"}"#).expect("write tokens");
    assert!(onboarding_path.exists());

    let (router, _pool) = test_router_with_pool(dir.path()).await;

    let (status, json, _) =
        post_json_noauth(router, "/api/settings/init", valid_config_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");

    // onboarding_tokens.json should be gone (moved).
    assert!(!onboarding_path.exists());

    // Token file should exist at the default account path.
    let target = accounts::account_token_path(dir.path(), DEFAULT_ACCOUNT_ID);
    assert!(target.exists());
    let content = std::fs::read_to_string(&target).expect("read migrated tokens");
    assert!(content.contains("tok123"));
}

#[tokio::test]
async fn token_migration_missing_source_is_noop() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // No onboarding_tokens.json — should not error.
    let (router, _pool) = test_router_with_pool(dir.path()).await;

    let (status, json, _) =
        post_json_noauth(router, "/api/settings/init", valid_config_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");

    // No token file at the default account path.
    let target = accounts::account_token_path(dir.path(), DEFAULT_ACCOUNT_ID);
    assert!(!target.exists());
}

// ============================================================
// Idempotency
// ============================================================

#[tokio::test]
async fn double_init_returns_409() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, _pool) = test_router_with_pool(dir.path()).await;

    let body = valid_config_body();

    // First call succeeds.
    let (status, _, _) = post_json_noauth(router.clone(), "/api/settings/init", body.clone()).await;
    assert_eq!(status, StatusCode::OK);

    // Second call returns 409.
    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn double_init_with_x_profile_first_wins() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, pool) = test_router_with_pool(dir.path()).await;

    let mut body = valid_config_body();
    body["x_profile"] = serde_json::json!({
        "x_user_id": "first",
        "x_username": "firstuser",
        "x_display_name": "First User"
    });

    // First call succeeds and sets profile.
    let (status, _, _) = post_json_noauth(router.clone(), "/api/settings/init", body).await;
    assert_eq!(status, StatusCode::OK);

    // Second call with different profile returns 409.
    let mut body2 = valid_config_body();
    body2["x_profile"] = serde_json::json!({
        "x_user_id": "second",
        "x_username": "seconduser",
        "x_display_name": "Second User"
    });
    let (status, _, _) = post_json_noauth(router, "/api/settings/init", body2).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Profile should still be from the first call.
    let account = accounts::get_account(&pool, DEFAULT_ACCOUNT_ID)
        .await
        .expect("db query")
        .expect("default account should exist");
    assert_eq!(account.x_user_id.as_deref(), Some("first"));
    assert_eq!(account.x_username.as_deref(), Some("firstuser"));
}

#[tokio::test]
async fn init_with_invalid_x_profile_returns_400() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (router, _pool) = test_router_with_pool(dir.path()).await;

    let mut body = valid_config_body();
    // Invalid x_profile — missing required fields.
    body["x_profile"] = serde_json::json!({ "bad_field": true });

    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("invalid x_profile"));

    // Config should NOT have been created (fail fast).
    assert!(!dir.path().join("config.toml").exists());
}
