//! Contract tests for the compose and draft endpoints, covering both legacy
//! and new thread-blocks payloads.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tuitbot_core::storage;

use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

const TEST_TOKEN: &str = "test-token-abc123";

async fn test_router() -> axum::Router {
    let pool = storage::init_test_db().await.expect("init test db");
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let state = Arc::new(AppState {
        db: pool,
        config_path: std::path::PathBuf::from("/tmp/test-config.toml"),
        data_dir: std::path::PathBuf::from("/tmp"),
        event_tx,
        api_token: TEST_TOKEN.to_string(),
        passphrase_hash: tokio::sync::RwLock::new(None),
        bind_host: "127.0.0.1".to_string(),
        bind_port: 3001,
        login_attempts: Mutex::new(std::collections::HashMap::new()),
        content_generators: Mutex::new(std::collections::HashMap::new()),
        runtimes: Mutex::new(std::collections::HashMap::new()),
        circuit_breaker: None,
        watchtower_cancel: None,
        content_sources: Default::default(),
        deployment_mode: Default::default(),
    });

    tuitbot_server::build_router(state)
}

async fn post_json(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

async fn patch_json(
    router: axum::Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("PATCH")
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes.to_bytes()).expect("parse JSON");

    (status, json)
}

async fn get_json(router: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .uri(path)
        .header("Authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .expect("build request");

    let response = router.oneshot(req).await.expect("send request");
    let status = response.status();
    let body = response.into_body().collect().await.expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body.to_bytes()).expect("parse JSON");

    (status, json)
}

fn two_blocks() -> serde_json::Value {
    serde_json::json!([
        {"id": "block-1", "text": "First tweet of the thread", "media_paths": [], "order": 0},
        {"id": "block-2", "text": "Second tweet of the thread", "media_paths": [], "order": 1}
    ])
}

// ============================================================
// Legacy compatibility tests
// ============================================================

#[tokio::test]
async fn legacy_compose_tweet_still_works() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Hello from the legacy endpoint!"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["status"].as_str().unwrap() == "scheduled"
            || body["status"].as_str().unwrap() == "queued_for_approval"
    );
}

#[tokio::test]
async fn legacy_compose_thread_still_works() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "[\"First tweet\",\"Second tweet\"]"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["status"].as_str().unwrap() == "scheduled"
            || body["status"].as_str().unwrap() == "queued_for_approval"
    );
}

#[tokio::test]
async fn legacy_compose_thread_empty_array_rejected() {
    let router = test_router().await;
    let (status, _) = post_json(
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
async fn legacy_compose_tweet_over_limit_rejected() {
    let router = test_router().await;
    let long_text = "a".repeat(281);
    let (status, _) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": long_text
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn legacy_compose_thread_single_tweet_over_limit_rejected() {
    let router = test_router().await;
    let long_text = "a".repeat(281);
    let content = serde_json::to_string(&vec!["Short tweet", &long_text]).unwrap();
    let (status, _) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": content
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================
// Blocks compose tests
// ============================================================

#[tokio::test]
async fn compose_thread_with_blocks_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "ignored",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["block_ids"].is_array());
    let block_ids = body["block_ids"].as_array().unwrap();
    assert_eq!(block_ids.len(), 2);
    assert_eq!(block_ids[0], "block-1");
    assert_eq!(block_ids[1], "block-2");
}

#[tokio::test]
async fn compose_thread_blocks_precedence_over_content() {
    // When both blocks and content are provided, blocks take precedence.
    // The content field ("invalid") is ignored.
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "this is not valid JSON array",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["block_ids"].is_array());
}

#[tokio::test]
async fn compose_thread_blocks_with_media_paths() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "b1", "text": "Check this image", "media_paths": ["photo1.jpg", "photo2.jpg"], "order": 0},
                {"id": "b2", "text": "And this one", "media_paths": ["photo3.jpg"], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["block_ids"].is_array());
}

#[tokio::test]
async fn compose_thread_blocks_roundtrip() {
    let router = test_router().await;

    // Compose with blocks (creates a scheduled content item).
    let (status, compose_body) = post_json(
        router.clone(),
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "ignored",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = compose_body["id"].as_i64().unwrap();

    // Read the content back via the calendar endpoint covering a wide range.
    let (status, calendar) = get_json(
        router,
        "/api/content/calendar?from=2000-01-01T00:00:00Z&to=2099-01-01T00:00:00Z",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Find our item.
    let items = calendar.as_array().unwrap();
    let item = items.iter().find(|i| i["id"].as_i64() == Some(id));
    assert!(item.is_some(), "scheduled item should appear in calendar");

    // Verify the content is the blocks payload.
    let content = item.unwrap()["content"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert!(parsed["blocks"].is_array());
    assert_eq!(parsed["blocks"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["blocks"][0]["id"], "block-1");
}

// ============================================================
// Blocks validation rejection tests
// ============================================================

#[tokio::test]
async fn compose_blocks_empty_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("must not be empty"));
}

#[tokio::test]
async fn compose_blocks_single_block_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "only", "text": "Lone tweet", "media_paths": [], "order": 0}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("at least 2 blocks"));
}

#[tokio::test]
async fn compose_blocks_duplicate_ids_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "dupe", "text": "First", "media_paths": [], "order": 0},
                {"id": "dupe", "text": "Second", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("duplicate block ID"));
}

#[tokio::test]
async fn compose_blocks_non_contiguous_order_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": "First", "media_paths": [], "order": 0},
                {"id": "b", "text": "Third", "media_paths": [], "order": 2}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("contiguous sequence"));
}

#[tokio::test]
async fn compose_blocks_order_not_starting_at_zero_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": "First", "media_paths": [], "order": 1},
                {"id": "b", "text": "Second", "media_paths": [], "order": 2}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("contiguous sequence"));
}

#[tokio::test]
async fn compose_blocks_empty_text_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": "   ", "media_paths": [], "order": 0},
                {"id": "b", "text": "Second", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("empty text"));
}

#[tokio::test]
async fn compose_blocks_text_over_limit_rejected() {
    let router = test_router().await;
    let long_text = "a".repeat(281);
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": long_text, "media_paths": [], "order": 0},
                {"id": "b", "text": "Short", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("exceeds"));
}

#[tokio::test]
async fn compose_blocks_too_many_media_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": "Text", "media_paths": ["1","2","3","4","5"], "order": 0},
                {"id": "b", "text": "Second", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("too many media"));
}

#[tokio::test]
async fn compose_blocks_empty_id_rejected() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "", "text": "First", "media_paths": [], "order": 0},
                {"id": "b", "text": "Second", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("empty ID"));
}

// ============================================================
// Draft with blocks tests
// ============================================================

#[tokio::test]
async fn create_draft_with_blocks_accepted() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "thread",
            "content": "ignored",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "draft");
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn edit_draft_with_blocks_accepted() {
    let router = test_router().await;

    // Create a draft first.
    let (status, create_body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "thread",
            "content": "[\"old first\",\"old second\"]"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = create_body["id"].as_i64().unwrap();

    // Edit with blocks.
    let (status, body) = patch_json(
        router,
        &format!("/api/content/drafts/{id}"),
        serde_json::json!({
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "draft");
}

#[tokio::test]
async fn edit_draft_content_or_blocks_required() {
    let router = test_router().await;

    // Create a draft.
    let (status, create_body) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Hello"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let id = create_body["id"].as_i64().unwrap();

    // Edit with neither content nor blocks.
    let (status, body) = patch_json(
        router,
        &format!("/api/content/drafts/{id}"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("content")
        .then_some(true)
        .or(body["error"]
            .as_str()
            .unwrap()
            .contains("blocks")
            .then_some(true))
        .is_some());
}

#[tokio::test]
async fn list_drafts_returns_blocks_in_content() {
    let router = test_router().await;

    // Create a draft with blocks.
    let (status, _) = post_json(
        router.clone(),
        "/api/content/drafts",
        serde_json::json!({
            "content_type": "thread",
            "content": "ignored",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // List drafts.
    let (status, body) = get_json(router, "/api/content/drafts").await;
    assert_eq!(status, StatusCode::OK);
    let drafts = body.as_array().unwrap();
    assert!(!drafts.is_empty());

    // The content field should contain the blocks payload.
    let content = drafts[0]["content"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert!(parsed["blocks"].is_array());
}

// ============================================================
// Edge cases
// ============================================================

#[tokio::test]
async fn compose_tweet_ignores_blocks() {
    let router = test_router().await;
    let (status, body) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "tweet",
            "content": "Just a tweet",
            "blocks": two_blocks()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // No block_ids in response for tweet type.
    assert!(body.get("block_ids").is_none());
}

#[tokio::test]
async fn compose_blocks_with_urls_respects_weighted_length() {
    let router = test_router().await;
    // 250 chars of text + a long URL. Weighted = 250 + 23 = 273, under 280.
    let padding = "a".repeat(250);
    let text = format!("{padding} https://example.com/{}", "x".repeat(76));
    assert!(text.len() > 280); // Raw length is over 280.

    let (status, _) = post_json(
        router,
        "/api/content/compose",
        serde_json::json!({
            "content_type": "thread",
            "content": "",
            "blocks": [
                {"id": "a", "text": text, "media_paths": [], "order": 0},
                {"id": "b", "text": "Short second tweet", "media_paths": [], "order": 1}
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
