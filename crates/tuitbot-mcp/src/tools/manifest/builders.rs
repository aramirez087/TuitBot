//! Tool entry builder helpers and shared profile/error constants.

use crate::contract::error_code::ErrorCode;

use super::types::{Lane, Profile, ToolCategory, ToolEntry};

// ── Profile constants ────────────────────────────────────────────────────

/// All six profiles.
pub(super) const ALL_SIX: &[Profile] = &[
    Profile::Readonly,
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
    Profile::UtilityReadonly,
    Profile::UtilityWrite,
];
/// All four original profiles (no utility profiles).
#[allow(dead_code)]
pub(super) const ALL_FOUR: &[Profile] = &[
    Profile::Readonly,
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
];
/// Write + Admin + ApiReadonly + UtilityWrite (extended reads).
pub(super) const WRITE_UP_AND_API_RO_AND_UTIL_WRITE: &[Profile] = &[
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
    Profile::UtilityWrite,
];
/// Write + Admin + ApiReadonly (not minimal readonly).
pub(super) const WRITE_UP_AND_API_RO: &[Profile] =
    &[Profile::ApiReadonly, Profile::Write, Profile::Admin];
/// Write + Admin (standard operating profiles).
pub(super) const WRITE_UP: &[Profile] = &[Profile::Write, Profile::Admin];
/// Write + Admin + UtilityWrite (mutation tools).
pub(super) const WRITE_UP_AND_UTIL_WRITE: &[Profile] =
    &[Profile::Write, Profile::Admin, Profile::UtilityWrite];
/// Admin only — universal request tools for ad-hoc X API v2 access.
pub(super) const ADMIN_ONLY: &[Profile] = &[Profile::Admin];
/// Api-readonly only.
pub(super) const API_RO: &[Profile] = &[Profile::ApiReadonly];

// ── Error code constants ─────────────────────────────────────────────────

/// X API read errors.
pub(super) const X_READ_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
];

/// X API read errors + no-user-id.
pub(super) const X_READ_USER_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
];

/// X API write errors (includes policy codes — policy codes only apply in
/// the workflow profile; the API profile skips policy gating).
pub(super) const X_WRITE_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
    ErrorCode::TweetTooLong,
    ErrorCode::ScraperMutationBlocked,
    ErrorCode::PolicyDeniedBlocked,
    ErrorCode::PolicyDeniedRateLimited,
    ErrorCode::PolicyDeniedHardRule,
    ErrorCode::PolicyDeniedUserRule,
    ErrorCode::PolicyError,
];

/// X API engage errors (policy codes only apply in the workflow profile).
pub(super) const X_ENGAGE_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
    ErrorCode::ScraperMutationBlocked,
    ErrorCode::PolicyDeniedBlocked,
    ErrorCode::PolicyDeniedRateLimited,
    ErrorCode::PolicyDeniedHardRule,
    ErrorCode::PolicyDeniedUserRule,
    ErrorCode::PolicyError,
];

/// Universal X API request errors (read-only — x_get).
pub(super) const X_REQUEST_READ_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
    ErrorCode::XRequestBlocked,
];

/// Universal X API request mutation errors (x_post, x_put, x_delete).
/// Includes policy denial codes because mutations are gateway-gated.
pub(super) const X_REQUEST_MUTATION_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
    ErrorCode::XRequestBlocked,
    ErrorCode::PolicyDeniedBlocked,
    ErrorCode::PolicyDeniedRateLimited,
    ErrorCode::PolicyDeniedHardRule,
    ErrorCode::PolicyDeniedUserRule,
    ErrorCode::PolicyError,
];

/// Database-only errors.
pub(super) const DB_ERR: &[ErrorCode] = &[ErrorCode::DbError];

/// LLM errors.
pub(super) const LLM_ERR: &[ErrorCode] = &[ErrorCode::LlmNotConfigured, ErrorCode::LlmError];

// ── Builder helpers ──────────────────────────────────────────────────────

/// Shorthand constructors.
#[allow(clippy::too_many_arguments)]
pub(super) fn tool(
    name: &str,
    category: ToolCategory,
    lane: Lane,
    mutation: bool,
    requires_x_client: bool,
    requires_llm: bool,
    requires_db: bool,
    profiles: &[Profile],
    error_codes: &[ErrorCode],
) -> ToolEntry {
    ToolEntry {
        name: name.to_owned(),
        category,
        lane,
        mutation,
        requires_x_client,
        requires_llm,
        requires_db,
        requires_scopes: vec![],
        requires_user_auth: false,
        requires_elevated_access: false,
        profiles: profiles.to_vec(),
        possible_error_codes: error_codes.to_vec(),
    }
}

/// Shorthand for X API tools that need scope and auth metadata.
#[allow(clippy::too_many_arguments)]
pub(super) fn x_tool(
    name: &str,
    category: ToolCategory,
    lane: Lane,
    mutation: bool,
    requires_db: bool,
    scopes: &[&str],
    requires_user_auth: bool,
    requires_elevated_access: bool,
    profiles: &[Profile],
    error_codes: &[ErrorCode],
) -> ToolEntry {
    ToolEntry {
        name: name.to_owned(),
        category,
        lane,
        mutation,
        requires_x_client: true,
        requires_llm: false,
        requires_db,
        requires_scopes: scopes.iter().map(|s| (*s).to_string()).collect(),
        requires_user_auth,
        requires_elevated_access,
        profiles: profiles.to_vec(),
        possible_error_codes: error_codes.to_vec(),
    }
}
