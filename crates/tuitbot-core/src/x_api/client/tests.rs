use super::*;
use crate::error::XApiError;
use crate::x_api::XApiClient;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_client(server: &MockServer) -> XApiHttpClient {
    XApiHttpClient::with_base_url("test-token".to_string(), server.uri())
}

#[tokio::test]
async fn search_tweets_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .and(query_param("query", "rust"))
        .and(query_param("max_results", "10"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "1", "text": "Rust is great", "author_id": "a1"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let result = client.search_tweets("rust", 10, None, None).await;
    let resp = result.expect("search");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].text, "Rust is great");
}

#[tokio::test]
async fn search_tweets_with_since_id() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .and(query_param("since_id", "999"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "meta": {"result_count": 0}
        })))
        .mount(&server)
        .await;

    let result = client.search_tweets("test", 10, Some("999"), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn post_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/tweets"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "new_123", "text": "Hello world"}
        })))
        .mount(&server)
        .await;

    let result = client.post_tweet("Hello world").await;
    let tweet = result.expect("post");
    assert_eq!(tweet.id, "new_123");
}

#[tokio::test]
async fn reply_to_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/tweets"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "reply_1", "text": "Nice point!"}
        })))
        .mount(&server)
        .await;

    let result = client.reply_to_tweet("Nice point!", "original_1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_me_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "u1",
                "username": "testuser",
                "name": "Test User",
                "public_metrics": {
                    "followers_count": 100,
                    "following_count": 50,
                    "tweet_count": 500
                }
            }
        })))
        .mount(&server)
        .await;

    let user = client.get_me().await.expect("get me");
    assert_eq!(user.username, "testuser");
    assert_eq!(user.public_metrics.followers_count, 100);
}

#[tokio::test]
async fn error_429_maps_to_rate_limited() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({"detail": "Too Many Requests"})),
        )
        .mount(&server)
        .await;

    let result = client.search_tweets("test", 10, None, None).await;
    assert!(matches!(result, Err(XApiError::RateLimited { .. })));
}

#[tokio::test]
async fn error_401_maps_to_auth_expired() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({"detail": "Unauthorized"})),
        )
        .mount(&server)
        .await;

    let result = client.get_me().await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));
}

#[tokio::test]
async fn error_403_maps_to_forbidden() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            serde_json::json!({"detail": "You are not permitted to use this endpoint"}),
        ))
        .mount(&server)
        .await;

    let result = client.search_tweets("test", 10, None, None).await;
    match result {
        Err(XApiError::Forbidden { message }) => {
            assert!(message.contains("not permitted"));
        }
        other => panic!("expected Forbidden, got: {other:?}"),
    }
}

#[tokio::test]
async fn error_403_scope_message_maps_to_scope_insufficient() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            serde_json::json!({"detail": "Missing required OAuth scope: tweet.write"}),
        ))
        .mount(&server)
        .await;

    let result = client.search_tweets("test", 10, None, None).await;
    match result {
        Err(XApiError::ScopeInsufficient { message }) => {
            assert!(message.contains("scope"));
        }
        other => panic!("expected ScopeInsufficient, got: {other:?}"),
    }
}

#[tokio::test]
async fn error_500_maps_to_api_error() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({"detail": "Internal Server Error"})),
        )
        .mount(&server)
        .await;

    let result = client.get_me().await;
    match result {
        Err(XApiError::ApiError { status, .. }) => assert_eq!(status, 500),
        other => panic!("expected ApiError, got: {other:?}"),
    }
}

#[tokio::test]
async fn error_messages_are_redacted() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(ResponseTemplate::new(500).set_body_json(
            serde_json::json!({"detail": "access_token=abc123 Authorization: Bearer secrettoken"}),
        ))
        .mount(&server)
        .await;

    let result = client.get_me().await;
    match result {
        Err(XApiError::ApiError { message, .. }) => {
            assert!(!message.contains("abc123"));
            assert!(!message.contains("secrettoken"));
            assert!(message.contains("***REDACTED***"));
        }
        other => panic!("expected ApiError, got: {other:?}"),
    }
}

#[tokio::test]
async fn parse_rate_limit_headers_works() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-rate-limit-remaining", "42".parse().unwrap());
    headers.insert("x-rate-limit-reset", "1700000000".parse().unwrap());

    let info = XApiHttpClient::parse_rate_limit_headers(&headers);
    assert_eq!(info.remaining, Some(42));
    assert_eq!(info.reset_at, Some(1700000000));
}

#[tokio::test]
async fn set_access_token_updates() {
    let client = XApiHttpClient::new("old-token".to_string());
    client.set_access_token("new-token".to_string()).await;

    let token = client.access_token.read().await;
    assert_eq!(*token, "new-token");
}

#[tokio::test]
async fn get_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "12345",
                "text": "Hello",
                "author_id": "a1",
                "public_metrics": {"like_count": 5, "retweet_count": 1, "reply_count": 0, "quote_count": 0}
            }
        })))
        .mount(&server)
        .await;

    let tweet = client.get_tweet("12345").await.expect("get tweet");
    assert_eq!(tweet.id, "12345");
    assert_eq!(tweet.public_metrics.like_count, 5);
}

#[tokio::test]
async fn get_mentions_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/mentions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "m1", "text": "@testuser hello", "author_id": "a2"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_mentions("u1", None, None)
        .await
        .expect("mentions");
    assert_eq!(resp.data.len(), 1);
}

#[tokio::test]
async fn quote_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/tweets"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "qt_1", "text": "Great thread! https://x.com/user/status/999"}
        })))
        .mount(&server)
        .await;

    let result = client.quote_tweet("Great thread!", "999").await;
    let tweet = result.expect("quote tweet");
    assert_eq!(tweet.id, "qt_1");
}

#[tokio::test]
async fn like_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/users/u1/likes"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"liked": true}
        })))
        .mount(&server)
        .await;

    let result = client.like_tweet("u1", "t1").await.expect("like");
    assert!(result);
}

#[tokio::test]
async fn follow_user_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/users/u1/following"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"following": true}
        })))
        .mount(&server)
        .await;

    let result = client.follow_user("u1", "target1").await.expect("follow");
    assert!(result);
}

#[tokio::test]
async fn unfollow_user_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/users/u1/following/target1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"following": false}
        })))
        .mount(&server)
        .await;

    let result = client
        .unfollow_user("u1", "target1")
        .await
        .expect("unfollow");
    assert!(!result);
}

#[tokio::test]
async fn like_tweet_rate_limited() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/users/u1/likes"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({"detail": "Too Many Requests"})),
        )
        .mount(&server)
        .await;

    let result = client.like_tweet("u1", "t1").await;
    assert!(matches!(result, Err(XApiError::RateLimited { .. })));
}

#[tokio::test]
async fn unfollow_user_auth_expired() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/users/u1/following/target1"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({"detail": "Unauthorized"})),
        )
        .mount(&server)
        .await;

    let result = client.unfollow_user("u1", "target1").await;
    assert!(matches!(result, Err(XApiError::AuthExpired)));
}

#[tokio::test]
async fn retweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/users/u1/retweets"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"retweeted": true}
        })))
        .mount(&server)
        .await;

    let result = client.retweet("u1", "t1").await.expect("retweet");
    assert!(result);
}

#[tokio::test]
async fn unretweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/users/u1/retweets/t1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"retweeted": false}
        })))
        .mount(&server)
        .await;

    let result = client.unretweet("u1", "t1").await.expect("unretweet");
    assert!(!result);
}

#[tokio::test]
async fn delete_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/tweets/t1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"deleted": true}
        })))
        .mount(&server)
        .await;

    let result = client.delete_tweet("t1").await.expect("delete");
    assert!(result);
}

#[tokio::test]
async fn get_home_timeline_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/timelines/reverse_chronological"))
        .and(query_param("max_results", "10"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "ht1", "text": "Home tweet", "author_id": "a1"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_home_timeline("u1", 10, None)
        .await
        .expect("home timeline");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].text, "Home tweet");
}

#[tokio::test]
async fn unlike_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/users/u1/likes/t1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"liked": false}
        })))
        .mount(&server)
        .await;

    let result = client.unlike_tweet("u1", "t1").await.expect("unlike");
    assert!(!result);
}

#[tokio::test]
async fn get_followers_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/followers"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "f1", "username": "follower1", "name": "Follower One"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_followers("u1", 10, None)
        .await
        .expect("followers");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].username, "follower1");
}

#[tokio::test]
async fn get_following_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/following"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "fw1", "username": "following1", "name": "Following One"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_following("u1", 10, None)
        .await
        .expect("following");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].username, "following1");
}

#[tokio::test]
async fn get_user_by_id_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "u123",
                "username": "iduser",
                "name": "ID User",
                "public_metrics": {"followers_count": 42, "following_count": 10, "tweet_count": 99}
            }
        })))
        .mount(&server)
        .await;

    let user = client.get_user_by_id("u123").await.expect("user by id");
    assert_eq!(user.id, "u123");
    assert_eq!(user.username, "iduser");
    assert_eq!(user.public_metrics.followers_count, 42);
}

#[tokio::test]
async fn get_liked_tweets_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/liked_tweets"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "lt1", "text": "Liked tweet", "author_id": "a1"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_liked_tweets("u1", 10, None)
        .await
        .expect("liked tweets");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].text, "Liked tweet");
}

#[tokio::test]
async fn get_bookmarks_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users/u1/bookmarks"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "bk1", "text": "Bookmarked tweet", "author_id": "a1"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_bookmarks("u1", 10, None)
        .await
        .expect("bookmarks");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].text, "Bookmarked tweet");
}

#[tokio::test]
async fn bookmark_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("POST"))
        .and(path("/users/u1/bookmarks"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"bookmarked": true}
        })))
        .mount(&server)
        .await;

    let result = client.bookmark_tweet("u1", "t1").await.expect("bookmark");
    assert!(result);
}

#[tokio::test]
async fn unbookmark_tweet_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/users/u1/bookmarks/t1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"bookmarked": false}
        })))
        .mount(&server)
        .await;

    let result = client
        .unbookmark_tweet("u1", "t1")
        .await
        .expect("unbookmark");
    assert!(!result);
}

#[tokio::test]
async fn get_users_by_ids_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/users"))
        .and(query_param("ids", "u1,u2,u3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": "u1", "username": "alice", "name": "Alice"},
                {"id": "u2", "username": "bob", "name": "Bob"},
                {"id": "u3", "username": "carol", "name": "Carol"}
            ],
            "meta": {"result_count": 3}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_users_by_ids(&["u1", "u2", "u3"])
        .await
        .expect("users by ids");
    assert_eq!(resp.data.len(), 3);
    assert_eq!(resp.data[0].username, "alice");
}

#[tokio::test]
async fn get_tweet_liking_users_success() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/t1/liking_users"))
        .and(query_param("max_results", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "lu1", "username": "liker1", "name": "Liker One"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let resp = client
        .get_tweet_liking_users("t1", 10, None)
        .await
        .expect("liking users");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].username, "liker1");
}

#[tokio::test]
async fn search_tweets_with_pagination_token() {
    let server = MockServer::start().await;
    let client = setup_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/tweets/search/recent"))
        .and(query_param("pagination_token", "next_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "p1", "text": "Page 2 tweet", "author_id": "a1"}],
            "meta": {"result_count": 1}
        })))
        .mount(&server)
        .await;

    let result = client
        .search_tweets("test", 10, None, Some("next_abc"))
        .await;
    let resp = result.expect("search with pagination");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].id, "p1");
}
