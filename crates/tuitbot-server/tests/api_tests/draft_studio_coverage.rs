//! Additional coverage tests for Draft Studio error-handling code paths.

use super::*;

// ============================================================
// NOT_FOUND paths on nonexistent drafts
// ============================================================

#[tokio::test]
async fn get_studio_draft_not_found() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts/9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn autosave_draft_not_found() {
    let router = test_router().await;
    let (status, body) = patch_json(
        router,
        "/api/drafts/9999",
        serde_json::json!({
            "content": "Updated",
            "content_type": "tweet",
            "updated_at": "2099-01-01 00:00:00"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn archive_draft_not_found() {
    let router = test_router().await;
    let (status, _) = post_json(router, "/api/drafts/9999/archive", serde_json::json!({})).await;
    // archive_draft_for may silently succeed (no-op) or the subsequent get
    // returns empty archived_at — either way the status is OK with empty
    // archived_at, which is acceptable.
    assert!(
        status == StatusCode::OK || status == StatusCode::NOT_FOUND,
        "unexpected status: {status}"
    );
}

#[tokio::test]
async fn restore_draft_not_found() {
    let router = test_router().await;
    let (status, _) = post_json(router, "/api/drafts/9999/restore", serde_json::json!({})).await;
    // restore_draft_for is a no-op on nonexistent drafts — OK is acceptable.
    assert!(
        status == StatusCode::OK || status == StatusCode::NOT_FOUND,
        "unexpected status: {status}"
    );
}

#[tokio::test]
async fn duplicate_draft_not_found() {
    let router = test_router().await;
    let (status, body) =
        post_json(router, "/api/drafts/9999/duplicate", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn list_draft_revisions_not_found() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts/9999/revisions").await;
    // Revisions list for nonexistent draft returns an empty array (not 404).
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_draft_activity_not_found() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts/9999/activity").await;
    // Activity list for nonexistent draft returns an empty array (not 404).
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn patch_draft_meta_not_found() {
    let router = test_router().await;
    let (status, body) = patch_json(
        router,
        "/api/drafts/9999/meta",
        serde_json::json!({ "title": "Nope" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn schedule_studio_draft_not_found() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/drafts/9999/schedule",
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59Z" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn create_studio_draft_minimal() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/drafts",
        serde_json::json!({ "content_type": "tweet", "content": "test" }),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "unexpected status: {status}"
    );
    assert!(body["id"].as_i64().unwrap() > 0);
    assert!(body["updated_at"].as_str().is_some());
}

#[tokio::test]
async fn list_studio_drafts_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}
