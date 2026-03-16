//! Read tool tests + error mapping + not-configured edge cases.

use super::*;

#[tokio::test]
async fn get_tweet_by_id_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_tweet_by_id(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "t1");
    assert!(parsed["meta"]["elapsed_ms"].is_number());
}

#[tokio::test]
async fn get_user_by_username_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_user_by_username(&state, "testuser").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["username"], "testuser");
}

#[tokio::test]
async fn search_tweets_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = search_tweets(&state, "rust", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["meta"]["result_count"], 1);
}

#[tokio::test]
async fn get_home_timeline_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_home_timeline(&state, 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["text"], "Home tweet");
}

#[tokio::test]
async fn get_x_usage_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_x_usage(&state, 7).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert!(parsed["data"]["summary"].is_object());
}

#[tokio::test]
async fn search_tweets_with_pagination() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = search_tweets(&state, "rust", 10, None, Some("next_token")).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
}

// ── Error mapping tests ─────────────────────────────────────────────

#[tokio::test]
async fn error_maps_rate_limited() {
    let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
    let result = search_tweets(&state, "test", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert_eq!(parsed["error"]["retryable"], true);
}

#[tokio::test]
async fn error_maps_auth_expired() {
    let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
    let result = get_user_mentions(&state, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn error_maps_forbidden() {
    let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
    let result = post_tweet(&state, "test", None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_forbidden");
}

#[tokio::test]
async fn error_maps_api_error() {
    let state = make_state(Some(Box::new(ErrorXApiClient)), Some("u1".into())).await;
    let result = get_tweet_by_id(&state, "nonexistent").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_api_error");
}

// ── Not configured / missing user ID ────────────────────────────────

#[tokio::test]
async fn x_not_configured_when_no_client() {
    let state = make_state(None, None).await;
    let result = get_tweet_by_id(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
    assert_eq!(parsed["error"]["retryable"], false);
}

#[tokio::test]
async fn like_tweet_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = like_tweet(&state, "t1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_by_id_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_user_by_id(&state, "uid123").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["id"], "uid123");
}

#[tokio::test]
async fn get_followers_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_followers(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "follower1");
}

#[tokio::test]
async fn get_following_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_following(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "following1");
}

#[tokio::test]
async fn get_liked_tweets_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_liked_tweets(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "lt1");
}

#[tokio::test]
async fn get_bookmarks_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_bookmarks(&state, 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["id"], "bk1");
}

#[tokio::test]
async fn get_bookmarks_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = get_bookmarks(&state, 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_users_by_ids_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_users_by_ids(&state, &["id1", "id2"]).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_tweet_liking_users_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_tweet_liking_users(&state, "t1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["data"][0]["username"], "liker1");
}

#[tokio::test]
async fn get_user_tweets_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_user_tweets(&state, "u2", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
}

#[tokio::test]
async fn get_user_mentions_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = get_user_mentions(&state, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_mentions_success() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = get_user_mentions(&state, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
    assert!(parsed["data"]["meta"]["result_count"].is_number());
}

#[tokio::test]
async fn get_home_timeline_no_user_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), None).await;
    let result = get_home_timeline(&state, 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn search_tweets_not_configured() {
    let state = make_state(None, None).await;
    let result = search_tweets(&state, "test", 10, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_by_username_not_configured() {
    let state = make_state(None, None).await;
    let result = get_user_by_username(&state, "someone").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_followers_not_configured() {
    let state = make_state(None, None).await;
    let result = get_followers(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_following_not_configured() {
    let state = make_state(None, None).await;
    let result = get_following(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_liked_tweets_not_configured() {
    let state = make_state(None, None).await;
    let result = get_liked_tweets(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_by_id_not_configured() {
    let state = make_state(None, None).await;
    let result = get_user_by_id(&state, "u1").await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_users_by_ids_not_configured() {
    let state = make_state(None, None).await;
    let result = get_users_by_ids(&state, &["id1"]).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_tweet_liking_users_not_configured() {
    let state = make_state(None, None).await;
    let result = get_tweet_liking_users(&state, "t1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_tweets_not_configured() {
    let state = make_state(None, None).await;
    let result = get_user_tweets(&state, "u1", 10, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn get_user_mentions_not_configured() {
    let state = make_state(None, None).await;
    let result = get_user_mentions(&state, None, None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_not_configured");
}

#[tokio::test]
async fn search_tweets_with_since_id() {
    let state = make_state(Some(Box::new(MockXApiClient)), Some("u1".into())).await;
    let result = search_tweets(&state, "rust", 10, Some("since123"), None).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["success"], true);
}
