//! Structural parity test: every manifest-declared tool must be callable.
//!
//! For each profile, verifies that every tool in the manifest is backed by
//! either a curated `#[tool]` handler in the server module or a generated
//! endpoint in the spec pack. This catches drift where a manifest entry
//! exists but no handler can serve it.

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap};

    use crate::spec::SPEC_ENDPOINTS;
    use crate::tools::manifest::{generate_manifest, Profile};

    /// Extract `#[tool]`-annotated function names from a server source file.
    fn extract_tool_fn_names(source: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut saw_tool_attr = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[tool") {
                saw_tool_attr = true;
                continue;
            }
            if saw_tool_attr {
                if let Some(rest) = trimmed.strip_prefix("async fn ") {
                    if let Some(paren) = rest.find('(') {
                        names.insert(rest[..paren].to_string());
                    }
                }
                saw_tool_attr = false;
            }
        }
        names
    }

    /// Spec-generated tool names, indexed by which profiles they belong to.
    fn spec_tools_by_profile() -> HashMap<Profile, BTreeSet<String>> {
        let mut map: HashMap<Profile, BTreeSet<String>> = HashMap::new();
        for ep in SPEC_ENDPOINTS {
            for profile in ep.profiles {
                map.entry(*profile)
                    .or_default()
                    .insert(ep.tool_name.to_string());
            }
        }
        map
    }

    /// All spec-generated tool names as a flat set.
    fn all_spec_tool_names() -> BTreeSet<String> {
        SPEC_ENDPOINTS
            .iter()
            .map(|ep| ep.tool_name.to_string())
            .collect()
    }

    /// For a profile, every manifest tool must be either:
    /// (a) a curated handler in the server source, OR
    /// (b) a generated spec endpoint assigned to this profile.
    ///
    /// Generated tools are dispatched through the universal request tools
    /// (x_get, x_post, x_put, x_delete) — they do not need individual handlers.
    fn assert_parity_for_profile(profile: Profile, server_source: &str, server_label: &str) {
        let manifest = generate_manifest();
        let manifest_tools: BTreeSet<String> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&profile))
            .map(|t| t.name.clone())
            .collect();

        let curated_handlers = extract_tool_fn_names(server_source);
        let spec_by_profile = spec_tools_by_profile();
        let spec_tools = spec_by_profile.get(&profile).cloned().unwrap_or_default();

        let mut missing = Vec::new();
        for tool_name in &manifest_tools {
            let has_curated = curated_handlers.contains(tool_name);
            let has_spec = spec_tools.contains(tool_name);
            if !has_curated && !has_spec {
                missing.push(tool_name.clone());
            }
        }

        assert!(
            missing.is_empty(),
            "PARITY VIOLATION in {server_label}: {} manifest tool(s) have no \
             handler (curated) or spec endpoint (generated): {missing:?}",
            missing.len()
        );

        // Reverse check: every curated handler must appear in the manifest.
        let mut orphan_handlers = Vec::new();
        for handler in &curated_handlers {
            if !manifest_tools.contains(handler) {
                orphan_handlers.push(handler.clone());
            }
        }
        assert!(
            orphan_handlers.is_empty(),
            "ORPHAN HANDLERS in {server_label}: {} handler(s) not in manifest: \
             {orphan_handlers:?}",
            orphan_handlers.len()
        );
    }

    #[test]
    fn readonly_manifest_runtime_parity() {
        let source = include_str!("../../server/readonly.rs");
        assert_parity_for_profile(Profile::Readonly, source, "readonly");
    }

    #[test]
    fn api_readonly_manifest_runtime_parity() {
        let source = include_str!("../../server/api_readonly.rs");
        assert_parity_for_profile(Profile::ApiReadonly, source, "api-readonly");
    }

    #[test]
    fn write_manifest_runtime_parity() {
        let source = include_str!("../../server/write.rs");
        assert_parity_for_profile(Profile::Write, source, "write");
    }

    #[test]
    fn admin_manifest_runtime_parity() {
        let source = include_str!("../../server/admin.rs");
        assert_parity_for_profile(Profile::Admin, source, "admin");
    }

    // ── Cross-profile consistency ────────────────────────────────────

    /// Every spec endpoint must appear in the manifest for each profile it declares.
    #[test]
    fn spec_endpoints_are_in_manifest_for_declared_profiles() {
        let manifest = generate_manifest();
        let manifest_by_name: BTreeMap<String, &crate::tools::manifest::ToolEntry> =
            manifest.tools.iter().map(|t| (t.name.clone(), t)).collect();

        for ep in SPEC_ENDPOINTS {
            let entry = manifest_by_name.get(ep.tool_name).unwrap_or_else(|| {
                panic!("spec endpoint '{}' not found in manifest", ep.tool_name)
            });

            for profile in ep.profiles {
                assert!(
                    entry.profiles.contains(profile),
                    "spec endpoint '{}' declares profile {:?} but manifest \
                     entry does not include it (has {:?})",
                    ep.tool_name,
                    profile,
                    entry.profiles
                );
            }
        }
    }

    /// No tool appears in the manifest without a backing implementation.
    #[test]
    fn every_manifest_tool_has_backing_implementation() {
        let manifest = generate_manifest();
        let spec_names = all_spec_tool_names();

        // Curated handlers across all server files.
        let all_handlers: BTreeSet<String> = [
            include_str!("../../server/readonly.rs"),
            include_str!("../../server/api_readonly.rs"),
            include_str!("../../server/write.rs"),
            include_str!("../../server/admin.rs"),
        ]
        .iter()
        .flat_map(|src| extract_tool_fn_names(src))
        .collect();

        let mut unimplemented = Vec::new();
        for tool in &manifest.tools {
            let has_curated = all_handlers.contains(&tool.name);
            let has_spec = spec_names.contains(&tool.name);
            if !has_curated && !has_spec {
                unimplemented.push(tool.name.clone());
            }
        }

        assert!(
            unimplemented.is_empty(),
            "UNIMPLEMENTED TOOLS: {} tool(s) in manifest with no backing: \
             {unimplemented:?}",
            unimplemented.len()
        );
    }

    /// Manifest tool count must equal curated handlers + spec endpoints
    /// (after dedup — a tool should not be both curated and spec-generated).
    #[test]
    fn no_tool_is_both_curated_and_generated() {
        let spec_names = all_spec_tool_names();

        let all_handlers: BTreeSet<String> = [
            include_str!("../../server/readonly.rs"),
            include_str!("../../server/api_readonly.rs"),
            include_str!("../../server/write.rs"),
            include_str!("../../server/admin.rs"),
        ]
        .iter()
        .flat_map(|src| extract_tool_fn_names(src))
        .collect();

        let overlap: Vec<&String> = all_handlers
            .iter()
            .filter(|h| spec_names.contains(*h))
            .collect();

        assert!(
            overlap.is_empty(),
            "OVERLAP: {} tool(s) are both curated handlers AND spec endpoints: \
             {overlap:?}",
            overlap.len()
        );
    }
}
