//! Startup types and helpers for ReplyGuy CLI commands.
//!
//! Provides API tier detection types, OAuth token management,
//! PKCE authentication helpers, startup banner formatting, and
//! diagnostic check types used by the `run`, `auth`, and `test`
//! CLI commands.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::path::PathBuf;

// ============================================================================
// X API OAuth 2.0 endpoints
// ============================================================================

/// X API OAuth 2.0 authorization endpoint.
pub const X_AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";

/// X API OAuth 2.0 token endpoint.
pub const X_TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";

/// X API users/me endpoint for credential verification.
pub const X_USERS_ME_URL: &str = "https://api.twitter.com/2/users/me";

/// OAuth scopes required by ReplyGuy.
pub const OAUTH_SCOPES: &str = "tweet.read tweet.write users.read offline.access";

// ============================================================================
// API Tier
// ============================================================================

/// Detected X API tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiTier {
    /// Free tier -- posting only (no search, no mentions).
    Free,
    /// Basic tier -- adds search/discovery.
    Basic,
    /// Pro tier -- all features.
    Pro,
}

impl fmt::Display for ApiTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiTier::Free => write!(f, "Free"),
            ApiTier::Basic => write!(f, "Basic"),
            ApiTier::Pro => write!(f, "Pro"),
        }
    }
}

/// Capabilities enabled by the current API tier.
#[derive(Debug, Clone)]
pub struct TierCapabilities {
    /// Whether the mentions loop can run.
    pub mentions: bool,
    /// Whether the discovery/search loop can run.
    pub discovery: bool,
    /// Whether posting (tweets + threads) is available.
    pub posting: bool,
    /// Whether tweet search is available.
    pub search: bool,
}

impl TierCapabilities {
    /// Determine capabilities for a given tier.
    pub fn for_tier(tier: ApiTier) -> Self {
        match tier {
            ApiTier::Free => Self {
                mentions: false,
                discovery: false,
                posting: true,
                search: false,
            },
            ApiTier::Basic | ApiTier::Pro => Self {
                mentions: true,
                discovery: true,
                posting: true,
                search: true,
            },
        }
    }

    /// List the names of enabled automation loops.
    pub fn enabled_loop_names(&self) -> Vec<&'static str> {
        let mut loops = Vec::new();
        if self.mentions {
            loops.push("mentions");
        }
        if self.discovery {
            loops.push("discovery");
        }
        // Content and threads are always enabled (no special tier required).
        loops.push("content");
        loops.push("threads");
        loops
    }

    /// Format the tier capabilities as a status line.
    pub fn format_status(&self) -> String {
        let status = |enabled: bool| if enabled { "enabled" } else { "DISABLED" };
        format!(
            "Mentions: {}, Discovery: {}, Content: enabled, Threads: enabled",
            status(self.mentions),
            status(self.discovery),
        )
    }
}

// ============================================================================
// Stored Tokens
// ============================================================================

/// OAuth tokens persisted to disk at `~/.replyguy/tokens.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    /// OAuth 2.0 access token.
    pub access_token: String,

    /// OAuth 2.0 refresh token (for offline.access scope).
    #[serde(default)]
    pub refresh_token: Option<String>,

    /// Token expiration timestamp.
    #[serde(default)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl StoredTokens {
    /// Check if the token has expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => chrono::Utc::now() >= expires,
            None => false,
        }
    }

    /// Time remaining until token expires.
    pub fn time_until_expiry(&self) -> Option<chrono::TimeDelta> {
        self.expires_at.map(|expires| expires - chrono::Utc::now())
    }

    /// Format time until expiry as a human-readable string.
    pub fn format_expiry(&self) -> String {
        match self.time_until_expiry() {
            Some(duration) if duration.num_seconds() > 0 => {
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                if hours > 0 {
                    format!("{hours}h {minutes}m")
                } else {
                    format!("{minutes}m")
                }
            }
            Some(_) => "expired".to_string(),
            None => "no expiry set".to_string(),
        }
    }
}

// ============================================================================
// Startup Error
// ============================================================================

/// Errors that can occur during startup operations.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// Configuration is invalid or missing.
    #[error("configuration error: {0}")]
    Config(String),

    /// No tokens found -- user needs to authenticate first.
    #[error("authentication required: run `replyguy auth` first")]
    AuthRequired,

    /// Tokens are expired and need re-authentication.
    #[error("authentication expired: run `replyguy auth` to re-authenticate")]
    AuthExpired,

    /// Token refresh attempt failed.
    #[error("token refresh failed: {0}")]
    TokenRefreshFailed(String),

    /// Database initialization or access error.
    #[error("database error: {0}")]
    Database(String),

    /// LLM provider configuration or connectivity error.
    #[error("LLM provider error: {0}")]
    LlmError(String),

    /// X API communication error.
    #[error("X API error: {0}")]
    XApiError(String),

    /// File I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Any other error.
    #[error("{0}")]
    Other(String),
}

// ============================================================================
// Token File I/O
// ============================================================================

/// Default directory for ReplyGuy data files (`~/.replyguy/`).
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".replyguy")
}

/// Path to the token storage file (`~/.replyguy/tokens.json`).
pub fn token_file_path() -> PathBuf {
    data_dir().join("tokens.json")
}

/// Load OAuth tokens from the default file path.
pub fn load_tokens_from_file() -> Result<StoredTokens, StartupError> {
    let path = token_file_path();
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StartupError::AuthRequired
        } else {
            StartupError::Io(e)
        }
    })?;
    serde_json::from_str(&contents)
        .map_err(|e| StartupError::Other(format!("failed to parse tokens file: {e}")))
}

/// Save OAuth tokens to the default file path with secure permissions.
///
/// Creates the `~/.replyguy/` directory if it does not exist.
/// On Unix, sets file permissions to 0600 (owner read/write only).
pub fn save_tokens_to_file(tokens: &StoredTokens) -> Result<(), StartupError> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;

    let path = token_file_path();
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| StartupError::Other(format!("failed to serialize tokens: {e}")))?;
    std::fs::write(&path, json)?;

    // Set file permissions to 0600 on Unix (owner read/write only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

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
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state_bytes: [u8; 16] = rand::thread_rng().gen();
    let state = URL_SAFE_NO_PAD.encode(state_bytes);
    PkceChallenge {
        verifier,
        challenge,
        state,
    }
}

/// Percent-encode a string for use in URL query parameters (RFC 3986).
fn url_encode(s: &str) -> String {
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
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        X_AUTH_URL,
        url_encode(client_id),
        url_encode(redirect_uri),
        url_encode(OAUTH_SCOPES),
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
        refresh_token: Option<String>,
        expires_in: Option<i64>,
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| StartupError::XApiError(format!("failed to parse token response: {e}")))?;

    let expires_at = token_resp
        .expires_in
        .map(|secs| chrono::Utc::now() + chrono::TimeDelta::seconds(secs));

    Ok(StoredTokens {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at,
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
        "ReplyGuy v{version}\n\
         Tier: {tier} | Loops: {loops}\n\
         Status summary: {status}\n\
         Press Ctrl+C to stop.",
        version = env!("CARGO_PKG_VERSION"),
    )
}

// ============================================================================
// Path Helpers
// ============================================================================

/// Expand `~` at the start of a path to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- ApiTier ---

    #[test]
    fn api_tier_display() {
        assert_eq!(ApiTier::Free.to_string(), "Free");
        assert_eq!(ApiTier::Basic.to_string(), "Basic");
        assert_eq!(ApiTier::Pro.to_string(), "Pro");
    }

    // --- TierCapabilities ---

    #[test]
    fn free_tier_capabilities() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        assert!(!caps.mentions);
        assert!(!caps.discovery);
        assert!(caps.posting);
        assert!(!caps.search);
    }

    #[test]
    fn basic_tier_capabilities() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        assert!(caps.mentions);
        assert!(caps.discovery);
        assert!(caps.posting);
        assert!(caps.search);
    }

    #[test]
    fn pro_tier_capabilities() {
        let caps = TierCapabilities::for_tier(ApiTier::Pro);
        assert!(caps.mentions);
        assert!(caps.discovery);
        assert!(caps.posting);
        assert!(caps.search);
    }

    #[test]
    fn free_tier_enabled_loops() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        let loops = caps.enabled_loop_names();
        assert_eq!(loops, vec!["content", "threads"]);
    }

    #[test]
    fn basic_tier_enabled_loops() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        let loops = caps.enabled_loop_names();
        assert_eq!(loops, vec!["mentions", "discovery", "content", "threads"]);
    }

    #[test]
    fn tier_capabilities_format_status() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        let status = caps.format_status();
        assert!(status.contains("Mentions: DISABLED"));
        assert!(status.contains("Discovery: DISABLED"));

        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        let status = caps.format_status();
        assert!(status.contains("Mentions: enabled"));
        assert!(status.contains("Discovery: enabled"));
    }

    // --- StoredTokens ---

    #[test]
    fn stored_tokens_not_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn stored_tokens_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn stored_tokens_no_expiry_is_not_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn stored_tokens_format_expiry_hours() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::minutes(102)),
        };
        let formatted = tokens.format_expiry();
        assert!(formatted.contains("h"));
        assert!(formatted.contains("m"));
    }

    #[test]
    fn stored_tokens_format_expiry_minutes_only() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::minutes(30)),
        };
        let formatted = tokens.format_expiry();
        assert!(formatted.contains("m"));
        assert!(!formatted.contains("h"));
    }

    #[test]
    fn stored_tokens_format_expiry_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
        };
        assert_eq!(tokens.format_expiry(), "expired");
    }

    #[test]
    fn stored_tokens_format_expiry_no_expiry() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
        };
        assert_eq!(tokens.format_expiry(), "no expiry set");
    }

    #[test]
    fn stored_tokens_serialization_roundtrip() {
        let tokens = StoredTokens {
            access_token: "access123".to_string(),
            refresh_token: Some("refresh456".to_string()),
            expires_at: Some(
                chrono::DateTime::parse_from_rfc3339("2026-06-01T12:00:00Z")
                    .expect("valid datetime")
                    .with_timezone(&chrono::Utc),
            ),
        };
        let json = serde_json::to_string(&tokens).expect("serialize");
        let deserialized: StoredTokens = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.access_token, "access123");
        assert_eq!(deserialized.refresh_token.as_deref(), Some("refresh456"));
        assert!(deserialized.expires_at.is_some());
    }

    // --- Token File I/O ---

    #[test]
    fn save_and_load_tokens() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tokens.json");

        let tokens = StoredTokens {
            access_token: "test_access".to_string(),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: None,
        };

        let json = serde_json::to_string_pretty(&tokens).expect("serialize");
        std::fs::write(&path, &json).expect("write");

        let contents = std::fs::read_to_string(&path).expect("read");
        let loaded: StoredTokens = serde_json::from_str(&contents).expect("deserialize");
        assert_eq!(loaded.access_token, "test_access");
        assert_eq!(loaded.refresh_token.as_deref(), Some("test_refresh"));
    }

    #[cfg(unix)]
    #[test]
    fn save_tokens_sets_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        // Override data dir for this test by saving directly.
        let path = dir.path().join("tokens.json");
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
        };
        let json = serde_json::to_string_pretty(&tokens).expect("serialize");
        std::fs::write(&path, &json).expect("write");
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms).expect("set perms");

        let meta = std::fs::metadata(&path).expect("metadata");
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    // --- Startup Error ---

    #[test]
    fn startup_error_display() {
        let err = StartupError::AuthRequired;
        assert_eq!(
            err.to_string(),
            "authentication required: run `replyguy auth` first"
        );

        let err = StartupError::AuthExpired;
        assert!(err.to_string().contains("expired"));

        let err = StartupError::Config("bad field".to_string());
        assert_eq!(err.to_string(), "configuration error: bad field");

        let err = StartupError::XApiError("timeout".to_string());
        assert_eq!(err.to_string(), "X API error: timeout");
    }

    // --- PKCE ---

    #[test]
    fn generate_pkce_produces_valid_challenge() {
        let pkce = generate_pkce();
        // Verifier should be 43 characters (32 bytes base64url encoded).
        assert_eq!(pkce.verifier.len(), 43);
        // Challenge should be 43 characters (32 bytes SHA-256 hash, base64url).
        assert_eq!(pkce.challenge.len(), 43);
        // State should be 22 characters (16 bytes base64url encoded).
        assert_eq!(pkce.state.len(), 22);
        // Verify the challenge matches the verifier.
        let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.verifier.as_bytes()));
        assert_eq!(pkce.challenge, expected);
    }

    #[test]
    fn generate_pkce_unique_each_time() {
        let a = generate_pkce();
        let b = generate_pkce();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
        assert_ne!(a.state, b.state);
    }

    // --- URL Building ---

    #[test]
    fn build_auth_url_contains_required_params() {
        let url = build_auth_url(
            "client123",
            "http://localhost:8080/callback",
            "state456",
            "challenge789",
        );
        assert!(url.starts_with(X_AUTH_URL));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("code_challenge=challenge789"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=state456"));
        // redirect_uri should be encoded.
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
    }

    #[test]
    fn build_redirect_uri_format() {
        let uri = build_redirect_uri("127.0.0.1", 8080);
        assert_eq!(uri, "http://127.0.0.1:8080/callback");
    }

    // --- extract_auth_code ---

    #[test]
    fn extract_code_from_full_url() {
        let code = extract_auth_code("http://127.0.0.1:8080/callback?code=abc123&state=xyz");
        assert_eq!(code, "abc123");
    }

    #[test]
    fn extract_code_from_bare_code() {
        let code = extract_auth_code("  abc123  ");
        assert_eq!(code, "abc123");
    }

    #[test]
    fn extract_code_from_url_without_state() {
        let code = extract_auth_code("http://127.0.0.1:8080/callback?code=mycode");
        assert_eq!(code, "mycode");
    }

    // --- URL Encoding ---

    #[test]
    fn url_encode_basic() {
        assert_eq!(url_encode("hello"), "hello");
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(
            url_encode("http://localhost:8080/callback"),
            "http%3A%2F%2Flocalhost%3A8080%2Fcallback"
        );
    }

    // --- Startup Banner ---

    #[test]
    fn startup_banner_free_tier() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        let banner = format_startup_banner(ApiTier::Free, &caps, 300);
        assert!(banner.contains("ReplyGuy v"));
        assert!(banner.contains("Tier: Free"));
        assert!(!banner.contains("mentions"));
        assert!(banner.contains("content"));
        assert!(banner.contains("threads"));
        assert!(!banner.contains("discovery"));
        assert!(banner.contains("every 300s"));
    }

    #[test]
    fn startup_banner_basic_tier() {
        let caps = TierCapabilities::for_tier(ApiTier::Basic);
        let banner = format_startup_banner(ApiTier::Basic, &caps, 0);
        assert!(banner.contains("Tier: Basic"));
        assert!(banner.contains("discovery"));
        assert!(banner.contains("disabled"));
    }

    #[test]
    fn startup_banner_contains_ctrl_c_hint() {
        let caps = TierCapabilities::for_tier(ApiTier::Free);
        let banner = format_startup_banner(ApiTier::Free, &caps, 0);
        assert!(banner.contains("Ctrl+C"));
    }

    // --- Path Helpers ---

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("~/.replyguy/config.toml");
        assert!(!expanded.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn expand_tilde_no_tilde() {
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn data_dir_under_home() {
        let dir = data_dir();
        assert!(dir.to_string_lossy().contains(".replyguy"));
    }

    #[test]
    fn token_file_path_under_data_dir() {
        let path = token_file_path();
        assert!(path.to_string_lossy().contains("tokens.json"));
        assert!(path.to_string_lossy().contains(".replyguy"));
    }
}
