use super::*;

#[tokio::test]
async fn connector_link_not_configured() {
    let router = test_router().await;
    // POST /link without connector config in TOML → 400 "not configured".
    let (status, body) = post_json(
        router,
        "/api/connectors/google-drive/link",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"].as_str().unwrap().contains("not configured")
            || body["error"].as_str().unwrap().contains("not set")
    );
}

#[tokio::test]
async fn connector_link_requires_auth() {
    let router = test_router().await;
    // POST /link without Bearer token → 401.
    let req = Request::builder()
        .method("POST")
        .uri("/api/connectors/google-drive/link")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn connector_callback_missing_params() {
    let router = test_router().await;
    // GET /callback without code/state → 400.
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connector_callback_invalid_state() {
    let router = test_router().await;
    // GET /callback with unknown state → 400.
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback?code=abc&state=unknown")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = response.into_body().collect().await.expect("read body");
    let body: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("invalid or expired"));
}

#[tokio::test]
async fn connector_callback_is_auth_exempt() {
    let router = test_router().await;
    // GET /callback without auth should return 400 (param validation), not 401.
    let req = Request::builder()
        .uri("/api/connectors/google-drive/callback")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    // Should be 400 (missing params), NOT 401 (unauthorized).
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connector_status_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/connectors/google-drive/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["connections"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn connector_status_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/connectors/google-drive/status")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn connector_status_with_connection() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Seed a connection row.
    tuitbot_core::storage::watchtower::insert_connection(
        &pool,
        "google_drive",
        Some("test@example.com"),
        Some("Test Drive"),
    )
    .await
    .expect("insert connection");

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
        scraper_health: None,
        watchtower_cancel: tokio::sync::RwLock::new(None),
        content_sources: tokio::sync::RwLock::new(Default::default()),
        connector_config: Default::default(),
        deployment_mode: Default::default(),

        pending_oauth: Mutex::new(std::collections::HashMap::new()),
        token_managers: Mutex::new(std::collections::HashMap::new()),
        x_client_id: String::new(),
        semantic_index: None,
        embedding_provider: None,
    });
    let router = tuitbot_server::build_router(state);

    let (status, body) = get_json(router, "/api/connectors/google-drive/status").await;
    assert_eq!(status, StatusCode::OK);

    let conns = body["connections"].as_array().unwrap();
    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0]["account_email"], "test@example.com");
    assert_eq!(conns[0]["connector_type"], "google_drive");
    // Must NOT contain encrypted_credentials.
    assert!(conns[0]["encrypted_credentials"].is_null());
}

#[tokio::test]
async fn connector_disconnect_not_found() {
    let router = test_router().await;
    let (status, body) = delete_json(router, "/api/connectors/google-drive/99999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn connector_disconnect_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/connectors/google-drive/1")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================
// Credential isolation (Session 3)
// ============================================================

/// Create a test router with a dedicated temp directory for data files.

#[tokio::test]
async fn credential_isolation_two_accounts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "Account A").await;
    let acct_b = create_test_account(&pool, "Account B").await;

    // Import scraper session for account A
    let (status, body) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct_a,
        serde_json::json!({
            "auth_token": "auth_a",
            "ct0": "ct0_a",
            "username": "user_a"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "import A: {body}");
    assert_eq!(body["username"], "user_a");

    // Import scraper session for account B
    let (status, body) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct_b,
        serde_json::json!({
            "auth_token": "auth_b",
            "ct0": "ct0_b",
            "username": "user_b"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "import B: {body}");
    assert_eq!(body["username"], "user_b");

    // Verify each account sees its own session
    let (status, body) =
        get_json_for(router.clone(), "/api/settings/scraper-session", &acct_a).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], true);
    assert_eq!(body["username"], "user_a");

    let (status, body) =
        get_json_for(router.clone(), "/api/settings/scraper-session", &acct_b).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], true);
    assert_eq!(body["username"], "user_b");

    // Delete account A's session — B should be unaffected
    let (status, _) =
        delete_json_for(router.clone(), "/api/settings/scraper-session", &acct_a).await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) =
        get_json_for(router.clone(), "/api/settings/scraper-session", &acct_a).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], false);

    let (status, body) =
        get_json_for(router.clone(), "/api/settings/scraper-session", &acct_b).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["exists"], true,
        "B's session should survive A's delete"
    );
    assert_eq!(body["username"], "user_b");
}

/// Test: can_post is isolated per account (one has credentials, one doesn't).
#[tokio::test]
async fn can_post_isolated_per_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_with = create_test_account(&pool, "Has Creds").await;
    let acct_without = create_test_account(&pool, "No Creds").await;

    // Import scraper session only for acct_with
    let (status, _) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct_with,
        serde_json::json!({
            "auth_token": "auth_test",
            "ct0": "ct0_test",
            "username": "test_user"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Check runtime status for acct_with — should have can_post = true
    let (status, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_with).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["can_post"], true,
        "account with session should can_post"
    );

    // Check runtime status for acct_without — should have can_post = false
    let (status, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_without).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["can_post"], false,
        "account without session should not can_post"
    );
}

/// Test: x-auth/status returns not linked for a fresh account.
#[tokio::test]
async fn x_auth_status_no_credentials() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Fresh Account").await;

    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["oauth_linked"], false);
    assert_eq!(body["scraper_linked"], false);
    assert_eq!(body["has_credentials"], false);
}

/// Test: x-auth/start returns an authorization URL and state.
#[tokio::test]
async fn x_auth_start_returns_auth_url() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Auth Start Test").await;

    let (status, body) = post_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/start"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "start: {body}");
    assert!(
        body["authorization_url"]
            .as_str()
            .unwrap()
            .contains("oauth2/authorize"),
        "should contain auth URL"
    );
    assert!(
        body["state"].as_str().is_some(),
        "should return state parameter"
    );
}

/// Test: x-auth/status reflects scraper session as well.
#[tokio::test]
async fn x_auth_status_reflects_scraper_session() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct = create_test_account(&pool, "Scraper Status Test").await;

    // No credentials yet
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["has_credentials"], false);

    // Import scraper session
    let (status, _) = post_json_for(
        router.clone(),
        "/api/settings/scraper-session",
        &acct,
        serde_json::json!({
            "auth_token": "auth_test",
            "ct0": "ct0_test",
            "username": "scraper_user"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Now x-auth/status should show scraper_linked
    let (status, body) = get_json(
        router.clone(),
        &format!("/api/accounts/{acct}/x-auth/status"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["scraper_linked"], true);
    assert_eq!(body["has_credentials"], true);
    assert_eq!(body["oauth_linked"], false);
}

// ============================================================
// Runtime isolation (Session 4)
// ============================================================

/// Test: runtime status returns per-account provider_backend from effective config.
#[tokio::test]
async fn runtime_status_per_account_provider_backend() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Default account gets provider_backend from config.toml ("scraper")
    let (status, body) = get_json_for(
        router.clone(),
        "/api/runtime/status",
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["provider_backend"], "scraper");

    // Create non-default account with x_api override
    let acct_b = create_test_account(&pool, "X API Account").await;
    tuitbot_core::storage::accounts::update_account(
        &pool,
        &acct_b,
        tuitbot_core::storage::accounts::UpdateAccountParams {
            label: None,
            x_user_id: None,
            x_username: None,
            x_display_name: None,
            x_avatar_url: None,
            config_overrides: Some(r#"{"x_api": {"provider_backend": "x_api"}}"#),
            token_path: None,
            status: None,
        },
    )
    .await
    .expect("update account");

    // Account B should see provider_backend = "x_api"
    let (status, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_b).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["provider_backend"], "x_api",
        "non-default account should see overridden provider_backend"
    );

    // Default account should still see "scraper"
    let (status, body) = get_json_for(
        router.clone(),
        "/api/runtime/status",
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["provider_backend"], "scraper",
        "default account should be unaffected by non-default override"
    );
}
