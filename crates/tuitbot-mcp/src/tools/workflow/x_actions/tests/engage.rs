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

#[tokio::test]
async fn unlike_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = unlike_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["liked"], false);
}

#[tokio::test]
async fn bookmark_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = bookmark_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["bookmarked"], true);
}

#[tokio::test]
async fn unbookmark_tweet_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = unbookmark_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["bookmarked"], false);
}

#[tokio::test]
async fn like_tweet_not_configured() {
    let state = make_state(None, Some("u1".into())).await;
    let result = like_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn follow_user_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = follow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn retweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = retweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn unretweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = unretweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

// ── More not-configured / no-user-id edge cases ─────────────────────

#[tokio::test]
async fn bookmark_tweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = bookmark_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn unbookmark_tweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = unbookmark_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn unlike_tweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = unlike_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn follow_user_not_configured() {
    let state = make_state(None, Some("u1".into())).await;
    let result = follow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn unfollow_user_not_configured() {
    let state = make_state(None, Some("u1".into())).await;
    let result = unfollow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn retweet_not_configured() {
    let state = make_state(None, Some("u1".into())).await;
    let result = retweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn unretweet_not_configured() {
    let state = make_state(None, Some("u1".into())).await;
    let result = unretweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

// ── Policy gate on engage tools ─────────────────────────────────────

#[tokio::test]
async fn like_tweet_blocked_by_policy() {
    let mut config = blocked_config();
    config.mcp_policy.blocked_tools = vec!["like_tweet".to_string()];
    let state =
        make_state_with_config(Some(Box::new(MockXApiClient)), Some("u1".into()), config).await;
    let result = like_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "policy_denied_blocked");
}

#[tokio::test]
async fn follow_user_routed_to_approval() {
    let mut config = approval_config();
    config.mcp_policy.require_approval_for = vec!["follow_user".to_string()];
    let state =
        make_state_with_config(Some(Box::new(MockXApiClient)), Some("u1".into()), config).await;
    let result = follow_user(&state, "target1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["routed_to_approval"], true);
}

#[tokio::test]
async fn retweet_dry_run_returns_would_execute() {
    let state = make_state_with_config(
        Some(Box::new(MockXApiClient)),
        Some("u1".into()),
        dry_run_config(),
    )
    .await;
    let result = retweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["dry_run"], true);
    assert_eq!(parsed["data"]["would_execute"], "retweet");
}
