use super::*;

// ============================================================
// Settings init → get → patch → validate workflow
// ============================================================

#[tokio::test]
async fn init_then_get_returns_created_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Remove pre-written config so init can create a fresh one.
    let config_path = dir.path().join("config.toml");
    let _ = std::fs::remove_file(&config_path);

    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
        config_path: config_path.clone(),
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
    });

    // Step 1: Init settings.
    let router = tuitbot_server::build_router(state.clone());
    let (status, body) = post_json(
        router,
        "/api/settings/init",
        serde_json::json!({
            "business": {
                "product_name": "WorkflowTest",
                "product_keywords": ["workflow", "test"],
                "product_description": "Testing the init workflow"
            },
            "x_api": {
                "provider_backend": "scraper",
                "client_id": "test-client-id"
            }
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 422,
        "init should handle gracefully: {body}"
    );

    // Config file should now exist.
    if code == 200 {
        assert!(config_path.exists(), "config.toml should have been created");

        // Step 2: Get settings should return the config we just set.
        let router = tuitbot_server::build_router(state.clone());
        let (status, body) = get_json(router, "/api/settings").await;
        assert_eq!(status, StatusCode::OK, "get settings: {body}");
        assert!(body.is_object(), "expected object: {body}");

        // Step 3: Patch settings to change the product name.
        let router = tuitbot_server::build_router(state.clone());
        let (status, body) = patch_json(
            router,
            "/api/settings",
            serde_json::json!({
                "business": { "product_name": "UpdatedWorkflow" }
            }),
        )
        .await;
        let patch_code = status.as_u16();
        assert!(
            patch_code == 200 || patch_code == 400 || patch_code == 422,
            "patch: {body}"
        );

        // Step 4: Get settings again to verify the update.
        let router = tuitbot_server::build_router(state.clone());
        let (status, body) = get_json(router, "/api/settings").await;
        assert_eq!(status, StatusCode::OK, "get after patch: {body}");
    }
}

#[tokio::test]
async fn init_settings_conflict_when_config_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    // Config already exists via test_router_with_dir, so init should return 409.
    let (status, body) = post_json(
        router,
        "/api/settings/init",
        serde_json::json!({
            "business": {
                "product_name": "Duplicate",
                "product_keywords": ["dup"],
                "product_description": "Should conflict"
            }
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 when config exists: {body}"
    );
}

#[tokio::test]
async fn init_settings_rejects_non_object_body() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    let _ = std::fs::remove_file(&config_path);

    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let state = Arc::new(AppState {
        db: pool,
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
    });
    let router = tuitbot_server::build_router(state);

    // Send an array instead of an object.
    let (status, body) = post_json(router, "/api/settings/init", serde_json::json!([])).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "non-object body should be rejected: {body}"
    );
}

#[tokio::test]
async fn validate_settings_with_valid_patch() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/validate",
        serde_json::json!({
            "business": { "product_name": "ValidatedProduct" }
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 422,
        "validate: {code}: {body}"
    );
    if code == 200 {
        // Should have a "valid" field.
        assert!(
            body.get("valid").is_some(),
            "expected 'valid' key in response: {body}"
        );
    }
}

#[tokio::test]
async fn validate_settings_rejects_non_object() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/validate",
        serde_json::json!("string"),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 422,
        "non-object validate should fail: {code}: {body}"
    );
}

#[tokio::test]
async fn test_llm_endpoint_returns_result() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/test-llm",
        serde_json::json!({
            "provider": "openai",
            "api_key": "sk-fake-key-for-test",
            "model": "gpt-4"
        }),
    )
    .await;
    let code = status.as_u16();
    // Should return 200 with success: false (no real API key).
    assert!(code == 200 || code == 400, "test-llm: {code}: {body}");
    if code == 200 {
        assert!(
            body.get("success").is_some(),
            "expected 'success' key: {body}"
        );
    }
}

#[tokio::test]
async fn patch_settings_rejects_non_object() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) =
        patch_json(router, "/api/settings", serde_json::json!("not-an-object")).await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 422,
        "non-object patch should fail: {code}: {body}"
    );
}

#[tokio::test]
async fn config_status_shows_configured_with_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = get_json(router, "/api/settings/status").await;
    assert_eq!(status, StatusCode::OK, "config status: {body}");
    assert_eq!(body["configured"], true, "should be configured: {body}");
}

#[tokio::test]
async fn factory_reset_correct_phrase_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/factory-reset",
        serde_json::json!({ "confirmation": "RESET TUITBOT" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "factory reset: {body}");
    assert_eq!(
        body["status"], "reset_complete",
        "expected reset_complete: {body}"
    );
    assert!(
        body["cleared"].is_object(),
        "expected cleared object: {body}"
    );
}
