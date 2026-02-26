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

    /// Tools that are admin-only (universal request tools).
    fn admin_only_tools() -> &'static [&'static str] {
        &["x_get", "x_post", "x_put", "x_delete"]
    }

    /// Extract `#[tool]`-annotated function names from server source code.
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
                saw_tool_attr = false;
            }
        }
        names
    }

    // ── Source-level isolation ───────────────────────────────────────

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
            assert!(!source.contains(module), "readonly.rs imports: {module}");
        }
    }

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
                "api_readonly.rs imports: {module}"
            );
        }
    }

    #[test]
    fn readonly_server_no_write_imports() {
        let source = include_str!("../server/readonly.rs");
        for path in ["kernel::write", "kernel::engage", "kernel::media"] {
            assert!(
                !source.contains(path),
                "readonly.rs has mutation import: {path}"
            );
        }
    }

    #[test]
    fn api_readonly_server_no_write_imports() {
        let source = include_str!("../server/api_readonly.rs");
        for path in ["kernel::write", "kernel::engage", "kernel::media"] {
            assert!(
                !source.contains(path),
                "api_readonly.rs has mutation import: {path}"
            );
        }
    }

    /// write.rs must NOT register universal request tool handlers.
    #[test]
    fn write_server_does_not_register_universal_tools() {
        let source = include_str!("../server/write.rs");
        let fn_names = extract_tool_fn_names(source);
        let admin_tools: HashSet<&str> = admin_only_tools().iter().copied().collect();
        for name in &fn_names {
            assert!(
                !admin_tools.contains(name.as_str()),
                "SAFETY VIOLATION: write.rs registers admin-only tool '{name}'"
            );
        }
    }

    /// admin.rs MUST register all universal request tool handlers.
    #[test]
    fn admin_server_registers_universal_tools() {
        let source = include_str!("../server/admin.rs");
        let fn_names: HashSet<String> = extract_tool_fn_names(source).into_iter().collect();
        for &tool in admin_only_tools() {
            assert!(
                fn_names.contains(tool),
                "admin.rs is missing universal tool handler '{tool}'"
            );
        }
    }

    // ── Lane isolation ──────────────────────────────────────────────

    #[test]
    fn write_admin_only_tools_have_workflow_lane() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            let write_admin_only = !t.profiles.contains(&Profile::Readonly)
                && !t.profiles.contains(&Profile::ApiReadonly)
                && (t.profiles.contains(&Profile::Write) || t.profiles.contains(&Profile::Admin));
            if write_admin_only {
                assert_eq!(
                    t.lane,
                    Lane::Workflow,
                    "tool {} is write/admin-only but has lane {:?}",
                    t.name,
                    t.lane
                );
            }
        }
    }

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

    #[test]
    fn readonly_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .count();
        assert_eq!(count, 14, "Readonly has {count} tools (expected 14)");
    }

    #[test]
    fn api_readonly_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .count();
        assert_eq!(count, 40, "ApiReadonly has {count} tools (expected 40)");
    }

    #[test]
    fn write_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Write))
            .count();
        // 68 curated write + 36 generated - 4 admin-only = 104
        assert_eq!(count, 104, "Write has {count} tools (expected 104)");
    }

    #[test]
    fn admin_profile_tool_count() {
        let manifest = generate_manifest();
        let count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Admin))
            .count();
        // 72 curated + 36 generated = 108 (superset of write)
        assert_eq!(count, 108, "Admin has {count} tools (expected 108)");
    }

    // ── Mutation safety ─────────────────────────────────────────────

    #[test]
    fn readonly_has_no_mutation_tools() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(!t.mutation, "mutation tool {} found in Readonly", t.name);
            }
        }
    }

    #[test]
    fn api_readonly_has_no_mutation_tools() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::ApiReadonly) {
                assert!(!t.mutation, "mutation tool {} found in ApiReadonly", t.name);
            }
        }
    }

    #[test]
    fn readonly_is_subset_of_api_readonly() {
        let manifest = generate_manifest();
        let api_ro: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(
                    api_ro.contains(t.name.as_str()),
                    "Readonly tool {} not in ApiReadonly",
                    t.name
                );
            }
        }
    }

    #[test]
    fn write_is_subset_of_admin() {
        let manifest = generate_manifest();
        let admin: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Admin))
            .map(|t| t.name.as_str())
            .collect();
        let write_count = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Write))
            .count();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Write) {
                assert!(
                    admin.contains(t.name.as_str()),
                    "Write tool '{}' not in Admin",
                    t.name
                );
            }
        }
        assert!(
            admin.len() > write_count,
            "Admin ({}) should have strictly more tools than Write ({})",
            admin.len(),
            write_count
        );
    }

    // ── Explicit mutation denylist ──────────────────────────────────

    #[test]
    fn readonly_denies_known_mutation_tools() {
        let manifest = generate_manifest();
        let ro: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .map(|t| t.name.as_str())
            .collect();
        for &name in mutation_denylist() {
            assert!(!ro.contains(name), "SAFETY: mutation '{name}' in readonly");
        }
    }

    #[test]
    fn api_readonly_denies_known_mutation_tools() {
        let manifest = generate_manifest();
        let api_ro: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();
        for &name in mutation_denylist() {
            assert!(
                !api_ro.contains(name),
                "SAFETY: mutation '{name}' in api-readonly"
            );
        }
    }

    #[test]
    fn denylist_matches_manifest_mutation_set() {
        let manifest = generate_manifest();
        let denylist: HashSet<&str> = mutation_denylist().iter().copied().collect();
        let mutations: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.mutation)
            .map(|t| t.name.as_str())
            .collect();
        for name in &mutations {
            assert!(denylist.contains(name), "mutation '{name}' not in denylist");
        }
        for &name in &denylist {
            assert!(
                mutations.contains(name),
                "denylist '{name}' not mutation:true"
            );
        }
    }

    #[test]
    fn write_profile_excludes_admin_only_tools() {
        let manifest = generate_manifest();
        let write: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Write))
            .map(|t| t.name.as_str())
            .collect();
        for &name in admin_only_tools() {
            assert!(
                !write.contains(name),
                "SAFETY: admin-only tool '{name}' in write profile"
            );
        }
    }

    #[test]
    fn admin_profile_includes_admin_only_tools() {
        let manifest = generate_manifest();
        let admin: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Admin))
            .map(|t| t.name.as_str())
            .collect();
        for &name in admin_only_tools() {
            assert!(admin.contains(name), "admin-only '{name}' not in admin");
        }
    }

    // ── Category-level guards ────────────────────────────────────────

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
                    "tool '{}' has mutation category {:?} in read-only profile",
                    t.name, t.category
                );
            }
        }
    }

    #[test]
    fn every_mutation_tool_is_write_or_admin_only() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.mutation {
                let has_readonly = t.profiles.contains(&Profile::Readonly)
                    || t.profiles.contains(&Profile::ApiReadonly);
                assert!(
                    !has_readonly,
                    "mutation '{}' has read-only profile: {:?}",
                    t.name, t.profiles
                );
            }
        }
    }

    // ── Dependency guards ────────────────────────────────────────────

    #[test]
    fn readonly_tools_require_no_db() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            if t.profiles.contains(&Profile::Readonly) {
                assert!(!t.requires_db, "Readonly '{}' has requires_db", t.name);
            }
        }
    }

    #[test]
    fn readonly_tools_require_no_llm() {
        let manifest = generate_manifest();
        for t in &manifest.tools {
            let in_ro = t.profiles.contains(&Profile::Readonly)
                || t.profiles.contains(&Profile::ApiReadonly);
            if in_ro {
                assert!(!t.requires_llm, "read-only '{}' has requires_llm", t.name);
            }
        }
    }

    // ── Server source cross-checks ───────────────────────────────────

    #[test]
    fn readonly_server_tool_count_matches() {
        let source = include_str!("../server/readonly.rs");
        let fn_names = extract_tool_fn_names(source);
        assert_eq!(
            fn_names.len(),
            10,
            "readonly.rs has {} tools (expected 10): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    #[test]
    fn api_readonly_server_tool_count_matches() {
        let source = include_str!("../server/api_readonly.rs");
        let fn_names = extract_tool_fn_names(source);
        assert_eq!(
            fn_names.len(),
            20,
            "api_readonly.rs has {} tools (expected 20): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    #[test]
    fn write_server_tool_count() {
        let source = include_str!("../server/write.rs");
        let fn_names = extract_tool_fn_names(source);
        // 72 curated - 4 admin-only universal request tools = 68
        assert_eq!(
            fn_names.len(),
            68,
            "write.rs has {} tools (expected 68): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    #[test]
    fn admin_server_tool_count() {
        let source = include_str!("../server/admin.rs");
        let fn_names = extract_tool_fn_names(source);
        // All 72 curated tools including universal request tools
        assert_eq!(
            fn_names.len(),
            72,
            "admin.rs has {} tools (expected 72): {:?}",
            fn_names.len(),
            fn_names
        );
    }

    #[test]
    fn readonly_servers_contain_no_denylist_functions() {
        let denylist: HashSet<&str> = mutation_denylist().iter().copied().collect();
        for (label, source) in [
            ("readonly.rs", include_str!("../server/readonly.rs")),
            ("api_readonly.rs", include_str!("../server/api_readonly.rs")),
        ] {
            for name in &extract_tool_fn_names(source) {
                assert!(
                    !denylist.contains(name.as_str()),
                    "SAFETY: {label} has denylist fn '{name}'"
                );
            }
        }
    }

    // ── Error code guards ────────────────────────────────────────────

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
                        "read-only '{}' has policy error '{}'",
                        t.name,
                        code.as_str()
                    );
                }
            }
        }
    }

    #[test]
    fn api_readonly_is_superset_of_readonly() {
        let manifest = generate_manifest();
        let ro: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::Readonly))
            .map(|t| t.name.as_str())
            .collect();
        let api_ro: HashSet<&str> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(&Profile::ApiReadonly))
            .map(|t| t.name.as_str())
            .collect();
        for name in &ro {
            assert!(
                api_ro.contains(name),
                "Readonly '{name}' missing from ApiReadonly"
            );
        }
        assert!(
            api_ro.len() > ro.len(),
            "ApiReadonly ({}) should be > Readonly ({})",
            api_ro.len(),
            ro.len()
        );
    }

    // ── Profile manifest contract ───────────────────────────────────

    #[test]
    fn profile_manifest_count_matches_len() {
        use crate::state::Profile as SP;
        for p in [SP::Write, SP::Admin, SP::Readonly, SP::ApiReadonly] {
            let m = crate::tools::manifest::generate_profile_manifest(p);
            assert_eq!(
                m.tool_count,
                m.tools.len(),
                "count mismatch for {}: {} vs {}",
                m.profile,
                m.tool_count,
                m.tools.len()
            );
        }
    }

    #[test]
    fn profile_manifest_readonly_no_mutations() {
        use crate::state::Profile as SP;
        for p in [SP::Readonly, SP::ApiReadonly] {
            let m = crate::tools::manifest::generate_profile_manifest(p);
            for t in &m.tools {
                assert!(!t.mutation, "mutation {} in {} manifest", t.name, m.profile);
            }
        }
    }

    #[test]
    fn profile_manifest_field_matches_request() {
        use crate::state::Profile as SP;
        for (p, expected) in [
            (SP::Write, "write"),
            (SP::Admin, "admin"),
            (SP::Readonly, "readonly"),
            (SP::ApiReadonly, "api-readonly"),
        ] {
            let m = crate::tools::manifest::generate_profile_manifest(p);
            assert_eq!(m.profile, expected);
        }
    }

    #[test]
    fn profile_manifest_tools_sorted() {
        use crate::state::Profile as SP;
        for p in [SP::Write, SP::Admin, SP::Readonly, SP::ApiReadonly] {
            let m = crate::tools::manifest::generate_profile_manifest(p);
            let names: Vec<&str> = m.tools.iter().map(|t| t.name.as_str()).collect();
            let mut sorted = names.clone();
            sorted.sort();
            assert_eq!(names, sorted, "unsorted for {}", m.profile);
        }
    }

    #[test]
    fn profile_manifest_serde_roundtrip() {
        use crate::state::Profile as SP;
        for p in [SP::Write, SP::Admin, SP::Readonly, SP::ApiReadonly] {
            let m = crate::tools::manifest::generate_profile_manifest(p);
            let json = serde_json::to_string_pretty(&m).unwrap();
            let d: crate::tools::manifest::ProfileManifest = serde_json::from_str(&json).unwrap();
            assert_eq!(m, d, "roundtrip failed for {}", m.profile);
        }
    }

    #[test]
    fn state_profile_to_manifest_profile_conversion() {
        use crate::state::Profile as SP;
        assert_eq!(Profile::from(SP::Write), Profile::Write);
        assert_eq!(Profile::from(SP::Admin), Profile::Admin);
        assert_eq!(Profile::from(SP::Readonly), Profile::Readonly);
        assert_eq!(Profile::from(SP::ApiReadonly), Profile::ApiReadonly);
    }

    #[test]
    fn profile_manifest_version_triplet() {
        use crate::state::Profile as SP;
        let m = crate::tools::manifest::generate_profile_manifest(SP::Write);
        assert!(m.tuitbot_mcp_version.contains('.'));
        assert_eq!(m.mcp_schema_version, crate::spec::MCP_SCHEMA_VERSION);
        assert_eq!(m.x_api_spec_version, crate::spec::X_API_SPEC_VERSION);
    }

    #[test]
    fn score_tweet_does_not_require_db() {
        let manifest = generate_manifest();
        let t = manifest
            .tools
            .iter()
            .find(|t| t.name == "score_tweet")
            .expect("score_tweet must exist");
        assert!(!t.requires_db);
        assert!(!t
            .possible_error_codes
            .iter()
            .any(|c| c.as_str() == "db_error"));
    }
}
