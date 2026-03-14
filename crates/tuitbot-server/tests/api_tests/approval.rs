use super::*;

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
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
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
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
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
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
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
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool.clone(),
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


// ============================================================
// X Auth: Unlink
// ============================================================

#[tokio::test]
async fn x_auth_unlink_removes_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Unlink Test").await;

    // Write a mock token file.
    let token_path = tuitbot_core::storage::accounts::account_token_path(dir.path(), &acct);
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "test_access".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec!["tweet.read".to_string()],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("save tokens");
    assert!(token_path.exists(), "token file should exist before unlink");

    // Verify status shows linked.
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["oauth_linked"], true);

    // Unlink.
    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/tokens"),
        &acct,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "unlink: {body}");
    assert_eq!(body["deleted"], true);
    assert!(!token_path.exists(), "token file should be deleted");

    // Verify status shows unlinked.
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["oauth_linked"], false);
    assert_eq!(body["has_credentials"], false);
}

/// Test: unlinking when no tokens exist returns deleted: false (no error).
#[tokio::test]
async fn x_auth_unlink_no_tokens_returns_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Unlink Empty").await;

    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/tokens"),
        &acct,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], false);
}

/// Test: unlinking account A's OAuth does not affect account B's scraper session.
#[tokio::test]
async fn x_auth_unlink_cross_account_isolation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "Account A").await;
    let acct_b = create_test_account(&pool, "Account B").await;

    // Give A OAuth tokens.
    let token_path = tuitbot_core::storage::accounts::account_token_path(dir.path(), &acct_a);
    let tokens = tuitbot_core::x_api::auth::Tokens {
        access_token: "a_access".to_string(),
        refresh_token: "a_refresh".to_string(),
        expires_at: chrono::Utc::now() + chrono::TimeDelta::hours(2),
        scopes: vec![],
    };
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path).expect("save A tokens");

    // Give B a scraper session.
    let (status, _) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct_b,
        serde_json::json!({
            "auth_token": "b_auth",
            "ct0": "b_ct0",
            "username": "user_b"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Unlink A's OAuth.
    let (status, body) = delete_json_for(
        router.clone(),
        &format!("/api/accounts/{acct_a}/x-auth/tokens"),
        &acct_a,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], true);

    // A should have no credentials.
    let (_, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct_a}/x-auth/status"),
    )
    .await;
    assert_eq!(body["oauth_linked"], false);
    assert_eq!(body["has_credentials"], false);

    // B should still have scraper session.
    let (_, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct_b}/x-auth/status"),
    )
    .await;
    assert_eq!(body["scraper_linked"], true);
    assert_eq!(body["has_credentials"], true);
}

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
    tuitbot_core::x_api::auth::save_tokens(&tokens, &token_path)
        .expect("write dummy tokens.json");

    (router, pool, dir)
}

#[tokio::test]
async fn approval_approve_item_succeeds() {
    let (router, pool, _dir) = router_with_pool_and_tokens().await;
    let id = tuitbot_core::storage::approval_queue::enqueue(
        &pool, "tweet", "", "", "Approve me", "General", "", 0.9, "[]",
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
        &pool, "tweet", "", "", "Pending item", "General", "", 0.8, "[]",
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
        &pool, "reply", "t1", "@user", "Reject me", "Rust", "", 0.5, "[]",
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
        &pool, "tweet", "", "", "Will be rejected", "Topic", "", 0.3, "[]",
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
        &pool, "tweet", "", "", "Fresh item", "General", "", 0.8, "[]",
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
        &pool, "tweet", "", "", "Original content", "General", "", 0.8, "[]",
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
