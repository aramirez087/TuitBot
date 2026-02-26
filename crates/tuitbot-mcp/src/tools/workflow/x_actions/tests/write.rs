//! Write tool tests: post, reply, quote, delete, thread, length validation, media, dry-run.

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
    let result = upload_media(&state, "/tmp/file.bmp", None, false).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}

#[tokio::test]
async fn upload_media_dry_run_unsupported() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = upload_media(&state, "/tmp/file.bmp", None, true).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "unsupported_media_type");
}

// ── Dry-run validation tests ────────────────────────────────────────

#[tokio::test]
async fn post_tweet_dry_run_valid() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = post_tweet_dry_run(&state, "Hello dry-run!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["valid"], true);
    assert_eq!(parsed["data"]["text"], "Hello dry-run!");
    assert_eq!(parsed["data"]["has_media"], false);
    assert_eq!(parsed["data"]["media_count"], 0);
}

#[tokio::test]
async fn post_tweet_dry_run_with_media() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let media = vec!["m1".to_string(), "m2".to_string()];
    let result = post_tweet_dry_run(&state, "With media!", Some(&media)).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["has_media"], true);
    assert_eq!(parsed["data"]["media_count"], 2);
    assert_eq!(parsed["data"]["media_ids"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn post_tweet_dry_run_too_long() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let long_text = "a".repeat(281);
    let result = post_tweet_dry_run(&state, &long_text, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

#[tokio::test]
async fn post_tweet_dry_run_no_x_client() {
    let state = make_state(None, None).await;
    let result = post_tweet_dry_run(&state, "No client needed!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["valid"], true);
    assert_eq!(parsed["data"]["x_client_available"], false);
}

#[tokio::test]
async fn post_thread_dry_run_valid() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let tweets = vec![
        "Thread start".to_string(),
        "Thread middle".to_string(),
        "Thread end".to_string(),
    ];
    let result = post_thread_dry_run(&state, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["valid"], true);
    assert_eq!(parsed["data"]["tweet_count"], 3);

    let tweet_validations = parsed["data"]["tweets"].as_array().unwrap();
    assert_eq!(tweet_validations.len(), 3);
    assert_eq!(tweet_validations[0]["chain_action"], "post_tweet");
    assert_eq!(
        tweet_validations[1]["chain_action"],
        "reply_to_tweet(parent=tweet_0)"
    );
    assert_eq!(
        tweet_validations[2]["chain_action"],
        "reply_to_tweet(parent=tweet_1)"
    );
}

#[tokio::test]
async fn post_thread_dry_run_with_media() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let tweets = vec!["Tweet 1".to_string(), "Tweet 2".to_string()];
    let media = vec![
        vec!["m1".to_string()],
        vec!["m2".to_string(), "m3".to_string()],
    ];
    let result = post_thread_dry_run(&state, &tweets, Some(&media)).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);

    let tweet_validations = parsed["data"]["tweets"].as_array().unwrap();
    assert_eq!(tweet_validations[0]["has_media"], true);
    assert_eq!(
        tweet_validations[0]["media_ids"].as_array().unwrap().len(),
        1
    );
    assert_eq!(tweet_validations[1]["has_media"], true);
    assert_eq!(
        tweet_validations[1]["media_ids"].as_array().unwrap().len(),
        2
    );
}

#[tokio::test]
async fn post_thread_dry_run_empty() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = post_thread_dry_run(&state, &[], None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "invalid_input");
}

#[tokio::test]
async fn post_thread_dry_run_too_long() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let tweets = vec!["Short".to_string(), "a".repeat(281)];
    let result = post_thread_dry_run(&state, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "tweet_too_long");
}

#[tokio::test]
async fn post_thread_dry_run_no_x_client() {
    let state = make_state(None, None).await;
    let tweets = vec!["T1".to_string(), "T2".to_string()];
    let result = post_thread_dry_run(&state, &tweets, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["valid"], true);
    assert_eq!(parsed["data"]["x_client_available"], false);
}

// ── Policy dry-run test ─────────────────────────────────────────────

#[tokio::test]
async fn post_tweet_dry_run_policy_blocked() {
    let config = blocked_config();
    let state =
        make_state_with_config(Some(Box::new(MockXApiClient)), Some("u1".into()), config).await;
    let result = post_tweet_dry_run(&state, "This should be blocked", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    // Policy blocked means policy_would_allow is false
    assert_eq!(parsed["data"]["policy_would_allow"], false);
}
