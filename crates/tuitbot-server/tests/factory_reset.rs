//! Integration tests for the factory-reset endpoint.
//!
//! Validates auth enforcement, confirmation phrase checking, data clearing,
//! cookie clearing, idempotency, config status after reset, and re-onboarding.

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

/// Build an AppState backed by an in-memory DB and a real temp directory.
async fn test_state_with_dir(dir: &std::path::Path) -> Arc<AppState> {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    Arc::new(AppState {
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
    })
}

/// Send a POST with JSON body and bearer auth, returning status + JSON + headers.
async fn post_json_auth(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value, axum::http::HeaderMap) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
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

/// Send a POST with JSON body and NO auth.
async fn post_json_noauth(router: axum::Router, path: &str, body: serde_json::Value) -> StatusCode {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    response.status()
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

/// Send a POST with JSON body and NO auth, returning full response.
async fn post_json_noauth_full(
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
// Auth enforcement
// ============================================================

#[tokio::test]
async fn factory_reset_requires_auth() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;
    let router = tuitbot_server::build_router(state);

    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let status = post_json_noauth(router, "/api/settings/factory-reset", body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================
// Confirmation phrase validation
// ============================================================

#[tokio::test]
async fn factory_reset_rejects_wrong_confirmation() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;
    let router = tuitbot_server::build_router(state);

    // Wrong phrase.
    let body = serde_json::json!({ "confirmation": "wrong" });
    let (status, json, _) =
        post_json_auth(router.clone(), "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("incorrect confirmation"));

    // Case mismatch.
    let body = serde_json::json!({ "confirmation": "reset tuitbot" });
    let (status, _, _) = post_json_auth(router.clone(), "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Empty string.
    let body = serde_json::json!({ "confirmation": "" });
    let (status, _, _) = post_json_auth(router, "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Happy path
// ============================================================

#[tokio::test]
async fn factory_reset_success() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;

    // Seed: config file.
    std::fs::write(
        dir.path().join("config.toml"),
        "[business]\nproduct_name = \"Test\"\n",
    )
    .unwrap();

    // Seed: passphrase hash.
    passphrase::create_passphrase_hash(dir.path(), "test passphrase here").unwrap();

    // Seed: media directory with a file.
    let media_dir = dir.path().join("media");
    std::fs::create_dir_all(&media_dir).unwrap();
    std::fs::write(media_dir.join("test.jpg"), b"fake image").unwrap();

    let router = tuitbot_server::build_router(state.clone());

    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let (status, json, headers) =
        post_json_auth(router.clone(), "/api/settings/factory-reset", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "reset_complete");

    let cleared = &json["cleared"];
    assert_eq!(cleared["tables_cleared"], 30);
    // Migration seeds 1 account + 2 account_roles = at least 3 rows.
    assert!(cleared["rows_deleted"].as_u64().unwrap() >= 3);
    assert_eq!(cleared["config_deleted"], true);
    assert_eq!(cleared["passphrase_deleted"], true);
    assert_eq!(cleared["media_deleted"], true);

    // Verify files deleted.
    assert!(!dir.path().join("config.toml").exists());
    assert!(!dir.path().join("passphrase_hash").exists());
    assert!(!dir.path().join("media").exists());

    // Verify tables are empty by running a second reset (should find 0 rows).
    let body2 = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let (status2, json2, _) = post_json_auth(router, "/api/settings/factory-reset", body2).await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(json2["cleared"]["rows_deleted"], 0);

    // Verify Set-Cookie header clears session.
    let cookie = headers
        .get("set-cookie")
        .expect("expected Set-Cookie header")
        .to_str()
        .unwrap();
    assert!(cookie.contains("tuitbot_session=;"));
    assert!(cookie.contains("Max-Age=0"));

    // Verify in-memory passphrase cleared.
    assert!(state.passphrase_hash.read().await.is_none());
}

// ============================================================
// Idempotency
// ============================================================

#[tokio::test]
async fn factory_reset_idempotent() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;

    // Write config so first reset can delete it.
    std::fs::write(dir.path().join("config.toml"), "[business]\n").unwrap();

    let router = tuitbot_server::build_router(state.clone());

    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });

    // First reset.
    let (status1, json1, _) =
        post_json_auth(router.clone(), "/api/settings/factory-reset", body.clone()).await;
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(json1["status"], "reset_complete");
    assert_eq!(json1["cleared"]["config_deleted"], true);

    // Second reset on already-reset instance.
    let (status2, json2, _) = post_json_auth(router, "/api/settings/factory-reset", body).await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(json2["status"], "reset_complete");
    assert_eq!(json2["cleared"]["rows_deleted"], 0);
    assert_eq!(json2["cleared"]["config_deleted"], false);
}

// ============================================================
// Config status after reset
// ============================================================

#[tokio::test]
async fn factory_reset_clears_config_status() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;

    // Create config so status shows configured=true.
    std::fs::write(dir.path().join("config.toml"), "[business]\n").unwrap();
    passphrase::create_passphrase_hash(dir.path(), "test passphrase here").unwrap();

    let router = tuitbot_server::build_router(state);

    // Verify configured before reset.
    let (status, json) = get_json_noauth(router.clone(), "/api/settings/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["configured"], true);
    assert_eq!(json["claimed"], true);

    // Reset.
    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let (status, _, _) = post_json_auth(router.clone(), "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::OK);

    // Verify unconfigured after reset.
    let (status, json) = get_json_noauth(router, "/api/settings/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["configured"], false);
    assert_eq!(json["claimed"], false);
}

// ============================================================
// Cookie clearing
// ============================================================

#[tokio::test]
async fn factory_reset_cookie_clearing() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;
    let router = tuitbot_server::build_router(state);

    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let (status, _, headers) = post_json_auth(router, "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::OK);

    let cookie = headers
        .get("set-cookie")
        .expect("Set-Cookie header required")
        .to_str()
        .unwrap();
    assert!(cookie.contains("tuitbot_session=;"), "should clear session");
    assert!(cookie.contains("HttpOnly"), "should be HttpOnly");
    assert!(cookie.contains("SameSite=Strict"), "should be SameSite");
    assert!(cookie.contains("Path=/"), "should have Path=/");
    assert!(cookie.contains("Max-Age=0"), "should expire immediately");
}

// ============================================================
// Re-onboarding after reset
// ============================================================

#[tokio::test]
async fn factory_reset_allows_re_onboarding() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let state = test_state_with_dir(dir.path()).await;

    // Create config so we can reset it.
    std::fs::write(dir.path().join("config.toml"), "[business]\n").unwrap();

    let router = tuitbot_server::build_router(state);

    // Reset.
    let body = serde_json::json!({ "confirmation": "RESET TUITBOT" });
    let (status, _, _) = post_json_auth(router.clone(), "/api/settings/factory-reset", body).await;
    assert_eq!(status, StatusCode::OK);

    // Re-onboard via init (auth-exempt route).
    let config_body = valid_config_body();
    let (status, json, _) = post_json_noauth_full(router, "/api/settings/init", config_body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "created");
    assert!(json["config"].is_object());
}
