//! Startup types and helpers for Tuitbot CLI commands.
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

use crate::x_api::scopes::{self, ScopeAnalysis, REQUIRED_SCOPES};

// ============================================================================
// X API OAuth 2.0 endpoints
// ============================================================================

/// X API OAuth 2.0 authorization endpoint.
pub const X_AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";

/// X API OAuth 2.0 token endpoint.
pub const X_TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";

/// X API users/me endpoint for credential verification.
pub const X_USERS_ME_URL: &str = "https://api.twitter.com/2/users/me";

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

/// OAuth tokens persisted to disk at `~/.tuitbot/tokens.json`.
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

    /// Granted OAuth scopes returned by X during token exchange.
    #[serde(default)]
    pub scopes: Vec<String>,
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

    /// Whether this token file includes scope metadata.
    pub fn has_scope_info(&self) -> bool {
        !self.scopes.is_empty()
    }

    /// Check whether a specific scope is granted.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|granted| granted == scope)
    }

    /// Analyze granted scopes versus required Tuitbot scopes.
    pub fn analyze_scopes(&self) -> ScopeAnalysis {
        scopes::analyze_scopes(&self.scopes)
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
    #[error("authentication required: run `tuitbot auth` first")]
    AuthRequired,

    /// Tokens are expired and need re-authentication.
    #[error("authentication expired: run `tuitbot auth` to re-authenticate")]
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

/// Default directory for Tuitbot data files (`~/.tuitbot/`).
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tuitbot")
}

/// Path to the token storage file (`~/.tuitbot/tokens.json`).
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
/// Creates the `~/.tuitbot/` directory if it does not exist.
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

/// Resolve the database path by loading the config file and reading `storage.db_path`.
///
/// Falls back to `~/.tuitbot/tuitbot.db` if the config cannot be loaded.
/// Returns an error if the resolved `db_path` is empty, whitespace-only,
/// or points to an existing directory.
pub fn resolve_db_path(config_path: &str) -> Result<PathBuf, crate::error::ConfigError> {
    use crate::config::Config;
    let config = match Config::load(Some(config_path)) {
        Ok(c) => c,
        Err(_) => return Ok(data_dir().join("tuitbot.db")),
    };

    validate_db_path(&config.storage.db_path)
}

/// Validate and expand a `storage.db_path` value.
///
/// Rejects empty, whitespace-only, and directory paths with a clear error.
pub fn validate_db_path(raw: &str) -> Result<PathBuf, crate::error::ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(crate::error::ConfigError::InvalidValue {
            field: "storage.db_path".to_string(),
            message: "must not be empty or whitespace-only".to_string(),
        });
    }
    let expanded = expand_tilde(trimmed);
    if expanded.is_dir() {
        return Err(crate::error::ConfigError::InvalidValue {
            field: "storage.db_path".to_string(),
            message: format!("'{}' is a directory, must point to a file", trimmed),
        });
    }
    Ok(expanded)
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
            scopes: vec![],
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn stored_tokens_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() - chrono::TimeDelta::hours(1)),
            scopes: vec![],
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn stored_tokens_no_expiry_is_not_expired() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn stored_tokens_format_expiry_hours() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::minutes(102)),
            scopes: vec![],
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
            scopes: vec![],
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
            scopes: vec![],
        };
        assert_eq!(tokens.format_expiry(), "expired");
    }

    #[test]
    fn stored_tokens_format_expiry_no_expiry() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
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
            scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
        };
        let json = serde_json::to_string(&tokens).expect("serialize");
        let deserialized: StoredTokens = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.access_token, "access123");
        assert_eq!(deserialized.refresh_token.as_deref(), Some("refresh456"));
        assert!(deserialized.expires_at.is_some());
        assert_eq!(
            deserialized.scopes,
            vec!["tweet.read".to_string(), "tweet.write".to_string()]
        );
    }

    #[test]
    fn stored_tokens_deserialize_without_scopes_defaults_empty() {
        let json = r#"{
            "access_token": "access123",
            "refresh_token": "refresh456",
            "expires_at": "2026-06-01T12:00:00Z"
        }"#;

        let tokens: StoredTokens = serde_json::from_str(json).expect("deserialize");
        assert!(tokens.scopes.is_empty());
        assert!(!tokens.has_scope_info());
    }

    #[test]
    fn stored_tokens_scope_helpers_work() {
        let tokens = StoredTokens {
            access_token: "access123".to_string(),
            refresh_token: Some("refresh456".to_string()),
            expires_at: None,
            scopes: vec!["tweet.read".to_string(), "users.read".to_string()],
        };

        assert!(tokens.has_scope_info());
        assert!(tokens.has_scope("tweet.read"));
        assert!(!tokens.has_scope("tweet.write"));
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
            scopes: vec!["tweet.read".to_string()],
        };

        let json = serde_json::to_string_pretty(&tokens).expect("serialize");
        std::fs::write(&path, &json).expect("write");

        let contents = std::fs::read_to_string(&path).expect("read");
        let loaded: StoredTokens = serde_json::from_str(&contents).expect("deserialize");
        assert_eq!(loaded.access_token, "test_access");
        assert_eq!(loaded.refresh_token.as_deref(), Some("test_refresh"));
        assert_eq!(loaded.scopes, vec!["tweet.read".to_string()]);
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
            scopes: vec![],
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
            "authentication required: run `tuitbot auth` first"
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
        assert!(banner.contains("Tuitbot v"));
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
        let expanded = expand_tilde("~/.tuitbot/config.toml");
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
        assert!(dir.to_string_lossy().contains(".tuitbot"));
    }

    #[test]
    fn token_file_path_under_data_dir() {
        let path = token_file_path();
        assert!(path.to_string_lossy().contains("tokens.json"));
        assert!(path.to_string_lossy().contains(".tuitbot"));
    }

    // --- extract_callback_state ---

    #[test]
    fn extract_callback_state_from_url() {
        let state =
            extract_callback_state("http://127.0.0.1:8080/callback?code=abc123&state=mystate456");
        assert_eq!(state, "mystate456");
    }

    #[test]
    fn extract_callback_state_no_state() {
        let state = extract_callback_state("http://127.0.0.1:8080/callback?code=abc123");
        assert_eq!(state, "");
    }

    #[test]
    fn extract_callback_state_state_only() {
        let state = extract_callback_state("state=xyz789");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn extract_callback_state_with_http_suffix() {
        // Simulates raw HTTP request line
        let state = extract_callback_state("/callback?code=abc&state=test123 HTTP/1.1");
        assert_eq!(state, "test123");
    }

    // --- validate_db_path ---

    #[test]
    fn validate_db_path_empty_rejected() {
        let result = validate_db_path("");
        assert!(result.is_err());
    }

    #[test]
    fn validate_db_path_whitespace_rejected() {
        let result = validate_db_path("   ");
        assert!(result.is_err());
    }

    #[test]
    fn validate_db_path_valid_path() {
        let result = validate_db_path("/tmp/test.db");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test.db"));
    }

    #[test]
    fn validate_db_path_tilde_expansion() {
        let result = validate_db_path("~/.tuitbot/test.db");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.to_string_lossy().starts_with('~'));
        assert!(path.to_string_lossy().contains("test.db"));
    }

    #[test]
    fn validate_db_path_directory_rejected() {
        // Use a known existing directory (cross-platform)
        let tmp = std::env::temp_dir();
        let result = validate_db_path(tmp.to_str().unwrap());
        assert!(result.is_err());
    }

    // --- expand_tilde edge cases ---

    #[test]
    fn expand_tilde_bare_tilde() {
        let expanded = expand_tilde("~");
        // Should expand to home directory, not just "~"
        assert!(!expanded.to_string_lossy().ends_with('~') || expanded == PathBuf::from("~"));
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home);
        }
    }

    #[test]
    fn expand_tilde_relative_path() {
        let expanded = expand_tilde("relative/path");
        assert_eq!(expanded, PathBuf::from("relative/path"));
    }

    // --- StoredTokens edge cases ---

    #[test]
    fn stored_tokens_analyze_scopes_returns_analysis() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec!["tweet.read".to_string(), "tweet.write".to_string()],
        };
        let analysis = tokens.analyze_scopes();
        // Should complete without panic and return valid analysis
        assert!(
            !analysis.granted.is_empty()
                || analysis.missing.is_empty()
                || !analysis.missing.is_empty()
        );
    }

    #[test]
    fn stored_tokens_time_until_expiry_some() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(2)),
            scopes: vec![],
        };
        let duration = tokens.time_until_expiry();
        assert!(duration.is_some());
        assert!(duration.unwrap().num_minutes() > 100);
    }

    #[test]
    fn stored_tokens_time_until_expiry_none() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec![],
        };
        assert!(tokens.time_until_expiry().is_none());
    }

    // --- StartupError variants ---

    #[test]
    fn startup_error_all_variants_display() {
        let errors = vec![
            StartupError::Config("bad".to_string()),
            StartupError::AuthRequired,
            StartupError::AuthExpired,
            StartupError::TokenRefreshFailed("fail".to_string()),
            StartupError::Database("db err".to_string()),
            StartupError::LlmError("llm err".to_string()),
            StartupError::XApiError("api err".to_string()),
            StartupError::Other("other".to_string()),
        ];
        for err in &errors {
            let msg = err.to_string();
            assert!(!msg.is_empty());
        }
    }

    // --- URL encoding edge cases ---

    #[test]
    fn url_encode_special_chars() {
        assert_eq!(url_encode("a b+c"), "a%20b%2Bc");
        assert_eq!(url_encode("foo@bar"), "foo%40bar");
        assert_eq!(url_encode("~valid_chars.-"), "~valid_chars.-");
    }

    #[test]
    fn url_encode_empty() {
        assert_eq!(url_encode(""), "");
    }

    // --- resolve_db_path ---

    #[test]
    fn resolve_db_path_nonexistent_config_falls_back() {
        let result = resolve_db_path("/nonexistent/path/to/config.toml");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("tuitbot.db"));
    }

    #[test]
    fn resolve_db_path_valid_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[storage]\ndb_path = \"~/.tuitbot/custom.db\"\n",
        )
        .expect("write config");
        let result = resolve_db_path(config_path.to_str().unwrap());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("custom.db"));
    }

    #[test]
    fn resolve_db_path_empty_db_path_in_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, "[storage]\ndb_path = \"\"\n").expect("write config");
        let result = resolve_db_path(config_path.to_str().unwrap());
        assert!(result.is_err());
    }

    // --- StartupError Io variant ---

    #[test]
    fn startup_error_io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let startup_err = StartupError::Io(io_err);
        let msg = startup_err.to_string();
        assert!(msg.contains("missing"), "got: {msg}");
    }

    // --- StoredTokens: scope analysis with required scopes ---

    #[test]
    fn stored_tokens_analyze_scopes_missing_scopes() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: vec!["tweet.read".to_string()], // missing several required
        };
        let analysis = tokens.analyze_scopes();
        assert!(!analysis.missing.is_empty(), "should have missing scopes");
        assert!(!analysis.granted.is_empty(), "should have granted scopes");
    }

    #[test]
    fn stored_tokens_analyze_scopes_all_granted() {
        let tokens = StoredTokens {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: REQUIRED_SCOPES.iter().map(|s| s.to_string()).collect(),
        };
        let analysis = tokens.analyze_scopes();
        assert!(analysis.missing.is_empty(), "all scopes should be granted");
    }

    // --- load_tokens_from_file error cases (without touching real ~/.tuitbot) ---

    #[test]
    fn save_and_load_roundtrip_via_file_io() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tokens.json");
        let tokens = StoredTokens {
            access_token: "at".to_string(),
            refresh_token: Some("rt".to_string()),
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
            scopes: vec!["offline.access".to_string()],
        };
        let json = serde_json::to_string_pretty(&tokens).expect("serialize");
        std::fs::write(&path, &json).expect("write");

        let loaded_json = std::fs::read_to_string(&path).expect("read");
        let loaded: StoredTokens = serde_json::from_str(&loaded_json).expect("parse");
        assert_eq!(loaded.access_token, "at");
        assert_eq!(loaded.refresh_token.as_deref(), Some("rt"));
        assert!(loaded.has_scope("offline.access"));
        assert!(!loaded.is_expired());
    }
}
