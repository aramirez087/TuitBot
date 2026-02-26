//! Shared application state for the MCP server.
//!
//! Bundles the database pool, configuration, optional LLM provider,
//! and optional X API client so that all tool handlers can access
//! them through the server struct.
//!
//! Two state structs exist for three runtime profiles:
//! - [`AppState`] / [`SharedState`]: full profile (DB + LLM + X client).
//! - [`ApiState`] / [`SharedApiState`]: readonly / api-readonly profiles (X client only, no DB).

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use tuitbot_core::config::Config;
use tuitbot_core::llm::LlmProvider;
use tuitbot_core::storage::DbPool;
use tuitbot_core::x_api::XApiClient;

use crate::tools::idempotency::IdempotencyStore;

// ── Runtime profile ─────────────────────────────────────────────────

/// MCP server runtime profile.
///
/// - **`Full`** — full TuitBot growth features. Default. All 60+ tools.
/// - **`Readonly`** — read-only X tools. No DB, no LLM, no mutations.
/// - **`ApiReadonly`** — broader read-only X tools. No DB, no LLM, no mutations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Full,
    Readonly,
    ApiReadonly,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Readonly => write!(f, "readonly"),
            Self::ApiReadonly => write!(f, "api-readonly"),
        }
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "full" => Ok(Self::Full),
            "readonly" => Ok(Self::Readonly),
            "api-readonly" => Ok(Self::ApiReadonly),
            other => Err(format!(
                "unknown profile '{other}'. Valid profiles: full, readonly, api-readonly"
            )),
        }
    }
}

// ── Full profile state ──────────────────────────────────────────────

/// Shared state accessible by all MCP tool handlers (full profile).
pub struct AppState {
    /// SQLite connection pool.
    pub pool: DbPool,
    /// Loaded and validated configuration.
    pub config: Config,
    /// Optional LLM provider (None if not configured or creation failed).
    pub llm_provider: Option<Box<dyn LlmProvider>>,
    /// Optional X API client (None if tokens not available).
    pub x_client: Option<Box<dyn XApiClient>>,
    /// Authenticated user ID from X API (cached from get_me on startup).
    pub authenticated_user_id: Option<String>,
    /// Idempotency guard for mutation dedup.
    pub idempotency: Arc<IdempotencyStore>,
}

/// Thread-safe reference to shared full-profile state.
pub type SharedState = Arc<AppState>;

// ── Readonly / api-readonly profile state ───────────────────────────

/// Lightweight state for readonly / api-readonly profiles (no DB, no LLM).
///
/// The X client is non-optional: a readonly profile with no X client has
/// zero usable tools, so `run_api_server` fails fast if tokens are missing.
pub struct ApiState {
    /// Loaded configuration.
    pub config: Config,
    /// X API client (required for readonly profiles).
    pub x_client: Box<dyn XApiClient>,
    /// Authenticated user ID from X API (from get_me on startup).
    pub authenticated_user_id: String,
    /// Idempotency guard for mutation dedup.
    pub idempotency: Arc<IdempotencyStore>,
}

/// Thread-safe reference to shared readonly-profile state.
pub type SharedApiState = Arc<ApiState>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_display() {
        assert_eq!(Profile::Full.to_string(), "full");
        assert_eq!(Profile::Readonly.to_string(), "readonly");
        assert_eq!(Profile::ApiReadonly.to_string(), "api-readonly");
    }

    #[test]
    fn profile_from_str_valid() {
        assert_eq!(Profile::from_str("full").unwrap(), Profile::Full);
        assert_eq!(Profile::from_str("readonly").unwrap(), Profile::Readonly);
        assert_eq!(
            Profile::from_str("api-readonly").unwrap(),
            Profile::ApiReadonly
        );
        // Case-insensitive variants
        assert_eq!(Profile::from_str("FULL").unwrap(), Profile::Full);
        assert_eq!(Profile::from_str("ReadOnly").unwrap(), Profile::Readonly);
        assert_eq!(
            Profile::from_str("API-READONLY").unwrap(),
            Profile::ApiReadonly
        );
    }

    #[test]
    fn profile_from_str_invalid() {
        let err = Profile::from_str("unknown").unwrap_err();
        assert!(err.contains("unknown profile"));
        assert!(err.contains("full, readonly, api-readonly"));
    }

    #[test]
    fn profile_roundtrip() {
        for variant in [Profile::Full, Profile::Readonly, Profile::ApiReadonly] {
            let s = variant.to_string();
            let parsed: Profile = s.parse().unwrap();
            assert_eq!(parsed, variant, "roundtrip failed for {s}");
        }
    }
}
