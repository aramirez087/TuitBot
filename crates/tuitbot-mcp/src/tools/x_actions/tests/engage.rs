//! Engage tool tests (like, follow, retweet) + policy gate tests.

use super::*;

// ── Engage success tests ────────────────────────────────────────────

#[tokio::test]
async fn like_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = like_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["liked"], true);
}

#[tokio::test]
async fn follow_user_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = follow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["following"], true);
}

#[tokio::test]
async fn unfollow_user_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = unfollow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["following"], false);
}

#[tokio::test]
async fn retweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = retweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["retweeted"], true);
}

#[tokio::test]
async fn unretweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = unretweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["retweeted"], false);
}

// ── Policy gate tests ───────────────────────────────────────────────

#[tokio::test]
async fn post_tweet_blocked_by_policy() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        blocked_config(),
    )
    .await;
    let result = post_tweet(&state, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "policy_denied_blocked");
}

#[tokio::test]
async fn post_tweet_routed_to_approval() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        approval_config(),
    )
    .await;
    let result = post_tweet(&state, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["routed_to_approval"], true);
    assert!(parsed["data"]["approval_queue_id"].is_number());
}

#[tokio::test]
async fn post_tweet_allowed_when_not_gated() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        allowed_config(),
    )
    .await;
    let result = post_tweet(&state, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "new_1");
}

#[tokio::test]
async fn dry_run_returns_would_execute() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        dry_run_config(),
    )
    .await;
    let result = post_tweet(&state, "Hello!", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["would_execute"], "post_tweet");
}

#[tokio::test]
async fn composer_mode_forces_approval_for_all() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        composer_config(),
    )
    .await;
    let result = unfollow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["routed_to_approval"], true);
}

#[tokio::test]
async fn delete_tweet_always_requires_approval() {
    // The hard rule at priority 0 forces RequireApproval for Delete category,
    // regardless of blocked_tools configuration.
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        blocked_config(),
    )
    .await;
    let result = delete_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["routed_to_approval"], true);
}
