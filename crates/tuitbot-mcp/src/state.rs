//! Shared application state for the MCP server.
//!
//! Bundles the database pool, configuration, optional LLM provider,
//! and optional X API client so that all tool handlers can access
//! them through the server struct.
//!
//! Two state structs exist for four runtime profiles:
//! - [`AppState`] / [`SharedState`]: write / admin profiles (DB + LLM + X client).
//! - [`ReadonlyState`] / [`SharedReadonlyState`]: readonly / api-readonly profiles (X client only, no DB).

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
/// - **`Readonly`** — minimal read-only X tools. No DB, no LLM, no mutations.
/// - **`ApiReadonly`** — broader read-only X tools. No DB, no LLM, no mutations.
/// - **`Write`** — standard operating profile. All typed tools including mutations. Default.
/// - **`Admin`** — superset of Write. Adds universal request tools (`x_get`/`x_post`/`x_put`/`x_delete`)
///   for arbitrary X API endpoint access. Only when explicitly configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Readonly,
    ApiReadonly,
    Write,
    Admin,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Readonly => write!(f, "readonly"),
            Self::ApiReadonly => write!(f, "api-readonly"),
            Self::Write => write!(f, "write"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "readonly" => Ok(Self::Readonly),
            "api-readonly" => Ok(Self::ApiReadonly),
            "write" | "full" => Ok(Self::Write),
            "admin" => Ok(Self::Admin),
            other => Err(format!(
                "unknown profile '{other}'. Valid profiles: readonly, api-readonly, write, admin"
            )),
        }
    }
}

// ── Write / Admin profile state ─────────────────────────────────────

/// Shared state accessible by all MCP tool handlers (write / admin profiles).
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
    /// OAuth scopes granted by the current token (empty if no token loaded).
    pub granted_scopes: Vec<String>,
    /// Idempotency guard for mutation dedup.
    pub idempotency: Arc<IdempotencyStore>,
}

/// Thread-safe reference to shared full-profile state.
pub type SharedState = Arc<AppState>;

// ── Readonly / api-readonly profile state ───────────────────────────

/// Lightweight state for readonly / api-readonly profiles (no DB, no LLM).
///
/// The X client is non-optional: a readonly profile with no X client has
/// zero usable tools, so the server fails fast if tokens are missing.
/// No idempotency store — read-only profiles perform no mutations.
pub struct ReadonlyState {
    /// Loaded configuration.
    pub config: Config,
    /// X API client (required for readonly profiles).
    pub x_client: Box<dyn XApiClient>,
    /// Authenticated user ID from X API (from get_me on startup).
    pub authenticated_user_id: String,
}

/// Thread-safe reference to shared readonly-profile state.
pub type SharedReadonlyState = Arc<ReadonlyState>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_display() {
        assert_eq!(Profile::Readonly.to_string(), "readonly");
        assert_eq!(Profile::ApiReadonly.to_string(), "api-readonly");
        assert_eq!(Profile::Write.to_string(), "write");
        assert_eq!(Profile::Admin.to_string(), "admin");
    }

    #[test]
    fn profile_from_str_valid() {
        assert_eq!(Profile::from_str("readonly").unwrap(), Profile::Readonly);
        assert_eq!(
            Profile::from_str("api-readonly").unwrap(),
            Profile::ApiReadonly
        );
        assert_eq!(Profile::from_str("write").unwrap(), Profile::Write);
        assert_eq!(Profile::from_str("admin").unwrap(), Profile::Admin);
        // Case-insensitive variants
        assert_eq!(Profile::from_str("WRITE").unwrap(), Profile::Write);
        assert_eq!(Profile::from_str("ReadOnly").unwrap(), Profile::Readonly);
        assert_eq!(
            Profile::from_str("API-READONLY").unwrap(),
            Profile::ApiReadonly
        );
        assert_eq!(Profile::from_str("ADMIN").unwrap(), Profile::Admin);
    }

    #[test]
    fn profile_from_str_legacy_full_maps_to_write() {
        assert_eq!(Profile::from_str("full").unwrap(), Profile::Write);
        assert_eq!(Profile::from_str("FULL").unwrap(), Profile::Write);
    }

    #[test]
    fn profile_from_str_invalid() {
        let err = Profile::from_str("unknown").unwrap_err();
        assert!(err.contains("unknown profile"));
        assert!(err.contains("readonly, api-readonly, write, admin"));
    }

    #[test]
    fn profile_roundtrip() {
        for variant in [
            Profile::Readonly,
            Profile::ApiReadonly,
            Profile::Write,
            Profile::Admin,
        ] {
            let s = variant.to_string();
            let parsed: Profile = s.parse().unwrap();
            assert_eq!(parsed, variant, "roundtrip failed for {s}");
        }
    }
}
