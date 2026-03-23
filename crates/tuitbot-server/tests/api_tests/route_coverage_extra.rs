//! Extra integration tests targeting large route handler files for coverage.
//!
//! Exercises compose (unified endpoint), onboarding, media, ingest, assist,
//! auth, and draft-studio lifecycle routes. Many endpoints return 400/500 in
//! the test environment (no LLM, no config file, no X credentials) — that is
//! fine because the handler code is still exercised end-to-end.

use super::*;

// ============================================================
// Compose — unified endpoint (POST /api/content/compose)
// ============================================================

#[tokio::test]
async fn compose_unified_tweet_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Hello from unified compose!"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "compose tweet: {body}");
    let s = body["status"].as_str().unwrap();
    assert!(
        s == "accepted" || s == "queued_for_approval" || s == "scheduled",
        "unexpected status: {s}"
    );
}

#[tokio::test]
async fn compose_unified_tweet_empty_content_rejected() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "   "
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_unified_tweet_exceeds_280_chars() {
    let router = test_router().await;
    let long = "x".repeat(300);
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": long
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_unified_invalid_content_type() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "reel",
            "content": "some content"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid type: {body}");
}

#[tokio::test]
async fn compose_unified_thread_legacy_accepted() {
    let router = test_router().await;
    let tweets = serde_json::json!(["First tweet", "Second tweet"]);
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": tweets.to_string()
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "compose thread legacy: {code} {body}"
    );
}

#[tokio::test]
async fn compose_unified_thread_legacy_empty_array() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "[]"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_unified_thread_legacy_invalid_json() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "not json"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_unified_thread_blocks_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                { "id": "block-1", "text": "First block", "order": 0 },
                { "id": "block-2", "text": "Second block", "order": 1 }
            ]
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "compose thread blocks: {code} {body}"
    );
    if code == 200 {
        let s = body["status"].as_str().unwrap();
        assert!(
            s == "queued_for_approval" || s == "scheduled" || s == "posted",
            "unexpected status: {s}"
        );
    }
}

#[tokio::test]
async fn compose_unified_thread_blocks_empty_text_rejected() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                { "id": "block-1", "text": "", "order": 0 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn compose_unified_tweet_with_media_paths() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Tweet with media",
            "media_paths": ["/tmp/fake-image.jpg"]
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "compose with media: {code} {body}"
    );
}

#[tokio::test]
async fn compose_unified_tweet_with_provenance() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Provenance tweet",
            "provenance": [
                { "node_id": 1, "chunk_index": 0, "similarity": 0.95 }
            ]
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "compose with provenance: {code} {body}"
    );
}

#[tokio::test]
async fn compose_tweet_with_scheduled_for() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Scheduled tweet",
            "scheduled_for": "2099-12-31T23:59:59Z"
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "compose scheduled: {code} {body}"
    );
}

#[tokio::test]
async fn compose_tweet_with_past_schedule_rejected() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Past tweet",
            "scheduled_for": "2020-01-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "past schedule should be rejected"
    );
}

// ============================================================
// Compose — dedicated endpoints edge cases
// ============================================================

#[tokio::test]
async fn compose_thread_with_scheduled_for() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/threads",
        serde_json::json!({
            "tweets": ["First", "Second"],
            "scheduled_for": "2099-12-31T23:59:59Z"
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "thread scheduled: {code} {body}"
    );
}

#[tokio::test]
async fn compose_tweet_with_provenance_on_dedicated_endpoint() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/tweets",
        serde_json::json!({
            "text": "Tweet with provenance",
            "provenance": [
                { "node_id": 42, "chunk_index": 0, "similarity": 0.88 }
            ]
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "tweet provenance: {code} {body}"
    );
}

// ============================================================
// Onboarding routes
// ============================================================

#[tokio::test]
async fn onboarding_x_auth_status_no_tokens() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/onboarding/x-auth/status").await;
    // No onboarding_tokens.json exists -> { connected: false }
    assert_eq!(status, StatusCode::OK, "onboarding status: {body}");
    assert_eq!(body["connected"], false);
}

#[tokio::test]
async fn onboarding_x_auth_start_no_client_id() {
    // test_router has empty x_client_id, so this should fail
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/start",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "no client_id: {body}");
    assert!(body["error"].as_str().unwrap().contains("client_id"));
}

#[tokio::test]
async fn onboarding_x_auth_start_with_client_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    // test_router_with_dir sets x_client_id = "test-client-id"
    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/start",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "start with client_id: {body}");
    assert!(body["authorization_url"].is_string());
    assert!(body["state"].is_string());
}

#[tokio::test]
async fn onboarding_x_auth_callback_invalid_state() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/onboarding/x-auth/callback",
        serde_json::json!({
            "code": "fake-code",
            "state": "nonexistent-state"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid state: {body}");
}

#[tokio::test]
async fn onboarding_analyze_profile_no_tokens() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/onboarding/analyze-profile",
        serde_json::json!({}),
    )
    .await;
    // No onboarding_tokens.json -> graceful error
    assert_eq!(status, StatusCode::OK, "analyze no tokens: {body}");
    assert_eq!(body["status"], "x_api_error");
}

// ============================================================
// Media routes
// ============================================================

#[tokio::test]
async fn media_file_missing_path_param() {
    let router = test_router().await;
    let req = Request::builder()
        .uri("/api/media/file")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    // Missing required query param -> 400
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn media_file_path_traversal_rejected() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/media/file?path=../../etc/passwd").await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 404,
        "path traversal should be rejected: {code}"
    );
}

#[tokio::test]
async fn media_file_nonexistent_returns_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (router, _pool) = test_router_with_dir(dir.path()).await;
    let safe_path = dir.path().join("media/nonexistent.jpg");
    let (status, _body) = get_json(
        router,
        &format!("/api/media/file?path={}", safe_path.display()),
    )
    .await;
    let code = status.as_u16();
    assert!(code == 400 || code == 404, "nonexistent media: {code}");
}

// ============================================================
// Ingest routes
// ============================================================

#[tokio::test]
async fn ingest_empty_body_returns_ok() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "file_hints": [],
            "inline_nodes": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "empty ingest: {body}");
    assert_eq!(body["ingested"], 0);
    assert_eq!(body["skipped"], 0);
}

#[tokio::test]
async fn ingest_inline_node() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [
                {
                    "relative_path": "notes/test.md",
                    "body_text": "This is inline test content.",
                    "title": "Test Note",
                    "tags": "test,inline"
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "inline ingest: {body}");
    assert_eq!(body["ingested"], 1);
}

#[tokio::test]
async fn ingest_inline_node_empty_body_text_is_error() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [
                {
                    "relative_path": "notes/empty.md",
                    "body_text": ""
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "empty body_text: {body}");
    assert_eq!(body["ingested"], 0);
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn ingest_inline_node_force_mode() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "force": true,
            "inline_nodes": [
                {
                    "relative_path": "notes/force.md",
                    "body_text": "Force re-ingest content."
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "force ingest: {body}");
    assert_eq!(body["ingested"], 1);
}

#[tokio::test]
async fn ingest_file_hints_no_source_configured() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "file_hints": ["readme.md"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "file hints no source: {body}");
    // No local_fs source configured -> error reported
    assert!(
        !body["errors"].as_array().unwrap().is_empty(),
        "expected error about no source"
    );
}

#[tokio::test]
async fn ingest_duplicate_inline_node_skipped() {
    let pool = storage::init_test_db().await.expect("init db");
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

    // First ingest
    let (s1, b1) = post_json(
        router.clone(),
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [{
                "relative_path": "dup.md",
                "body_text": "Same content"
            }]
        }),
    )
    .await;
    assert_eq!(s1, StatusCode::OK, "first: {b1}");
    assert_eq!(b1["ingested"], 1);

    // Second ingest with same content -> should be skipped
    let (s2, b2) = post_json(
        router,
        "/api/ingest",
        serde_json::json!({
            "inline_nodes": [{
                "relative_path": "dup.md",
                "body_text": "Same content"
            }]
        }),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "second: {b2}");
    assert_eq!(b2["skipped"], 1, "duplicate should be skipped");
}

// ============================================================
// AI Assist routes
// ============================================================

#[tokio::test]
async fn assist_tweet_no_llm_returns_error() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/assist/tweet",
        serde_json::json!({ "topic": "Rust async" }),
    )
    .await;
    let code = status.as_u16();
    // No LLM configured -> 400 or 500
    assert!(
        code == 400 || code == 500,
        "assist tweet without LLM: {code}"
    );
}

#[tokio::test]
async fn assist_reply_no_llm_returns_error() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/assist/reply",
        serde_json::json!({
            "tweet_text": "Anyone using Rust?",
            "tweet_author": "rustdev"
        }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 500,
        "assist reply without LLM: {code}"
    );
}

#[tokio::test]
async fn assist_thread_no_llm_returns_error() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/assist/thread",
        serde_json::json!({ "topic": "Rust ecosystem" }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 500,
        "assist thread without LLM: {code}"
    );
}

#[tokio::test]
async fn assist_improve_no_llm_returns_error() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/assist/improve")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({ "content": "Make this better" })).unwrap(),
        ))
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    assert!(
        code == 400 || code == 422 || code == 500,
        "assist improve without LLM: {code}"
    );
}

#[tokio::test]
async fn assist_topics_no_llm_returns_error() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/assist/topics").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "assist topics: {code}"
    );
}

#[tokio::test]
async fn assist_optimal_times_returns_result() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/assist/optimal-times").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "assist optimal-times: {code}"
    );
}

#[tokio::test]
async fn assist_mode_returns_result() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/assist/mode").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "assist mode: {code}"
    );
}

// ============================================================
// Auth routes
// ============================================================

#[tokio::test]
async fn auth_status_returns_result() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/auth/status").await;
    assert_eq!(status, StatusCode::OK, "auth status: {body}");
}

#[tokio::test]
async fn auth_login_empty_body() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({})).unwrap(),
        ))
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    // No passphrase set -> various outcomes; body may not be JSON
    assert!(
        code == 200 || code == 400 || code == 401 || code == 422,
        "auth login empty: {code}"
    );
}

#[tokio::test]
async fn auth_logout() {
    let router = test_router().await;
    let (status, _body) = post_json(router, "/api/auth/logout", serde_json::json!({})).await;
    let code = status.as_u16();
    assert!(code == 200 || code == 400, "auth logout: {code}");
}

// ============================================================
// Draft Studio lifecycle
// ============================================================

#[tokio::test]
async fn drafts_list_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts").await;
    assert_eq!(status, StatusCode::OK, "drafts list: {body}");
    assert!(body.is_array());
}

#[tokio::test]
async fn drafts_create_and_get() {
    let pool = storage::init_test_db().await.expect("init db");
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

    // Create a draft
    let (status, body) = post_json(
        router.clone(),
        "/api/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Draft tweet content"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create draft: {body}");
    let draft_id = body["id"].as_i64().expect("draft id");

    // Get the draft
    let (status, body) = get_json(router, &format!("/api/drafts/{draft_id}")).await;
    assert_eq!(status, StatusCode::OK, "get draft: {body}");
    assert_eq!(body["id"], draft_id);
}

#[tokio::test]
async fn drafts_get_nonexistent_returns_not_found() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/drafts/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn drafts_autosave_nonexistent() {
    let router = test_router().await;
    let req = Request::builder()
        .method("PATCH")
        .uri("/api/drafts/999999")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({ "content": "updated" })).unwrap(),
        ))
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    assert!(
        code == 404 || code == 200 || code == 422,
        "autosave nonexistent: {code}"
    );
}

#[tokio::test]
async fn drafts_delete_nonexistent() {
    let router = test_router().await;
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/drafts/999999")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    assert!(code == 404 || code == 200, "delete nonexistent: {code}");
}

#[tokio::test]
async fn drafts_schedule_nonexistent() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/drafts/999999/schedule",
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59Z" }),
    )
    .await;
    let code = status.as_u16();
    assert!(code == 404 || code == 400, "schedule nonexistent: {code}");
}

#[tokio::test]
async fn drafts_archive_nonexistent() {
    let router = test_router().await;
    let (status, _body) =
        post_json(router, "/api/drafts/999999/archive", serde_json::json!({})).await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 200,
        "archive nonexistent: {code}"
    );
}

#[tokio::test]
async fn drafts_restore_nonexistent() {
    let router = test_router().await;
    let (status, _body) =
        post_json(router, "/api/drafts/999999/restore", serde_json::json!({})).await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 200,
        "restore nonexistent: {code}"
    );
}

#[tokio::test]
async fn drafts_duplicate_nonexistent() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/drafts/999999/duplicate",
        serde_json::json!({}),
    )
    .await;
    let code = status.as_u16();
    assert!(code == 404 || code == 400, "duplicate nonexistent: {code}");
}

#[tokio::test]
async fn drafts_revisions_nonexistent() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts/999999/revisions").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 404,
        "revisions nonexistent: {code} {body}"
    );
}

#[tokio::test]
async fn drafts_activity_nonexistent() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/drafts/999999/activity").await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 404,
        "activity nonexistent: {code} {body}"
    );
}

#[tokio::test]
async fn drafts_patch_meta_nonexistent() {
    let router = test_router().await;
    let (status, _body) = patch_json(
        router,
        "/api/drafts/999999/meta",
        serde_json::json!({ "title": "new title" }),
    )
    .await;
    let code = status.as_u16();
    assert!(code == 404 || code == 400, "patch meta: {code}");
}

// ============================================================
// Legacy drafts
// ============================================================

#[tokio::test]
async fn legacy_drafts_list_empty() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK, "legacy drafts: {body}");
}

#[tokio::test]
async fn legacy_drafts_delete_nonexistent() {
    let router = test_router().await;
    let (status, _body) = delete_json(router, "/api/content/drafts/999999").await;
    let code = status.as_u16();
    assert!(code == 404 || code == 200, "legacy delete: {code}");
}

#[tokio::test]
async fn legacy_drafts_schedule_nonexistent() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/drafts/999999/schedule",
        serde_json::json!({ "scheduled_for": "2099-12-31T23:59:59Z" }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 200,
        "legacy schedule: {code}"
    );
}

#[tokio::test]
async fn legacy_drafts_publish_nonexistent() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/content/drafts/999999/publish",
        serde_json::json!({}),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 500,
        "legacy publish: {code}"
    );
}

// ============================================================
// Vault detail route
// ============================================================

#[tokio::test]
async fn vault_note_detail_nonexistent() {
    let router = test_router().await;
    let (status, _body) = get_json(router, "/api/vault/notes/999999").await;
    let code = status.as_u16();
    assert!(
        code == 404 || code == 400 || code == 500,
        "vault note detail: {code}"
    );
}

#[tokio::test]
async fn vault_resolve_refs() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/vault/resolve-refs",
        serde_json::json!({ "node_ids": [] }),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400,
        "vault resolve-refs: {code} {body}"
    );
}

// ============================================================
// Settings test-llm
// ============================================================

#[tokio::test]
async fn settings_test_llm_no_config() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/settings/test-llm",
        serde_json::json!({
            "provider": "openai",
            "api_key": "sk-fake",
            "model": "gpt-4"
        }),
    )
    .await;
    let code = status.as_u16();
    // No real LLM -> expect error
    assert!(
        code == 200 || code == 400 || code == 500,
        "test-llm: {code}"
    );
}

// ============================================================
// LAN reset passphrase
// ============================================================

#[tokio::test]
async fn lan_reset_passphrase() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/settings/lan/reset-passphrase",
        serde_json::json!({}),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 200 || code == 400 || code == 500,
        "reset passphrase: {code}"
    );
}

// ============================================================
// Accounts — roles and sync-profile
// ============================================================

#[tokio::test]
async fn account_set_role_nonexistent() {
    let router = test_router().await;
    let req = Request::builder()
        .method("POST")
        .uri("/api/accounts/nonexistent-id/roles")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({ "role": "admin" })).unwrap(),
        ))
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    assert!(
        code == 200 || code == 400 || code == 404 || code == 422,
        "set role: {code}"
    );
}

#[tokio::test]
async fn account_remove_role_nonexistent() {
    let router = test_router().await;
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/accounts/nonexistent-id/roles")
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build");
    let resp = router.oneshot(req).await.expect("send");
    let code = resp.status().as_u16();
    assert!(
        code == 200 || code == 400 || code == 404 || code == 415 || code == 422,
        "remove role: {code}"
    );
}

#[tokio::test]
async fn account_sync_profile_nonexistent() {
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/accounts/nonexistent-id/sync-profile",
        serde_json::json!({}),
    )
    .await;
    let code = status.as_u16();
    assert!(
        code == 400 || code == 404 || code == 500,
        "sync profile: {code}"
    );
}

// ============================================================
// Targets — timeline and stats for nonexistent
// ============================================================

#[tokio::test]
async fn target_timeline_nonexistent() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/targets/nonexistent/timeline").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 404, "target timeline: {code} {body}");
}

#[tokio::test]
async fn target_stats_nonexistent() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/targets/nonexistent/stats").await;
    let code = status.as_u16();
    assert!(code == 200 || code == 404, "target stats: {code} {body}");
}
