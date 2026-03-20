//! Tests for startup module split by domain.
//!
//! - `tier`   — ApiTier and TierCapabilities
//! - `tokens` — StoredTokens, file I/O, scope analysis
//! - `auth`   — StartupError, PKCE, URL building, banner
//! - `paths`  — Path helpers, validate_db_path, resolve_db_path, callback state

mod auth;
mod paths;
mod tier;
mod tokens;
