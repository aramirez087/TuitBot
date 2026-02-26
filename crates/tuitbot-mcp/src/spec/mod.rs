//! X API spec pack and tool generator.
//!
//! This module defines every X API v2 endpoint as a declarative
//! [`EndpointDef`] struct and provides a generator that produces
//! [`ToolEntry`] records for the manifest system.
//!
//! ## Architecture
//!
//! ```text
//! endpoints.rs   (static endpoint definitions)
//!       │
//!       ▼
//! generator.rs   (EndpointDef → ToolEntry + JSON Schema)
//!       │
//!       ▼
//! manifest.rs    (all_tools() merges curated + generated)
//! ```
//!
//! ## Versioning
//!
//! Three independent version strings:
//! - **`X_API_SPEC_VERSION`**: semver for the spec pack itself.
//! - **`MCP_SCHEMA_VERSION`**: schema version for the manifest format.
//! - **`tuitbot_mcp_version`**: crate version from `CARGO_PKG_VERSION`.

mod endpoints;
mod generator;
mod params;

#[cfg(test)]
mod tests;

pub use endpoints::SPEC_ENDPOINTS;
pub use generator::{generate_spec_tools, generate_tool_schemas, ToolSchema};
pub use params::{EndpointDef, HttpMethod, ParamDef, ParamType};

/// Semantic version of the X API spec pack.
///
/// Bump when endpoints are added, removed, or modified.
pub const X_API_SPEC_VERSION: &str = "1.0.0";

/// Schema version for the manifest format.
pub const MCP_SCHEMA_VERSION: &str = "1.2";
