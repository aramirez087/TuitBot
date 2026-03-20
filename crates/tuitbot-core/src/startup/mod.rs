//! Startup types and helpers for Tuitbot CLI commands.
//!
//! Provides API tier detection types, OAuth token management,
//! PKCE authentication helpers, startup banner formatting, and
//! diagnostic check types used by the `run`, `auth`, and `test`
//! CLI commands.
//!
//! ## Module layout
//! - `config`   — ApiTier, TierCapabilities, StoredTokens, StartupError
//! - `db`       — token file I/O, path helpers (data_dir, expand_tilde, validate_db_path)
//! - `services` — PKCE, OAuth URL building, token exchange, credential verification, banner

pub mod config;
pub mod db;
pub mod services;

#[cfg(test)]
mod tests;

// Re-export the entire public API so callers use `startup::*` unchanged.
pub use config::{ApiTier, StartupError, StoredTokens, TierCapabilities};
pub use db::{
    data_dir, expand_tilde, load_tokens_from_file, resolve_db_path, save_tokens_to_file,
    token_file_path, validate_db_path,
};
pub use services::{
    build_auth_url, build_redirect_uri, exchange_auth_code, extract_auth_code,
    extract_callback_state, format_startup_banner, generate_pkce, verify_credentials,
    PkceChallenge, X_AUTH_URL, X_TOKEN_URL, X_USERS_ME_URL,
};
