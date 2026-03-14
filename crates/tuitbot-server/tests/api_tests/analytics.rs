use super::*;

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
async fn runtime_status_initially_stopped() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/runtime/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["running"], false);
}

#[tokio::test]
async fn runtime_start_and_stop() {
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
async fn post_ingest_inline_returns_200() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [{
                "relative_path": "notes/test.md",
                "body_text": "Some test content about Rust.",
                "title": "Test Note"
            }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ingested"], 1);
    assert_eq!(body["skipped"], 0);
    assert!(body["duration_ms"].is_number());
}

#[tokio::test]
async fn post_ingest_requires_auth() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/ingest")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({"inline_nodes": []})).unwrap(),
        ))
        .expect("build request");
    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_ingest_idempotent() {
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
    let router = tuitbot_server::build_router(state);

    let body = serde_json::json!({
        "inline_nodes": [{
            "relative_path": "notes/idea.md",
            "body_text": "Content that won't change."
        }]
    });

    // First ingest
    let (status, resp1) = post_json(router.clone(), "/api/ingest", body.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp1["ingested"], 1);

    // Second ingest — same content, same hash → skipped
    let (status, resp2) = post_json(router, "/api/ingest", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp2["ingested"], 0);
    assert_eq!(resp2["skipped"], 1);
}

#[tokio::test]
async fn post_ingest_empty_body() {
    let router = test_router().await;
    let (status, body) = post_json(router, "/api/ingest", serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ingested"], 0);
    assert_eq!(body["skipped"], 0);
}

#[tokio::test]
async fn post_ingest_empty_body_text_errors() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [{
                "relative_path": "notes/empty.md",
                "body_text": ""
            }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ingested"], 0);
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
}

// ============================================================
// Config status (unauthenticated, includes capabilities)
// ============================================================

#[tokio::test]
async fn config_status_includes_capabilities() {
    let router = test_router().await;
    // No auth header — this endpoint is public.
    let req = Request::builder()
        .uri("/api/settings/status")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.expect("read body");
    let body: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    // Must include the three expected fields.
    assert!(body["configured"].is_boolean());
    assert!(body["deployment_mode"].is_string());
    assert!(body["capabilities"].is_object());

    // Default mode is desktop.
    assert_eq!(body["deployment_mode"], "desktop");
    assert_eq!(body["capabilities"]["local_folder"], true);
    assert_eq!(body["capabilities"]["file_picker_native"], true);
    assert_eq!(body["capabilities"]["preferred_source_default"], "local_fs");
}

#[tokio::test]
async fn config_status_capabilities_match_cloud_mode() {
    use tuitbot_core::config::DeploymentMode;

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
        deployment_mode: DeploymentMode::Cloud,

        pending_oauth: Mutex::new(std::collections::HashMap::new()),
        token_managers: Mutex::new(std::collections::HashMap::new()),
        x_client_id: String::new(),
    });
    let router = tuitbot_server::build_router(state);

    let req = Request::builder()
        .uri("/api/settings/status")
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.expect("read body");
    let body: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    assert_eq!(body["deployment_mode"], "cloud");
    assert_eq!(body["capabilities"]["local_folder"], false);
    assert_eq!(body["capabilities"]["manual_local_path"], false);
    assert_eq!(body["capabilities"]["file_picker_native"], false);
    assert_eq!(body["capabilities"]["google_drive"], true);
    assert_eq!(
        body["capabilities"]["preferred_source_default"],
        "google_drive"
    );
}


#[tokio::test]
async fn load_effective_config_per_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[x_api]
provider_backend = "scraper"
client_id = "test-client-id"

[business]
product_name = "BaseProduct"
product_keywords = ["base"]
"#,
    )
    .expect("write config");

    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let state = AppState {
        db: pool.clone(),
        config_path,
        data_dir: dir.path().to_path_buf(),
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
    };

    // Default account: base config
    let config = state
        .load_effective_config(tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID)
        .await
        .unwrap();
    assert_eq!(config.x_api.provider_backend, "scraper");
    assert_eq!(config.business.product_name, "BaseProduct");

    // Create non-default account with overrides
    let acct_id = uuid::Uuid::new_v4().to_string();
    tuitbot_core::storage::accounts::create_account(&pool, &acct_id, "Override Test")
        .await
        .expect("create account");
    tuitbot_core::storage::accounts::update_account(
        &pool,
        &acct_id,
        tuitbot_core::storage::accounts::UpdateAccountParams {
            label: None,
            x_user_id: None,
            x_username: None,
            x_display_name: None,
            x_avatar_url: None,
            config_overrides: Some(
                r#"{"x_api": {"provider_backend": "x_api"}, "business": {"product_name": "OverriddenProduct"}}"#,
            ),
            token_path: None,
            status: None,
        },
    )
    .await
    .expect("update account");

    // Non-default account: merged config
    let config = state.load_effective_config(&acct_id).await.unwrap();
    assert_eq!(config.x_api.provider_backend, "x_api");
    assert_eq!(config.business.product_name, "OverriddenProduct");
    // Base keywords should still be inherited
    assert_eq!(config.x_api.client_id, "test-client-id");
}

// ============================================================
// OAuth unlink (Session 8)
// ============================================================

/// Test: unlinking OAuth tokens deletes the token file and updates status.

// ============================================================
// Analytics summary + recent-performance (Task 3.4)
// ============================================================

#[tokio::test]
async fn analytics_summary_returns_expected_shape() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/summary").await;
    assert_eq!(status, StatusCode::OK);
    // Shape check: must have followers, actions_today, engagement, top_topics.
    assert!(body["followers"].is_object(), "missing followers object");
    assert!(body["actions_today"].is_object(), "missing actions_today object");
    assert!(body["engagement"].is_object(), "missing engagement object");
    assert!(body["top_topics"].is_array(), "missing top_topics array");
    // Numeric follower fields.
    assert!(body["followers"]["current"].is_number());
    assert!(body["followers"]["change_7d"].is_number());
    assert!(body["followers"]["change_30d"].is_number());
}

#[tokio::test]
async fn analytics_summary_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/analytics/summary")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn analytics_recent_performance_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/recent-performance").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected JSON array");
}

#[tokio::test]
async fn analytics_recent_performance_honours_limit_param() {
    let router = test_router().await;
    // With no data in the DB the result should be empty regardless of limit.
    let (status, body) = get_json(router, "/api/analytics/recent-performance?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() <= 5);
}

#[tokio::test]
async fn analytics_followers_honours_days_param() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/followers?days=7").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn analytics_topics_honours_limit_param() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/topics?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() <= 5);
}
