//! Token refresh response types and refresh logic.

use serde::Deserialize;

/// Response from the OAuth 2.0 token refresh endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: String,
}
