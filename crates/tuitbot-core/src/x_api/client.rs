//! Reqwest-based X API v2 HTTP client implementation.
//!
//! Provides `XApiHttpClient` which implements the `XApiClient` trait
//! using reqwest for HTTP requests with proper error mapping and
//! rate limit header parsing.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::XApiError;

use super::types::{
    MentionResponse, PostTweetRequest, PostTweetResponse, PostedTweet, RateLimitInfo, ReplyTo,
    SearchResponse, SingleTweetResponse, Tweet, User, UserResponse, XApiErrorResponse,
};
use super::XApiClient;

/// Default X API v2 base URL.
const DEFAULT_BASE_URL: &str = "https://api.x.com/2";

/// Standard tweet fields requested on every query.
const TWEET_FIELDS: &str = "public_metrics,created_at,author_id,conversation_id";

/// Standard expansions requested on every query.
const EXPANSIONS: &str = "author_id";

/// Standard user fields requested on every query.
const USER_FIELDS: &str = "username,public_metrics";

/// HTTP client for the X API v2.
///
/// Uses reqwest with Bearer token authentication. The access token
/// is stored behind an `Arc<RwLock>` so the token manager can
/// update it transparently after a refresh.
pub struct XApiHttpClient {
    client: reqwest::Client,
    base_url: String,
    access_token: Arc<RwLock<String>>,
}

impl XApiHttpClient {
    /// Create a new X API HTTP client with the given access token.
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            access_token: Arc::new(RwLock::new(access_token)),
        }
    }

    /// Create a new client with a custom base URL (for testing with wiremock).
    pub fn with_base_url(access_token: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            access_token: Arc::new(RwLock::new(access_token)),
        }
    }

    /// Get a shared reference to the access token lock for token manager integration.
    pub fn access_token_lock(&self) -> Arc<RwLock<String>> {
        self.access_token.clone()
    }

    /// Update the access token (used by token manager after refresh).
    pub async fn set_access_token(&self, token: String) {
        let mut lock = self.access_token.write().await;
        *lock = token;
    }

    /// Parse rate limit headers from an X API response.
    fn parse_rate_limit_headers(headers: &reqwest::header::HeaderMap) -> RateLimitInfo {
        let remaining = headers
            .get("x-rate-limit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let reset_at = headers
            .get("x-rate-limit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        RateLimitInfo {
            remaining,
            reset_at,
        }
    }

    /// Map an HTTP error response to a typed `XApiError`.
    async fn map_error_response(response: reqwest::Response) -> XApiError {
        let status = response.status().as_u16();
        let rate_info = Self::parse_rate_limit_headers(response.headers());

        let body = response.text().await.unwrap_or_default();
        let error_detail = serde_json::from_str::<XApiErrorResponse>(&body).ok();

        let message = error_detail
            .as_ref()
            .and_then(|e| e.detail.clone())
            .unwrap_or_else(|| body.clone());

        match status {
            429 => {
                let retry_after = rate_info.reset_at.and_then(|reset| {
                    let now = chrono::Utc::now().timestamp() as u64;
                    reset.checked_sub(now)
                });
                XApiError::RateLimited { retry_after }
            }
            401 => XApiError::AuthExpired,
            403 => XApiError::Forbidden { message },
            _ => XApiError::ApiError { status, message },
        }
    }

    /// Send a GET request and handle common error patterns.
    async fn get(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<reqwest::Response, XApiError> {
        let token = self.access_token.read().await;
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&*token)
            .query(query)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        let rate_info = Self::parse_rate_limit_headers(response.headers());
        tracing::debug!(
            path,
            remaining = ?rate_info.remaining,
            reset_at = ?rate_info.reset_at,
            "X API response"
        );

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::map_error_response(response).await)
        }
    }

    /// Send a POST request with JSON body and handle common error patterns.
    async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, XApiError> {
        let token = self.access_token.read().await;
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&*token)
            .json(body)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        let rate_info = Self::parse_rate_limit_headers(response.headers());
        tracing::debug!(
            path,
            remaining = ?rate_info.remaining,
            reset_at = ?rate_info.reset_at,
            "X API response"
        );

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::map_error_response(response).await)
        }
    }
}

#[async_trait::async_trait]
impl XApiClient for XApiHttpClient {
    async fn search_tweets(
        &self,
        query: &str,
        max_results: u32,
        since_id: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(query = %query, max_results = max_results, "Search tweets");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("query", query),
            ("max_results", &max_str),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let since_id_owned;
        if let Some(sid) = since_id {
            since_id_owned = sid.to_string();
            params.push(("since_id", &since_id_owned));
        }

        let response = self.get("/tweets/search/recent", &params).await?;
        let resp: SearchResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        tracing::debug!(
            query = %query,
            results = resp.data.len(),
            "Search tweets completed",
        );
        Ok(resp)
    }

    async fn get_mentions(
        &self,
        user_id: &str,
        since_id: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        let path = format!("/users/{user_id}/mentions");
        let mut params = vec![
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let since_id_owned;
        if let Some(sid) = since_id {
            since_id_owned = sid.to_string();
            params.push(("since_id", &since_id_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<MentionResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        tracing::debug!(chars = text.len(), "Posting tweet");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn reply_to_tweet(
        &self,
        text: &str,
        in_reply_to_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(in_reply_to = %in_reply_to_id, chars = text.len(), "Posting reply");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: in_reply_to_id.to_string(),
            }),
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, XApiError> {
        let path = format!("/tweets/{tweet_id}");
        let params = [
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let response = self.get(&path, &params).await?;
        let resp: SingleTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_me(&self) -> Result<User, XApiError> {
        let params = [("user.fields", USER_FIELDS)];

        let response = self.get("/users/me", &params).await?;
        let resp: UserResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn get_user_tweets(
        &self,
        user_id: &str,
        max_results: u32,
    ) -> Result<SearchResponse, XApiError> {
        let path = format!("/users/{user_id}/tweets");
        let max_str = max_results.to_string();
        let params = [
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
    }

    async fn follow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Result<(), XApiError> {
        let path = format!("/users/{source_user_id}/following");
        let body = serde_json::json!({ "target_user_id": target_user_id });

        self.post_json(&path, &body).await?;
        Ok(())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, XApiError> {
        let path = format!("/users/by/username/{username}");
        let params = [("user.fields", USER_FIELDS)];

        let response = self.get(&path, &params).await?;
        let resp: UserResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let result = client.search_tweets("rust", 10, None).await;
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

        let result = client.search_tweets("test", 10, Some("999")).await;
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

        let result = client.search_tweets("test", 10, None).await;
        assert!(matches!(result, Err(XApiError::RateLimited { .. })));
    }

    #[tokio::test]
    async fn error_401_maps_to_auth_expired() {
        let server = MockServer::start().await;
        let client = setup_client(&server).await;

        Mock::given(method("GET"))
            .and(path("/users/me"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"detail": "Unauthorized"})),
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

        let result = client.search_tweets("test", 10, None).await;
        match result {
            Err(XApiError::Forbidden { message }) => {
                assert!(message.contains("not permitted"));
            }
            other => panic!("expected Forbidden, got: {other:?}"),
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

        let resp = client.get_mentions("u1", None).await.expect("mentions");
        assert_eq!(resp.data.len(), 1);
    }
}
