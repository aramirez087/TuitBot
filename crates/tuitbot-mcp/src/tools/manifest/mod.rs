//! Machine-readable tool manifest for the MCP server.
//!
//! Provides a [`ToolManifest`] describing every registered tool: its name,
//! category, mutation flag, dependency requirements, available profiles, and
//! possible error codes. Generated programmatically — the snapshot test in this
//! module ensures the manifest JSON artifact never drifts from source.
//!
//! ## Module layout
//!
//! | Module              | Contents                                                  |
//! |---------------------|-----------------------------------------------------------|
//! | `types`             | `ToolManifest`, `ToolEntry`, `ProfileManifest`, enums     |
//! | `profile`           | `From<state::Profile>` impl, `generate_profile_manifest`  |
//! | `builders`          | `tool()`, `x_tool()`, profile/error constants             |
//! | `curated_workflow`  | Non-X curated tool entries (analytics, config, composite) |
//! | `curated_x_api`     | X API curated tool entries (reads, writes, engages)       |

mod builders;
mod curated_composite;
mod curated_workflow;
mod curated_x_api;
mod profile;
mod types;

#[cfg(test)]
mod tests;

pub use profile::generate_profile_manifest;
pub use types::{Lane, Profile, ProfileManifest, ToolCategory, ToolEntry, ToolManifest};

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

/// All tools (curated + generated), sorted by name.
fn all_tools() -> Vec<ToolEntry> {
    let mut tools = all_curated_tools();
    tools.extend(crate::spec::generate_spec_tools());
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

/// Hand-crafted curated tools (Layer 1): workflow + composite + X API tools.
fn all_curated_tools() -> Vec<ToolEntry> {
    let mut tools = curated_workflow::workflow_tools();
    tools.extend(curated_composite::composite_tools());
    tools.extend(curated_x_api::x_api_tools());
    tools
}
