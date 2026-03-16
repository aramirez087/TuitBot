use super::*;

// ============================================================
// Approval happy-path mutations (Task 3.4)
// ============================================================

/// Build a router and pool with a real tempdir containing a dummy tokens.json
/// so that approve/reject routes pass the X-auth existence check.
/// Returns (router, pool, tempdir) — the caller must hold `tempdir` alive.
async fn router_with_pool_and_tokens() -> (
    axum::Router,
    tuitbot_core::storage::DbPool,
    tempfile::TempDir,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Write a dummy tokens.json for the default account so approve/reject
    // routes pass the `token_path.exists()` guard.
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
async fn approval_approve_item_succeeds() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Approve me",
        "General",
        "",
        0.9,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = post_json(
        router,
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({"actor": "dashboard"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "approved");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn approval_approved_item_removed_from_pending_list() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Pending item",
        "General",
        "",
        0.8,
        "[]",
    )
    .await
    .expect("enqueue");

    // Approve it.
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Pending list should now be empty.
    let (status, body) = get_json(router, "/api/approval?status=pending").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn approval_reject_item_succeeds() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "reply",
        "t1",
        "@user",
        "Reject me",
        "Rust",
        "",
        0.5,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = post_json(
        router,
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({"actor": "dashboard", "notes": "off-topic"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "rejected");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn approval_reject_sets_status_in_db() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Will be rejected",
        "Topic",
        "",
        0.3,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Confirm rejected items show up in the rejected list.
    let (status, body) = get_json(router, "/api/approval?status=rejected").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status"], "rejected");
}

#[tokio::test]
async fn approval_approve_all_clears_pending_queue() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    // Seed 3 pending items.
    for i in 0..3u32 {
        tuitbot_core::storage::approval_queue::enqueue(
            &pool,
            "tweet",
            "",
            "",
            &format!("Bulk item {i}"),
            "General",
            "",
            0.8,
            "[]",
        )
        .await
        .expect("enqueue");
    }

    let (status, body) = post_json(
        router.clone(),
        "/api/approval/approve-all",
        serde_json::json!({"max": 10}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 3, "approve-all should report count=3");
    assert!(body["ids"].is_array());

    // Pending queue should now be empty.
    let (status, body) = get_json(router, "/api/approval?status=pending").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn approval_approve_all_respects_max_limit() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    for i in 0..5u32 {
        tuitbot_core::storage::approval_queue::enqueue(
            &pool,
            "tweet",
            "",
            "",
            &format!("Item {i}"),
            "General",
            "",
            0.7,
            "[]",
        )
        .await
        .expect("enqueue");
    }

    let (status, body) = post_json(
        router,
        "/api/approval/approve-all",
        serde_json::json!({"max": 2}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["count"].as_u64().unwrap_or(0) <= 2,
        "should not approve more than max"
    );
}

#[tokio::test]
async fn approval_edit_history_empty_for_new_item() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Fresh item",
        "General",
        "",
        0.8,
        "[]",
    )
    .await
    .expect("enqueue");

    let (status, body) = get_json(router, &format!("/api/approval/{id}/history")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0, "no edits yet");
}

#[tokio::test]
async fn approval_edit_history_records_edits() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool,
        "tweet",
        "",
        "",
        "Original content",
        "General",
        "",
        0.8,
        "[]",
    )
    .await
    .expect("enqueue");

    // Edit the item.
    let (status, _) = patch_json(
        router.clone(),
        &format!("/api/approval/{id}"),
        serde_json::json!({"content": "Edited content", "editor": "tester"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // History should now have one entry.
    let (status, body) = get_json(router, &format!("/api/approval/{id}/history")).await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    // EditHistoryEntry fields: old_value holds the pre-edit content.
    assert_eq!(arr[0]["old_value"], "Original content");
    assert_eq!(arr[0]["new_value"], "Edited content");
    assert_eq!(arr[0]["field"], "generated_content");
}

// ---------------------------------------------------------------------------
// Helpers for idempotency / auth tests
// ---------------------------------------------------------------------------

/// Seed a single pending approval item into `pool` and return its ID.
async fn seed_pending_item(pool: &tuitbot_core::storage::DbPool) -> i64 {
    tuitbot_core::storage::approval_queue::enqueue(
        pool,
        "reply",
        "tweet_test_123",
        "@testauthor",
        "Test idempotency content",
        "General",
        "",
        75.0,
        "[]",
    )
    .await
    .expect("seed pending item")
}

// ---------------------------------------------------------------------------
// Auth guard tests (parity with discovery routes)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn approval_list_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/approval")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn approval_approve_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/approval/1/approve")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from("{}"))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Idempotency / state-guard tests (safety-critical)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn approval_approve_already_approved_returns_409() {
    // Guard: approve_item rejects re-approval with 409 Conflict.
    // Prevents double-post to X API via retry or race condition.
    let (router, pool, _dir) = router_with_pool_and_tokens().await;

    let id = seed_pending_item(&pool).await;

    // First approve — must succeed.
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "first approve must succeed");

    // Second approve on an already-approved item — must be 409.
    let (status2, body2) = post_json(
        router,
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "double-approve must return 409 Conflict, got {status2}: {body2}"
    );
}

#[tokio::test]
async fn approval_reject_already_rejected_returns_409() {
    // Guard: reject_item rejects re-rejection with 409 Conflict.
    let (router, pool, _dir) = router_with_pool_and_tokens().await;

    let id = seed_pending_item(&pool).await;

    // First reject — must succeed.
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "first reject must succeed");

    // Second reject on an already-rejected item — must be 409.
    let (status2, body2) = post_json(
        router,
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "double-reject must return 409 Conflict, got {status2}: {body2}"
    );
}

// ---------------------------------------------------------------------------
// Cross-state guard tests (approve-rejected, reject-approved, scheduled)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn approval_approve_already_rejected_returns_409() {
    // Guard: cannot approve an item that was already rejected.
    let (router, pool, _dir) = router_with_pool_and_tokens().await;

    let id = seed_pending_item(&pool).await;

    // Reject first.
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({"actor": "dashboard"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "first reject must succeed");

    // Now try to approve the rejected item — must be 409.
    let (status2, body2) = post_json(
        router,
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "approve-after-reject must return 409 Conflict, got {status2}: {body2}"
    );
}

#[tokio::test]
async fn approval_reject_already_approved_returns_409() {
    // Guard: cannot reject an item that was already approved.
    let (router, pool, _dir) = router_with_pool_and_tokens().await;

    let id = seed_pending_item(&pool).await;

    // Approve first.
    let (status, _) = post_json(
        router.clone(),
        &format!("/api/approval/{id}/approve"),
        serde_json::json!({"actor": "dashboard"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "first approve must succeed");

    // Now try to reject the approved item — must be 409.
    let (status2, body2) = post_json(
        router,
        &format!("/api/approval/{id}/reject"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status2,
        StatusCode::CONFLICT,
        "reject-after-approve must return 409 Conflict, got {status2}: {body2}"
    );
}
