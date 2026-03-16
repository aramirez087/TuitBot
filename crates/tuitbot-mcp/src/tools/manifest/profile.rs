//! Profile conversion and profile-specific manifest generation.

use super::types::{Profile, ProfileManifest};

impl From<crate::state::Profile> for Profile {
    fn from(p: crate::state::Profile) -> Self {
        match p {
            crate::state::Profile::Readonly => Self::Readonly,
            crate::state::Profile::ApiReadonly => Self::ApiReadonly,
            crate::state::Profile::Write => Self::Write,
            crate::state::Profile::Admin => Self::Admin,
            crate::state::Profile::UtilityReadonly => Self::UtilityReadonly,
            crate::state::Profile::UtilityWrite => Self::UtilityWrite,
        }
    }
}

/// Generate a profile-specific manifest with version metadata.
///
/// Merges curated + generated tools, filters to the requested profile,
/// sorts alphabetically by tool name for deterministic output, and
/// populates the version triplet.
pub fn generate_profile_manifest(profile: crate::state::Profile) -> ProfileManifest {
    let manifest = super::generate_manifest();
    let manifest_profile = Profile::from(profile);
    let mut tools: Vec<super::types::ToolEntry> = manifest
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
