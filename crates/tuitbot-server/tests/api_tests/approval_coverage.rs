//! Additional coverage tests for approval queue edge cases and error paths.

use super::*;

// ============================================================
// Empty-state and pagination tests
// ============================================================

#[tokio::test]
async fn approval_list_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn approval_list_with_pagination() {
    let router = test_router().await;
    // With query params on an empty DB — should still return a valid array.
    let (status, body) = get_json(router, "/api/approval?status=pending").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn approval_stats_empty_db() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pending"], 0);
    assert_eq!(body["approved"], 0);
    assert_eq!(body["rejected"], 0);
}

// ============================================================
// Not-found error paths (supplement existing tests)
// ============================================================

#[tokio::test]
async fn approval_approve_not_found_with_body() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/approval/9999/approve",
        serde_json::json!({ "actor": "tester", "notes": "nope" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn approval_reject_not_found_with_body() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/approval/9999/reject",
        serde_json::json!({ "actor": "tester", "notes": "nah" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn approval_edit_not_found_with_content() {
    let router = test_router().await;
    let (status, body) = patch_json(
        router,
        "/api/approval/9999",
        serde_json::json!({ "content": "Does not exist" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

// ============================================================
// Batch approve on empty queue
// ============================================================

#[tokio::test]
async fn approval_approve_all_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Write dummy tokens so the X-auth guard passes.
    let token_path = tuitbot_core::storage::accounts::account_token_path(
        dir.path(),
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
    );
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("write dummy tokens.json");

    let (status, body) =
        post_json(router, "/api/approval/approve-all", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 0);
    assert!(body["ids"].as_array().unwrap().is_empty());
}

// ============================================================
// Export endpoint coverage
// ============================================================

#[tokio::test]
async fn approval_export_csv_empty() {
    let router = test_router().await;

    let req = axum::http::Request::builder()
        .uri("/api/approval/export?format=csv")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(axum::body::Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);
    let ct = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("text/csv"));
}

#[tokio::test]
async fn approval_export_json_empty() {
    let router = test_router().await;

    let req = axum::http::Request::builder()
        .uri("/api/approval/export?format=json")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(axum::body::Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);
    let ct = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("application/json"));
}

// ============================================================
// Edit history on nonexistent item
// ============================================================

#[tokio::test]
async fn approval_edit_history_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval/9999/history").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Status guard: approve-after-scheduled (cross-state, P0 safety)
// ---------------------------------------------------------------------------

/// Build a router backed by a real tempdir with dummy tokens.json.
async fn router_with_tokens_and_pool() -> (
    axum::Router,
    tuitbot_core::storage::DbPool,
    tempfile::TempDir,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let token_path = tuitbot_core::storage::accounts::account_token_path(
        dir.path(),
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
    );
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("write dummy tokens.json");

    (router, pool, dir)
}

#[tokio::test]
async fn approval_approve_scheduled_item_returns_409() {
    // Guard: cannot re-approve an item whose status is "scheduled".
    // Prevents duplicate scheduled_content rows from double-approval.
    let (router, pool, _dir) = router_with_tokens_and_pool().await;

    let future = (chrono::Utc::now() + chrono::TimeDelta::hours(24))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // Seed item with a future scheduled_for so approve_item bridges to "scheduled".
    let id = tuitbot_core::storage::approval_queue::enqueue_with_context_for(
        &pool,
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
        "tweet",
        "",
        "",
        "Scheduled content",
        "General",
        "",
        0.9,
        "[]",
        None,
        None,
        Some(&future),
    )
    .await
    .expect("enqueue scheduled item");

    // First approve — bridges item to "scheduled" status.
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "first approve must succeed: {body}");
    assert_eq!(body["status"], "scheduled");

    // Second approve on a scheduled item — must be 409.
    let (status2, body2) = post_json(
        router,
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "approve-after-scheduled must return 409 Conflict, got {status2}: {body2}"
    );
}
