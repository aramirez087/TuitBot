//! Integration tests for Draft Studio revision restore endpoint.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::storage;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::AccountWsEvent;

const TEST_TOKEN: &str = "test-token-abc123";

async fn test_router() -> axum::Router {
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

/// Helper: create a draft and return (id, updated_at).
async fn create_test_draft(router: &axum::Router, content: &str) -> (i64, String) {
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        serde_json::json!({ "content": content }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create_test_draft failed: {body}");
    let id = body["id"].as_i64().unwrap();
    let updated_at = body["updated_at"].as_str().unwrap().to_string();
    (id, updated_at)
}

// ============================================================
// Restore from revision
// ============================================================

#[tokio::test]
async fn restore_from_revision_updates_content() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Version 1").await;

    // Create a manual revision snapshot of "Version 1"
    let (_, rev_body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions"),
        serde_json::json!({ "trigger_kind": "manual" }),
    )
    .await;
    let rev_id = rev_body["id"].as_i64().unwrap();

    // Edit the draft to "Version 2"
    let (status, _) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Version 2",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Restore from the revision
    let (status, restored) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions/{rev_id}/restore"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["content"].as_str().unwrap(), "Version 1");
}

#[tokio::test]
async fn restore_creates_pre_restore_snapshot() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Version 1").await;

    // Create a manual revision
    let (_, rev_body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions"),
        serde_json::json!({ "trigger_kind": "manual" }),
    )
    .await;
    let rev_id = rev_body["id"].as_i64().unwrap();

    // Edit to "Version 2"
    let (_, patch_result) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Version 2",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    let _new_updated_at = patch_result["updated_at"].as_str().unwrap();

    // Restore from revision
    post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions/{rev_id}/restore"),
        serde_json::json!({}),
    )
    .await;

    // Check revisions: should have manual + pre_restore
    let (_, revisions) = get_json(router, &format!("/api/drafts/{id}/revisions")).await;
    let revs = revisions.as_array().unwrap();
    assert_eq!(revs.len(), 2);
    // Newest first
    assert_eq!(revs[0]["trigger_kind"].as_str().unwrap(), "pre_restore");
    assert_eq!(revs[0]["content"].as_str().unwrap(), "Version 2");
    assert_eq!(revs[1]["trigger_kind"].as_str().unwrap(), "manual");
    assert_eq!(revs[1]["content"].as_str().unwrap(), "Version 1");
}

#[tokio::test]
async fn restore_logs_revision_restored_activity() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Version 1").await;

    // Create a revision
    let (_, rev_body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions"),
        serde_json::json!({}),
    )
    .await;
    let rev_id = rev_body["id"].as_i64().unwrap();

    // Restore
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions/{rev_id}/restore"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Check activity
    let (_, activity) = get_json(router, &format!("/api/drafts/{id}/activity")).await;
    let acts = activity.as_array().unwrap();
    // Should have: revision_restored (newest), created (oldest)
    assert!(acts.len() >= 2);
    assert_eq!(acts[0]["action"].as_str().unwrap(), "revision_restored");

    // Check detail contains the revision id
    let detail: serde_json::Value =
        serde_json::from_str(acts[0]["detail"].as_str().unwrap()).unwrap();
    assert_eq!(detail["from_revision_id"].as_i64().unwrap(), rev_id);
}

#[tokio::test]
async fn restore_nonexistent_revision_returns_404() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Content").await;

    let (status, _) = post_json(
        router,
        &format!("/api/drafts/{id}/revisions/9999/restore"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn restore_nonexistent_draft_returns_404() {
    let router = test_router().await;

    let (status, _) = post_json(
        router,
        "/api/drafts/9999/revisions/1/restore",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn restore_preserves_current_state_then_reverts() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Original").await;

    // Create a revision of "Original"
    let (_, rev_body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions"),
        serde_json::json!({ "trigger_kind": "manual" }),
    )
    .await;
    let rev_id = rev_body["id"].as_i64().unwrap();

    // Edit to "Modified"
    let (_, patch_result) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Modified",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    let updated_at_2 = patch_result["updated_at"].as_str().unwrap().to_string();

    // Edit again to "Modified Again"
    patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Modified Again",
            "content_type": "tweet",
            "updated_at": updated_at_2
        }),
    )
    .await;

    // Restore to "Original"
    let (status, restored) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions/{rev_id}/restore"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(restored["content"].as_str().unwrap(), "Original");

    // The pre_restore snapshot should capture "Modified Again"
    let (_, revisions) = get_json(router, &format!("/api/drafts/{id}/revisions")).await;
    let revs = revisions.as_array().unwrap();
    let pre_restore = revs
        .iter()
        .find(|r| r["trigger_kind"] == "pre_restore")
        .unwrap();
    assert_eq!(pre_restore["content"].as_str().unwrap(), "Modified Again");
}
