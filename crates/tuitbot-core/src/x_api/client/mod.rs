//! Reqwest-based X API v2 HTTP client implementation.
//!
//! Provides `XApiHttpClient` which implements the `XApiClient` trait
//! using reqwest for HTTP requests with proper error mapping and
//! rate limit header parsing.

mod trait_impl;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::XApiError;
use crate::safety::redact::redact_secrets;
use crate::storage::{self, DbPool};

use super::types::{RateLimitInfo, XApiErrorResponse};

/// Default X API v2 base URL.
const DEFAULT_BASE_URL: &str = "https://api.x.com/2";

/// Default X API v1.1 media upload base URL.
const DEFAULT_UPLOAD_BASE_URL: &str = "https://upload.twitter.com/1.1";

/// Standard tweet fields requested on every query.
pub(crate) const TWEET_FIELDS: &str = "public_metrics,created_at,author_id,conversation_id";

/// Standard expansions requested on every query.
pub(crate) const EXPANSIONS: &str = "author_id";

/// Standard user fields requested on every query.
pub(crate) const USER_FIELDS: &str = "username,public_metrics";

/// HTTP client for the X API v2.
///
/// Uses reqwest with Bearer token authentication. The access token
/// is stored behind an `Arc<RwLock>` so the token manager can
/// update it transparently after a refresh.
pub struct XApiHttpClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) upload_base_url: String,
    pub(crate) access_token: Arc<RwLock<String>>,
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
    pub(crate) fn parse_rate_limit_headers(headers: &reqwest::header::HeaderMap) -> RateLimitInfo {
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
    pub(crate) async fn map_error_response(response: reqwest::Response) -> XApiError {
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
    pub(crate) fn record_usage(&self, path: &str, method: &str, status_code: u16) {
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
    pub(crate) async fn get(
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
    pub(crate) async fn delete(&self, path: &str) -> Result<reqwest::Response, XApiError> {
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
    pub(crate) async fn post_json<T: serde::Serialize>(
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
