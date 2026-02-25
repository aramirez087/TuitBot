//! Write tool tests: post, reply, quote, delete, thread, length validation, media.

use super::*;

// ── Success path tests ──────────────────────────────────────────────

#[tokio::test]
async fn post_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = post_tweet(&state, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "new_1");
}

#[tokio::test]
async fn quote_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = quote_tweet(&state, "Great!", "qt_id", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "qt_1");
}

#[tokio::test]
async fn delete_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = delete_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["deleted"], true);
}

#[tokio::test]
async fn post_tweet_with_media_ids() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let media = vec!["media_1".to_string()];
    let result = post_tweet(&state, "Check this!", Some(&media)).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "new_1");
}

// ── Thread tests ────────────────────────────────────────────────────

#[tokio::test]
async fn post_thread_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let tweets = vec![
        "Tweet 1".to_string(),
        "Tweet 2".to_string(),
        "Tweet 3".to_string(),
    ];
    let result = post_thread(&state, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["tweet_count"], 3);
    assert_eq!(parsed["data"]["root_tweet_id"], "new_1");
    assert_eq!(
        parsed["data"]["thread_tweet_ids"].as_array().unwrap().len(),
        3
    );
}

#[tokio::test]
async fn post_thread_empty_error() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = post_thread(&state, &[], None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "invalid_input");
}

#[tokio::test]
async fn post_thread_one_too_long() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let tweets = vec![
        "Short".to_string(),
        "a".repeat(281),
        "Also short".to_string(),
    ];
    let result = post_thread(&state, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

// ── Length validation tests ─────────────────────────────────────────

#[tokio::test]
async fn post_tweet_too_long_rejected() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let long_text = "a".repeat(281);
    let result = post_tweet(&state, &long_text, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

#[tokio::test]
async fn reply_too_long_rejected() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let long_text = "a".repeat(281);
    let result = reply_to_tweet(&state, &long_text, "t1", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

// ── Media support test ──────────────────────────────────────────────

#[tokio::test]
async fn upload_media_unsupported_extension() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, "/tmp/file.bmp").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}
