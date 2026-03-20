//! API tier types, token storage model, and startup error type.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::x_api::scopes::{self, ScopeAnalysis};

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
