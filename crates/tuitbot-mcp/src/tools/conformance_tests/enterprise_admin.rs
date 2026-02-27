//! Deterministic conformance tests for enterprise admin tool family (7 tools).
//!
//! Covers both Compliance (4 tools) and Stream Rules (3 tools), all
//! restricted to Admin profile with elevated access and policy gating.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::spec::{generate_spec_tools, generate_tool_schemas, SPEC_ENDPOINTS};
    use crate::tools::manifest::{generate_manifest, Lane, Profile, ToolCategory, ToolEntry};

    /// Compliance read tools (3 reads + 1 usage read = 4).
    const COMPLIANCE_READS: &[&str] = &[
        "x_v2_compliance_job_by_id",
        "x_v2_compliance_jobs",
        "x_v2_stream_rules_list",
        "x_v2_usage_tweets",
    ];

    /// Compliance/stream mutation tools (3).
    const COMPLIANCE_MUTATIONS: &[&str] = &[
        "x_v2_compliance_job_create",
        "x_v2_stream_rules_add",
        "x_v2_stream_rules_delete",
    ];

    fn all_enterprise_tools() -> Vec<&'static str> {
        let mut all: Vec<&str> = COMPLIANCE_READS.to_vec();
        all.extend_from_slice(COMPLIANCE_MUTATIONS);
        all
    }

    fn find_tool<'a>(tools: &'a [ToolEntry], name: &str) -> &'a ToolEntry {
        tools
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("Enterprise tool '{}' not found in manifest", name))
    }

    // ── Existence ────────────────────────────────────────────────────

    #[test]
    fn all_enterprise_tools_exist_in_manifest() {
        let manifest = generate_manifest();
        let names: BTreeSet<&str> = manifest.tools.iter().map(|t| t.name.as_str()).collect();
        for tool_name in all_enterprise_tools() {
            assert!(
                names.contains(tool_name),
                "Enterprise tool '{tool_name}' missing from manifest"
            );
        }
    }

    #[test]
    fn all_enterprise_tools_exist_in_spec_endpoints() {
        let spec_names: BTreeSet<&str> = SPEC_ENDPOINTS.iter().map(|e| e.tool_name).collect();
        for tool_name in all_enterprise_tools() {
            assert!(
                spec_names.contains(tool_name),
                "Enterprise tool '{tool_name}' missing from SPEC_ENDPOINTS"
            );
        }
    }

    // ── Category ─────────────────────────────────────────────────────

    #[test]
    fn enterprise_tools_have_compliance_category() {
        let manifest = generate_manifest();
        for tool_name in all_enterprise_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.category,
                ToolCategory::Compliance,
                "Enterprise tool '{tool_name}' has wrong category: {:?}",
                t.category
            );
        }
    }

    // ── Mutation classification ──────────────────────────────────────

    #[test]
    fn compliance_reads_are_not_mutations() {
        let manifest = generate_manifest();
        for tool_name in COMPLIANCE_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.mutation,
                "Enterprise read '{tool_name}' incorrectly marked as mutation"
            );
        }
    }

    #[test]
    fn compliance_mutations_are_mutations() {
        let manifest = generate_manifest();
        for tool_name in COMPLIANCE_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.mutation,
                "Enterprise mutation '{tool_name}' not marked as mutation"
            );
        }
    }

    // ── Profile assignments (all Admin-only) ─────────────────────────

    #[test]
    fn all_enterprise_tools_are_admin_only() {
        let manifest = generate_manifest();
        for tool_name in all_enterprise_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.profiles,
                vec![Profile::Admin],
                "Enterprise tool '{tool_name}' should be Admin-only, got {:?}",
                t.profiles
            );
        }
    }

    #[test]
    fn enterprise_tools_not_in_write_profile() {
        let manifest = generate_manifest();
        let write_tools: BTreeSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Write))
            .map(|t| t.name.as_str())
            .collect();
        for tool_name in all_enterprise_tools() {
            assert!(
                !write_tools.contains(tool_name),
                "SAFETY: Enterprise tool '{tool_name}' leaked into Write profile"
            );
        }
    }

    #[test]
    fn enterprise_tools_not_in_readonly_profiles() {
        let manifest = generate_manifest();
        for tool_name in all_enterprise_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.profiles.contains(&Profile::Readonly),
                "SAFETY: Enterprise tool '{tool_name}' in Readonly"
            );
            assert!(
                !t.profiles.contains(&Profile::ApiReadonly),
                "SAFETY: Enterprise tool '{tool_name}' in ApiReadonly"
            );
        }
    }

    // ── Lane and dependency ──────────────────────────────────────────

    #[test]
    fn compliance_reads_use_shared_lane() {
        let manifest = generate_manifest();
        for tool_name in COMPLIANCE_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Shared,
                "Enterprise read '{tool_name}' should use Shared lane"
            );
        }
    }

    #[test]
    fn compliance_mutations_use_workflow_lane() {
        let manifest = generate_manifest();
        for tool_name in COMPLIANCE_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Workflow,
                "Enterprise mutation '{tool_name}' should use Workflow lane"
            );
        }
    }

    #[test]
    fn compliance_mutations_require_db() {
        let manifest = generate_manifest();
        for tool_name in COMPLIANCE_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_db,
                "Enterprise mutation '{tool_name}' should require DB for audit"
            );
        }
    }

    // ── Elevated access ──────────────────────────────────────────────

    #[test]
    fn all_enterprise_tools_require_elevated_access() {
        let manifest = generate_manifest();
        for tool_name in all_enterprise_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_elevated_access,
                "Enterprise tool '{tool_name}' should require elevated access \
                 (admin-only)"
            );
        }
    }

    // ── Host routing ─────────────────────────────────────────────────

    #[test]
    fn enterprise_tools_use_default_host() {
        for ep in SPEC_ENDPOINTS {
            if all_enterprise_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.host, None,
                    "Enterprise tool '{}' should use default api.x.com host",
                    ep.tool_name
                );
            }
        }
    }

    // ── API version ──────────────────────────────────────────────────

    #[test]
    fn enterprise_tools_use_v2_api() {
        for ep in SPEC_ENDPOINTS {
            if all_enterprise_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.api_version, "v2",
                    "Enterprise tool '{}' should use v2 API version",
                    ep.tool_name
                );
            }
        }
    }

    // ── OAuth scopes ─────────────────────────────────────────────────

    #[test]
    fn compliance_job_tools_require_compliance_write_scope() {
        let manifest = generate_manifest();
        let compliance_job_tools = [
            "x_v2_compliance_jobs",
            "x_v2_compliance_job_by_id",
            "x_v2_compliance_job_create",
        ];
        for tool_name in &compliance_job_tools {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"compliance.write".to_string()),
                "Compliance job tool '{tool_name}' missing compliance.write scope"
            );
        }
    }

    #[test]
    fn usage_tweets_requires_usage_read_scope() {
        let manifest = generate_manifest();
        let t = find_tool(&manifest.tools, "x_v2_usage_tweets");
        assert!(
            t.requires_scopes.contains(&"usage.read".to_string()),
            "x_v2_usage_tweets missing usage.read scope"
        );
    }

    #[test]
    fn stream_rules_require_tweet_read_scope() {
        let manifest = generate_manifest();
        let stream_tools = [
            "x_v2_stream_rules_list",
            "x_v2_stream_rules_add",
            "x_v2_stream_rules_delete",
        ];
        for tool_name in &stream_tools {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"tweet.read".to_string()),
                "Stream rule tool '{tool_name}' missing tweet.read scope"
            );
        }
    }

    // ── Schema generation ────────────────────────────────────────────

    #[test]
    fn enterprise_tools_have_valid_schemas() {
        let schemas = generate_tool_schemas();
        for tool_name in all_enterprise_tools() {
            let schema = schemas
                .iter()
                .find(|s| s.name == tool_name)
                .unwrap_or_else(|| panic!("no schema for '{tool_name}'"));

            assert!(
                !schema.description.is_empty(),
                "{tool_name}: empty description"
            );
            assert!(
                schema.path.starts_with('/'),
                "{tool_name}: path must start with /"
            );
            assert_eq!(schema.group, "compliance", "{tool_name}: wrong group");
            assert!(
                schema.host.is_none(),
                "{tool_name}: should have no host override"
            );
            assert_eq!(schema.api_version, "v2", "{tool_name}: must use v2");
            assert!(
                schema.input_schema.is_object(),
                "{tool_name}: schema must be object"
            );
        }
    }

    // ── Stream rules delete uses POST ────────────────────────────────

    #[test]
    fn stream_rules_delete_uses_post_method() {
        let ep = SPEC_ENDPOINTS
            .iter()
            .find(|e| e.tool_name == "x_v2_stream_rules_delete")
            .expect("x_v2_stream_rules_delete not found");
        assert_eq!(
            format!("{:?}", ep.method),
            "Post",
            "x_v2_stream_rules_delete should use POST (X API design quirk)"
        );
    }

    // ── Aggregate counts ─────────────────────────────────────────────

    #[test]
    fn compliance_family_has_exactly_7_tools() {
        let count = SPEC_ENDPOINTS
            .iter()
            .filter(|ep| ep.category == ToolCategory::Compliance)
            .count();
        assert_eq!(count, 7, "Compliance family has {count} tools (expected 7)");
    }

    #[test]
    fn compliance_family_has_4_reads_and_3_mutations() {
        let tools = generate_spec_tools();
        let compliance_tools: Vec<&_> = tools
            .iter()
            .filter(|t| t.category == ToolCategory::Compliance)
            .collect();
        let reads = compliance_tools.iter().filter(|t| !t.mutation).count();
        let mutations = compliance_tools.iter().filter(|t| t.mutation).count();
        assert_eq!(reads, 4, "Compliance reads: {reads} (expected 4)");
        assert_eq!(
            mutations, 3,
            "Compliance mutations: {mutations} (expected 3)"
        );
    }
}
