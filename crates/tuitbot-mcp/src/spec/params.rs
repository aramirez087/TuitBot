//! Parameter and endpoint type definitions for the X API spec pack.

use crate::contract::error_code::ErrorCode;
use crate::tools::manifest::{Profile, ToolCategory};

/// HTTP method for an X API endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    /// Whether this method performs a mutation (POST/PUT/DELETE).
    pub fn is_mutation(self) -> bool {
        !matches!(self, Self::Get)
    }
}

/// Type of a tool parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// A string value (user ID, tweet ID, query text, etc.).
    String,
    /// An integer value (max_results, etc.).
    Integer,
    /// A boolean value.
    Boolean,
    /// A comma-separated list of string values (expansions, fields).
    StringArray,
}

/// Definition of a single tool parameter.
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Parameter name as it appears in the MCP tool schema.
    pub name: &'static str,
    /// Parameter type.
    pub param_type: ParamType,
    /// Whether the parameter is required.
    pub required: bool,
    /// Human-readable description for the JSON Schema.
    pub description: &'static str,
    /// Default value (as a string literal), if any.
    pub default: Option<&'static str>,
}

/// Declarative definition of a single X API endpoint.
///
/// The generator converts each `EndpointDef` into a [`ToolEntry`] for the
/// manifest and can produce a JSON Schema for the tool's input.
#[derive(Debug, Clone)]
pub struct EndpointDef {
    /// MCP tool name (e.g. `"x_v2_lists_get"`).
    pub tool_name: &'static str,
    /// Human-readable description for the tool.
    pub description: &'static str,
    /// HTTP method.
    pub method: HttpMethod,
    /// URL path template (e.g. `"/2/lists/{id}/tweets"`).
    /// Path parameters use `{name}` placeholders.
    pub path: &'static str,
    /// Functional category for manifest grouping.
    pub category: ToolCategory,
    /// Which profiles include this tool.
    pub profiles: &'static [Profile],
    /// Required OAuth 2.0 scopes.
    pub scopes: &'static [&'static str],
    /// Tool parameters (path params + query params + body fields).
    pub params: &'static [ParamDef],
    /// Error codes this tool may return.
    pub error_codes: &'static [ErrorCode],
    /// API version group (e.g. "v2", "v1.1").
    pub api_version: &'static str,
    /// Endpoint group for naming hierarchy (e.g. "lists", "mutes").
    pub group: &'static str,
}
