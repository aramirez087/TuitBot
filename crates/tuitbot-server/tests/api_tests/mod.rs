//! Integration tests for the tuitbot-server API routes.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::storage;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::AccountWsEvent;

mod analytics;
mod analytics_summary; // Task 3.4: analytics summary + recent-performance endpoints
mod approval;
mod approval_coverage; // Additional approval queue edge-case coverage
mod approval_workflow; // Task 3.4: approval happy-path mutations (approve/reject/bulk/history)
mod compose;
mod connectors_coverage; // Connector routes (link, status, disconnect) coverage
mod content;
mod content_drafts; // Task 3.4: legacy /api/content/drafts CRUD + publish + schedule
mod coverage_gaps;
mod deep_handler_coverage; // Deep happy-path integration tests for large handlers
mod discovery;
mod discovery_feed; // Task 3.4: discovery feed + queue-reply routes
mod draft_studio_coverage; // Additional draft studio error-path coverage
mod mcp_policy; // Task 3.8: /api/mcp/* policy + telemetry coverage
mod route_coverage_extra; // Extra compose, onboarding, media, ingest, assist, draft-studio coverage
mod settings_accounts; // Settings, accounts, activity, connectors, vault, content, and misc route coverage
mod settings_init_workflow; // Settings init → get → patch → validate workflow coverage
mod x_auth; // X OAuth unlink tests // Task 3.8: integration tests for 0%-coverage server routes

/// The test API token used across all tests.
pub const TEST_TOKEN: &str = "test-token-abc123";

/// Create the test router backed by an in-memory SQLite database.
pub async fn test_router() -> axum::Router {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
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

    tuitbot_server::build_router(state)
}

/// Helper: send a GET request with auth and parse JSON from the response.
pub async fn get_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let body = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body.to_bytes()).expect("parse JSON");

    (status, json)
}

/// Helper: send a POST request with auth and JSON body.
pub async fn post_json(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

/// Helper: send a PATCH request with auth and JSON body.
pub async fn patch_json(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("PATCH")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

/// Helper: send a DELETE request with auth.
pub async fn delete_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

// ============================================================
// Isolation helpers (shared by discovery and content modules)
// ============================================================

pub async fn test_router_with_dir(dir: &std::path::Path) -> (axum::Router, storage::DbPool) {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a minimal valid config.toml
    let config_path = dir.join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[x_api]
provider_backend = "scraper"
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_keywords = ["test"]
"#,
    )
    .expect("write config");

    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path,
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
        x_client_id: "test-client-id".to_string(),
    });

    (tuitbot_server::build_router(state), pool)
}

/// Helper: send a GET request with auth + X-Account-Id header.
pub async fn get_json_for(
    router: axum::Router,
    path: &str,
    account_id: &str,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("X-Account-Id", account_id)
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let body = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body.to_bytes()).expect("parse JSON");
    (status, json)
}

/// Helper: send a POST request with auth + X-Account-Id header.
pub async fn post_json_for(
    router: axum::Router,
    path: &str,
    account_id: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .header("X-Account-Id", account_id)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");
    (status, json)
}

/// Helper: send a DELETE request with auth + X-Account-Id header.
pub async fn delete_json_for(
    router: axum::Router,
    path: &str,
    account_id: &str,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("X-Account-Id", account_id)
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");
    (status, json)
}

/// Create a non-default account, returning its account_id.
pub async fn create_test_account(pool: &storage::DbPool, label: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    tuitbot_core::storage::accounts::create_account(pool, &id, label)
        .await
        .expect("create account");
    id
}
