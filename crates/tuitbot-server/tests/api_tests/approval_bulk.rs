//! Tests for F3: bulk approve/reject and account-filter on approval list.

use super::*;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Seed N pending items and return their IDs.
async fn seed_pending(pool: &tuitbot_core::storage::DbPool, n: u32) -> Vec<i64> {
    let mut ids = Vec::with_capacity(n as usize);
    for i in 0..n {
        let id = tuitbot_core::storage::approval_queue::enqueue(
            pool,
            "tweet",
            "",
            "",
            &format!("Bulk test item {i}"),
            "General",
            "",
            0.8,
            "[]",
        )
        .await
        .expect("enqueue");
        ids.push(id);
    }
    ids
}

// ─── Bulk approve ────────────────────────────────────────────────────────────

#[tokio::test]
async fn bulk_approve_happy_path() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    let ids = seed_pending(&pool, 3).await;

    let (status, body) = post_json(
        router.clone(),
        "/api/approval/bulk/approve",
        serde_json::json!({ "ids": ids }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "bulk approve should return 200");
    assert_eq!(body["approved"], 3, "all 3 should be approved");
    assert_eq!(body["failed"], 0, "no failures expected");

    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for r in results {
        assert!(r["ok"].as_bool().unwrap(), "each item should be ok");
    }

    // Verify queue is now empty (all moved out of pending).
    let (s, pending) = get_json(router, "/api/approval?status=pending").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        pending.as_array().unwrap().len(),
        0,
        "pending queue must be empty after bulk approve"
    );
}

#[tokio::test]
async fn bulk_approve_partial_failure_already_approved() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    let ids = seed_pending(&pool, 2).await;

    // Approve the first item directly so it's no longer pending.
    tuitbot_core::storage::approval_queue::update_status_with_review_for(
        &pool,
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
        ids[0],
        "approved",
        &Default::default(),
    )
    .await
    .expect("pre-approve");

    let (status, body) = post_json(
        router,
        "/api/approval/bulk/approve",
        serde_json::json!({ "ids": ids }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // ids[0] was already approved → should fail; ids[1] is still pending → should succeed.
    assert_eq!(body["approved"], 1, "only 1 should succeed");
    assert_eq!(body["failed"], 1, "1 should fail (already approved)");

    let results = body["results"].as_array().unwrap();
    let r0 = results.iter().find(|r| r["id"] == ids[0]).unwrap();
    let r1 = results.iter().find(|r| r["id"] == ids[1]).unwrap();
    assert!(
        !r0["ok"].as_bool().unwrap(),
        "pre-approved item should fail"
    );
    assert!(r1["ok"].as_bool().unwrap(), "pending item should succeed");
    assert!(
        r0["error"].as_str().unwrap().contains("pending"),
        "error should mention expected status"
    );
}

#[tokio::test]
async fn bulk_approve_not_found_reported_per_item() {
    let (router, _pool, _dir) = approval_router_with_tokens().await;

    let (status, body) = post_json(
        router,
        "/api/approval/bulk/approve",
        serde_json::json!({ "ids": [999_999_i64, 888_888_i64] }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["approved"], 0);
    assert_eq!(body["failed"], 2);
    for r in body["results"].as_array().unwrap() {
        assert!(!r["ok"].as_bool().unwrap());
        assert!(r["error"].as_str().unwrap().contains("not found"));
    }
}

#[tokio::test]
async fn bulk_approve_empty_ids_is_noop() {
    let (router, _pool, _dir) = approval_router_with_tokens().await;

    let (status, body) = post_json(
        router,
        "/api/approval/bulk/approve",
        serde_json::json!({ "ids": [] }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["approved"], 0);
    assert_eq!(body["failed"], 0);
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn bulk_approve_requires_tokens() {
    // Use a router WITHOUT tokens — should get 400.
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/approval/bulk/approve",
        serde_json::json!({ "ids": [1_i64] }),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "should require X auth tokens"
    );
    assert!(
        body["error"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains("auth")
            || body["error"]
                .as_str()
                .unwrap_or("")
                .to_lowercase()
                .contains("authenticated"),
        "error should mention authentication: {:?}",
        body
    );
}

// ─── Bulk reject ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn bulk_reject_happy_path() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    let ids = seed_pending(&pool, 3).await;

    let (status, body) = post_json(
        router.clone(),
        "/api/approval/bulk/reject",
        serde_json::json!({ "ids": ids }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["rejected"], 3);
    assert_eq!(body["failed"], 0);

    // Verify items are now rejected.
    let (s, rejected) = get_json(router, "/api/approval?status=rejected").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        rejected.as_array().unwrap().len(),
        3,
        "all 3 should be in rejected queue"
    );
}

#[tokio::test]
async fn bulk_reject_partial_failure_already_rejected() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    let ids = seed_pending(&pool, 2).await;

    // Pre-reject the first item.
    tuitbot_core::storage::approval_queue::update_status_with_review_for(
        &pool,
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
        ids[0],
        "rejected",
        &Default::default(),
    )
    .await
    .expect("pre-reject");

    let (status, body) = post_json(
        router,
        "/api/approval/bulk/reject",
        serde_json::json!({ "ids": ids }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["rejected"], 1);
    assert_eq!(body["failed"], 1);

    let results = body["results"].as_array().unwrap();
    let r0 = results.iter().find(|r| r["id"] == ids[0]).unwrap();
    let r1 = results.iter().find(|r| r["id"] == ids[1]).unwrap();
    assert!(!r0["ok"].as_bool().unwrap());
    assert!(r1["ok"].as_bool().unwrap());
}

#[tokio::test]
async fn bulk_reject_not_found_reported_per_item() {
    let (router, _pool, _dir) = approval_router_with_tokens().await;

    let (status, body) = post_json(
        router,
        "/api/approval/bulk/reject",
        serde_json::json!({ "ids": [777_777_i64] }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["rejected"], 0);
    assert_eq!(body["failed"], 1);
    assert!(body["results"][0]["error"]
        .as_str()
        .unwrap()
        .contains("not found"));
}

// ─── Account filter on list ──────────────────────────────────────────────────

#[tokio::test]
async fn approval_list_account_id_param_matches_context() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    seed_pending(&pool, 2).await;

    // account_id matching the default account should return items.
    let url = format!(
        "/api/approval?status=pending&account_id={}",
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID
    );
    let (status, body) = get_json(router, &url).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn approval_list_account_id_param_mismatch_falls_back_to_context() {
    let (router, pool, _dir) = approval_router_with_tokens().await;
    seed_pending(&pool, 2).await;

    // Mismatched account_id should silently use the auth context account.
    let url = "/api/approval?status=pending&account_id=00000000-dead-beef-cafe-000000000001";
    let (status, body) = get_json(router, url).await;
    // Should still return 200 with the default account's items (not the fake ID's empty set).
    assert_eq!(status, StatusCode::OK);
    // The items belong to the auth context account, so they should still be returned.
    assert_eq!(body.as_array().unwrap().len(), 2);
}

// ─── Helper: router with token dir (re-uses approval.rs pattern) ─────────────

async fn approval_router_with_tokens() -> (
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
