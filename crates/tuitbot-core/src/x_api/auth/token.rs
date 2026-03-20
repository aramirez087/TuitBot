//! Token manager: persistence, loading, and automatic refresh.

use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;

use crate::error::XApiError;

use super::{save_tokens, TokenRefreshResponse, Tokens, REFRESH_WINDOW_SECS, TOKEN_URL};

/// Manages token persistence, loading, and automatic refresh.
pub struct TokenManager {
    tokens: Arc<RwLock<Tokens>>,
    client_id: String,
    http_client: reqwest::Client,
    token_path: std::path::PathBuf,
    /// Serializes refresh attempts so only one runs at a time.
    /// X API refresh tokens are single-use, so concurrent refreshes
    /// would invalidate the token used by the second caller.
    refresh_lock: tokio::sync::Mutex<()>,
}

impl TokenManager {
    /// Create a new token manager with the given tokens and client configuration.
    pub fn new(tokens: Tokens, client_id: String, token_path: std::path::PathBuf) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(tokens)),
            client_id,
            http_client: reqwest::Client::new(),
            token_path,
            refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Get the current access token, refreshing if needed.
    pub async fn get_access_token(&self) -> Result<String, XApiError> {
        self.refresh_if_needed().await?;
        let tokens = self.tokens.read().await;
        Ok(tokens.access_token.clone())
    }

    /// Get a shared reference to the tokens lock for direct access.
    pub fn tokens_lock(&self) -> Arc<RwLock<Tokens>> {
        self.tokens.clone()
    }

    /// Refresh the access token if it is within 5 minutes of expiring.
    ///
    /// Acquires `refresh_lock` to prevent concurrent refresh attempts.
    /// X API refresh tokens are single-use, so a second concurrent refresh
    /// with the old token would fail and revoke the session.
    pub async fn refresh_if_needed(&self) -> Result<(), XApiError> {
        // Fast path: no refresh needed.
        {
            let tokens = self.tokens.read().await;
            let seconds_until_expiry = tokens
                .expires_at
                .signed_duration_since(Utc::now())
                .num_seconds();
            if seconds_until_expiry >= REFRESH_WINDOW_SECS {
                return Ok(());
            }
        }

        // Serialize refresh attempts.
        let _guard = self.refresh_lock.lock().await;

        // Re-check after acquiring the lock — another caller may have
        // already refreshed while we were waiting.
        {
            let tokens = self.tokens.read().await;
            let seconds_until_expiry = tokens
                .expires_at
                .signed_duration_since(Utc::now())
                .num_seconds();
            if seconds_until_expiry >= REFRESH_WINDOW_SECS {
                return Ok(());
            }
        }

        self.do_refresh().await
    }

    /// Perform the token refresh.
    async fn do_refresh(&self) -> Result<(), XApiError> {
        let refresh_token = {
            let tokens = self.tokens.read().await;
            tokens.refresh_token.clone()
        };

        tracing::info!("Refreshing X API access token");

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.client_id),
        ];

        let response = self
            .http_client
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                status,
                body_len = body.len(),
                "Token refresh failed (response body redacted)"
            );
            return Err(XApiError::AuthExpired);
        }

        let body: TokenRefreshResponse = response
            .json()
            .await
            .map_err(|e| XApiError::Network { source: e })?;

        let new_tokens = Tokens {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_at: Utc::now() + chrono::Duration::seconds(body.expires_in),
            scopes: body
                .scope
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        };

        tracing::info!(
            expires_at = %new_tokens.expires_at,
            "Token refreshed successfully"
        );

        // Update in memory
        {
            let mut tokens = self.tokens.write().await;
            *tokens = new_tokens.clone();
        }

        // Persist to disk
        save_tokens(&new_tokens, &self.token_path).map_err(|e| {
            tracing::error!(error = %e, "Failed to save refreshed tokens");
            XApiError::ApiError {
                status: 0,
                message: format!("Failed to save tokens: {e}"),
            }
        })?;

        Ok(())
    }
}
