//! Integration tests for legacy content/drafts routes (Task 3.4).
//!
//! Covers:
//!   GET    /api/content/drafts                — list
//!   POST   /api/content/drafts                — create (tweet + thread)
//!   PATCH  /api/content/drafts/{id}           — edit
//!   DELETE /api/content/drafts/{id}           — delete
//!   POST   /api/content/drafts/{id}/publish   — publish (error: no X creds)
//!   POST   /api/content/drafts/{id}/schedule  — schedule (happy path)

use super::*;

// ---------------------------------------------------------------------------
// GET /api/content/drafts — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_list_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected array, got: {body}");
}

#[tokio::test]
async fn content_drafts_list_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/content/drafts")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn content_drafts_list_empty_on_fresh_db() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.as_array().unwrap().len(),
        0,
        "fresh db should have 0 drafts"
    );
}

// ---------------------------------------------------------------------------
// POST /api/content/drafts — create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_create_tweet_returns_id() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Hello world draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create tweet draft: {body}");
    assert!(body["id"].is_number(), "response should include id: {body}");
}

#[tokio::test]
async fn content_drafts_create_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/content/drafts")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            r#"{"content_type":"tweet","content":"test","source":"manual"}"#,
        ))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn content_drafts_create_reply_type_returns_id() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "reply",
            "content": "This is a reply draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create reply draft: {body}");
    assert!(body["id"].is_number());
}

#[tokio::test]
async fn content_drafts_create_appears_in_list() {
    let router = test_router().await;

    // Create a draft.
    let (status, body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Listable draft",
            "source": "manual"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = body["id"].as_i64().unwrap();

    // List should contain it.
    let (status, list) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK);
    let arr = list.as_array().unwrap();
    assert!(
        arr.iter().any(|d| d["id"].as_i64() == Some(id)),
        "created draft {id} should appear in list"
    );
}

// ---------------------------------------------------------------------------
// PATCH /api/content/drafts/{id} — edit
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_edit_updates_content() {
    let router = test_router().await;

    // Create.
    let (_, body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Original draft text",
            "source": "manual"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Edit.
    let (status, edit_body) = patch_json(
        router,
        &format!("/api/content/drafts/{id}"),
        serde_json::json!({"content": "Updated draft text"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "edit draft: {edit_body}");
}

#[tokio::test]
async fn content_drafts_edit_nonexistent_silently_succeeds() {
    // The legacy edit_draft handler performs a no-op SQL UPDATE for missing IDs
    // (does not 404). This test documents the current permissive behavior.
    let router = test_router().await;
    let (status, _) = patch_json(
        router,
        "/api/content/drafts/999999",
        serde_json::json!({"content": "Updated"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn content_drafts_edit_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("PATCH")
        .uri("/api/content/drafts/1")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"content":"hi"}"#))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// DELETE /api/content/drafts/{id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_delete_removes_draft() {
    let router = test_router().await;

    // Create.
    let (_, body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "To be deleted",
            "source": "manual"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Delete.
    let (status, _) = delete_json(router.clone(), &format!("/api/content/drafts/{id}")).await;
    assert_eq!(status, StatusCode::OK);

    // Confirm removed from list.
    let (_, list) = get_json(router, "/api/content/drafts").await;
    let arr = list.as_array().unwrap();
    assert!(
        !arr.iter().any(|d| d["id"].as_i64() == Some(id)),
        "deleted draft {id} should not appear in list"
    );
}

#[tokio::test]
async fn content_drafts_delete_nonexistent_silently_succeeds() {
    // delete_draft uses a no-op SQL DELETE — missing IDs return 200 (not 404).
    // This documents the current permissive behavior.
    let router = test_router().await;
    let (status, _) = delete_json(router, "/api/content/drafts/999999").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn content_drafts_delete_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("DELETE")
        .uri("/api/content/drafts/1")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// POST /api/content/drafts/{id}/publish — requires X creds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_publish_without_x_creds_returns_error() {
    let router = test_router().await;

    // Create a draft to publish.
    let (_, body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Publish me",
            "source": "manual"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Publish without X API tokens → should fail.
    let (status, err_body) = post_json(
        router,
        &format!("/api/content/drafts/{id}/publish"),
        serde_json::json!({}),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
        "publish without X creds should return 400 or 500, got {status}: {err_body}"
    );
}

#[tokio::test]
async fn content_drafts_publish_nonexistent_returns_client_error() {
    // publish_draft runs require_post_capable() before the ID lookup.
    // In the test router (scraper backend, no session file) this returns 400.
    // If a post-capable session existed, a missing ID would return 404.
    let router = test_router().await;
    let (status, _) = post_json(
        router,
        "/api/content/drafts/999999/publish",
        serde_json::json!({}),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND,
        "expected 400 (no session) or 404 (not found), got {status}"
    );
}

#[tokio::test]
async fn content_drafts_publish_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/content/drafts/1/publish")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from("{}"))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// POST /api/content/drafts/{id}/schedule
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_drafts_schedule_happy_path_returns_ok() {
    let router = test_router().await;

    // Create a draft.
    let (_, body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Scheduled tweet draft",
            "source": "manual"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Schedule it for the future.
    let future_ts = "2027-01-01T12:00:00Z";
    let (status, sched_body) = post_json(
        router,
        &format!("/api/content/drafts/{id}/schedule"),
        serde_json::json!({"scheduled_for": future_ts}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "schedule draft: {sched_body}");
}

#[tokio::test]
async fn content_drafts_schedule_nonexistent_silently_succeeds() {
    // schedule_draft uses a no-op SQL UPDATE — missing IDs return 200 (not 404).
    // This documents the current permissive behavior.
    let router = test_router().await;
    let (status, _) = post_json(
        router,
        "/api/content/drafts/999999/schedule",
        serde_json::json!({"scheduled_for": "2027-01-01T12:00:00Z"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn content_drafts_schedule_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/content/drafts/1/schedule")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            r#"{"scheduled_for":"2027-01-01T12:00:00Z"}"#,
        ))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
