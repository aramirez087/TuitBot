//! Machine-readable tool manifest for the MCP server.
//!
//! Provides a [`ToolManifest`] describing every registered tool: its name,
//! category, mutation flag, dependency requirements, available profiles, and
//! possible error codes. Generated programmatically — the snapshot test in this
//! module ensures the manifest JSON artifact never drifts from source.

use serde::{Deserialize, Serialize};

use crate::contract::error_code::ErrorCode;

/// Top-level manifest containing all tool entries.
#[derive(Debug, Serialize)]
pub struct ToolManifest {
    /// Schema version for the manifest format.
    pub version: &'static str,
    /// All registered tools.
    pub tools: Vec<ToolEntry>,
}

/// Metadata for a single tool.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolEntry {
    /// Tool name as registered in the MCP server (e.g. `"x_post_tweet"`).
    pub name: String,
    /// Functional category.
    pub category: ToolCategory,
    /// Module lane (shared vs workflow).
    pub lane: Lane,
    /// Whether this tool performs a mutation (write/engage).
    pub mutation: bool,
    /// Whether the tool requires an authenticated X API client.
    pub requires_x_client: bool,
    /// Whether the tool requires an LLM provider.
    pub requires_llm: bool,
    /// Whether the tool requires database access.
    pub requires_db: bool,
    /// OAuth scopes required by this tool (empty for non-X tools).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_scopes: Vec<String>,
    /// Whether the tool requires OAuth user-context authentication.
    #[serde(default, skip_serializing_if = "is_false")]
    pub requires_user_auth: bool,
    /// Whether the tool requires elevated (admin) access.
    #[serde(default, skip_serializing_if = "is_false")]
    pub requires_elevated_access: bool,
    /// Which profiles include this tool.
    pub profiles: Vec<Profile>,
    /// Error codes this tool may return.
    pub possible_error_codes: Vec<ErrorCode>,
}

fn is_false(v: &bool) -> bool {
    !v
}

/// Functional category for grouping tools.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Read,
    Write,
    Engage,
    Media,
    Analytics,
    Approval,
    Content,
    Discovery,
    Scoring,
    Config,
    Health,
    Policy,
    Telemetry,
    Context,
    Composite,
    Meta,
    List,
    Moderation,
}

/// MCP server profile.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Readonly,
    ApiReadonly,
    Write,
    Admin,
}

/// Module lane: whether a tool lives in the shared `tools/` root or
/// inside `tools/workflow/`.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    /// Shared tools available to all profiles (config, scoring, response).
    Shared,
    /// Workflow-only tools behind `tools/workflow/`.
    Workflow,
}

/// Build the complete tool manifest from the source-of-truth lookup table.
///
/// Merges curated Layer 1 tools with generated Layer 2 tools from the spec pack.
/// Output is sorted alphabetically by tool name for determinism.
pub fn generate_manifest() -> ToolManifest {
    ToolManifest {
        version: crate::spec::MCP_SCHEMA_VERSION,
        tools: all_tools(),
    }
}

// ── Profile-specific manifest ────────────────────────────────────────────

impl From<crate::state::Profile> for Profile {
    fn from(p: crate::state::Profile) -> Self {
        match p {
            crate::state::Profile::Readonly => Self::Readonly,
            crate::state::Profile::ApiReadonly => Self::ApiReadonly,
            crate::state::Profile::Write => Self::Write,
            crate::state::Profile::Admin => Self::Admin,
        }
    }
}

/// Profile-specific manifest with version metadata and filtered tool list.
///
/// Contains a version triplet:
/// - `tuitbot_mcp_version`: crate version from `CARGO_PKG_VERSION`
/// - `mcp_schema_version`: manifest format version
/// - `x_api_spec_version`: spec pack version for generated tools
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileManifest {
    pub tuitbot_mcp_version: String,
    pub mcp_schema_version: String,
    pub x_api_spec_version: String,
    pub profile: String,
    pub tool_count: usize,
    pub tools: Vec<ToolEntry>,
}

/// Generate a profile-specific manifest with version metadata.
///
/// Merges curated + generated tools, filters to the requested profile,
/// sorts alphabetically by tool name for deterministic output, and
/// populates the version triplet.
pub fn generate_profile_manifest(profile: crate::state::Profile) -> ProfileManifest {
    let manifest = generate_manifest();
    let manifest_profile = Profile::from(profile);
    let mut tools: Vec<ToolEntry> = manifest
        .tools
        .into_iter()
        .filter(|t| t.profiles.contains(&manifest_profile))
        .collect();
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    ProfileManifest {
        tuitbot_mcp_version: env!("CARGO_PKG_VERSION").to_string(),
        mcp_schema_version: crate::spec::MCP_SCHEMA_VERSION.to_string(),
        x_api_spec_version: crate::spec::X_API_SPEC_VERSION.to_string(),
        profile: profile.to_string(),
        tool_count: tools.len(),
        tools,
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Shorthand constructors.
#[allow(clippy::too_many_arguments)]
fn tool(
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
fn x_tool(
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

/// All four profiles.
const ALL_FOUR: &[Profile] = &[
    Profile::Readonly,
    Profile::ApiReadonly,
    Profile::Write,
    Profile::Admin,
];
/// Write + Admin + ApiReadonly (not minimal readonly).
const WRITE_UP_AND_API_RO: &[Profile] = &[Profile::ApiReadonly, Profile::Write, Profile::Admin];
/// Write + Admin (standard operating profiles).
const WRITE_UP: &[Profile] = &[Profile::Write, Profile::Admin];
/// Admin only — universal request tools for ad-hoc X API v2 access.
const ADMIN_ONLY: &[Profile] = &[Profile::Admin];
/// Api-readonly only.
const API_RO: &[Profile] = &[Profile::ApiReadonly];

/// X API read errors.
const X_READ_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
];

/// X API read errors + no-user-id.
const X_READ_USER_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
];

/// X API write errors (includes policy codes — policy codes only apply in
/// the workflow profile; the API profile skips policy gating).
const X_WRITE_ERR: &[ErrorCode] = &[
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
const X_ENGAGE_ERR: &[ErrorCode] = &[
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

/// Universal X API request errors.
const X_REQUEST_ERR: &[ErrorCode] = &[
    ErrorCode::XNotConfigured,
    ErrorCode::XRateLimited,
    ErrorCode::XAuthExpired,
    ErrorCode::XForbidden,
    ErrorCode::XNetworkError,
    ErrorCode::XApiError,
    ErrorCode::XRequestBlocked,
];

/// Database-only errors.
const DB_ERR: &[ErrorCode] = &[ErrorCode::DbError];

/// LLM errors.
const LLM_ERR: &[ErrorCode] = &[ErrorCode::LlmNotConfigured, ErrorCode::LlmError];

/// All tools (curated + generated), sorted by name.
pub fn all_tools() -> Vec<ToolEntry> {
    let mut tools = all_curated_tools();
    tools.extend(crate::spec::generate_spec_tools());
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

/// Hand-crafted curated tools (Layer 1).
fn all_curated_tools() -> Vec<ToolEntry> {
    vec![
        // ── Analytics ────────────────────────────────────────────────
        tool(
            "get_stats",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_follower_trend",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Action Log ───────────────────────────────────────────────
        tool(
            "get_action_log",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_action_counts",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Mutation Audit ──────────────────────────────────────────
        tool(
            "get_recent_mutations",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_mutation_detail",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Rate Limits ──────────────────────────────────────────────
        tool(
            "get_rate_limits",
            ToolCategory::Policy,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Replies ──────────────────────────────────────────────────
        tool(
            "get_recent_replies",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_reply_count_today",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Target Accounts ──────────────────────────────────────────
        tool(
            "list_target_accounts",
            ToolCategory::Discovery,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Discovery ────────────────────────────────────────────────
        tool(
            "list_unreplied_tweets",
            ToolCategory::Discovery,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Scoring (pure function on &Config — no DB needed) ────────
        tool(
            "score_tweet",
            ToolCategory::Scoring,
            Lane::Shared,
            false,
            false,
            false,
            false,
            ALL_FOUR,
            &[ErrorCode::InvalidInput],
        ),
        // ── Approval Queue ───────────────────────────────────────────
        tool(
            "list_pending_approvals",
            ToolCategory::Approval,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_pending_count",
            ToolCategory::Approval,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "approve_item",
            ToolCategory::Approval,
            Lane::Workflow,
            true,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::DbError,
                ErrorCode::NotFound,
                ErrorCode::XNotConfigured,
                ErrorCode::XApiError,
            ],
        ),
        tool(
            "reject_item",
            ToolCategory::Approval,
            Lane::Workflow,
            true,
            false,
            false,
            true,
            WRITE_UP,
            &[ErrorCode::DbError, ErrorCode::NotFound],
        ),
        tool(
            "approve_all",
            ToolCategory::Approval,
            Lane::Workflow,
            true,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::DbError,
                ErrorCode::XNotConfigured,
                ErrorCode::XApiError,
            ],
        ),
        // ── Content Generation ───────────────────────────────────────
        tool(
            "generate_reply",
            ToolCategory::Content,
            Lane::Workflow,
            false,
            false,
            true,
            true,
            WRITE_UP,
            LLM_ERR,
        ),
        tool(
            "generate_tweet",
            ToolCategory::Content,
            Lane::Workflow,
            false,
            false,
            true,
            true,
            WRITE_UP,
            LLM_ERR,
        ),
        tool(
            "generate_thread",
            ToolCategory::Content,
            Lane::Workflow,
            false,
            false,
            true,
            true,
            WRITE_UP,
            LLM_ERR,
        ),
        // ── Config ───────────────────────────────────────────────────
        tool(
            "get_config",
            ToolCategory::Config,
            Lane::Shared,
            false,
            false,
            false,
            false,
            ALL_FOUR,
            &[],
        ),
        tool(
            "validate_config",
            ToolCategory::Config,
            Lane::Shared,
            false,
            false,
            false,
            false,
            WRITE_UP_AND_API_RO,
            &[],
        ),
        // ── Capabilities & Health ────────────────────────────────────
        tool(
            "get_capabilities",
            ToolCategory::Meta,
            Lane::Shared,
            false,
            false,
            false,
            false,
            WRITE_UP_AND_API_RO,
            &[],
        ),
        tool(
            "health_check",
            ToolCategory::Health,
            Lane::Shared,
            false,
            false,
            false,
            false,
            ALL_FOUR,
            &[],
        ),
        // ── Mode & Policy ────────────────────────────────────────────
        tool(
            "get_mode",
            ToolCategory::Meta,
            Lane::Shared,
            false,
            false,
            false,
            false,
            WRITE_UP_AND_API_RO,
            &[],
        ),
        tool(
            "get_policy_status",
            ToolCategory::Policy,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "compose_tweet",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::XApiError,
                ErrorCode::DbError,
                ErrorCode::InvalidInput,
                ErrorCode::TweetTooLong,
                ErrorCode::PolicyDeniedBlocked,
                ErrorCode::PolicyDeniedRateLimited,
                ErrorCode::PolicyDeniedHardRule,
                ErrorCode::PolicyDeniedUserRule,
                ErrorCode::PolicyError,
            ],
        ),
        // ── Discovery Feed & Topics ──────────────────────────────────
        tool(
            "get_discovery_feed",
            ToolCategory::Discovery,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "suggest_topics",
            ToolCategory::Content,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── X API Core Read (in all 3 profiles) ─────────────────────
        x_tool(
            "get_tweet_by_id",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_user_by_username",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_ERR,
        ),
        x_tool(
            "x_search_tweets",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_user_mentions",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_USER_ERR,
        ),
        x_tool(
            "x_get_user_tweets",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_home_timeline",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_USER_ERR,
        ),
        x_tool(
            "x_get_user_by_id",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["users.read"],
            true,
            false,
            ALL_FOUR,
            X_READ_ERR,
        ),
        // ── X API Extended Read (workflow + api-readonly) ────────────
        x_tool(
            "x_get_followers",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["follows.read", "users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_following",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["follows.read", "users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_liked_tweets",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["like.read", "users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_bookmarks",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["bookmark.read", "users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_USER_ERR,
        ),
        x_tool(
            "x_get_users_by_ids",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_ERR,
        ),
        x_tool(
            "x_get_tweet_liking_users",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["tweet.read", "users.read"],
            true,
            false,
            WRITE_UP_AND_API_RO,
            X_READ_ERR,
        ),
        tool(
            "get_x_usage",
            ToolCategory::Analytics,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── X API Read (api-readonly only) ──────────────────────────
        x_tool(
            "x_get_me",
            ToolCategory::Read,
            Lane::Shared,
            false,
            false,
            &["users.read"],
            true,
            false,
            API_RO,
            X_READ_ERR,
        ),
        // ── X API Write (workflow only) ─────────────────────────────
        x_tool(
            "x_post_tweet",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_WRITE_ERR,
        ),
        x_tool(
            "x_reply_to_tweet",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_WRITE_ERR,
        ),
        x_tool(
            "x_quote_tweet",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_WRITE_ERR,
        ),
        x_tool(
            "x_delete_tweet",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_WRITE_ERR,
        ),
        x_tool(
            "x_post_thread",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::XRateLimited,
                ErrorCode::XAuthExpired,
                ErrorCode::XForbidden,
                ErrorCode::XNetworkError,
                ErrorCode::XApiError,
                ErrorCode::TweetTooLong,
                ErrorCode::InvalidInput,
                ErrorCode::ThreadPartialFailure,
                ErrorCode::ScraperMutationBlocked,
                ErrorCode::PolicyDeniedBlocked,
                ErrorCode::PolicyDeniedRateLimited,
                ErrorCode::PolicyDeniedHardRule,
                ErrorCode::PolicyDeniedUserRule,
                ErrorCode::PolicyError,
            ],
        ),
        // ── X API Engage (workflow only) ─────────────────────────────
        x_tool(
            "x_like_tweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["like.read", "like.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_unlike_tweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["like.read", "like.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_follow_user",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["follows.read", "follows.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_unfollow_user",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["follows.read", "follows.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_retweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_unretweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["tweet.read", "tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_bookmark_tweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["bookmark.read", "bookmark.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        x_tool(
            "x_unbookmark_tweet",
            ToolCategory::Engage,
            Lane::Workflow,
            true,
            true,
            &["bookmark.read", "bookmark.write", "users.read"],
            true,
            false,
            WRITE_UP,
            X_ENGAGE_ERR,
        ),
        // ── X API Media (workflow only) ─────────────────────────────
        x_tool(
            "x_upload_media",
            ToolCategory::Media,
            Lane::Workflow,
            true,
            false,
            &["tweet.write", "users.read"],
            true,
            false,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::UnsupportedMediaType,
                ErrorCode::FileReadError,
                ErrorCode::MediaUploadError,
                ErrorCode::XApiError,
                ErrorCode::ScraperMutationBlocked,
            ],
        ),
        // ── Dry-run validation tools ──────────────────────────────────
        tool(
            "x_post_tweet_dry_run",
            ToolCategory::Write,
            Lane::Workflow,
            false, // dry-run is not a mutation
            false,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::TweetTooLong,
                ErrorCode::InvalidInput,
                ErrorCode::PolicyError,
            ],
        ),
        tool(
            "x_post_thread_dry_run",
            ToolCategory::Write,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::TweetTooLong,
                ErrorCode::InvalidInput,
                ErrorCode::PolicyError,
            ],
        ),
        // ── Context Intelligence ─────────────────────────────────────
        tool(
            "get_author_context",
            ToolCategory::Context,
            Lane::Workflow,
            false,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::ContextError,
                ErrorCode::DbError,
            ],
        ),
        tool(
            "recommend_engagement_action",
            ToolCategory::Context,
            Lane::Workflow,
            false,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::RecommendationError,
                ErrorCode::DbError,
            ],
        ),
        tool(
            "topic_performance_snapshot",
            ToolCategory::Context,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            &[ErrorCode::TopicError, ErrorCode::DbError],
        ),
        // ── Telemetry ────────────────────────────────────────────────
        tool(
            "get_mcp_tool_metrics",
            ToolCategory::Telemetry,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        tool(
            "get_mcp_error_breakdown",
            ToolCategory::Telemetry,
            Lane::Workflow,
            false,
            false,
            false,
            true,
            WRITE_UP,
            DB_ERR,
        ),
        // ── Composite ────────────────────────────────────────────────
        tool(
            "find_reply_opportunities",
            ToolCategory::Composite,
            Lane::Workflow,
            false,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::XNotConfigured,
                ErrorCode::InvalidInput,
                ErrorCode::XApiError,
                ErrorCode::DbError,
            ],
        ),
        tool(
            "draft_replies_for_candidates",
            ToolCategory::Composite,
            Lane::Workflow,
            false,
            false,
            true,
            true,
            WRITE_UP,
            &[
                ErrorCode::InvalidInput,
                ErrorCode::LlmNotConfigured,
                ErrorCode::LlmError,
                ErrorCode::DbError,
            ],
        ),
        tool(
            "propose_and_queue_replies",
            ToolCategory::Composite,
            Lane::Workflow,
            true,
            true,
            false,
            true,
            WRITE_UP,
            &[
                ErrorCode::InvalidInput,
                ErrorCode::XNotConfigured,
                ErrorCode::XApiError,
                ErrorCode::DbError,
                ErrorCode::PolicyDeniedBlocked,
                ErrorCode::PolicyDeniedRateLimited,
                ErrorCode::PolicyDeniedHardRule,
                ErrorCode::PolicyDeniedUserRule,
                ErrorCode::PolicyError,
            ],
        ),
        tool(
            "generate_thread_plan",
            ToolCategory::Composite,
            Lane::Workflow,
            false,
            false,
            true,
            false,
            WRITE_UP,
            &[
                ErrorCode::LlmNotConfigured,
                ErrorCode::LlmError,
                ErrorCode::InvalidInput,
            ],
        ),
        // ── Universal X API Request (admin only) ────────────────────
        x_tool(
            "x_get",
            ToolCategory::Read,
            Lane::Workflow,
            false,
            false,
            &[],
            true,
            true,
            ADMIN_ONLY,
            X_REQUEST_ERR,
        ),
        x_tool(
            "x_post",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            false,
            &[],
            true,
            true,
            ADMIN_ONLY,
            X_REQUEST_ERR,
        ),
        x_tool(
            "x_put",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            false,
            &[],
            true,
            true,
            ADMIN_ONLY,
            X_REQUEST_ERR,
        ),
        x_tool(
            "x_delete",
            ToolCategory::Write,
            Lane::Workflow,
            true,
            false,
            &[],
            true,
            true,
            ADMIN_ONLY,
            X_REQUEST_ERR,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn manifest_generates_without_panic() {
        let manifest = generate_manifest();
        assert_eq!(manifest.version, crate::spec::MCP_SCHEMA_VERSION);
        assert!(!manifest.tools.is_empty());
    }

    #[test]
    fn no_duplicate_tool_names() {
        let manifest = generate_manifest();
        let mut seen = HashSet::new();
        for t in &manifest.tools {
            assert!(
                seen.insert(t.name.as_str()),
                "duplicate tool name: {}",
                t.name
            );
        }
    }

    #[test]
    fn all_tools_have_at_least_one_profile() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            assert!(!t.profiles.is_empty(), "tool {} has no profiles", t.name);
        }
    }

    #[test]
    fn mutation_tools_require_x_or_db() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.mutation {
                assert!(
                    t.requires_x_client || t.requires_db,
                    "mutation tool {} requires neither x_client nor db",
                    t.name
                );
            }
        }
    }

    #[test]
    fn error_codes_are_valid_variants() {
        let all_codes: HashSet<ErrorCode> = ErrorCode::ALL.iter().copied().collect();
        let manifest = generate_manifest();
        for t in &manifest.tools {
            for &code in &t.possible_error_codes {
                assert!(
                    all_codes.contains(&code),
                    "tool {} references unknown error code {:?}",
                    t.name,
                    code
                );
            }
        }
    }

    #[test]
    fn category_counts() {
        let manifest = generate_manifest();
        let mut cats: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for t in &manifest.tools {
            let cat = serde_json::to_string(&t.category).unwrap();
            *cats.entry(Box::leak(cat.into_boxed_str())).or_default() += 1;
        }
        // Sanity: we have tools in multiple categories
        assert!(
            cats.len() >= 10,
            "expected at least 10 categories, got {}",
            cats.len()
        );
    }

    #[test]
    fn manifest_snapshot() {
        let manifest = generate_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let expected_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../roadmap/artifacts/session-06-tool-manifest.json"
        );
        let expected = std::fs::read_to_string(expected_path);
        match expected {
            Ok(content) => {
                // Normalize CRLF/LF so snapshot checks are stable across OSes.
                let normalize_newlines = |s: &str| s.replace("\r\n", "\n");
                assert_eq!(
                    normalize_newlines(&json).trim(),
                    normalize_newlines(&content).trim(),
                    "Tool manifest has drifted from snapshot. \
                     Regenerate with: cargo test -p tuitbot-mcp manifest -- --ignored"
                );
            }
            Err(_) => {
                // First run: write the snapshot.
                std::fs::write(expected_path, &json).unwrap();
            }
        }
    }
}
