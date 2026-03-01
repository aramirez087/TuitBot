//! Remote connector module for OAuth-based source linking.
//!
//! Provides the `RemoteConnector` trait and implementations for
//! linking, refreshing, and revoking user-owned connections to
//! remote services (e.g. Google Drive). Credentials are encrypted
//! at rest via AES-256-GCM (see `crypto` submodule).

pub mod crypto;
pub mod google_drive;
#[cfg(test)]
mod tests;

use async_trait::async_trait;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from remote connector operations.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("connector not configured: {0}")]
    NotConfigured(String),

    #[error("invalid OAuth state")]
    InvalidState,

    #[error("token exchange failed: {0}")]
    TokenExchange(String),

    #[error("token refresh failed: {0}")]
    TokenRefresh(String),

    #[error("revocation failed: {0}")]
    Revocation(String),

    #[error("encryption error: {0}")]
    Encryption(String),

    #[error("storage error: {source}")]
    Storage {
        #[from]
        source: crate::error::StorageError,
    },

    #[error("network error: {0}")]
    Network(String),
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

/// Tokens received from an initial OAuth code exchange.
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_secs: i64,
    pub scope: String,
}

/// A refreshed access token (refresh token stays the same).
pub struct RefreshedToken {
    pub access_token: String,
    pub expires_in_secs: i64,
}

/// Basic user identity from the OAuth provider.
pub struct UserInfo {
    pub email: String,
    pub display_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Abstraction over OAuth-based remote connectors.
///
/// Each connector knows how to build authorization URLs, exchange
/// codes for tokens, refresh access tokens, revoke connections,
/// and fetch basic user info.
#[async_trait]
pub trait RemoteConnector: Send + Sync {
    /// Returns the connector type identifier (e.g. `"google_drive"`).
    fn connector_type(&self) -> &str;

    /// Build the OAuth authorization URL the user should visit.
    fn authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, ConnectorError>;

    /// Exchange an authorization code for tokens.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<TokenSet, ConnectorError>;

    /// Refresh an access token using encrypted refresh-token material.
    async fn refresh_access_token(
        &self,
        encrypted_refresh: &[u8],
        key: &[u8],
    ) -> Result<RefreshedToken, ConnectorError>;

    /// Revoke a connection (best-effort).
    async fn revoke(&self, encrypted_refresh: &[u8], key: &[u8]) -> Result<(), ConnectorError>;

    /// Fetch basic user identity from the provider.
    async fn user_info(&self, access_token: &str) -> Result<UserInfo, ConnectorError>;
}
