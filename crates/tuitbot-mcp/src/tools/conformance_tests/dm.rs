//! Deterministic conformance tests for Direct Message tool family (8 tools).
//!
//! Validates that every DM endpoint in the spec pack produces correct
//! manifest entries, tool schemas, and profile assignments.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::spec::{generate_tool_schemas, SPEC_ENDPOINTS};
    use crate::tools::manifest::{generate_manifest, Lane, Profile, ToolCategory, ToolEntry};

    /// All 8 DM tool names from the charter.
    const DM_READS: &[&str] = &[
        "x_v2_dm_conversations",
        "x_v2_dm_conversation_by_id",
        "x_v2_dm_events",
        "x_v2_dm_events_by_conversation",
        "x_v2_dm_events_by_participant",
    ];

    const DM_MUTATIONS: &[&str] = &[
        "x_v2_dm_send_in_conversation",
        "x_v2_dm_send_to_participant",
        "x_v2_dm_create_group",
    ];

    fn all_dm_tools() -> Vec<&'static str> {
        let mut all: Vec<&str> = DM_READS.to_vec();
        all.extend_from_slice(DM_MUTATIONS);
        all
    }

    fn find_tool<'a>(tools: &'a [ToolEntry], name: &str) -> &'a ToolEntry {
        tools
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("DM tool '{}' not found in manifest", name))
    }

    // ── Existence ────────────────────────────────────────────────────

    #[test]
    fn all_dm_tools_exist_in_manifest() {
        let manifest = generate_manifest();
        let names: BTreeSet<&str> = manifest.tools.iter().map(|t| t.name.as_str()).collect();
        for tool_name in all_dm_tools() {
            assert!(
                names.contains(tool_name),
                "DM tool '{tool_name}' missing from manifest"
            );
        }
    }

    #[test]
    fn all_dm_tools_exist_in_spec_endpoints() {
        let spec_names: BTreeSet<&str> = SPEC_ENDPOINTS.iter().map(|e| e.tool_name).collect();
        for tool_name in all_dm_tools() {
            assert!(
                spec_names.contains(tool_name),
                "DM tool '{tool_name}' missing from SPEC_ENDPOINTS"
            );
        }
    }

    // ── Category ─────────────────────────────────────────────────────

    #[test]
    fn dm_tools_have_direct_message_category() {
        let manifest = generate_manifest();
        for tool_name in all_dm_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.category,
                ToolCategory::DirectMessage,
                "DM tool '{tool_name}' has wrong category: {:?}",
                t.category
            );
        }
    }

    // ── Mutation classification ──────────────────────────────────────

    #[test]
    fn dm_reads_are_not_mutations() {
        let manifest = generate_manifest();
        for tool_name in DM_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.mutation,
                "DM read '{tool_name}' incorrectly marked as mutation"
            );
        }
    }

    #[test]
    fn dm_mutations_are_mutations() {
        let manifest = generate_manifest();
        for tool_name in DM_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.mutation,
                "DM mutation '{tool_name}' not marked as mutation"
            );
        }
    }

    // ── Profile assignments ──────────────────────────────────────────

    #[test]
    fn dm_reads_available_in_api_readonly_write_admin_utility_write() {
        let manifest = generate_manifest();
        let expected_profiles = [
            Profile::ApiReadonly,
            Profile::Write,
            Profile::Admin,
            Profile::UtilityWrite,
        ];
        for tool_name in DM_READS {
            let t = find_tool(&manifest.tools, tool_name);
            for profile in &expected_profiles {
                assert!(
                    t.profiles.contains(profile),
                    "DM read '{tool_name}' missing profile {:?}",
                    profile
                );
            }
            // Must NOT be in Readonly
            assert!(
                !t.profiles.contains(&Profile::Readonly),
                "DM read '{tool_name}' should not be in Readonly"
            );
        }
    }

    #[test]
    fn dm_mutations_available_in_write_admin_utility_write() {
        let manifest = generate_manifest();
        let expected_profiles = [Profile::Write, Profile::Admin, Profile::UtilityWrite];
        for tool_name in DM_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            for profile in &expected_profiles {
                assert!(
                    t.profiles.contains(profile),
                    "DM mutation '{tool_name}' missing profile {:?}",
                    profile
                );
            }
            // Must NOT be in Readonly or ApiReadonly
            assert!(
                !t.profiles.contains(&Profile::Readonly),
                "DM mutation '{tool_name}' should not be in Readonly"
            );
            assert!(
                !t.profiles.contains(&Profile::ApiReadonly),
                "DM mutation '{tool_name}' should not be in ApiReadonly"
            );
        }
    }

    // ── Lane and dependency ──────────────────────────────────────────

    #[test]
    fn dm_reads_use_shared_lane() {
        let manifest = generate_manifest();
        for tool_name in DM_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Shared,
                "DM read '{tool_name}' should use Shared lane"
            );
        }
    }

    #[test]
    fn dm_mutations_with_utility_use_shared_lane() {
        // DM mutations are in UtilityWrite, so they use Shared lane
        // (utility profile mutations bypass the workflow audit layer).
        let manifest = generate_manifest();
        for tool_name in DM_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Shared,
                "DM mutation '{tool_name}' should use Shared lane \
                 (utility profile bypass)"
            );
        }
    }

    // ── OAuth scopes ─────────────────────────────────────────────────

    #[test]
    fn dm_reads_require_dm_read_scope() {
        let manifest = generate_manifest();
        for tool_name in DM_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"dm.read".to_string()),
                "DM read '{tool_name}' missing dm.read scope"
            );
        }
    }

    #[test]
    fn dm_mutations_require_dm_write_scope() {
        let manifest = generate_manifest();
        for tool_name in DM_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"dm.write".to_string()),
                "DM mutation '{tool_name}' missing dm.write scope"
            );
        }
    }

    // ── Host ─────────────────────────────────────────────────────────

    #[test]
    fn dm_tools_use_default_host() {
        for ep in SPEC_ENDPOINTS {
            if all_dm_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.host, None,
                    "DM tool '{}' should use default api.x.com host",
                    ep.tool_name
                );
            }
        }
    }

    // ── API version ──────────────────────────────────────────────────

    #[test]
    fn dm_tools_use_v2_api() {
        for ep in SPEC_ENDPOINTS {
            if all_dm_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.api_version, "v2",
                    "DM tool '{}' should use v2 API version",
                    ep.tool_name
                );
            }
        }
    }

    // ── Schema generation ────────────────────────────────────────────

    #[test]
    fn dm_tools_have_valid_schemas() {
        let schemas = generate_tool_schemas();
        for tool_name in all_dm_tools() {
            let schema = schemas
                .iter()
                .find(|s| s.name == tool_name)
                .unwrap_or_else(|| panic!("no schema for DM tool '{tool_name}'"));

            assert!(
                !schema.description.is_empty(),
                "{tool_name}: empty description"
            );
            assert!(
                schema.path.starts_with('/'),
                "{tool_name}: path must start with /"
            );
            assert_eq!(schema.group, "direct_messages", "{tool_name}: wrong group");
            assert!(
                schema.host.is_none(),
                "{tool_name}: should have no host override"
            );
            assert!(
                schema.input_schema.is_object(),
                "{tool_name}: schema must be object"
            );
        }
    }

    // ── Elevated access ──────────────────────────────────────────────

    #[test]
    fn dm_tools_do_not_require_elevated_access() {
        let manifest = generate_manifest();
        for tool_name in all_dm_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.requires_elevated_access,
                "DM tool '{tool_name}' should not require elevated access \
                 (DM tools are not admin-only)"
            );
        }
    }

    // ── Aggregate count ──────────────────────────────────────────────

    #[test]
    fn dm_family_has_exactly_8_tools() {
        let count = SPEC_ENDPOINTS
            .iter()
            .filter(|ep| ep.category == ToolCategory::DirectMessage)
            .count();
        assert_eq!(count, 8, "DM family has {count} tools (expected 8)");
    }
}
