//! Shared constants: profile shorthands, error code sets, and common param definitions.

use crate::contract::error_code::ErrorCode;
use crate::tools::manifest::Profile;

use crate::spec::params::{ParamDef, ParamType};

// ── Profile shorthands ─────────────────────────────────────────────────

/// All six profiles (original four + utility pair).
pub(super) const ALL_SIX: &[Profile] = &[
    Profile::Readonly,
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
    Profile::UtilityReadonly,
    Profile::UtilityWrite,
];
/// Write + Admin + ApiReadonly + UtilityWrite.
pub(super) const WRITE_UP_AND_API_RO_AND_UTIL_WRITE: &[Profile] = &[
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
    Profile::UtilityWrite,
];
/// Write + Admin + UtilityWrite.
pub(super) const WRITE_UP_AND_UTIL_WRITE: &[Profile] =
    &[Profile::Write, Profile::Admin, Profile::UtilityWrite];
/// Admin profile only (enterprise/elevated-access tools).
pub(super) const ADMIN_ONLY: &[Profile] = &[Profile::Admin];

// ── Error code sets ────────────────────────────────────────────────────

pub(super) const X_READ_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
];

pub(super) const X_WRITE_ERR: &[ErrorCode] = &[
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

// ── Common parameter definitions ───────────────────────────────────────

pub(super) const PARAM_ID: ParamDef = ParamDef {
    name: "id",
    param_type: ParamType::String,
    required: true,
    description: "Resource ID",
    default: None,
};

pub(super) const PARAM_USER_ID: ParamDef = ParamDef {
    name: "user_id",
    param_type: ParamType::String,
    required: true,
    description: "User ID (authenticated user for /me endpoints)",
    default: None,
};

pub(super) const PARAM_TARGET_USER_ID: ParamDef = ParamDef {
    name: "target_user_id",
    param_type: ParamType::String,
    required: true,
    description: "Target user ID",
    default: None,
};

pub(super) const PARAM_MAX_RESULTS: ParamDef = ParamDef {
    name: "max_results",
    param_type: ParamType::Integer,
    required: false,
    description: "Maximum number of results to return",
    default: Some("100"),
};

pub(super) const PARAM_PAGINATION_TOKEN: ParamDef = ParamDef {
    name: "pagination_token",
    param_type: ParamType::String,
    required: false,
    description: "Token for paginating through results",
    default: None,
};

pub(super) const PARAM_TWEET_FIELDS: ParamDef = ParamDef {
    name: "tweet_fields",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated tweet fields to include (e.g. created_at,public_metrics)",
    default: None,
};

pub(super) const PARAM_USER_FIELDS: ParamDef = ParamDef {
    name: "user_fields",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated user fields to include (e.g. created_at,public_metrics)",
    default: None,
};

pub(super) const PARAM_EXPANSIONS: ParamDef = ParamDef {
    name: "expansions",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated expansions (e.g. author_id,referenced_tweets.id)",
    default: None,
};

pub(super) const PARAM_LIST_FIELDS: ParamDef = ParamDef {
    name: "list_fields",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated list fields (e.g. created_at,member_count,owner_id)",
    default: None,
};

pub(super) const PARAM_SPACE_FIELDS: ParamDef = ParamDef {
    name: "space_fields",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated space fields (e.g. host_ids,title,state)",
    default: None,
};

pub(super) const PARAM_DM_EVENT_FIELDS: ParamDef = ParamDef {
    name: "dm_event_fields",
    param_type: ParamType::StringArray,
    required: false,
    description: "Comma-separated DM event fields (e.g. id,text,event_type,created_at,sender_id)",
    default: None,
};

pub(super) const PARAM_PARTICIPANT_ID: ParamDef = ParamDef {
    name: "participant_id",
    param_type: ParamType::String,
    required: true,
    description: "User ID of the DM conversation participant",
    default: None,
};

// ── Ads-specific parameter definitions ────────────────────────────────

pub(super) const PARAM_ACCOUNT_ID: ParamDef = ParamDef {
    name: "account_id",
    param_type: ParamType::String,
    required: true,
    description: "Ads account ID",
    default: None,
};

pub(super) const PARAM_CAMPAIGN_ID: ParamDef = ParamDef {
    name: "campaign_id",
    param_type: ParamType::String,
    required: true,
    description: "Campaign ID",
    default: None,
};

pub(super) const PARAM_WITH_DELETED: ParamDef = ParamDef {
    name: "with_deleted",
    param_type: ParamType::Boolean,
    required: false,
    description: "Include deleted entities in the response",
    default: Some("false"),
};

pub(super) const PARAM_ADS_CURSOR: ParamDef = ParamDef {
    name: "cursor",
    param_type: ParamType::String,
    required: false,
    description: "Cursor for paginating through Ads API results",
    default: None,
};

pub(super) const PARAM_ADS_COUNT: ParamDef = ParamDef {
    name: "count",
    param_type: ParamType::Integer,
    required: false,
    description: "Number of results per page (Ads API, default: 200)",
    default: Some("200"),
};

// ── Compliance-specific parameter definitions ─────────────────────────

pub(super) const PARAM_COMPLIANCE_TYPE: ParamDef = ParamDef {
    name: "type",
    param_type: ParamType::String,
    required: true,
    description: "Compliance job type: tweets or users",
    default: None,
};

pub(super) const PARAM_COMPLIANCE_STATUS: ParamDef = ParamDef {
    name: "status",
    param_type: ParamType::String,
    required: false,
    description: "Filter jobs by status: created, in_progress, complete, expired, failed",
    default: None,
};

pub(super) const PARAM_USAGE_DAYS: ParamDef = ParamDef {
    name: "days",
    param_type: ParamType::Integer,
    required: false,
    description: "Number of days of usage data to retrieve (default: 7)",
    default: Some("7"),
};

pub(super) const PARAM_STREAM_RULE_VALUE: ParamDef = ParamDef {
    name: "value",
    param_type: ParamType::String,
    required: true,
    description: "The stream rule filter expression (e.g. 'cat has:images')",
    default: None,
};

pub(super) const PARAM_STREAM_RULE_TAG: ParamDef = ParamDef {
    name: "tag",
    param_type: ParamType::String,
    required: false,
    description: "Optional label/tag for the stream rule",
    default: None,
};

pub(super) const PARAM_STREAM_RULE_IDS: ParamDef = ParamDef {
    name: "rule_ids",
    param_type: ParamType::StringArray,
    required: true,
    description: "Comma-separated rule IDs to delete",
    default: None,
};
