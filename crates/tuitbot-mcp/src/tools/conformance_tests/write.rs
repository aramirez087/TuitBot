use crate::kernel::write;
use crate::tools::test_mocks::{assert_conformant_success, MockXApiClient};

#[tokio::test]
async fn conformance_write_post_tweet() {
    let json = write::post_tweet(&MockXApiClient, "Hello!", None).await;
    assert_conformant_success(&json, "post_tweet");
}

#[tokio::test]
async fn conformance_write_reply_to_tweet() {
    let json = write::reply_to_tweet(&MockXApiClient, "Great!", "t1", None).await;
    assert_conformant_success(&json, "reply_to_tweet");
}

#[tokio::test]
async fn conformance_write_quote_tweet() {
    let json = write::quote_tweet(&MockXApiClient, "So true!", "t1").await;
    assert_conformant_success(&json, "quote_tweet");
}

#[tokio::test]
async fn conformance_write_delete_tweet() {
    let json = write::delete_tweet(&MockXApiClient, "t1").await;
    assert_conformant_success(&json, "delete_tweet");
}

#[tokio::test]
async fn conformance_write_post_thread() {
    let tweets = vec!["First".to_string(), "Second".to_string()];
    let json = write::post_thread(&MockXApiClient, &tweets, None).await;
    assert_conformant_success(&json, "post_thread");
}
