use serde_json::Value;

use crate::kernel::{engage, read, write};
use crate::tools::test_mocks::MockXApiClient;

use super::generation::{GoldenErrorProvider, GoldenMockProvider};

#[tokio::test]
async fn golden_single_tweet_has_required_keys() {
    let json = read::get_tweet(&GoldenMockProvider, "t1").await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let data = &parsed["data"];
    for key in ["id", "text", "author_id", "created_at", "public_metrics"] {
        assert!(data.get(key).is_some(), "get_tweet missing key: {key}");
    }
}

#[tokio::test]
async fn golden_single_user_has_required_keys() {
    let json = read::get_user_by_username(&GoldenMockProvider, "alice").await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let data = &parsed["data"];
    for key in ["id", "username", "name", "public_metrics"] {
        assert!(
            data.get(key).is_some(),
            "get_user_by_username missing key: {key}"
        );
    }
}

#[tokio::test]
async fn golden_tweet_list_has_data_and_meta() {
    let json = read::search_tweets(&GoldenMockProvider, "q", 10, None, None).await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["data"]["data"].is_array(), "missing data.data[]");
    assert!(parsed["data"]["meta"].is_object(), "missing data.meta");
    assert!(
        parsed["meta"]["pagination"].is_object(),
        "missing meta.pagination"
    );
}

#[tokio::test]
async fn golden_users_list_has_data_and_meta() {
    let json = read::get_followers(&GoldenMockProvider, "u1", 10, None).await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["data"]["data"].is_array(), "missing data.data[]");
    assert!(parsed["data"]["meta"].is_object(), "missing data.meta");
    assert!(
        parsed["meta"]["pagination"].is_object(),
        "missing meta.pagination"
    );
}

#[tokio::test]
async fn golden_write_result_has_id_and_text() {
    let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["data"]["id"].is_string(), "missing data.id");
    assert!(parsed["data"]["text"].is_string(), "missing data.text");
}

#[tokio::test]
async fn golden_engage_result_has_action_and_id() {
    let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["data"]["liked"].is_boolean(), "missing data.liked");
    assert!(
        parsed["data"]["tweet_id"].is_string(),
        "missing data.tweet_id"
    );
}

#[tokio::test]
async fn golden_error_rate_limited_has_retry_after() {
    let json = read::get_tweet(&GoldenErrorProvider, "t1").await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed["error"]["retry_after_ms"].is_number(),
        "missing error.retry_after_ms"
    );
    assert_eq!(parsed["error"]["code"], "x_rate_limited");
    assert!(parsed["error"]["retryable"].as_bool().unwrap());
}

#[tokio::test]
async fn golden_error_auth_no_retry_after() {
    let json = read::get_user_by_username(&GoldenErrorProvider, "u").await;
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["error"]["code"], "x_auth_expired");
    assert!(!parsed["error"]["retryable"].as_bool().unwrap());
    assert!(
        parsed["error"].get("retry_after_ms").is_none(),
        "auth error should not have retry_after_ms"
    );
}
