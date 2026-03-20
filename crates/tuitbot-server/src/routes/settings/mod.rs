//! Settings endpoints for reading and updating the configuration.
//!
//! ## Module layout
//! - `handlers`   — all route handlers (onboarding, config, factory reset)
//! - `validation` — shared helpers, TOML merge/convert utilities

pub mod handlers;
pub mod validation;

#[cfg(test)]
mod tests;
// tests/ is a submodule directory: tests/mod.rs → toml, helpers

// Re-export public API so the router can reference `settings::*` unchanged.
pub use handlers::{config_status, get_settings, init_settings, patch_settings, validate_settings};
pub use validation::{factory_reset, get_defaults, merge_patch_and_parse, test_llm};

use serde::{Deserialize, Serialize};
use tuitbot_core::error::ConfigError;

// ---------------------------------------------------------------------------
// LLM test request/response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct TestLlmRequest {
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Factory reset types + constant (used by handlers + tests)
// ---------------------------------------------------------------------------

/// Confirmation phrase required for factory reset (case-sensitive, exact match).
pub(super) const FACTORY_RESET_PHRASE: &str = "RESET TUITBOT";

#[derive(Deserialize)]
pub struct FactoryResetRequest {
    pub confirmation: String,
}

#[derive(Serialize)]
pub(super) struct FactoryResetResponse {
    pub status: String,
    pub cleared: FactoryResetCleared,
}

#[derive(Serialize)]
pub(super) struct FactoryResetCleared {
    pub tables_cleared: u32,
    pub rows_deleted: u64,
    pub config_deleted: bool,
    pub passphrase_deleted: bool,
    pub media_deleted: bool,
    pub credentials_deleted: bool,
    pub runtimes_stopped: u32,
}

// ---------------------------------------------------------------------------
// Request / response types (shared across handlers + validation)
// ---------------------------------------------------------------------------

/// Request body for the optional claim object within `POST /api/settings/init`.
#[derive(Deserialize)]
pub(super) struct ClaimRequest {
    pub passphrase: String,
}

/// X profile data passed from the frontend during onboarding init.
///
/// Populated from the OAuth callback response so we can write the user's
/// X identity to the default account row atomically with config creation.
#[derive(Deserialize)]
pub(super) struct XProfileData {
    pub x_user_id: String,
    pub x_username: String,
    pub x_display_name: String,
    #[serde(default)]
    pub x_avatar_url: Option<String>,
}

#[derive(Serialize)]
pub(super) struct ValidationResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationErrorItem>,
}

#[derive(Serialize)]
pub(super) struct ValidationErrorItem {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
pub(super) struct TestResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Shared helper (used by handlers + tests)
// ---------------------------------------------------------------------------

pub(super) fn config_errors_to_response(errors: Vec<ConfigError>) -> Vec<ValidationErrorItem> {
    errors
        .into_iter()
        .map(|e| match e {
            ConfigError::MissingField { field } => ValidationErrorItem {
                field,
                message: "this field is required".to_string(),
            },
            ConfigError::InvalidValue { field, message } => ValidationErrorItem { field, message },
            other => ValidationErrorItem {
                field: String::new(),
                message: other.to_string(),
            },
        })
        .collect()
}
