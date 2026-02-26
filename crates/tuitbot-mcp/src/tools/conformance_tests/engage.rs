use crate::kernel::engage;
use crate::tools::test_mocks::{assert_conformant_success, MockXApiClient};

#[tokio::test]
async fn conformance_engage_like_tweet() {
    let json = engage::like_tweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "like_tweet");
}

#[tokio::test]
async fn conformance_engage_unlike_tweet() {
    let json = engage::unlike_tweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "unlike_tweet");
}

#[tokio::test]
async fn conformance_engage_follow_user() {
    let json = engage::follow_user(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "follow_user");
}

#[tokio::test]
async fn conformance_engage_unfollow_user() {
    let json = engage::unfollow_user(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "unfollow_user");
}

#[tokio::test]
async fn conformance_engage_retweet() {
    let json = engage::retweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "retweet");
}

#[tokio::test]
async fn conformance_engage_unretweet() {
    let json = engage::unretweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "unretweet");
}

#[tokio::test]
async fn conformance_engage_bookmark_tweet() {
    let json = engage::bookmark_tweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "bookmark_tweet");
}

#[tokio::test]
async fn conformance_engage_unbookmark_tweet() {
    let json = engage::unbookmark_tweet(&MockXApiClient, "u1", "t1").await;
    assert_conformant_success(&json, "unbookmark_tweet");
}
