//! Integration tests for the fresh-install claim bootstrap flow.
//!
//! Tests that `POST /api/settings/init` can accept an optional `claim` object
//! to create a passphrase hash and session in one step, and that
//! `GET /api/settings/status` reports the `claimed` state.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::auth::passphrase;
use tuitbot_core::storage;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

const TEST_TOKEN: &str = "test-token-abc123";

/// A valid minimal config body that passes `Config::validate()`.
fn valid_config_body() -> serde_json::Value {
    serde_json::json!({
        "business": {
            "product_name": "TestBot",
            "product_keywords": ["rust", "testing"]
        }
    })
}

/// Create a test router with an isolated temp directory for data_dir and config_path.
async fn test_router_with_dir(dir: &std::path::Path) -> axum::Router {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let state = Arc::new(AppState {
        db: pool,
        config_path: dir.join("config.toml"),
        data_dir: dir.to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: tokio::sync::RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(std::collections::HashMap::new()),
        content_generators: Mutex::new(std::collections::HashMap::new()),
        runtimes: Mutex::new(std::collections::HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: None,
        content_sources: Default::default(),
        deployment_mode: Default::default(),
        pending_oauth: Mutex::new(std::collections::HashMap::new()),
    });

    tuitbot_server::build_router(state)
}

/// Send a POST with JSON body (no auth).
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

/// Send a GET request (no auth).
async fn get_json_noauth(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

// ============================================================
// Claim bootstrap
// ============================================================

#[tokio::test]
async fn claim_creates_passphrase_and_session() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let router = test_router_with_dir(dir.path()).await;

    let mut body = valid_config_body();
    body["claim"] = serde_json::json!({ "passphrase": "alpha bravo charlie delta" });

    let (status, json, headers) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");
    assert!(
        json["csrf_token"].is_string(),
        "expected csrf_token in response"
    );
    assert!(json["config"].is_object(), "expected config in response");

    // Set-Cookie header must be present with tuitbot_session.
    let cookie = headers
        .get("set-cookie")
        .expect("expected Set-Cookie header")
        .to_str()
        .unwrap();
    assert!(
        cookie.contains("tuitbot_session="),
        "cookie should contain tuitbot_session"
    );
    assert!(cookie.contains("HttpOnly"), "cookie should be HttpOnly");

    // Hash file should exist and verify against the passphrase.
    assert!(passphrase::is_claimed(dir.path()));
    let hash = passphrase::load_passphrase_hash(dir.path())
        .unwrap()
        .unwrap();
    assert!(passphrase::verify_passphrase("alpha bravo charlie delta", &hash).unwrap());
}

#[tokio::test]
async fn claim_rejects_already_claimed() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Pre-create a passphrase hash.
    passphrase::create_passphrase_hash(dir.path(), "existing passphrase ok").unwrap();

    let router = test_router_with_dir(dir.path()).await;

    let mut body = valid_config_body();
    body["claim"] = serde_json::json!({ "passphrase": "new passphrase attempt" });

    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already claimed"));
}

#[tokio::test]
async fn claim_rejects_short_passphrase() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let router = test_router_with_dir(dir.path()).await;

    let mut body = valid_config_body();
    body["claim"] = serde_json::json!({ "passphrase": "short" });

    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"].as_str().unwrap().contains("8 characters"));
}

#[tokio::test]
async fn init_without_claim_works_as_before() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let router = test_router_with_dir(dir.path()).await;

    let body = valid_config_body();

    let (status, json, headers) = post_json_noauth(router, "/api/settings/init", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");
    assert!(json["config"].is_object());
    // No csrf_token or Set-Cookie without claim.
    assert!(
        json.get("csrf_token").is_none(),
        "should not have csrf_token without claim"
    );
    assert!(
        headers.get("set-cookie").is_none(),
        "should not have Set-Cookie without claim"
    );
    // No passphrase hash file.
    assert!(!passphrase::is_claimed(dir.path()));
}

#[tokio::test]
async fn double_init_returns_409() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let router = test_router_with_dir(dir.path()).await;

    let body = valid_config_body();

    // First call succeeds.
    let (status, _, _) = post_json_noauth(router.clone(), "/api/settings/init", body.clone()).await;
    assert_eq!(status, StatusCode::OK);

    // Second call returns 409.
    let (status, json, _) = post_json_noauth(router, "/api/settings/init", body).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(json["error"].as_str().unwrap().contains("already exists"));
}

// ============================================================
// Config status with claimed field
// ============================================================

#[tokio::test]
async fn config_status_includes_claimed_false() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let router = test_router_with_dir(dir.path()).await;

    let (status, json) = get_json_noauth(router, "/api/settings/status").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["configured"], false);
    assert_eq!(json["claimed"], false);
    assert!(json["deployment_mode"].is_string());
    assert!(json["capabilities"].is_object());
}

#[tokio::test]
async fn config_status_includes_claimed_true() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Pre-create passphrase hash.
    passphrase::create_passphrase_hash(dir.path(), "test passphrase here").unwrap();

    let router = test_router_with_dir(dir.path()).await;

    let (status, json) = get_json_noauth(router, "/api/settings/status").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["claimed"], true);
}

// ============================================================
// Session validity after claim
// ============================================================

#[tokio::test]
async fn init_with_claim_produces_valid_session() {
    let dir = tempfile::tempdir().expect("create temp dir");

    // Use a shared DB pool so session persists across requests.
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let state = Arc::new(AppState {
        db: pool,
        config_path: dir.path().join("config.toml"),
        data_dir: dir.path().to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: tokio::sync::RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(std::collections::HashMap::new()),
        content_generators: Mutex::new(std::collections::HashMap::new()),
        runtimes: Mutex::new(std::collections::HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: None,
        content_sources: Default::default(),
        deployment_mode: Default::default(),
        pending_oauth: Mutex::new(std::collections::HashMap::new()),
    });
    let router = tuitbot_server::build_router(state);

    // Claim during init.
    let mut body = valid_config_body();
    body["claim"] = serde_json::json!({ "passphrase": "alpha bravo charlie delta" });

    let (status, json, headers) =
        post_json_noauth(router.clone(), "/api/settings/init", body).await;
    assert_eq!(status, StatusCode::OK);

    let csrf_token = json["csrf_token"].as_str().unwrap();
    let cookie_value = headers
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Extract the raw session token from the Set-Cookie header.
    let session_token = cookie_value
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("tuitbot_session=")
        .unwrap();

    // Use the session cookie + CSRF token to make an authenticated request.
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth/status")
        .header("cookie", format!("tuitbot_session={session_token}"))
        .body(Body::empty())
        .expect("build request");

    let response = router.clone().oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.expect("read body");
    let auth_status: serde_json::Value =
        serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");
    assert_eq!(auth_status["authenticated"], true);
    assert_eq!(auth_status["csrf_token"], csrf_token);
}
