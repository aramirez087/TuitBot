//! Integration tests for the tuitbot-server API routes.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::storage;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

/// The test API token used across all tests.
const TEST_TOKEN: &str = "test-token-abc123";

/// Create the test router backed by an in-memory SQLite database.
async fn test_router() -> axum::Router {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });

    tuitbot_server::build_router(state)
}

/// Helper: send a GET request with auth and parse JSON from the response.
async fn get_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
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
async fn post_json(
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
async fn patch_json(
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
async fn delete_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
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
// Health (no auth required)
// ============================================================

#[tokio::test]
async fn health_returns_ok() {
    let router = test_router().await;
    // Health should work WITHOUT auth.
    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================
// Auth middleware
// ============================================================

#[tokio::test]
async fn auth_required_for_api_routes() {
    let router = test_router().await;
    // Request without auth header should be rejected.
    let req = Request::builder()
        .uri("/api/approval")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_succeeds_with_valid_token() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn auth_fails_with_wrong_token() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/approval")
        .header("Authorization", "Bearer wrong-token")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================
// Analytics (read-only)
// ============================================================

#[tokio::test]
async fn analytics_followers_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/followers").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn analytics_performance_returns_object() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/performance").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["avg_reply_engagement"].is_number());
}

#[tokio::test]
async fn analytics_topics_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/topics").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

// ============================================================
// Approval mutations
// ============================================================

#[tokio::test]
async fn approval_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn approval_approve_not_found() {
    let router = test_router().await;
    let (status, body) =
        post_json(router, "/api/approval/99999/approve", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn approval_reject_not_found() {
    let router = test_router().await;
    let (status, _) = post_json(router, "/api/approval/99999/reject", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn approval_stats_returns_counts() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // Seed data.
    tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "A", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");
    let id2 = tuitbot_core::storage::approval_queue::enqueue(
        &pool, "reply", "t1", "@u", "B", "Rust", "", 50.0, "[]",
    )
    .await
    .expect("enqueue");
    tuitbot_core::storage::approval_queue::update_status(&pool, id2, "approved")
        .await
        .expect("update");

    let (status, body) = get_json(router, "/api/approval/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pending"], 1);
    assert_eq!(body["approved"], 1);
    assert_eq!(body["rejected"], 0);
}

#[tokio::test]
async fn approval_list_with_status_filter() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // Seed: one pending, one approved.
    tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "Pending", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");
    let id2 = tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "Approved", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");
    tuitbot_core::storage::approval_queue::update_status(&pool, id2, "approved")
        .await
        .expect("update");

    // Default (pending only).
    let (status, body) = get_json(router.clone(), "/api/approval").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Approved only.
    let (status, body) = get_json(router.clone(), "/api/approval?status=approved").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["generated_content"], "Approved");

    // Both pending and approved.
    let (status, body) = get_json(router, "/api/approval?status=pending,approved").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn approval_edit_content() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "Original", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = patch_json(
        router,
        &format!("/api/approval/{id}"),
        serde_json::json!({"content": "Edited version"}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["generated_content"], "Edited version");
    assert_eq!(body["status"], "pending");
}

#[tokio::test]
async fn approval_edit_not_found() {
    let router = test_router().await;
    let (status, _) = patch_json(
        router,
        "/api/approval/99999",
        serde_json::json!({"content": "Something"}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn approval_edit_empty_content() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "Original", "General", "", 0.0, "[]",
    )
    .await
    .expect("enqueue");

    let (status, _) = patch_json(
        router,
        &format!("/api/approval/{id}"),
        serde_json::json!({"content": "   "}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Activity
// ============================================================

#[tokio::test]
async fn activity_returns_paginated_object() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_object());
    assert!(body["actions"].is_array());
    assert!(body["total"].is_number());
    assert!(body["limit"].is_number());
    assert!(body["offset"].is_number());
}

// ============================================================
// Replies
// ============================================================

#[tokio::test]
async fn replies_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/replies").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

// ============================================================
// Content (read + write)
// ============================================================

#[tokio::test]
async fn content_tweets_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/tweets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn content_threads_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/threads").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn compose_tweet_validation() {
    let router = test_router().await;
    // Empty text should fail.
    let (status, _) = post_json(
        router,
        "/api/content/tweets",
        serde_json::json!({"text": ""}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_tweet_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        serde_json::json!({"text": "Hello world!"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["status"].as_str().unwrap() == "accepted"
            || body["status"].as_str().unwrap() == "queued_for_approval"
    );
}

#[tokio::test]
async fn compose_thread_validation() {
    let router = test_router().await;
    let (status, _) = post_json(
        router,
        "/api/content/threads",
        serde_json::json!({"tweets": []}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Targets (read + write)
// ============================================================

#[tokio::test]
async fn targets_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn add_and_list_target() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // Add a target.
    let (status, body) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "elonmusk"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "added");

    // List targets — should contain the new one.
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    let targets = body.as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["username"], "elonmusk");
}

#[tokio::test]
async fn add_duplicate_target_fails() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // First add succeeds.
    let (status, _) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "testtarget"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Second add should conflict.
    let (status, _) = post_json(
        router,
        "/api/targets",
        serde_json::json!({"username": "testtarget"}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn remove_target_works() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // Add a target.
    let (status, _) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "removeme"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Remove it.
    let (status, body) = delete_json(router.clone(), "/api/targets/removeme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "removed");

    // List should be empty.
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn remove_nonexistent_target_fails() {
    let router = test_router().await;
    let (status, _) = delete_json(router, "/api/targets/nonexistent").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================
// Runtime
// ============================================================

#[tokio::test]
async fn runtime_status_initially_stopped() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/runtime/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["running"], false);
}

#[tokio::test]
async fn runtime_start_and_stop() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // Start runtime.
    let (status, body) =
        post_json(router.clone(), "/api/runtime/start", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "started");

    // Check status.
    let (status, body) = get_json(router.clone(), "/api/runtime/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["running"], true);

    // Start again should conflict.
    let (status, _) = post_json(router.clone(), "/api/runtime/start", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Stop runtime.
    let (status, body) =
        post_json(router.clone(), "/api/runtime/stop", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "stopped");

    // Stop again should conflict.
    let (status, _) = post_json(router, "/api/runtime/stop", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ============================================================
// Settings
// ============================================================

#[tokio::test]
async fn settings_get_returns_json() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    // Write a minimal config file.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"TestBot\"\n").unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path,
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["business"]["product_name"], "TestBot");
}

#[tokio::test]
async fn settings_patch_round_trips() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"OldName\"\n").unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path,
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        content_generator: None,
        runtime: Mutex::new(None),
    });
    let router = tuitbot_server::build_router(state);

    // PATCH a field.
    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "business": {"product_name": "NewName"}
            }))
            .unwrap(),
        ))
        .expect("build request");

    let response = router
        .clone()
        .oneshot(patch_req)
        .await
        .expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    // GET and verify.
    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["business"]["product_name"], "NewName");
}
