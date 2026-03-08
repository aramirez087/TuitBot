//! Integration tests for Draft Studio API endpoints (`/api/drafts`).

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
// Collection & CRUD
// ============================================================

#[tokio::test]
async fn list_drafts_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_draft_returns_id_and_updated_at() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/drafts",
        serde_json::json!({ "content": "Hello world" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().unwrap() > 0);
    assert!(body["updated_at"].as_str().is_some());
}

#[tokio::test]
async fn create_draft_with_title() {
    let router = test_router().await;
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        serde_json::json!({ "content": "Content", "title": "My Title" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = body["id"].as_i64().unwrap();

    // Verify title is set
    let (status, draft) = get_json(router, &format!("/api/drafts/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(draft["title"].as_str().unwrap(), "My Title");
}

#[tokio::test]
async fn create_draft_blank() {
    let router = test_router().await;
    let (status, body) = post_json(router, "/api/drafts", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn get_draft_by_id() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Test content").await;

    let (status, body) = get_json(router, &format!("/api/drafts/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"].as_i64().unwrap(), id);
    assert_eq!(body["content"].as_str().unwrap(), "Test content");
    assert_eq!(body["status"].as_str().unwrap(), "draft");
}

#[tokio::test]
async fn get_draft_not_found() {
    let router = test_router().await;
    let (status, _) = get_json(router, "/api/drafts/9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_drafts_returns_summaries() {
    let router = test_router().await;
    let long_content = "a".repeat(100);
    create_test_draft(&router, &long_content).await;

    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK);
    let drafts = body.as_array().unwrap();
    assert_eq!(drafts.len(), 1);

    let preview = drafts[0]["content_preview"].as_str().unwrap();
    assert!(
        preview.len() <= 60,
        "preview should be truncated: {preview}"
    );
    assert!(preview.ends_with("..."));
}

// ============================================================
// Autosave
// ============================================================

#[tokio::test]
async fn autosave_patch_updates_content() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Original").await;

    let (status, body) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Updated content",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"].as_i64().unwrap(), id);
    assert!(body["updated_at"].as_str().is_some());

    // Verify content changed
    let (_, draft) = get_json(router, &format!("/api/drafts/{id}")).await;
    assert_eq!(draft["content"].as_str().unwrap(), "Updated content");
}

#[tokio::test]
async fn autosave_patch_stale_write_returns_409() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Original").await;

    let (status, body) = patch_json(
        router,
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Should fail",
            "content_type": "tweet",
            "updated_at": "1999-01-01 00:00:00"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let error_body: serde_json::Value =
        serde_json::from_str(body["error"].as_str().unwrap()).unwrap();
    assert_eq!(error_body["error"], "stale_write");
    assert!(error_body["server_updated_at"].as_str().is_some());
}

#[tokio::test]
async fn autosave_does_not_create_revision() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Original").await;

    // Autosave
    let (status, _) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Updated",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Check no revisions
    let (_, revisions) = get_json(router, &format!("/api/drafts/{id}/revisions")).await;
    assert_eq!(revisions.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn autosave_does_not_log_activity() {
    let router = test_router().await;
    let (id, updated_at) = create_test_draft(&router, "Original").await;

    // Autosave
    let (status, _) = patch_json(
        router.clone(),
        &format!("/api/drafts/{id}"),
        serde_json::json!({
            "content": "Updated",
            "content_type": "tweet",
            "updated_at": updated_at
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Activity should only have "created", not an autosave entry
    let (_, activity) = get_json(router, &format!("/api/drafts/{id}/activity")).await;
    let entries = activity.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"].as_str().unwrap(), "created");
}

// ============================================================
// Metadata
// ============================================================

#[tokio::test]
async fn patch_meta_updates_title_and_notes() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Content").await;

    let (status, body) = patch_json(
        router,
        &format!("/api/drafts/{id}/meta"),
        serde_json::json!({ "title": "My Draft", "notes": "Some notes" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"].as_str().unwrap(), "My Draft");
    assert_eq!(body["notes"].as_str().unwrap(), "Some notes");
}

// ============================================================
// Workflow transitions
// ============================================================

#[tokio::test]
async fn schedule_draft_creates_revision_and_activity() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Schedule me").await;

    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/schedule"),
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"].as_str().unwrap(), "scheduled");
    assert_eq!(
        body["scheduled_for"].as_str().unwrap(),
        "2099-12-31T23:59:59"
    );

    // Check revision was created
    let (_, revisions) = get_json(router.clone(), &format!("/api/drafts/{id}/revisions")).await;
    let revs = revisions.as_array().unwrap();
    assert_eq!(revs.len(), 1);
    assert_eq!(revs[0]["trigger_kind"].as_str().unwrap(), "schedule");

    // Check activity was logged (created + scheduled)
    let (_, activity) = get_json(router, &format!("/api/drafts/{id}/activity")).await;
    let acts = activity.as_array().unwrap();
    assert_eq!(acts.len(), 2);
    // Newest first
    assert_eq!(acts[0]["action"].as_str().unwrap(), "scheduled");
    assert_eq!(acts[1]["action"].as_str().unwrap(), "created");
}

#[tokio::test]
async fn unschedule_returns_to_draft() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Unschedule me").await;

    // Schedule first
    post_json(
        router.clone(),
        &format!("/api/drafts/{id}/schedule"),
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59" }),
    )
    .await;

    // Unschedule
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/unschedule"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"].as_str().unwrap(), "draft");

    // Verify in DB
    let (_, draft) = get_json(router, &format!("/api/drafts/{id}")).await;
    assert_eq!(draft["status"].as_str().unwrap(), "draft");
    assert!(draft["scheduled_for"].is_null());
}

#[tokio::test]
async fn schedule_non_draft_fails() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Already scheduled").await;

    // Schedule once
    post_json(
        router.clone(),
        &format!("/api/drafts/{id}/schedule"),
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59" }),
    )
    .await;

    // Try to schedule again
    let (status, _) = post_json(
        router,
        &format!("/api/drafts/{id}/schedule"),
        serde_json::json!({ "scheduled_for": "2099-12-31T00:00:00" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Archive / Restore
// ============================================================

#[tokio::test]
async fn archive_hides_from_list() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Archive me").await;

    // Archive
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/archive"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["archived_at"].as_str().is_some());

    // List should be empty
    let (_, list) = get_json(router, "/api/drafts").await;
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn archive_shows_in_archived_list() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Archive me").await;

    // Archive
    post_json(
        router.clone(),
        &format!("/api/drafts/{id}/archive"),
        serde_json::json!({}),
    )
    .await;

    // Archived list should have 1 item
    let (_, list) = get_json(router, "/api/drafts?archived=true").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn restore_returns_to_list() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Restore me").await;

    // Archive then restore
    post_json(
        router.clone(),
        &format!("/api/drafts/{id}/archive"),
        serde_json::json!({}),
    )
    .await;
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/restore"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Should be back in the active list
    let (_, list) = get_json(router, "/api/drafts").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

// ============================================================
// Duplicate
// ============================================================

#[tokio::test]
async fn duplicate_creates_new_draft() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Duplicate me").await;

    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/duplicate"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let new_id = body["id"].as_i64().unwrap();
    assert_ne!(new_id, id);

    // Verify the new draft exists and is a draft
    let (_, draft) = get_json(router, &format!("/api/drafts/{new_id}")).await;
    assert_eq!(draft["status"].as_str().unwrap(), "draft");
    assert_eq!(draft["content"].as_str().unwrap(), "Duplicate me");
}

// ============================================================
// Revisions & Activity
// ============================================================

#[tokio::test]
async fn list_revisions_empty() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "No revisions").await;

    let (status, body) = get_json(router, &format!("/api/drafts/{id}/revisions")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_manual_revision() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Revision content").await;

    let (status, body) = post_json(
        router.clone(),
        &format!("/api/drafts/{id}/revisions"),
        serde_json::json!({ "trigger_kind": "manual" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().unwrap() > 0);

    // List should have 1
    let (_, revisions) = get_json(router, &format!("/api/drafts/{id}/revisions")).await;
    let revs = revisions.as_array().unwrap();
    assert_eq!(revs.len(), 1);
    assert_eq!(revs[0]["trigger_kind"].as_str().unwrap(), "manual");
    assert_eq!(revs[0]["content"].as_str().unwrap(), "Revision content");
}

#[tokio::test]
async fn list_activity_shows_created() {
    let router = test_router().await;
    let (id, _) = create_test_draft(&router, "Activity test").await;

    let (status, body) = get_json(router, &format!("/api/drafts/{id}/activity")).await;
    assert_eq!(status, StatusCode::OK);
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["action"].as_str().unwrap(), "created");
}

// ============================================================
// Backward compatibility
// ============================================================

#[tokio::test]
async fn legacy_list_drafts_still_works() {
    let router = test_router().await;

    // Create via legacy endpoint
    post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Legacy draft"
        }),
    )
    .await;

    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn legacy_edit_draft_still_works() {
    let router = test_router().await;

    // Create via legacy endpoint
    let (status, create_body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Original"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = create_body["id"].as_i64().unwrap();

    // Edit via legacy endpoint
    let (status, _) = patch_json(
        router,
        &format!("/api/content/drafts/{id}"),
        serde_json::json!({ "content": "Edited" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
