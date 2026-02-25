//! Reqwest-based X API v2 HTTP client implementation.
//!
//! Provides `XApiHttpClient` which implements the `XApiClient` trait
//! using reqwest for HTTP requests with proper error mapping and
//! rate limit header parsing.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::XApiError;
use crate::safety::redact::redact_secrets;
use crate::storage::{self, DbPool};

use super::types::{
    ActionResultResponse, DeleteTweetResponse, FollowUserRequest, LikeTweetRequest, MediaId,
    MediaPayload, MediaType, MentionResponse, PostTweetRequest, PostTweetResponse, PostedTweet,
    RateLimitInfo, ReplyTo, RetweetRequest, SearchResponse, SingleTweetResponse, Tweet, User,
    UserResponse, XApiErrorResponse,
};
use super::XApiClient;

/// Default X API v2 base URL.
const DEFAULT_BASE_URL: &str = "https://api.x.com/2";

/// Default X API v1.1 media upload base URL.
const DEFAULT_UPLOAD_BASE_URL: &str = "https://upload.twitter.com/1.1";

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
    upload_base_url: String,
    access_token: Arc<RwLock<String>>,
    pool: Arc<RwLock<Option<DbPool>>>,
}

impl XApiHttpClient {
    /// Create a new X API HTTP client with the given access token.
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            upload_base_url: DEFAULT_UPLOAD_BASE_URL.to_string(),
            access_token: Arc::new(RwLock::new(access_token)),
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new client with a custom base URL (for testing with wiremock).
    pub fn with_base_url(access_token: String, base_url: String) -> Self {
        let upload_base_url = base_url.clone();
        Self {
            client: reqwest::Client::new(),
            base_url,
            upload_base_url,
            access_token: Arc::new(RwLock::new(access_token)),
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the database pool for usage tracking.
    ///
    /// Called after DB initialization to enable fire-and-forget recording
    /// of every X API call.
    pub async fn set_pool(&self, pool: DbPool) {
        let mut lock = self.pool.write().await;
        *lock = Some(pool);
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

        let raw_body = response.text().await.unwrap_or_default();
        let error_detail = serde_json::from_str::<XApiErrorResponse>(&raw_body).ok();
        let body = redact_secrets(&raw_body);

        let message = error_detail
            .as_ref()
            .and_then(|e| e.detail.clone())
            .unwrap_or_else(|| body.clone());
        let message = redact_secrets(&message);

        match status {
            429 => {
                let retry_after = rate_info.reset_at.and_then(|reset| {
                    let now = chrono::Utc::now().timestamp() as u64;
                    reset.checked_sub(now)
                });
                XApiError::RateLimited { retry_after }
            }
            401 => XApiError::AuthExpired,
            403 if Self::is_scope_insufficient_message(&message) => {
                XApiError::ScopeInsufficient { message }
            }
            403 => XApiError::Forbidden { message },
            _ => XApiError::ApiError { status, message },
        }
    }

    fn is_scope_insufficient_message(message: &str) -> bool {
        let normalized = message.to_ascii_lowercase();
        normalized.contains("scope")
            && (normalized.contains("insufficient")
                || normalized.contains("missing")
                || normalized.contains("not granted")
                || normalized.contains("required"))
    }

    /// Record an API call in the usage tracking table (fire-and-forget).
    fn record_usage(&self, path: &str, method: &str, status_code: u16) {
        let pool_lock = self.pool.clone();
        let endpoint = path.to_string();
        let http_method = method.to_string();
        let cost = storage::x_api_usage::estimate_cost(&endpoint, &http_method);
        // Only record successful calls for cost (failed requests don't incur charges per X docs).
        let final_cost = if status_code < 400 { cost } else { 0.0 };
        tokio::spawn(async move {
            if let Some(pool) = pool_lock.read().await.as_ref() {
                if let Err(e) = storage::x_api_usage::insert_x_api_usage(
                    pool,
                    &endpoint,
                    &http_method,
                    status_code as i32,
                    final_cost,
                )
                .await
                {
                    tracing::warn!(error = %e, "Failed to record X API usage");
                }
            }
        });
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

        let status_code = response.status().as_u16();
        let rate_info = Self::parse_rate_limit_headers(response.headers());
        tracing::debug!(
            path,
            remaining = ?rate_info.remaining,
            reset_at = ?rate_info.reset_at,
            "X API response"
        );

        self.record_usage(path, "GET", status_code);

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::map_error_response(response).await)
        }
    }

    /// Send a DELETE request and handle common error patterns.
    async fn delete(&self, path: &str) -> Result<reqwest::Response, XApiError> {
        let token = self.access_token.read().await;
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&*token)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        let status_code = response.status().as_u16();
        let rate_info = Self::parse_rate_limit_headers(response.headers());
        tracing::debug!(
            path,
            remaining = ?rate_info.remaining,
            reset_at = ?rate_info.reset_at,
            "X API response"
        );

        self.record_usage(path, "DELETE", status_code);

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

        let status_code = response.status().as_u16();
        let rate_info = Self::parse_rate_limit_headers(response.headers());
        tracing::debug!(
            path,
            remaining = ?rate_info.remaining,
            reset_at = ?rate_info.reset_at,
            "X API response"
        );

        self.record_usage(path, "POST", status_code);

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
        pagination_token: Option<&str>,
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

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
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
        pagination_token: Option<&str>,
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

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
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
            media: None,
            quote_tweet_id: None,
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
            media: None,
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn upload_media(&self, data: &[u8], media_type: MediaType) -> Result<MediaId, XApiError> {
        super::media::upload_media(
            &self.client,
            &self.upload_base_url,
            &self.access_token.read().await,
            data,
            media_type,
        )
        .await
    }

    async fn post_tweet_with_media(
        &self,
        text: &str,
        media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(
            chars = text.len(),
            media_count = media_ids.len(),
            "Posting tweet with media"
        );
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
            media: Some(MediaPayload {
                media_ids: media_ids.to_vec(),
            }),
            quote_tweet_id: None,
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn reply_to_tweet_with_media(
        &self,
        text: &str,
        in_reply_to_id: &str,
        media_ids: &[String],
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(in_reply_to = %in_reply_to_id, chars = text.len(), media_count = media_ids.len(), "Posting reply with media");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: Some(ReplyTo {
                in_reply_to_tweet_id: in_reply_to_id.to_string(),
            }),
            media: Some(MediaPayload {
                media_ids: media_ids.to_vec(),
            }),
            quote_tweet_id: None,
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
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        let path = format!("/users/{user_id}/tweets");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
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

    async fn quote_tweet(
        &self,
        text: &str,
        quoted_tweet_id: &str,
    ) -> Result<PostedTweet, XApiError> {
        tracing::debug!(chars = text.len(), quoted = %quoted_tweet_id, "Posting quote tweet");
        let body = PostTweetRequest {
            text: text.to_string(),
            reply: None,
            media: None,
            quote_tweet_id: Some(quoted_tweet_id.to_string()),
        };

        let response = self.post_json("/tweets", &body).await?;
        let resp: PostTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data)
    }

    async fn like_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Liking tweet");
        let path = format!("/users/{user_id}/likes");
        let body = LikeTweetRequest {
            tweet_id: tweet_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn follow_user(&self, user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, target = %target_user_id, "Following user");
        let path = format!("/users/{user_id}/following");
        let body = FollowUserRequest {
            target_user_id: target_user_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn unfollow_user(&self, user_id: &str, target_user_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, target = %target_user_id, "Unfollowing user");
        let path = format!("/users/{user_id}/following/{target_user_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn retweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Retweeting");
        let path = format!("/users/{user_id}/retweets");
        let body = RetweetRequest {
            tweet_id: tweet_id.to_string(),
        };

        let response = self.post_json(&path, &body).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn unretweet(&self, user_id: &str, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(user_id = %user_id, tweet_id = %tweet_id, "Unretweeting");
        let path = format!("/users/{user_id}/retweets/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: ActionResultResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.result)
    }

    async fn delete_tweet(&self, tweet_id: &str) -> Result<bool, XApiError> {
        tracing::debug!(tweet_id = %tweet_id, "Deleting tweet");
        let path = format!("/tweets/{tweet_id}");

        let response = self.delete(&path).await?;
        let resp: DeleteTweetResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;
        Ok(resp.data.deleted)
    }

    async fn get_home_timeline(
        &self,
        user_id: &str,
        max_results: u32,
        pagination_token: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        tracing::debug!(user_id = %user_id, max_results = max_results, "Getting home timeline");
        let path = format!("/users/{user_id}/timelines/reverse_chronological");
        let max_str = max_results.to_string();
        let mut params = vec![
            ("max_results", max_str.as_str()),
            ("tweet.fields", TWEET_FIELDS),
            ("expansions", EXPANSIONS),
            ("user.fields", USER_FIELDS),
        ];

        let pagination_token_owned;
        if let Some(pt) = pagination_token {
            pagination_token_owned = pt.to_string();
            params.push(("pagination_token", &pagination_token_owned));
        }

        let response = self.get(&path, &params).await?;
        response
            .json::<SearchResponse>()
            .await
            .map_err(|e| XApiError::Network { source: e })
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
            .respond_with(
                ResponseTemplate::new(500).set_body_json(
                    serde_json::json!({"detail": "access_token=abc123 Authorization: Bearer secrettoken"}),
                ),
            )
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
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"detail": "Unauthorized"})),
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
}
