//! OAuth 2.0 PKCE authentication and token management for X API.
//!
//! Supports two authentication modes:
//! - **Manual**: User copies an authorization URL, pastes the code back.
//! - **Local callback**: CLI starts a temporary HTTP server to capture the code.
//!
//! Token management handles persistent storage, loading, and automatic
//! refresh before expiry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::XApiError;

mod oauth;
mod refresh;
mod token;

pub use oauth::{authenticate_callback, authenticate_manual};
pub use refresh::TokenRefreshResponse;
pub use token::TokenManager;

// ───────────────────────────────────────────────────────────────

/// X API OAuth 2.0 authorization endpoint.
pub const AUTH_URL: &str = "https://x.com/i/oauth2/authorize";

/// X API OAuth 2.0 token endpoint.
pub const TOKEN_URL: &str = "https://api.x.com/2/oauth2/token";

/// Pre-expiry refresh window in seconds (5 minutes).
pub const REFRESH_WINDOW_SECS: i64 = 300;

// ───────────────────────────────────────────────────────────────

/// Stored OAuth tokens with expiration tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    /// The Bearer access token.
    pub access_token: String,
    /// The refresh token for obtaining new access tokens.
    pub refresh_token: String,
    /// When the access token expires (UTC).
    pub expires_at: DateTime<Utc>,
    /// Granted OAuth scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
}

// ───────────────────────────────────────────────────────────────

/// Save tokens to disk as JSON with restricted permissions.
pub fn save_tokens(tokens: &Tokens, path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| format!("Failed to serialize tokens: {e}"))?;

    // Write token file with restricted permissions from the start (no TOCTOU window)
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| format!("Failed to create token file: {e}"))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write tokens: {e}"))?;
    }

    #[cfg(not(unix))]
    {
        std::fs::write(path, &json).map_err(|e| format!("Failed to write tokens: {e}"))?;
        tracing::warn!("Cannot set restrictive file permissions on non-Unix platform");
    }

    Ok(())
}

/// Load tokens from disk. Returns `None` if the file does not exist.
pub fn load_tokens(path: &std::path::Path) -> Result<Option<Tokens>, XApiError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let tokens: Tokens =
                serde_json::from_str(&contents).map_err(|e| XApiError::ApiError {
                    status: 0,
                    message: format!("Failed to parse tokens file: {e}"),
                })?;
            Ok(Some(tokens))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(XApiError::ApiError {
            status: 0,
            message: format!("Failed to read tokens file: {e}"),
        }),
    }
}

#[cfg(test)]
mod tests;
