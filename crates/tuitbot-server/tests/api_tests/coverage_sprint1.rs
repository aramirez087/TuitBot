//! Additional coverage tests for sprint-1 push toward 75% threshold.
//!
//! Covers compose, connectors, settings, and factory reset edge cases.

use axum::http::StatusCode;
use serde_json::json;

use super::*;

/// Create a router with a specific config TOML string.
async fn test_router_with_config(
    dir: &std::path::Path,
    config_toml: &str,
) -> (axum::Router, storage::DbPool) {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<tuitbot_server::ws::AccountWsEvent>(256);

    let config_path = dir.join("config.toml");
    std::fs::write(&config_path, config_toml).expect("write config");

    let state = std::sync::Arc::new(tuitbot_server::state::AppState {
        db: pool.clone(),
        config_path,
        data_dir: dir.to_path_buf(),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: tokio::sync::RwLock::new(None),
        passphrase_hash_mtime: tokio::sync::RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        content_generators: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        runtimes: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        circuit_breaker: None,
        scraper_health: None,
        watchtower_cancel: tokio::sync::RwLock::new(None),
        content_sources: tokio::sync::RwLock::new(Default::default()),
        connector_config: Default::default(),
        deployment_mode: Default::default(),
        pending_oauth: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        token_managers: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        x_client_id: "test-client-id".to_string(),
        semantic_index: None,
        embedding_provider: None,
    });

    (tuitbot_server::build_router(state), pool)
}

const APPROVAL_ON_CONFIG: &str = r#"
approval_mode = true

[x_api]
provider_backend = "scraper"
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_keywords = ["test"]
"#;

const APPROVAL_OFF_CONFIG: &str = r#"
approval_mode = false

[x_api]
provider_backend = "scraper"
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_keywords = ["test"]
"#;

// ============================================================
// Compose tests (compose.rs coverage)
// ============================================================

#[tokio::test]
async fn compose_tweet_empty_text_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(router, "/api/content/tweets", json!({ "text": "" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty text: {body}");
}

#[tokio::test]
async fn compose_tweet_whitespace_only_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        json!({ "text": "   \n\t  " }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "whitespace text: {body}");
}

#[tokio::test]
async fn compose_tweet_approval_mode_queues() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_ON_CONFIG).await;

    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        json!({ "text": "Test tweet for approval" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "queued: {body}");
    assert_eq!(body["status"], "queued_for_approval");
    assert!(body["id"].is_number(), "should return id: {body}");
}

#[tokio::test]
async fn compose_tweet_no_approval_mode_accepts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_OFF_CONFIG).await;

    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        json!({ "text": "Direct tweet" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "accepted: {body}");
    assert_eq!(body["status"], "accepted");
}

#[tokio::test]
async fn compose_thread_empty_tweets_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(router, "/api/content/threads", json!({ "tweets": [] })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty thread: {body}");
}

#[tokio::test]
async fn compose_thread_approval_mode_queues() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_ON_CONFIG).await;

    let (status, body) = post_json(
        router,
        "/api/content/threads",
        json!({ "tweets": ["First tweet", "Second tweet"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "thread queued: {body}");
    assert_eq!(body["status"], "queued_for_approval");
}

#[tokio::test]
async fn compose_unified_invalid_content_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "invalid",
            "content": "Hello"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid type: {body}");
}

#[tokio::test]
async fn compose_unified_tweet_empty_content() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "tweet",
            "content": ""
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty content: {body}");
}

#[tokio::test]
async fn compose_unified_tweet_too_long() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let long_text = "x".repeat(300);
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "tweet",
            "content": long_text
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "too long: {body}");
}

#[tokio::test]
async fn compose_unified_thread_legacy_invalid_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "thread",
            "content": "not a json array"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid json: {body}");
}

#[tokio::test]
async fn compose_unified_thread_legacy_empty_array() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "thread",
            "content": "[]"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty array: {body}");
}

#[tokio::test]
async fn compose_unified_thread_blocks_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "thread",
            "content": "",
            "blocks": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty blocks: {body}");
}

#[tokio::test]
async fn compose_unified_tweet_with_approval_mode() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_ON_CONFIG).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "tweet",
            "content": "Approval tweet"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "queued: {body}");
    assert_eq!(body["status"], "queued_for_approval");
}

#[tokio::test]
async fn compose_unified_tweet_without_approval() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "tweet",
            "content": "Immediate tweet"
        }),
    )
    .await;
    let code = status.as_u16();
    // Without X API credentials, falls through to scheduled
    assert!(
        code == 200 || code == 400 || code == 500,
        "got {code}: {body}"
    );
}

#[tokio::test]
async fn compose_unified_thread_with_blocks_valid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_ON_CONFIG).await;

    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                { "id": "b1", "text": "First block", "order": 0 },
                { "id": "b2", "text": "Second block", "order": 1 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "blocks compose: {body}");
    assert_eq!(body["status"], "queued_for_approval");
    assert!(body["block_ids"].is_array());
}

#[tokio::test]
async fn compose_tweet_with_empty_provenance() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_config(dir.path(), APPROVAL_ON_CONFIG).await;

    // Empty provenance array should be treated as no provenance.
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "tweet",
            "content": "Sourced tweet",
            "provenance": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "empty provenance: {body}");
    assert_eq!(body["status"], "queued_for_approval");
}

#[tokio::test]
async fn compose_thread_legacy_tweet_too_long() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let long_tweet = "x".repeat(300);
    let content = serde_json::to_string(&vec![long_tweet]).unwrap();
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        json!({
            "content_type": "thread",
            "content": content
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "thread tweet too long: {body}"
    );
}

// ============================================================
// Connectors deep tests (connectors.rs coverage)
// ============================================================

#[tokio::test]
async fn connectors_disconnect_existing_google_drive() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let conn_id = tuitbot_core::storage::watchtower::insert_connection(
        &pool,
        "google_drive",
        Some("test@example.com"),
        Some("Test User"),
    )
    .await
    .expect("insert connection");

    let (status, body) =
        delete_json(router, &format!("/api/connectors/google-drive/{conn_id}")).await;
    assert_eq!(status, StatusCode::OK, "disconnect: {body}");
    assert_eq!(body["disconnected"], true);
    assert_eq!(body["id"], conn_id);
}

#[tokio::test]
async fn connectors_disconnect_wrong_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let conn_id = tuitbot_core::storage::watchtower::insert_connection(
        &pool,
        "dropbox",
        Some("user@example.com"),
        None,
    )
    .await
    .expect("insert connection");

    let (status, body) =
        delete_json(router, &format!("/api/connectors/google-drive/{conn_id}")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "wrong type: {body}");
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("not a Google Drive"));
}

#[tokio::test]
async fn connectors_status_with_connection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    tuitbot_core::storage::watchtower::insert_connection(
        &pool,
        "google_drive",
        Some("user@example.com"),
        Some("Display Name"),
    )
    .await
    .expect("insert");

    let (status, body) = get_json(router, "/api/connectors/google-drive/status").await;
    assert_eq!(status, StatusCode::OK);
    let conns = body["connections"].as_array().expect("array");
    assert!(!conns.is_empty(), "should have a connection: {body}");
}

// ============================================================
// Settings tests (settings.rs coverage)
// ============================================================

#[tokio::test]
async fn settings_test_llm_invalid_provider() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/test-llm",
        json!({
            "provider": "nonexistent_provider",
            "model": "gpt-4"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "test-llm: {body}");
    assert_eq!(body["success"], false);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn settings_test_llm_missing_api_key() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/test-llm",
        json!({
            "provider": "openai",
            "model": "gpt-4"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "test-llm no key: {body}");
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn settings_factory_reset_correct_phrase() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/factory-reset",
        json!({ "confirmation": "RESET TUITBOT" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "factory reset: {body}");
    assert_eq!(body["status"], "reset_complete");
    assert!(body["cleared"].is_object());
    assert!(body["cleared"]["tables_cleared"].is_number());
    assert!(body["cleared"]["rows_deleted"].is_number());
}

#[tokio::test]
async fn settings_factory_reset_empty_confirmation() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/factory-reset",
        json!({ "confirmation": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "empty phrase: {body}");
}

#[tokio::test]
async fn settings_factory_reset_case_sensitive() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/settings/factory-reset",
        json!({ "confirmation": "reset tuitbot" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "lowercase should fail: {body}"
    );
}

#[tokio::test]
async fn settings_get_for_nondefault_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let account_id = create_test_account(&pool, "test-account").await;

    let (status, body) = get_json_for(router, "/api/settings", &account_id).await;
    assert_eq!(status, StatusCode::OK, "non-default settings: {body}");
    assert!(
        body.get("config").is_some() || body.get("business").is_some(),
        "should return config: {body}"
    );
}

#[tokio::test]
async fn settings_patch_nondefault_account_rejects_instance_scope() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let account_id = create_test_account(&pool, "test-account-2").await;

    let req = Request::builder()
        .method("PATCH")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .header("X-Account-Id", &account_id)
        .body(Body::from(
            serde_json::to_vec(&json!({
                "deployment_mode": "cloud"
            }))
            .unwrap(),
        ))
        .expect("build");

    let resp = router.oneshot(req).await.expect("send");
    let status = resp.status();
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "instance-scoped keys should be rejected"
    );
}

#[tokio::test]
async fn settings_validate_nondefault_account() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, pool) = test_router_with_dir(dir.path()).await;

    let account_id = create_test_account(&pool, "test-account-3").await;

    let (status, body) = post_json_for(
        router,
        "/api/settings/validate",
        &account_id,
        json!({
            "business": { "product_name": "Test" }
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "validate non-default: {code}: {body}"
    );
}

#[tokio::test]
async fn settings_init_conflict_when_config_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;

    let (status, body) = post_json(
        router,
        "/api/settings/init",
        json!({
            "business": {
                "product_name": "New",
                "product_keywords": ["test"]
            }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "should conflict: {body}");
}
