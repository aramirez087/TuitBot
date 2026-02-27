//! Deterministic conformance tests for Ads/Campaign tool family (16 tools).
//!
//! Validates that every Ads endpoint in the spec pack produces correct
//! manifest entries, tool schemas, host routing, and profile assignments.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::spec::{generate_spec_tools, generate_tool_schemas, SPEC_ENDPOINTS};
    use crate::tools::manifest::{generate_manifest, Lane, Profile, ToolCategory, ToolEntry};

    /// Ads read tools (9 tools).
    const ADS_READS: &[&str] = &[
        "x_ads_accounts",
        "x_ads_account_by_id",
        "x_ads_analytics",
        "x_ads_campaign_by_id",
        "x_ads_campaigns",
        "x_ads_funding_instruments",
        "x_ads_line_items",
        "x_ads_promoted_tweets",
        "x_ads_targeting_criteria",
    ];

    /// Ads mutation tools (7 tools).
    const ADS_MUTATIONS: &[&str] = &[
        "x_ads_campaign_create",
        "x_ads_campaign_delete",
        "x_ads_campaign_update",
        "x_ads_line_item_create",
        "x_ads_promoted_tweet_create",
        "x_ads_targeting_create",
        "x_ads_targeting_delete",
    ];

    fn all_ads_tools() -> Vec<&'static str> {
        let mut all: Vec<&str> = ADS_READS.to_vec();
        all.extend_from_slice(ADS_MUTATIONS);
        all
    }

    fn find_tool<'a>(tools: &'a [ToolEntry], name: &str) -> &'a ToolEntry {
        tools
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("Ads tool '{}' not found in manifest", name))
    }

    // ── Existence ────────────────────────────────────────────────────

    #[test]
    fn all_ads_tools_exist_in_manifest() {
        let manifest = generate_manifest();
        let names: BTreeSet<&str> = manifest.tools.iter().map(|t| t.name.as_str()).collect();
        for tool_name in all_ads_tools() {
            assert!(
                names.contains(tool_name),
                "Ads tool '{tool_name}' missing from manifest"
            );
        }
    }

    #[test]
    fn all_ads_tools_exist_in_spec_endpoints() {
        let spec_names: BTreeSet<&str> = SPEC_ENDPOINTS.iter().map(|e| e.tool_name).collect();
        for tool_name in all_ads_tools() {
            assert!(
                spec_names.contains(tool_name),
                "Ads tool '{tool_name}' missing from SPEC_ENDPOINTS"
            );
        }
    }

    // ── Category ─────────────────────────────────────────────────────

    #[test]
    fn ads_tools_have_ads_category() {
        let manifest = generate_manifest();
        for tool_name in all_ads_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.category,
                ToolCategory::Ads,
                "Ads tool '{tool_name}' has wrong category: {:?}",
                t.category
            );
        }
    }

    // ── Mutation classification ──────────────────────────────────────

    #[test]
    fn ads_reads_are_not_mutations() {
        let manifest = generate_manifest();
        for tool_name in ADS_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.mutation,
                "Ads read '{tool_name}' incorrectly marked as mutation"
            );
        }
    }

    #[test]
    fn ads_mutations_are_mutations() {
        let manifest = generate_manifest();
        for tool_name in ADS_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.mutation,
                "Ads mutation '{tool_name}' not marked as mutation"
            );
        }
    }

    // ── Profile assignments (all Admin-only) ─────────────────────────

    #[test]
    fn all_ads_tools_are_admin_only() {
        let manifest = generate_manifest();
        for tool_name in all_ads_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.profiles,
                vec![Profile::Admin],
                "Ads tool '{tool_name}' should be Admin-only, got {:?}",
                t.profiles
            );
        }
    }

    #[test]
    fn ads_tools_not_in_write_profile() {
        let manifest = generate_manifest();
        let write_tools: BTreeSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Write))
            .map(|t| t.name.as_str())
            .collect();
        for tool_name in all_ads_tools() {
            assert!(
                !write_tools.contains(tool_name),
                "SAFETY: Ads tool '{tool_name}' leaked into Write profile"
            );
        }
    }

    #[test]
    fn ads_tools_not_in_readonly_profiles() {
        let manifest = generate_manifest();
        for tool_name in all_ads_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                !t.profiles.contains(&Profile::Readonly),
                "SAFETY: Ads tool '{tool_name}' in Readonly"
            );
            assert!(
                !t.profiles.contains(&Profile::ApiReadonly),
                "SAFETY: Ads tool '{tool_name}' in ApiReadonly"
            );
        }
    }

    // ── Lane and dependency ──────────────────────────────────────────

    #[test]
    fn ads_reads_use_shared_lane() {
        let manifest = generate_manifest();
        for tool_name in ADS_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Shared,
                "Ads read '{tool_name}' should use Shared lane"
            );
        }
    }

    #[test]
    fn ads_mutations_use_workflow_lane() {
        let manifest = generate_manifest();
        for tool_name in ADS_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert_eq!(
                t.lane,
                Lane::Workflow,
                "Ads mutation '{tool_name}' should use Workflow lane"
            );
        }
    }

    #[test]
    fn ads_mutations_require_db() {
        let manifest = generate_manifest();
        for tool_name in ADS_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_db,
                "Ads mutation '{tool_name}' should require DB for audit"
            );
        }
    }

    // ── Elevated access ──────────────────────────────────────────────

    #[test]
    fn all_ads_tools_require_elevated_access() {
        let manifest = generate_manifest();
        for tool_name in all_ads_tools() {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_elevated_access,
                "Ads tool '{tool_name}' should require elevated access \
                 (admin-only)"
            );
        }
    }

    // ── Host routing ─────────────────────────────────────────────────

    #[test]
    fn ads_tools_target_ads_api_host() {
        for ep in SPEC_ENDPOINTS {
            if all_ads_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.host,
                    Some("ads-api.x.com"),
                    "Ads tool '{}' must target ads-api.x.com",
                    ep.tool_name
                );
            }
        }
    }

    // ── API version ──────────────────────────────────────────────────

    #[test]
    fn ads_tools_use_ads_v12_api() {
        for ep in SPEC_ENDPOINTS {
            if all_ads_tools().contains(&ep.tool_name) {
                assert_eq!(
                    ep.api_version, "ads-v12",
                    "Ads tool '{}' should use ads-v12 API version",
                    ep.tool_name
                );
            }
        }
    }

    // ── OAuth scopes ─────────────────────────────────────────────────

    #[test]
    fn ads_reads_require_ads_read_scope() {
        let manifest = generate_manifest();
        for tool_name in ADS_READS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"ads.read".to_string()),
                "Ads read '{tool_name}' missing ads.read scope"
            );
        }
    }

    #[test]
    fn ads_mutations_require_ads_write_scope() {
        let manifest = generate_manifest();
        for tool_name in ADS_MUTATIONS {
            let t = find_tool(&manifest.tools, tool_name);
            assert!(
                t.requires_scopes.contains(&"ads.write".to_string()),
                "Ads mutation '{tool_name}' missing ads.write scope"
            );
        }
    }

    // ── Schema generation ────────────────────────────────────────────

    #[test]
    fn ads_tools_have_valid_schemas() {
        let schemas = generate_tool_schemas();
        for tool_name in all_ads_tools() {
            let schema = schemas
                .iter()
                .find(|s| s.name == tool_name)
                .unwrap_or_else(|| panic!("no schema for Ads tool '{tool_name}'"));

            assert!(
                !schema.description.is_empty(),
                "{tool_name}: empty description"
            );
            assert!(
                schema.path.starts_with('/'),
                "{tool_name}: path must start with /"
            );
            assert_eq!(schema.group, "ads", "{tool_name}: wrong group");
            assert_eq!(
                schema.host.as_deref(),
                Some("ads-api.x.com"),
                "{tool_name}: must have ads-api.x.com host"
            );
            assert_eq!(
                schema.api_version, "ads-v12",
                "{tool_name}: must use ads-v12"
            );
            assert!(
                schema.input_schema.is_object(),
                "{tool_name}: schema must be object"
            );
        }
    }

    // ── Aggregate count ──────────────────────────────────────────────

    #[test]
    fn ads_family_has_exactly_16_tools() {
        let count = SPEC_ENDPOINTS
            .iter()
            .filter(|ep| ep.category == ToolCategory::Ads)
            .count();
        assert_eq!(count, 16, "Ads family has {count} tools (expected 16)");
    }

    #[test]
    fn ads_family_has_9_reads_and_7_mutations() {
        let tools = generate_spec_tools();
        let ads_tools: Vec<&_> = tools
            .iter()
            .filter(|t| t.category == ToolCategory::Ads)
            .collect();
        let reads = ads_tools.iter().filter(|t| !t.mutation).count();
        let mutations = ads_tools.iter().filter(|t| t.mutation).count();
        assert_eq!(reads, 9, "Ads reads: {reads} (expected 9)");
        assert_eq!(mutations, 7, "Ads mutations: {mutations} (expected 7)");
    }
}
