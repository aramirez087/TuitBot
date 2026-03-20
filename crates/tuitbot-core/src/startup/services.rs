//! PKCE generation, OAuth URL building, token exchange, credential verification,
//! and startup banner formatting.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::config::{ApiTier, StartupError, StoredTokens, TierCapabilities};

// ============================================================================
// X API OAuth 2.0 endpoints (re-exported via mod.rs)
// ============================================================================

/// X API OAuth 2.0 authorization endpoint.
pub const X_AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";

/// X API OAuth 2.0 token endpoint.
pub const X_TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";

/// X API users/me endpoint for credential verification.
pub const X_USERS_ME_URL: &str = "https://api.twitter.com/2/users/me";

// ============================================================================
// PKCE Authentication
// ============================================================================

/// PKCE code verifier and challenge pair.
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    /// The code verifier (sent during token exchange).
    pub verifier: String,
    /// The code challenge (sent during authorization).
    pub challenge: String,
    /// CSRF state parameter.
    pub state: String,
}

/// Generate a PKCE code verifier, challenge, and state parameter.
pub fn generate_pkce() -> PkceChallenge {
    use rand::Rng;
    let random_bytes: [u8; 32] = rand::rng().random();
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state_bytes: [u8; 16] = rand::rng().random();
    let state = URL_SAFE_NO_PAD.encode(state_bytes);
    PkceChallenge {
        verifier,
        challenge,
        state,
    }
}

/// Percent-encode a string for use in URL query parameters (RFC 3986).
pub(super) fn url_encode(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

/// Build the X API OAuth 2.0 authorization URL.
pub fn build_auth_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    use crate::x_api::scopes::REQUIRED_SCOPES;
    let oauth_scopes = REQUIRED_SCOPES.join(" ");
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&prompt=consent",
        X_AUTH_URL,
        url_encode(client_id),
        url_encode(redirect_uri),
        url_encode(&oauth_scopes),
        url_encode(state),
        url_encode(code_challenge),
    )
}

/// Build the redirect URI from config auth settings.
pub fn build_redirect_uri(callback_host: &str, callback_port: u16) -> String {
    format!("http://{callback_host}:{callback_port}/callback")
}

/// Exchange an authorization code for OAuth tokens.
pub async fn exchange_auth_code(
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<StoredTokens, StartupError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(X_TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
            ("client_id", client_id),
        ])
        .send()
        .await
        .map_err(|e| StartupError::XApiError(format!("token exchange request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(StartupError::XApiError(format!(
            "token exchange failed (HTTP {status}): {body}"
        )));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        #[serde(default)]
        refresh_token: Option<String>,
        #[serde(default)]
        expires_in: Option<i64>,
        #[serde(default)]
        scope: Option<String>,
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| StartupError::XApiError(format!("failed to parse token response: {e}")))?;

    let expires_at = token_resp
        .expires_in
        .map(|secs| chrono::Utc::now() + chrono::TimeDelta::seconds(secs));
    let scopes = token_resp
        .scope
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default();

    Ok(StoredTokens {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at,
        scopes,
    })
}

/// Verify OAuth credentials by calling the X API /2/users/me endpoint.
///
/// Returns the authenticated user's username on success.
pub async fn verify_credentials(access_token: &str) -> Result<String, StartupError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(X_USERS_ME_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            StartupError::XApiError(format!("credential verification request failed: {e}"))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(StartupError::XApiError(format!(
            "credential verification failed (HTTP {status}): {body}"
        )));
    }

    #[derive(Deserialize)]
    struct UserResponse {
        data: UserData,
    }

    #[derive(Deserialize)]
    struct UserData {
        username: String,
    }

    let user: UserResponse = resp
        .json()
        .await
        .map_err(|e| StartupError::XApiError(format!("failed to parse user response: {e}")))?;

    Ok(user.data.username)
}

/// Extract the authorization code from a callback URL or raw code string.
///
/// Accepts either a full URL (e.g., `http://127.0.0.1:8080/callback?code=XXX&state=YYY`)
/// or a bare authorization code.
pub fn extract_auth_code(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.contains("code=") {
        // Parse code from URL query parameters.
        if let Some(query) = trimmed.split('?').nth(1) {
            for pair in query.split('&') {
                if let Some(value) = pair.strip_prefix("code=") {
                    return value.to_string();
                }
            }
        }
    }
    trimmed.to_string()
}

/// Extract the `state` parameter from a callback URL or query string.
///
/// Returns an empty string if no `state` parameter is found.
pub fn extract_callback_state(input: &str) -> String {
    let query = if let Some(q) = input.split('?').nth(1) {
        // Strip HTTP version suffix if present (e.g. " HTTP/1.1").
        q.split_whitespace().next().unwrap_or(q)
    } else {
        input.trim()
    };
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("state=") {
            return value.to_string();
        }
    }
    String::new()
}

// ============================================================================
// Startup Banner
// ============================================================================

/// Format the startup banner printed when the agent starts.
pub fn format_startup_banner(
    tier: ApiTier,
    capabilities: &TierCapabilities,
    status_interval: u64,
) -> String {
    let loops = capabilities.enabled_loop_names().join(", ");
    let status = if status_interval > 0 {
        format!("every {status_interval}s")
    } else {
        "disabled".to_string()
    };
    format!(
        "Tuitbot v{version}\n\
         Tier: {tier} | Loops: {loops}\n\
         Status summary: {status}\n\
         Press Ctrl+C to stop.",
        version = env!("CARGO_PKG_VERSION"),
    )
}
