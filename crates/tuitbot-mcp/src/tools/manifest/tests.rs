//! Tests for the tool manifest.

use std::collections::HashSet;

#[test]
fn manifest_generates_without_panic() {
    let manifest = generate_manifest();
    assert_eq!(manifest.version, crate::spec::MCP_SCHEMA_VERSION);
    assert!(!manifest.tools.is_empty());
}

#[test]
fn no_duplicate_tool_names() {
    let manifest = generate_manifest();
    let mut seen = HashSet::new();
    for t in &manifest.tools {
        assert!(
            seen.insert(t.name.as_str()),
            "duplicate tool name: {}",
            t.name
        );
    }
}

#[test]
fn all_tools_have_at_least_one_profile() {
    let manifest = generate_manifest();
    for t in &manifest.tools {
        assert!(!t.profiles.is_empty(), "tool {} has no profiles", t.name);
    }
}

#[test]
fn mutation_tools_require_x_or_db() {
    let manifest = generate_manifest();
    for t in &manifest.tools {
        if t.mutation {
            assert!(
                t.requires_x_client || t.requires_db,
                "mutation tool {} requires neither x_client nor db",
                t.name
            );
        }
    }
}

#[test]
fn error_codes_are_valid_variants() {
    let all_codes: HashSet<ErrorCode> = ErrorCode::ALL.iter().copied().collect();
    let manifest = generate_manifest();
    for t in &manifest.tools {
        for &code in &t.possible_error_codes {
            assert!(
                all_codes.contains(&code),
                "tool {} references unknown error code {:?}",
                t.name,
                code
            );
        }
    }
}

#[test]
fn category_counts() {
    let manifest = generate_manifest();
    let mut cats: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for t in &manifest.tools {
        let cat = serde_json::to_string(&t.category).unwrap();
        *cats.entry(Box::leak(cat.into_boxed_str())).or_default() += 1;
    }
    // Sanity: we have tools in multiple categories
    assert!(
        cats.len() >= 10,
        "expected at least 10 categories, got {}",
        cats.len()
    );
}

// ── Utility profile boundary tests ────────────────────────────

#[test]
fn utility_readonly_contains_no_workflow_tools() {
    let manifest = generate_profile_manifest(crate::state::Profile::UtilityReadonly);
    for t in &manifest.tools {
        assert_ne!(
            t.lane,
            Lane::Workflow,
            "utility-readonly profile must not include workflow tool: {}",
            t.name
        );
    }
}

#[test]
fn utility_write_contains_no_workflow_tools() {
    let manifest = generate_profile_manifest(crate::state::Profile::UtilityWrite);
    for t in &manifest.tools {
        assert_ne!(
            t.lane,
            Lane::Workflow,
            "utility-write profile must not include workflow tool: {}",
            t.name
        );
    }
}

#[test]
fn utility_profiles_require_no_db_or_llm() {
    for profile in [
        crate::state::Profile::UtilityReadonly,
        crate::state::Profile::UtilityWrite,
    ] {
        let manifest = generate_profile_manifest(profile);
        for t in &manifest.tools {
            assert!(!t.requires_db, "{profile} tool {} requires DB", t.name);
            assert!(!t.requires_llm, "{profile} tool {} requires LLM", t.name);
        }
    }
}

#[test]
fn utility_readonly_contains_no_mutations() {
    let manifest = generate_profile_manifest(crate::state::Profile::UtilityReadonly);
    for t in &manifest.tools {
        assert!(
            !t.mutation,
            "utility-readonly tool {} is marked as mutation",
            t.name
        );
    }
}

#[test]
fn utility_profile_tool_counts() {
    let ro = generate_profile_manifest(crate::state::Profile::UtilityReadonly);
    let rw = generate_profile_manifest(crate::state::Profile::UtilityWrite);
    // utility-readonly: core reads + scoring + config + health
    assert!(
        ro.tool_count >= 10,
        "utility-readonly should have >=10 tools, got {}",
        ro.tool_count
    );
    // utility-write: superset of utility-readonly + writes + engages + extended reads
    assert!(
        rw.tool_count > ro.tool_count,
        "utility-write ({}) should have more tools than utility-readonly ({})",
        rw.tool_count,
        ro.tool_count
    );
}

#[test]
fn utility_write_is_superset_of_utility_readonly_tools() {
    let ro = generate_profile_manifest(crate::state::Profile::UtilityReadonly);
    let rw = generate_profile_manifest(crate::state::Profile::UtilityWrite);
    let rw_names: HashSet<&str> = rw.tools.iter().map(|t| t.name.as_str()).collect();
    // Every tool in utility-readonly manifest should also be in utility-write manifest
    for t in &ro.tools {
        assert!(
            rw_names.contains(t.name.as_str()),
            "utility-readonly tool {} is missing from utility-write profile",
            t.name
        );
    }
}

#[test]
#[ignore] // Run with: cargo test -p tuitbot-mcp write_utility_manifests -- --ignored
fn write_utility_manifests() {
    let docs_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/generated");
    for (profile, filename) in [
        (
            crate::state::Profile::Readonly,
            "mcp-manifest-readonly.json",
        ),
        (
            crate::state::Profile::ApiReadonly,
            "mcp-manifest-api-readonly.json",
        ),
        (crate::state::Profile::Write, "mcp-manifest-write.json"),
        (crate::state::Profile::Admin, "mcp-manifest-admin.json"),
        (
            crate::state::Profile::UtilityReadonly,
            "mcp-manifest-utility-readonly.json",
        ),
        (
            crate::state::Profile::UtilityWrite,
            "mcp-manifest-utility-write.json",
        ),
    ] {
        let manifest = generate_profile_manifest(profile);
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let path = format!("{docs_dir}/{filename}");
        std::fs::write(&path, format!("{json}\n")).unwrap();
        eprintln!("Wrote {path} ({} tools)", manifest.tool_count);
    }
}

#[test]
fn manifest_snapshot() {
    let manifest = generate_manifest();
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let expected_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../roadmap/artifacts/session-06-tool-manifest.json"
    );
    let expected = std::fs::read_to_string(expected_path);
    match expected {
        Ok(content) => {
            // Normalize CRLF/LF so snapshot checks are stable across OSes.
            let normalize_newlines = |s: &str| s.replace("\r\n", "\n");
            assert_eq!(
                normalize_newlines(&json).trim(),
                normalize_newlines(&content).trim(),
                "Tool manifest has drifted from snapshot. \
                 Regenerate with: cargo test -p tuitbot-mcp manifest -- --ignored"
            );
        }
        Err(_) => {
            // First run: write the snapshot.
            std::fs::write(expected_path, &json).unwrap();
        }
    }
}

use crate::contract::error_code::ErrorCode;
use crate::tools::manifest::{generate_manifest, generate_profile_manifest, Lane};
