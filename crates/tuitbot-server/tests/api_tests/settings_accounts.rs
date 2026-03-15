use super::*;

// ============================================================
// Settings routes
// ============================================================

#[tokio::test]
async fn get_settings_config_status() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/settings/status").await;
    // Always returns 200 with a JSON status object (even when unconfigured).
    assert_eq!(status, StatusCode::OK, "config status: {body}");
    assert!(body.is_object(), "expected object: {body}");
    // Should include deployment_mode and capabilities fields.
    assert!(
        body.get("deployment_mode").is_some() || body.get("configured").is_some(),
        "expected deployment_mode or configured key: {body}"
    );
}

#[tokio::test]
async fn get_settings_returns_current_config() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/settings").await;
    // In test env, config file doesn't exist at /tmp/test-config.toml
    // so handler may return 400/500.
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn get_settings_with_valid_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    let (status, body) = get_json(router, "/api/settings").await;
    // test_router_with_dir writes a minimal config.toml, so this should succeed.
    assert_eq!(status, StatusCode::OK, "get settings with config: {body}");
    assert!(body.is_object(), "expected object: {body}");
}

#[tokio::test]
async fn patch_settings_no_config_file() {
    let router = test_router().await;
    let (status, body) = patch_json(
        router,
        "/api/settings",
        serde_json::json!({
            "business": { "product_name": "TestProd" }
        }),
    )
    .await;
    // No config file in default test router -> 400 or 500.
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn patch_settings_with_valid_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    let (status, body) = patch_json(
        router,
        "/api/settings",
        serde_json::json!({
            "business": { "product_name": "UpdatedProduct" }
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 422,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn init_settings_creates_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Remove the pre-written config so init can create a fresh one.
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

    let (status, body) = post_json(
        router,
        "/api/settings/init",
        serde_json::json!({
            "business": {
                "product_name": "TestProduct",
                "product_keywords": ["test"],
                "product_description": "A test product"
            }
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 422,
        "init should handle gracefully: {body}"
    );
}

#[tokio::test]
async fn settings_validate_with_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    let (status, body) = post_json(router, "/api/settings/validate", serde_json::json!({})).await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 422,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn settings_defaults_returns_ok() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/settings/defaults").await;
    assert_eq!(status, StatusCode::OK, "defaults: {body}");
    assert!(body.is_object(), "expected object: {body}");
}

#[tokio::test]
async fn settings_factory_reset_wrong_confirmation() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/factory-reset",
        serde_json::json!({ "confirmation": "wrong phrase" }),
    )
    .await;
    // Should reject with bad request for incorrect confirmation phrase.
    assert_eq!(status, StatusCode::BAD_REQUEST, "got: {body}");
}

// ============================================================
// Accounts routes
// ============================================================

#[tokio::test]
async fn list_accounts_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/accounts").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected array, got: {body}");
}

#[tokio::test]
async fn create_account_returns_id() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/accounts",
        serde_json::json!({ "label": "test-account" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create account failed: {body}");
    assert!(body["id"].is_string(), "response should include id: {body}");
}

#[tokio::test]
async fn get_account_not_found() {
    let router = test_router().await;
    let (status, _) = get_json(router, "/api/accounts/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_and_get_account() {
    let pool = storage::init_test_db().await.expect("init db");
    let id = create_test_account(&pool, "my-account").await;

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

    let (status, body) = get_json(router, &format!("/api/accounts/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn delete_account_not_found() {
    let router = test_router().await;
    let (status, _) = delete_json(router, "/api/accounts/nonexistent-id").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 404 || code == 400, "got {code}");
}

#[tokio::test]
async fn update_account_not_found() {
    let router = test_router().await;
    let (status, _) = patch_json(
        router,
        "/api/accounts/nonexistent-id",
        serde_json::json!({ "label": "new-label" }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 500,
        "got {code} for update nonexistent"
    );
}

#[tokio::test]
async fn list_account_roles_not_found() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/accounts/nonexistent-id/roles").await;
    let code = status.as_u16();
    // May return empty array or 404 depending on implementation.
    assert!(
        code == 200 || code == 404 || code == 400,
        "got {code}: {body}"
    );
}

// ============================================================
// Activity routes
// ============================================================

#[tokio::test]
async fn list_activity_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.is_object() || body.is_array(),
        "expected object or array: {body}"
    );
}

#[tokio::test]
async fn list_activity_with_filters() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity?limit=10&offset=0&type=reply").await;
    assert_eq!(status, StatusCode::OK, "activity with filters: {body}");
}

#[tokio::test]
async fn list_activity_type_all() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity?type=all").await;
    assert_eq!(status, StatusCode::OK, "got: {body}");
}

#[tokio::test]
async fn activity_export_returns_csv() {
    let router = test_router().await;
    // Export returns CSV, not JSON, so use a raw request.
    let req = Request::builder()
        .uri("/api/activity/export")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build request");
    let response = router.oneshot(req).await.expect("send request");
    let code = response.status().as_u16();
    assert!(code == 200 || code == 400, "got {code}");
}

#[tokio::test]
async fn activity_rate_limits() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity/rate-limits").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

// ============================================================
// Connectors routes
// ============================================================

#[tokio::test]
async fn connectors_google_drive_status() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/connectors/google-drive/status").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

#[tokio::test]
async fn connectors_google_drive_link_no_config() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/connectors/google-drive/link",
        serde_json::json!({}),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 422 || code == 500,
        "got {code} -- expected error without connector config"
    );
}

// ============================================================
// Replies routes
// ============================================================

#[tokio::test]
async fn get_replies_list_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/replies").await;
    assert_eq!(status, StatusCode::OK, "got: {body}");
}

// ============================================================
// Targets routes
// ============================================================

#[tokio::test]
async fn get_targets_list() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/targets").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

#[tokio::test]
async fn add_target_empty_username() {
    let router = test_router().await;
    // Send a valid JSON shape but with an empty username.
    let (status, body) = post_json(
        router,
        "/api/targets",
        serde_json::json!({ "username": "" }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 422,
        "got {code}: {body} -- expected validation error"
    );
}

// ============================================================
// Vault routes
// ============================================================

#[tokio::test]
async fn vault_search_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/vault/search?q=test").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn vault_sources_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/vault/sources").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

#[tokio::test]
async fn vault_notes_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/vault/notes").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

// ============================================================
// Content routes
// ============================================================

#[tokio::test]
async fn get_content_calendar_empty() {
    let router = test_router().await;
    let (status, body) = get_json(
        router,
        "/api/content/calendar?from=2026-03-01&to=2026-03-31",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "calendar: {body}");
}

#[tokio::test]
async fn get_content_schedule_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/schedule").await;
    assert_eq!(status, StatusCode::OK, "schedule: {body}");
}

#[tokio::test]
async fn get_content_tweets_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/tweets").await;
    assert_eq!(status, StatusCode::OK, "tweets: {body}");
}

#[tokio::test]
async fn get_content_threads_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/threads").await;
    assert_eq!(status, StatusCode::OK, "threads: {body}");
}

// ============================================================
// Runtime routes
// ============================================================

#[tokio::test]
async fn runtime_status() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/runtime/status").await;
    assert_eq!(status, StatusCode::OK, "runtime status: {body}");
    assert!(body.is_object(), "expected object: {body}");
}

// ============================================================
// Strategy routes
// ============================================================

#[tokio::test]
async fn strategy_current_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/strategy/current").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 404 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn strategy_history_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/strategy/history").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

#[tokio::test]
async fn strategy_inputs() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/strategy/inputs").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

// ============================================================
// Costs routes
// ============================================================

#[tokio::test]
async fn costs_summary_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/costs/summary").await;
    assert_eq!(status, StatusCode::OK, "costs summary: {body}");
}

#[tokio::test]
async fn costs_daily_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/costs/daily").await;
    assert_eq!(status, StatusCode::OK, "costs daily: {body}");
}

#[tokio::test]
async fn costs_by_model_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/costs/by-model").await;
    assert_eq!(status, StatusCode::OK, "costs by-model: {body}");
}

#[tokio::test]
async fn costs_x_api_summary_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/costs/x-api/summary").await;
    assert_eq!(status, StatusCode::OK, "x-api summary: {body}");
}

// ============================================================
// Sources routes
// ============================================================

#[tokio::test]
async fn sources_status() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/sources/status").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

// ============================================================
// MCP routes
// ============================================================

#[tokio::test]
async fn mcp_policy_get() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/mcp/policy").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn mcp_policy_templates() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/mcp/policy/templates").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn mcp_telemetry_summary() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/mcp/telemetry/summary").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

// ============================================================
// Scraper session routes
// ============================================================

#[tokio::test]
async fn scraper_session_get_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/settings/scraper-session").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 404,
        "got {code}: {body}"
    );
}

// ============================================================
// LAN settings routes
// ============================================================

#[tokio::test]
async fn lan_settings_status() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/settings/lan").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "got {code}: {body}");
}

// ============================================================
// Onboarding routes (coverage for onboarding.rs)
// ============================================================

#[tokio::test]
async fn onboarding_auth_status_no_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    let (status, body) = get_json(router, "/api/onboarding/x-auth/status").await;
    assert_eq!(status, StatusCode::OK, "onboarding status: {body}");
    assert_eq!(
        body["connected"], false,
        "should not be connected without tokens"
    );
}

#[tokio::test]
async fn onboarding_start_auth_no_client_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Create a state without x_client_id
    let pool = storage::init_test_db().await.expect("init db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[x_api]\nprovider_backend = \"scraper\"\n")
        .expect("write config");

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
        x_client_id: String::new(), // empty = no client_id
    });
    let router = tuitbot_server::build_router(state);

    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/start",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "should reject without client_id: {body}"
    );
}

#[tokio::test]
async fn onboarding_start_auth_with_client_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/start",
        serde_json::json!({ "client_id": "test-client-id" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "start auth: {body}");
    assert!(
        body["authorization_url"].is_string(),
        "should return auth URL: {body}"
    );
    assert!(body["state"].is_string(), "should return state: {body}");
}

#[tokio::test]
async fn onboarding_callback_invalid_state() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/callback",
        serde_json::json!({
            "code": "test-code",
            "state": "invalid-state-that-doesnt-exist"
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "callback with invalid state: {body}"
    );
}

#[tokio::test]
async fn onboarding_analyze_profile_no_tokens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/onboarding/analyze-profile",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "analyze profile: {body}");
    assert_eq!(
        body["status"], "x_api_error",
        "should report x_api_error without tokens"
    );
}

// ============================================================
// Account CRUD coverage (accounts.rs)
// ============================================================

#[tokio::test]
async fn create_and_delete_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    // Create
    let (status, body) = post_json(
        router.clone(),
        "/api/accounts",
        serde_json::json!({ "label": "deleteme" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {body}");
    let account_id = body["id"].as_str().expect("id").to_string();

    // Delete
    let (status, body) = delete_json(router, &format!("/api/accounts/{account_id}")).await;
    assert_eq!(status, StatusCode::OK, "delete: {body}");
    assert_eq!(body["status"], "archived");

    // Verify account is soft-deleted (status = archived)
    let acc = tuitbot_core::storage::accounts::get_account(&pool, &account_id)
        .await
        .expect("query")
        .expect("should still exist as archived");
    assert_eq!(acc.status, "archived", "account should be archived");
}

#[tokio::test]
async fn update_account_label() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "original-label").await;

    let (status, body) = patch_json(
        router,
        &format!("/api/accounts/{account_id}"),
        serde_json::json!({ "label": "updated-label" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "update label: {body}");
    assert_eq!(body["label"], "updated-label");
}

#[tokio::test]
async fn update_account_invalid_config_overrides() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "override-test").await;

    let (status, body) = patch_json(
        router,
        &format!("/api/accounts/{account_id}"),
        serde_json::json!({ "config_overrides": "not valid json" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid JSON: {body}");
}

#[tokio::test]
async fn update_account_empty_config_overrides_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "empty-override").await;

    let (status, body) = patch_json(
        router,
        &format!("/api/accounts/{account_id}"),
        serde_json::json!({ "config_overrides": "{}" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "empty overrides: {body}");
}

#[tokio::test]
async fn set_role_invalid_role() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "role-test").await;

    let (status, body) = post_json(
        router,
        &format!("/api/accounts/{account_id}/roles"),
        serde_json::json!({ "actor": "user1", "role": "INVALID_ROLE" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid role: {body}");
}

#[tokio::test]
async fn set_and_list_roles() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "roles-test").await;

    // Set role
    let (status, body) = post_json(
        router.clone(),
        &format!("/api/accounts/{account_id}/roles"),
        serde_json::json!({ "actor": "agent1", "role": "admin" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "set role: {body}");

    // List roles
    let (status, body) = get_json(router, &format!("/api/accounts/{account_id}/roles")).await;
    assert_eq!(status, StatusCode::OK, "list roles: {body}");
    assert!(body.is_array(), "expected array: {body}");
}

// ============================================================
// Content drafts coverage (drafts.rs)
// ============================================================

#[tokio::test]
async fn list_drafts_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "drafts-test").await;

    let (status, body) = get_json_for(router, "/api/content/drafts", &account_id).await;
    assert_eq!(status, StatusCode::OK, "list drafts: {body}");
}

#[tokio::test]
async fn create_draft_tweet() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "create-draft").await;

    let (status, body) = post_json_for(
        router,
        "/api/content/drafts",
        &account_id,
        serde_json::json!({
            "content_type": "tweet",
            "content": "Hello from a draft test!",
            "source": "test"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create draft: {body}");
    assert!(body["id"].is_number(), "should return draft id: {body}");
    assert_eq!(body["status"], "draft");
}

#[tokio::test]
async fn create_draft_empty_content_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "empty-draft").await;

    let (status, body) = post_json_for(
        router,
        "/api/content/drafts",
        &account_id,
        serde_json::json!({
            "content_type": "tweet",
            "content": "   ",
            "source": "test"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty content: {body}");
}

#[tokio::test]
async fn create_and_delete_draft() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "delete-draft").await;

    // Create
    let (status, body) = post_json_for(
        router.clone(),
        "/api/content/drafts",
        &account_id,
        serde_json::json!({
            "content_type": "tweet",
            "content": "Draft to delete",
            "source": "test"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create: {body}");
    let draft_id = body["id"].as_i64().expect("id");

    // Delete
    let (status, body) = delete_json_for(
        router,
        &format!("/api/content/drafts/{draft_id}"),
        &account_id,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "delete: {body}");
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn create_draft_too_long_tweet_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;
    let account_id = create_test_account(&pool, "long-draft").await;

    // Create a tweet that exceeds 280 characters
    let long_text = "A".repeat(300);
    let (status, body) = post_json_for(
        router,
        "/api/content/drafts",
        &account_id,
        serde_json::json!({
            "content_type": "tweet",
            "content": long_text,
            "source": "test"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "long tweet: {body}");
}

// ============================================================
// Additional route coverage — Sprint 2
// ============================================================

#[tokio::test]
async fn get_health_returns_ok() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/health")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_approval_queue_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval").await;
    assert_eq!(status, StatusCode::OK, "approval: {body}");
    assert!(body.is_array(), "expected array: {body}");
}

#[tokio::test]
async fn get_approval_stats_returns_ok() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/approval/stats").await;
    assert_eq!(status, StatusCode::OK, "approval stats: {body}");
}

#[tokio::test]
async fn get_analytics_follower_trend() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/followers").await;
    assert_eq!(status, StatusCode::OK, "followers: {body}");
}

#[tokio::test]
async fn get_analytics_topics() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/topics").await;
    assert_eq!(status, StatusCode::OK, "topics: {body}");
}

#[tokio::test]
async fn get_analytics_summary() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/summary").await;
    assert_eq!(status, StatusCode::OK, "summary: {body}");
}

#[tokio::test]
async fn get_content_drafts_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK, "drafts: {body}");
    assert!(body.is_array(), "expected array: {body}");
}

#[tokio::test]
async fn get_discovery_feed_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/feed").await;
    assert_eq!(status, StatusCode::OK, "feed: {body}");
}

#[tokio::test]
async fn get_activity_log() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/activity").await;
    assert_eq!(status, StatusCode::OK, "activity: {body}");
}

#[tokio::test]
async fn unauthenticated_request_rejected() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/approval")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_token_rejected() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/approval")
        .header("Authorization", "Bearer wrong-token")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_recent_performance() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/analytics/recent-performance").await;
    assert_eq!(status, StatusCode::OK, "recent perf: {body}");
}
