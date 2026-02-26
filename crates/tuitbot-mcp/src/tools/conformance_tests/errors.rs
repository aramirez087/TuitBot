use serde_json::Value;

use crate::kernel::{read, utils};
use crate::tools::test_mocks::{assert_conformant_error, ErrorProvider};

#[tokio::test]
async fn conformance_error_rate_limited() {
    let json = read::search_tweets(&ErrorProvider, "test", 10, None, None).await;
    assert_conformant_error(&json, "search_tweets/rate_limited", "x_rate_limited");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["error"]["retry_after_ms"], 60000,
        "retry_after_ms should be 60000 for 60s retry"
    );
}

#[tokio::test]
async fn conformance_error_auth_expired() {
    let json = read::get_user_by_username(&ErrorProvider, "nobody").await;
    assert_conformant_error(&json, "get_user_by_username/auth_expired", "x_auth_expired");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed["error"].get("retry_after_ms").is_none(),
        "auth errors should not have retry_after_ms"
    );
}

#[tokio::test]
async fn conformance_error_network() {
    let json = read::get_followers(&ErrorProvider, "u1", 10, None).await;
    assert_conformant_error(&json, "get_followers/network", "x_network_error");
}

#[tokio::test]
async fn conformance_error_other() {
    let json = read::get_tweet(&ErrorProvider, "missing").await;
    assert_conformant_error(&json, "get_tweet/other", "x_api_error");
}

#[tokio::test]
async fn conformance_error_get_me_auth_expired() {
    let json = utils::get_me(&ErrorProvider).await;
    assert_conformant_error(&json, "get_me/auth_expired", "x_auth_expired");
}
