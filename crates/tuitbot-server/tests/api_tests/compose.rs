use super::*;

#[tokio::test]
async fn content_tweets_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/tweets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn content_threads_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/threads").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn compose_tweet_validation() {
    let router = test_router().await;
    // Empty text should fail.
    let (status, _) = post_json(
        router,
        "/api/content/tweets",
        serde_json::json!({"text": ""}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_tweet_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        serde_json::json!({"text": "Hello world!"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["status"].as_str().unwrap() == "accepted"
            || body["status"].as_str().unwrap() == "queued_for_approval"
    );
}

#[tokio::test]
async fn compose_thread_validation() {
    let router = test_router().await;
    let (status, _) = post_json(
        router,
        "/api/content/threads",
        serde_json::json!({"tweets": []}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Targets (read + write)
// ============================================================

#[tokio::test]
async fn targets_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn add_and_list_target() {
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

    // Add a target.
    let (status, body) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "elonmusk"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "added");

    // List targets — should contain the new one.
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    let targets = body.as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["username"], "elonmusk");
}

#[tokio::test]
async fn add_duplicate_target_fails() {
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

    // First add succeeds.
    let (status, _) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "testtarget"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Second add should conflict.
    let (status, _) = post_json(
        router,
        "/api/targets",
        serde_json::json!({"username": "testtarget"}),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn remove_target_works() {
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

    // Add a target.
    let (status, _) = post_json(
        router.clone(),
        "/api/targets",
        serde_json::json!({"username": "removeme"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Remove it.
    let (status, body) = delete_json(router.clone(), "/api/targets/removeme").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "removed");

    // List should be empty.
    let (status, body) = get_json(router, "/api/targets").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn remove_nonexistent_target_fails() {
    let router = test_router().await;
    let (status, _) = delete_json(router, "/api/targets/nonexistent").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================
// Runtime
// ============================================================

#[tokio::test]
async fn settings_get_returns_json() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a minimal config file.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"TestBot\"\n").unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path,
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

    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["business"]["product_name"], "TestBot");
}

#[tokio::test]
async fn settings_patch_round_trips() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[business]\nproduct_name = \"OldName\"\n").unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path,
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

    // PATCH a field.
    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "business": {"product_name": "NewName"}
            }))
            .unwrap(),
        ))
        .expect("build request");

    let response = router
        .clone()
        .oneshot(patch_req)
        .await
        .expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    // GET and verify.
    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["business"]["product_name"], "NewName");
}

// ============================================================
// Ingest
// ============================================================

#[tokio::test]
async fn runtime_isolation_start_stop() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let acct_a = create_test_account(&pool, "Runtime A").await;
    let acct_b = create_test_account(&pool, "Runtime B").await;

    // Both start as not running
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_a).await;
    assert_eq!(body["running"], false);
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_b).await;
    assert_eq!(body["running"], false);

    // Start runtime for A
    let (status, body) = post_json_for(
        router.clone(),
        "/api/runtime/start",
        &acct_a,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "start A: {body}");

    // A is running, B is not
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_a).await;
    assert_eq!(body["running"], true, "A should be running");
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_b).await;
    assert_eq!(body["running"], false, "B should not be running");

    // Start runtime for B
    let (status, _) = post_json_for(
        router.clone(),
        "/api/runtime/start",
        &acct_b,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Both running
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_a).await;
    assert_eq!(body["running"], true, "A should still be running");
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_b).await;
    assert_eq!(body["running"], true, "B should be running now");

    // Stop A — B should remain running
    let (status, _) = post_json_for(
        router.clone(),
        "/api/runtime/stop",
        &acct_a,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_a).await;
    assert_eq!(body["running"], false, "A should be stopped");
    let (_, body) = get_json_for(router.clone(), "/api/runtime/status", &acct_b).await;
    assert_eq!(body["running"], true, "B should survive A's stop");
}
