//! Core types for the MCP tool manifest.

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

pub(crate) fn is_false(v: &bool) -> bool {
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
    DirectMessage,
    Ads,
    Compliance,
}

/// MCP server profile.
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Readonly,
    ApiReadonly,
    Write,
    Admin,
    UtilityReadonly,
    UtilityWrite,
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
