use super::*;

#[tokio::test]
async fn settings_get_redacts_service_account_key() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a config file with a service_account_key.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[business]
product_name = "TestBot"

[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/secret/path/sa-key.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#,
    )
    .unwrap();

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

    // The service_account_key must be redacted.
    let source = &body["content_sources"]["sources"][0];
    assert_eq!(source["service_account_key"], "[redacted]");
    // Other fields remain intact.
    assert_eq!(source["folder_id"], "abc123");
    assert_eq!(source["source_type"], "google_drive");
}

// --- Session 06: backward-compatibility regression tests ---

#[tokio::test]
async fn settings_patch_preserves_legacy_sa_key() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a config with a legacy service_account_key.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[business]
product_name = "TestBot"

[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/secret/path/sa-key.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#,
    )
    .unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path: config_path.clone(),
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

    // PATCH an unrelated field (business.product_name).
    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "business": {"product_name": "UpdatedBot"}
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

    // Read the on-disk TOML and verify service_account_key is preserved.
    let on_disk = std::fs::read_to_string(&config_path).unwrap();
    assert!(
        on_disk.contains("service_account_key"),
        "PATCH should not remove service_account_key from disk"
    );
    assert!(
        on_disk.contains("/secret/path/sa-key.json"),
        "service_account_key value should be preserved on disk"
    );

    // GET and verify the name was updated.
    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["business"]["product_name"], "UpdatedBot");
}

#[tokio::test]
async fn settings_patch_response_redacts_sa_key() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a config with a legacy service_account_key.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[business]
product_name = "TestBot"

[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/secret/path/sa-key.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#,
    )
    .unwrap();

    let state = Arc::new(AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("/tmp"),
        config_path: config_path.clone(),
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

    // PATCH an unrelated field — the response must redact the SA key.
    let patch_req = Request::builder()
        .method("PATCH")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "business": {"product_name": "PatchedBot"}
            }))
            .unwrap(),
        ))
        .expect("build request");

    let response = router.oneshot(patch_req).await.expect("send request");
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let source = &body["content_sources"]["sources"][0];
    assert_eq!(
        source["service_account_key"], "[redacted]",
        "PATCH response must redact service_account_key"
    );
    // On-disk value must remain unredacted.
    let on_disk = std::fs::read_to_string(&config_path).unwrap();
    assert!(on_disk.contains("/secret/path/sa-key.json"));
}

#[tokio::test]
async fn settings_get_redacts_sa_key_alongside_connection_id() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    // Write a config with both service_account_key AND connection_id.
    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[business]
product_name = "TestBot"

[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc123"
service_account_key = "/secret/path/sa-key.json"
connection_id = 42
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#,
    )
    .unwrap();

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

    let source = &body["content_sources"]["sources"][0];
    // SA key is redacted.
    assert_eq!(source["service_account_key"], "[redacted]");
    // connection_id is returned intact.
    assert_eq!(source["connection_id"], 42);
    // Other fields intact.
    assert_eq!(source["folder_id"], "abc123");
    assert_eq!(source["source_type"], "google_drive");
}

#[tokio::test]
async fn settings_init_with_connection_id() {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let dir = tempfile::tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.toml");
    // Ensure config does NOT exist yet (init creates it).
    assert!(!config_path.exists());

    let state = Arc::new(AppState {
        db: pool,
        data_dir: dir.path().to_path_buf(),
        config_path: config_path.clone(),
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

    // POST /api/settings/init with a Google Drive source using connection_id.
    let init_body = serde_json::json!({
        "x_api": {
            "client_id": "test-client-id"
        },
        "business": {
            "product_name": "InitBot",
            "product_keywords": ["test"],
            "product_description": "A test bot",
            "industry_topics": ["testing"]
        },
        "llm": {
            "provider": "ollama",
            "model": "llama3.2"
        },
        "content_sources": {
            "sources": [{
                "source_type": "google_drive",
                "folder_id": "init_folder",
                "connection_id": 7,
                "watch": true,
                "file_patterns": ["*.md"],
                "loop_back_enabled": false
            }]
        }
    });

    let (status, _body) = post_json(router.clone(), "/api/settings/init", init_body).await;
    assert_eq!(status, StatusCode::OK, "init should succeed");

    // Verify the config file was created and round-trips.
    assert!(config_path.exists(), "config file should be created");
    let on_disk = std::fs::read_to_string(&config_path).unwrap();
    let config: tuitbot_core::config::Config =
        toml::from_str(&on_disk).expect("on-disk config should parse");
    assert_eq!(config.content_sources.sources.len(), 1);
    assert_eq!(config.content_sources.sources[0].connection_id, Some(7));
    assert_eq!(
        config.content_sources.sources[0].folder_id.as_deref(),
        Some("init_folder")
    );
    assert!(config.content_sources.sources[0]
        .service_account_key
        .is_none());

    // GET and verify connection_id is returned.
    let (status, body) = get_json(router, "/api/settings").await;
    assert_eq!(status, StatusCode::OK);
    let source = &body["content_sources"]["sources"][0];
    assert_eq!(source["connection_id"], 7);
    assert_eq!(source["folder_id"], "init_folder");
}

// ============================================================
// Connector endpoints
// ============================================================

#[tokio::test]
async fn content_generator_lazy_init_per_account() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Write config with LLM configured (will fail to actually create provider
    // without a valid API key, but we can verify the caching mechanism)
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
[x_api]
provider_backend = "scraper"
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_keywords = ["test"]

[llm]
provider = "openai"
api_key = "test-key-not-real"
"#,
    )
    .expect("write config");

    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<AccountWsEvent>(256);

    let state = Arc::new(AppState {
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
    });

    // Cache starts empty
    assert_eq!(state.content_generators.lock().await.len(), 0);

    // Lazy init for default account
    let result = state
        .get_or_create_content_generator(tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID)
        .await;
    assert!(
        result.is_ok(),
        "should create generator for default account"
    );
    assert_eq!(state.content_generators.lock().await.len(), 1);

    // Create a non-default account
    let acct_b = uuid::Uuid::new_v4().to_string();
    tuitbot_core::storage::accounts::create_account(&pool, &acct_b, "Account B")
        .await
        .expect("create account");

    // Lazy init for non-default account (inherits base config LLM settings)
    let result = state.get_or_create_content_generator(&acct_b).await;
    assert!(
        result.is_ok(),
        "should create generator for non-default account"
    );
    assert_eq!(state.content_generators.lock().await.len(), 2);

    // Second call should return cached instance (no additional entry)
    let gen1 = state
        .get_or_create_content_generator(&acct_b)
        .await
        .unwrap();
    let gen2 = state
        .get_or_create_content_generator(&acct_b)
        .await
        .unwrap();
    assert!(Arc::ptr_eq(&gen1, &gen2), "should return cached generator");
    assert_eq!(state.content_generators.lock().await.len(), 2);
}
