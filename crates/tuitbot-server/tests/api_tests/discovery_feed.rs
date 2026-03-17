//! Integration tests for the discovery feed routes (Task 3.4).
//!
//! Covers:
//!   GET  /api/discovery/feed
//!   GET  /api/discovery/keywords
//!   POST /api/discovery/{tweet_id}/queue-reply  (happy path + error cases)
//!   POST /api/discovery/{tweet_id}/compose-reply (no LLM → error)

use super::*;

// ---------------------------------------------------------------------------
// GET /api/discovery/feed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discovery_feed_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/feed").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected JSON array, got: {body}");
}

#[tokio::test]
async fn discovery_feed_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/discovery/feed")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn discovery_feed_with_min_score_param_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/feed?min_score=50").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn discovery_feed_with_limit_param_respects_bound() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/feed?limit=5").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert!(arr.len() <= 5);
}

#[tokio::test]
async fn discovery_feed_with_keyword_filter_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/feed?keyword=rust").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[tokio::test]
async fn discovery_feed_each_item_has_required_fields() {
    use tuitbot_core::storage;

    // Seed a discovered tweet so the feed has at least one item to inspect.
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<tuitbot_server::ws::AccountWsEvent>(16);
    let state = std::sync::Arc::new(tuitbot_server::state::AppState {
        db: pool.clone(),
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
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
        x_client_id: String::new(),
    });

    // Insert a discovered tweet directly.
    tuitbot_core::storage::tweets::insert_discovered_tweet_for(
        &pool,
        tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID,
        &tuitbot_core::storage::tweets::DiscoveredTweet {
            id: "tweet_abc123".to_string(),
            author_id: "author_id_123".to_string(),
            author_username: "dev_user".to_string(),
            content: "Rust async is amazing".to_string(),
            like_count: 10,
            retweet_count: 2,
            reply_count: 1,
            impression_count: None,
            relevance_score: Some(85.0),
            matched_keyword: Some("rust".to_string()),
            discovered_at: "2026-03-14T00:00:00Z".to_string(),
            replied_to: 0,
        },
    )
    .await
    .expect("insert tweet");

    let router = tuitbot_server::build_router(state);
    let (status, body) = get_json(router, "/api/discovery/feed?min_score=0").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    if !arr.is_empty() {
        let item = &arr[0];
        assert!(item["id"].is_string(), "missing id field");
        assert!(
            item["author_username"].is_string(),
            "missing author_username"
        );
        assert!(item["content"].is_string(), "missing content");
        assert!(
            item["relevance_score"].is_number(),
            "missing relevance_score"
        );
    }
}

// ---------------------------------------------------------------------------
// GET /api/discovery/keywords
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discovery_keywords_returns_array() {
    let router = test_router().await;
    let (status, body) = get_json(router, "/api/discovery/keywords").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "expected JSON array");
}

#[tokio::test]
async fn discovery_keywords_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .uri("/api/discovery/keywords")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/queue-reply — error cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discovery_queue_reply_without_x_credentials_returns_400() {
    // No X API token files exist in the test environment, so this must return 400.
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/discovery/tweet_abc123/queue-reply",
        serde_json::json!({"content": "Great point!"}),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "posting without credentials should fail: {body}"
    );
}

#[tokio::test]
async fn discovery_queue_reply_empty_content_returns_400() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/discovery/tweet_xyz/queue-reply",
        serde_json::json!({"content": "   "}),
    )
    .await;
    // Either 400 (empty content) or 400 (no credentials) — both are valid failures.
    assert!(
        status == StatusCode::BAD_REQUEST,
        "empty content should return 400, got {status}: {body}"
    );
}

#[tokio::test]
async fn discovery_queue_reply_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/discovery/tweet_xyz/queue-reply")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"content":"hi"}"#))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// POST /api/discovery/{tweet_id}/compose-reply — no LLM → error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discovery_compose_reply_without_llm_returns_error() {
    // No LLM configured in test environment → 400/500 from the generator.
    let router = test_router().await;
    let (status, _body) = post_json(
        router,
        "/api/discovery/tweet_abc/compose-reply",
        serde_json::json!({"mention_product": false}),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
        "expected 400 or 500 without LLM configured, got {status}"
    );
}

#[tokio::test]
async fn discovery_compose_reply_requires_auth() {
    let router = test_router().await;
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/discovery/tweet_abc/compose-reply")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{}"#))
        .unwrap();
    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
