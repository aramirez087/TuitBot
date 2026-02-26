use crate::kernel::{read, utils};
use crate::tools::test_mocks::{assert_conformant_success, MockProvider};

#[tokio::test]
async fn conformance_read_get_tweet() {
    let json = read::get_tweet(&MockProvider, "t1").await;
    assert_conformant_success(&json, "get_tweet");
}

#[tokio::test]
async fn conformance_read_get_user_by_username() {
    let json = read::get_user_by_username(&MockProvider, "alice").await;
    assert_conformant_success(&json, "get_user_by_username");
}

#[tokio::test]
async fn conformance_read_search_tweets() {
    let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
    assert_conformant_success(&json, "search_tweets");
}

#[tokio::test]
async fn conformance_read_get_user_mentions() {
    let json = read::get_user_mentions(&MockProvider, "u1", None, None).await;
    assert_conformant_success(&json, "get_user_mentions");
}

#[tokio::test]
async fn conformance_read_get_user_tweets() {
    let json = read::get_user_tweets(&MockProvider, "u1", 10, None).await;
    assert_conformant_success(&json, "get_user_tweets");
}

#[tokio::test]
async fn conformance_read_get_home_timeline() {
    let json = read::get_home_timeline(&MockProvider, "u1", 20, None).await;
    assert_conformant_success(&json, "get_home_timeline");
}

#[tokio::test]
async fn conformance_read_get_me() {
    let json = utils::get_me(&MockProvider).await;
    assert_conformant_success(&json, "get_me");
}

#[tokio::test]
async fn conformance_read_get_followers() {
    let json = read::get_followers(&MockProvider, "u1", 10, None).await;
    assert_conformant_success(&json, "get_followers");
}

#[tokio::test]
async fn conformance_read_get_following() {
    let json = read::get_following(&MockProvider, "u1", 10, None).await;
    assert_conformant_success(&json, "get_following");
}

#[tokio::test]
async fn conformance_read_get_user_by_id() {
    let json = read::get_user_by_id(&MockProvider, "u42").await;
    assert_conformant_success(&json, "get_user_by_id");
}

#[tokio::test]
async fn conformance_read_get_liked_tweets() {
    let json = read::get_liked_tweets(&MockProvider, "u1", 10, None).await;
    assert_conformant_success(&json, "get_liked_tweets");
}

#[tokio::test]
async fn conformance_read_get_bookmarks() {
    let json = read::get_bookmarks(&MockProvider, "u1", 10, None).await;
    assert_conformant_success(&json, "get_bookmarks");
}

#[tokio::test]
async fn conformance_read_get_users_by_ids() {
    let json = read::get_users_by_ids(&MockProvider, &["u1", "u2"]).await;
    assert_conformant_success(&json, "get_users_by_ids");
}

#[tokio::test]
async fn conformance_read_get_tweet_liking_users() {
    let json = read::get_tweet_liking_users(&MockProvider, "t1", 10, None).await;
    assert_conformant_success(&json, "get_tweet_liking_users");
}
