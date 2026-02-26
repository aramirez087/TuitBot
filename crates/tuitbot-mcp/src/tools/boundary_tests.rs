//! Boundary tests: verify structural isolation between profiles.
//!
//! These tests enforce that the read-only profile servers never import
//! mutation modules, and that the manifest Lane/Profile assignments are correct.

#[cfg(test)]
mod tests {
    use crate::contract::error_code::ErrorCode;
    use crate::tools::manifest::{generate_manifest, Lane, Profile, ToolCategory};
    use std::collections::HashSet;

    // ── Helpers ──────────────────────────────────────────────────────

    /// Hardcoded denylist of tools that perform mutations.
    /// Must stay in sync with `mutation: true` flags in the manifest.
    fn mutation_denylist() -> &'static [&'static str] {
        &[
            // ── Curated Layer 1 mutations ──
            "x_post_tweet",
            "x_reply_to_tweet",
            "x_quote_tweet",
            "x_delete_tweet",
            "x_post_thread",
            "x_like_tweet",
            "x_unlike_tweet",
            "x_follow_user",
            "x_unfollow_user",
            "x_retweet",
            "x_unretweet",
            "x_bookmark_tweet",
            "x_unbookmark_tweet",
            "x_upload_media",
            "approve_item",
            "approve_all",
            "reject_item",
            "propose_and_queue_replies",
            "compose_tweet",
            "x_post",
            "x_put",
            "x_delete",
            // ── Generated Layer 2 mutations ──
            "x_v2_blocks_create",
            "x_v2_blocks_delete",
            "x_v2_lists_create",
            "x_v2_lists_delete",
            "x_v2_lists_follow",
            "x_v2_lists_members_add",
            "x_v2_lists_members_remove",
            "x_v2_lists_pin",
            "x_v2_lists_unfollow",
            "x_v2_lists_update",
            "x_v2_mutes_create",
            "x_v2_mutes_delete",
            "x_v2_tweets_hide_reply",
            "x_v2_tweets_unhide_reply",
            "x_v2_users_pin_tweet",
            "x_v2_users_unpin_tweet",
        ]
    }

    /// Extract `#[tool]`-annotated function names from server source code.
    ///
    /// Scans lines: when `#[tool]` is found, captures the function name from
    /// the next `async fn <name>(` line.
    fn extract_tool_fn_names(source: &str) -> Vec<String> {
        let mut names = Vec::new();
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
                        names.push(rest[..paren].to_string());
                    }
                }
                // Reset regardless — we only look at the line immediately after #[tool].
                saw_tool_attr = false;
            }
        }
        names
    }

    // ── Source-level isolation ───────────────────────────────────────

    /// The readonly server source must not reference any workflow or mutation modules.
    #[test]
    fn readonly_server_does_not_import_workflow_modules() {
        let source = include_str!("../server/readonly.rs");

        let forbidden = [
            "tools::workflow::",
            "tools::analytics::",
            "tools::actions::",
            "tools::approval::",
            "tools::capabilities::",
            "tools::content::",
            "tools::context::",
            "tools::discovery::",
            "tools::health::",
            "tools::policy_gate::",
            "tools::rate_limits::",
            "tools::replies::",
            "tools::targets::",
            "tools::telemetry::",
            "tools::composite::",
            "tools::x_actions::",
        ];

        for module in &forbidden {
            assert!(
                !source.contains(module),
                "readonly.rs imports workflow module: {module}"
            );
        }
    }

    /// The api-readonly server source must not reference any workflow or mutation modules.
    #[test]
    fn api_readonly_server_does_not_import_workflow_modules() {
        let source = include_str!("../server/api_readonly.rs");

        let forbidden = [
            "tools::workflow::",
            "tools::analytics::",
            "tools::actions::",
            "tools::approval::",
            "tools::capabilities::",
            "tools::content::",
            "tools::context::",
            "tools::discovery::",
            "tools::health::",
            "tools::policy_gate::",
            "tools::rate_limits::",
            "tools::replies::",
            "tools::targets::",
            "tools::telemetry::",
            "tools::composite::",
            "tools::x_actions::",
        ];

        for module in &forbidden {
            assert!(
                !source.contains(module),
                "api_readonly.rs imports workflow module: {module}"
            );
        }
    }

    /// readonly.rs must not contain kernel::write, kernel::engage, or kernel::media imports.
    #[test]
    fn readonly_server_no_write_imports() {
        let source = include_str!("../server/readonly.rs");
        for path in ["kernel::write", "kernel::engage", "kernel::media"] {
            assert!(
                !source.contains(path),
                "readonly.rs contains mutation import: {path}"
            );
        }
    }

    /// api_readonly.rs must not contain kernel::write, kernel::engage, or kernel::media imports.
    #[test]
    fn api_readonly_server_no_write_imports() {
        let source = include_str!("../server/api_readonly.rs");
        for path in ["kernel::write", "kernel::engage", "kernel::media"] {
            assert!(
                !source.contains(path),
                "api_readonly.rs contains mutation import: {path}"
            );
        }
    }

    // ── Lane isolation ──────────────────────────────────────────────

    /// Every workflow-only tool (WF profile only) must have Lane::Workflow.
    #[test]
    fn workflow_only_tools_have_workflow_lane() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            let wf_only = t.profiles == vec![Profile::Workflow];
            if wf_only {
                assert_eq!(
                    t.lane,
                    Lane::Workflow,
                    "tool {} is WF-only but has lane {:?}",
                    t.name,
                    t.lane
                );
            }
        }
    }

    /// Every tool in a read-only profile must have Lane::Shared.
    #[test]
    fn readonly_tools_have_shared_lane() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            let in_ro = t.profiles.contains(&Profile::Readonly)
                || t.profiles.contains(&Profile::ApiReadonly);
            if in_ro {
                assert_eq!(
                    t.lane,
                    Lane::Shared,
                    "tool {} is in a read-only profile but has lane {:?}",
                    t.name,
                    t.lane
                );
            }
        }
    }

    // ── Profile tool counts ─────────────────────────────────────────

    /// Drift guard: Readonly profile tool count (10 curated + 4 generated).
    #[test]
    fn readonly_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .count();
        assert_eq!(
            count, 14,
            "Readonly profile has {count} tools (expected 14)"
        );
    }

    /// Drift guard: ApiReadonly profile tool count (20 curated + 20 generated).
    #[test]
    fn api_readonly_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .count();
        assert_eq!(
            count, 40,
            "ApiReadonly profile has {count} tools (expected 40)"
        );
    }

    /// Drift guard: Workflow profile tool count (68 curated + 36 generated).
    #[test]
    fn workflow_profile_tool_count() {
        let manifest = generate_manifest();
        let wf_count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Workflow))
            .count();
        assert_eq!(
            wf_count, 104,
            "Workflow profile has {wf_count} tools (expected 104)"
        );
    }

    // ── Mutation safety ─────────────────────────────────────────────

    /// No tool with `mutation: true` may appear in the Readonly profile.
    #[test]
    fn readonly_has_no_mutation_tools() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(
                    !t.mutation,
                    "mutation tool {} found in Readonly profile",
                    t.name
                );
            }
        }
    }

    /// No tool with `mutation: true` may appear in the ApiReadonly profile.
    #[test]
    fn api_readonly_has_no_mutation_tools() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::ApiReadonly) {
                assert!(
                    !t.mutation,
                    "mutation tool {} found in ApiReadonly profile",
                    t.name
                );
            }
        }
    }

    /// Every Readonly tool name must also exist in ApiReadonly (subset relationship).
    #[test]
    fn readonly_is_subset_of_api_readonly() {
        let manifest = generate_manifest();
        let api_ro_names: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();

        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(
                    api_ro_names.contains(t.name.as_str()),
                    "Readonly tool {} is not in ApiReadonly profile",
                    t.name
                );
            }
        }
    }

    // ── Explicit mutation denylist ──────────────────────────────────

    /// Hardcoded denylist check: no known mutation tool may appear in Readonly.
    #[test]
    fn readonly_denies_known_mutation_tools() {
        let manifest = generate_manifest();
        let ro_names: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .map(|t| t.name.as_str())
            .collect();

        for &name in mutation_denylist() {
            assert!(
                !ro_names.contains(name),
                "SAFETY VIOLATION: mutation tool '{name}' found in readonly profile"
            );
        }
    }

    /// Hardcoded denylist check: no known mutation tool may appear in ApiReadonly.
    #[test]
    fn api_readonly_denies_known_mutation_tools() {
        let manifest = generate_manifest();
        let api_ro_names: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();

        for &name in mutation_denylist() {
            assert!(
                !api_ro_names.contains(name),
                "SAFETY VIOLATION: mutation tool '{name}' found in api-readonly profile"
            );
        }
    }

    /// Cross-check: every `mutation: true` tool must be in the denylist,
    /// and every denylist entry must be `mutation: true` in the manifest.
    /// Catches stale denylists AND missed mutation flags.
    #[test]
    fn denylist_matches_manifest_mutation_set() {
        let manifest = generate_manifest();
        let denylist: HashSet<&str> = mutation_denylist().iter().copied().collect();

        let manifest_mutations: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.mutation)
            .map(|t| t.name.as_str())
            .collect();

        // Every manifest mutation tool must be in the denylist.
        for name in &manifest_mutations {
            assert!(
                denylist.contains(name),
                "manifest has mutation tool '{name}' not in denylist — add it"
            );
        }

        // Every denylist entry must be a mutation tool in the manifest.
        for &name in &denylist {
            assert!(
                manifest_mutations.contains(name),
                "denylist entry '{name}' is not mutation:true in manifest — stale entry?"
            );
        }
    }

    // ── Category-level guards ────────────────────────────────────────

    /// No tool with a mutation category (Write, Engage, Media, Approval) may
    /// appear in Readonly or ApiReadonly profiles.
    #[test]
    fn readonly_profiles_exclude_mutation_categories() {
        let manifest = generate_manifest();
        let mutation_cats = [
            ToolCategory::Write,
            ToolCategory::Engage,
            ToolCategory::Media,
            ToolCategory::Approval,
        ];

        for t in &manifest.tools {
            let in_ro = t.profiles.contains(&Profile::Readonly)
                || t.profiles.contains(&Profile::ApiReadonly);
            if in_ro && mutation_cats.contains(&t.category) {
                panic!(
                    "tool '{}' has mutation category {:?} but is in a read-only profile",
                    t.name, t.category
                );
            }
        }
    }

    /// Every tool with `mutation: true` must have `profiles == [Workflow]` only.
    #[test]
    fn every_mutation_tool_is_workflow_only() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.mutation {
                assert_eq!(
                    t.profiles,
                    vec![Profile::Workflow],
                    "mutation tool '{}' has profiles {:?} — expected [Workflow] only",
                    t.name,
                    t.profiles
                );
            }
        }
    }

    // ── Dependency guards ────────────────────────────────────────────

    /// No tool in the Readonly profile may require database access.
    #[test]
    fn readonly_tools_require_no_db() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(
                    !t.requires_db,
                    "Readonly tool '{}' has requires_db: true",
                    t.name
                );
            }
        }
    }

    /// No tool in Readonly or ApiReadonly profiles may require an LLM provider.
    #[test]
    fn readonly_tools_require_no_llm() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            let in_ro = t.profiles.contains(&Profile::Readonly)
                || t.profiles.contains(&Profile::ApiReadonly);
            if in_ro {
                assert!(
                    !t.requires_llm,
                    "read-only tool '{}' has requires_llm: true",
                    t.name
                );
            }
        }
    }

    // ── Server source cross-checks ───────────────────────────────────

    /// The number of `#[tool]` annotations in readonly.rs must match
    /// the manifest Readonly profile count (10).
    #[test]
    fn readonly_server_tool_count_matches_manifest() {
        let source = include_str!("../server/readonly.rs");
        let fn_names = extract_tool_fn_names(source);
        assert_eq!(
            fn_names.len(),
            10,
            "readonly.rs has {} #[tool] functions (expected 10): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    /// The number of `#[tool]` annotations in api_readonly.rs must match
    /// the manifest ApiReadonly profile count (20).
    #[test]
    fn api_readonly_server_tool_count_matches_manifest() {
        let source = include_str!("../server/api_readonly.rs");
        let fn_names = extract_tool_fn_names(source);
        assert_eq!(
            fn_names.len(),
            20,
            "api_readonly.rs has {} #[tool] functions (expected 20): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    /// No `#[tool]`-annotated function in readonly.rs or api_readonly.rs may
    /// share a name with any denylist entry.
    #[test]
    fn readonly_servers_contain_no_denylist_functions() {
        let denylist: HashSet<&str> = mutation_denylist().iter().copied().collect();

        for (label, source) in [
            ("readonly.rs", include_str!("../server/readonly.rs")),
            ("api_readonly.rs", include_str!("../server/api_readonly.rs")),
        ] {
            let fn_names = extract_tool_fn_names(source);
            for name in &fn_names {
                assert!(
                    !denylist.contains(name.as_str()),
                    "SAFETY VIOLATION: {label} contains denylist function '{name}'"
                );
            }
        }
    }

    // ── Tighten existing + error code guards ─────────────────────────

    /// No tool in Readonly or ApiReadonly profiles may declare policy error codes
    /// that only apply to mutation paths.
    #[test]
    fn readonly_profiles_have_no_policy_error_codes() {
        let manifest = generate_manifest();
        let policy_codes: HashSet<ErrorCode> = HashSet::from([
            ErrorCode::PolicyDeniedBlocked,
            ErrorCode::PolicyDeniedRateLimited,
            ErrorCode::PolicyDeniedHardRule,
            ErrorCode::PolicyDeniedUserRule,
            ErrorCode::PolicyError,
            ErrorCode::ScraperMutationBlocked,
        ]);

        for t in &manifest.tools {
            let in_ro = t.profiles.contains(&Profile::Readonly)
                || t.profiles.contains(&Profile::ApiReadonly);
            if in_ro {
                for code in &t.possible_error_codes {
                    assert!(
                        !policy_codes.contains(code),
                        "read-only tool '{}' declares policy error code '{}'",
                        t.name,
                        code.as_str()
                    );
                }
            }
        }
    }

    /// ApiReadonly must be a strict superset of Readonly:
    /// every Readonly tool is in ApiReadonly, and ApiReadonly has strictly more.
    #[test]
    fn api_readonly_is_superset_of_readonly() {
        let manifest = generate_manifest();
        let ro_names: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .map(|t| t.name.as_str())
            .collect();
        let api_ro_names: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();

        // Every Readonly tool must be in ApiReadonly.
        for name in &ro_names {
            assert!(
                api_ro_names.contains(name),
                "Readonly tool '{name}' missing from ApiReadonly"
            );
        }

        // ApiReadonly must have strictly more tools.
        assert!(
            api_ro_names.len() > ro_names.len(),
            "ApiReadonly ({}) should have strictly more tools than Readonly ({})",
            api_ro_names.len(),
            ro_names.len()
        );
    }

    // ── Regression ──────────────────────────────────────────────────

    // ── Profile manifest contract ───────────────────────────────────

    /// For each profile: tool_count == tools.len().
    #[test]
    fn profile_manifest_count_matches_len() {
        use crate::state::Profile as StateProfile;
        for profile in [
            StateProfile::Full,
            StateProfile::Readonly,
            StateProfile::ApiReadonly,
        ] {
            let m = crate::tools::manifest::generate_profile_manifest(profile);
            assert_eq!(
                m.tool_count,
                m.tools.len(),
                "tool_count mismatch for profile {}: count={}, len={}",
                m.profile,
                m.tool_count,
                m.tools.len()
            );
        }
    }

    /// Readonly and ApiReadonly manifests have zero mutation tools.
    #[test]
    fn profile_manifest_readonly_no_mutations() {
        use crate::state::Profile as StateProfile;
        for profile in [StateProfile::Readonly, StateProfile::ApiReadonly] {
            let m = crate::tools::manifest::generate_profile_manifest(profile);
            for t in &m.tools {
                assert!(
                    !t.mutation,
                    "mutation tool {} found in {} profile manifest",
                    t.name, m.profile
                );
            }
        }
    }

    /// Profile field matches the input profile string.
    #[test]
    fn profile_manifest_field_matches_request() {
        use crate::state::Profile as StateProfile;
        let cases = [
            (StateProfile::Full, "full"),
            (StateProfile::Readonly, "readonly"),
            (StateProfile::ApiReadonly, "api-readonly"),
        ];
        for (profile, expected) in cases {
            let m = crate::tools::manifest::generate_profile_manifest(profile);
            assert_eq!(
                m.profile, expected,
                "profile field mismatch: got {}, expected {}",
                m.profile, expected
            );
        }
    }

    /// Tool names are in ascending alphabetical order.
    #[test]
    fn profile_manifest_tools_sorted() {
        use crate::state::Profile as StateProfile;
        for profile in [
            StateProfile::Full,
            StateProfile::Readonly,
            StateProfile::ApiReadonly,
        ] {
            let m = crate::tools::manifest::generate_profile_manifest(profile);
            let names: Vec<&str> = m.tools.iter().map(|t| t.name.as_str()).collect();
            let mut sorted = names.clone();
            sorted.sort();
            assert_eq!(names, sorted, "tools not sorted for profile {}", m.profile);
        }
    }

    /// Serialize → deserialize → assert equal.
    #[test]
    fn profile_manifest_serde_roundtrip() {
        use crate::state::Profile as StateProfile;
        for profile in [
            StateProfile::Full,
            StateProfile::Readonly,
            StateProfile::ApiReadonly,
        ] {
            let m = crate::tools::manifest::generate_profile_manifest(profile);
            let json = serde_json::to_string_pretty(&m).unwrap();
            let deserialized: crate::tools::manifest::ProfileManifest =
                serde_json::from_str(&json).unwrap();
            assert_eq!(
                m, deserialized,
                "roundtrip failed for profile {}",
                m.profile
            );
        }
    }

    /// Full→Workflow, Readonly→Readonly, ApiReadonly→ApiReadonly.
    #[test]
    fn state_profile_to_manifest_profile_conversion() {
        use crate::state::Profile as StateProfile;
        assert_eq!(Profile::from(StateProfile::Full), Profile::Workflow);
        assert_eq!(Profile::from(StateProfile::Readonly), Profile::Readonly);
        assert_eq!(
            Profile::from(StateProfile::ApiReadonly),
            Profile::ApiReadonly
        );
    }

    /// Version triplet: tuitbot_mcp_version is semver, mcp_schema_version
    /// and x_api_spec_version are non-empty.
    #[test]
    fn profile_manifest_version_triplet() {
        use crate::state::Profile as StateProfile;
        let m = crate::tools::manifest::generate_profile_manifest(StateProfile::Full);
        assert!(
            m.tuitbot_mcp_version.contains('.'),
            "tuitbot_mcp_version '{}' does not look like semver",
            m.tuitbot_mcp_version
        );
        assert_eq!(m.mcp_schema_version, crate::spec::MCP_SCHEMA_VERSION);
        assert_eq!(m.x_api_spec_version, crate::spec::X_API_SPEC_VERSION);
    }

    // ── Regression ──────────────────────────────────────────────────

    /// Regression: score_tweet is a pure function on &Config, no DB required.
    #[test]
    fn score_tweet_does_not_require_db() {
        let manifest = generate_manifest();
        let score = manifest.tools.iter().find(|t| t.name == "score_tweet");
        let t = score.expect("score_tweet must be in manifest");
        assert!(
            !t.requires_db,
            "score_tweet should not require DB (pure function)"
        );
        assert!(
            !t.possible_error_codes
                .iter()
                .any(|c| c.as_str() == "db_error"),
            "score_tweet should not list DbError"
        );
    }
}
