//! Endpoint coverage report generator.
//!
//! Introspects the tool manifest to produce a machine-readable (JSON) and
//! human-readable (markdown) coverage report. Categorizes every tool by:
//! - Layer (curated L1 vs generated L2 spec-pack)
//! - Category (read, write, engage, list, moderation, etc.)
//! - Auth requirements (user-context, elevated access, specific scopes)
//! - Test coverage (kernel conformance tests vs untested)
//! - Tier gating (profile availability)

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::spec::SPEC_ENDPOINTS;
use crate::tools::manifest::{generate_manifest, Profile, ToolCategory, ToolEntry};
use crate::tools::test_mocks::artifacts_dir;

/// Coverage status for a single tool.
#[derive(Debug, Serialize)]
struct ToolCoverage {
    name: String,
    category: String,
    layer: String,
    mutation: bool,
    requires_x_client: bool,
    requires_llm: bool,
    requires_db: bool,
    requires_user_auth: bool,
    requires_elevated_access: bool,
    scopes: Vec<String>,
    profiles: Vec<String>,
    has_kernel_conformance_test: bool,
    has_contract_test: bool,
    has_live_test: bool,
    tier_gate: String,
}

/// Summary statistics for the coverage report.
#[derive(Debug, Serialize)]
struct CoverageSummary {
    total_tools: usize,
    curated_tools: usize,
    generated_tools: usize,
    mutation_tools: usize,
    readonly_tools: usize,
    x_client_required: usize,
    llm_required: usize,
    db_required: usize,
    user_auth_required: usize,
    elevated_access_required: usize,
    kernel_conformance_tested: usize,
    contract_tested: usize,
    live_tested: usize,
    untested: usize,
}

/// Breakdown by category.
#[derive(Debug, Serialize)]
struct CategoryBreakdown {
    category: String,
    total: usize,
    curated: usize,
    generated: usize,
    mutation_count: usize,
    tested_count: usize,
}

/// Per-profile tool counts.
#[derive(Debug, Serialize)]
struct ProfileBreakdown {
    profile: String,
    tool_count: usize,
    mutation_count: usize,
    read_count: usize,
}

/// The full coverage report.
#[derive(Debug, Serialize)]
struct CoverageReport {
    generated_at: String,
    mcp_schema_version: String,
    x_api_spec_version: String,
    summary: CoverageSummary,
    categories: Vec<CategoryBreakdown>,
    profiles: Vec<ProfileBreakdown>,
    tools: Vec<ToolCoverage>,
    coverage_gaps: Vec<String>,
    tier_gated_areas: Vec<String>,
    credential_gated_areas: Vec<String>,
}

/// Tools that have dedicated kernel conformance tests (in conformance_tests/).
const KERNEL_CONFORMANCE_TESTED: &[&str] = &[
    "get_tweet_by_id",
    "x_get_user_by_username",
    "x_search_tweets",
    "x_get_user_mentions",
    "x_get_user_tweets",
    "x_get_home_timeline",
    "x_get_me",
    "x_get_followers",
    "x_get_following",
    "x_get_user_by_id",
    "x_get_liked_tweets",
    "x_get_bookmarks",
    "x_get_users_by_ids",
    "x_get_tweet_liking_users",
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
];

/// Tools that have contract tests (in contract_tests.rs).
const CONTRACT_TESTED: &[&str] = &[
    "get_rate_limits",
    "get_action_log",
    "get_action_counts",
    "get_recent_replies",
    "get_reply_count_today",
    "list_target_accounts",
    "list_unreplied_tweets",
    "score_tweet",
    "get_config",
    "validate_config",
    "get_follower_trend",
    "get_top_topics",
    "list_pending",
    "get_pending_count",
    "approve_all",
    "get_mcp_tool_metrics",
    "get_mcp_error_breakdown",
    "topic_performance_snapshot",
    "get_recent_mutations",
    "get_mutation_detail",
];

/// Tools with live conformance tests (in conformance_tests/live.rs).
const LIVE_TESTED: &[&str] = &[
    "get_tweet_by_id",
    "x_get_user_by_username",
    "x_search_tweets",
    "x_get_me",
    "x_get_followers",
    "x_post_tweet",
    "x_delete_tweet",
    "x_like_tweet",
    "x_unlike_tweet",
];

fn layer_for_tool(tool: &ToolEntry) -> &'static str {
    // Spec-pack tools all start with x_v2_ or x_v1_
    let spec_names: BTreeSet<&str> = SPEC_ENDPOINTS.iter().map(|e| e.tool_name).collect();
    if spec_names.contains(tool.name.as_str()) {
        "generated (L2)"
    } else {
        "curated (L1)"
    }
}

fn category_str(cat: ToolCategory) -> &'static str {
    match cat {
        ToolCategory::Read => "read",
        ToolCategory::Write => "write",
        ToolCategory::Engage => "engage",
        ToolCategory::Media => "media",
        ToolCategory::Analytics => "analytics",
        ToolCategory::Approval => "approval",
        ToolCategory::Content => "content",
        ToolCategory::Discovery => "discovery",
        ToolCategory::Scoring => "scoring",
        ToolCategory::Config => "config",
        ToolCategory::Health => "health",
        ToolCategory::Policy => "policy",
        ToolCategory::Telemetry => "telemetry",
        ToolCategory::Context => "context",
        ToolCategory::Composite => "composite",
        ToolCategory::Meta => "meta",
        ToolCategory::List => "list",
        ToolCategory::Moderation => "moderation",
    }
}

fn profile_str(p: Profile) -> &'static str {
    match p {
        Profile::Readonly => "readonly",
        Profile::ApiReadonly => "api_readonly",
        Profile::Write => "write",
        Profile::Admin => "admin",
        Profile::UtilityReadonly => "utility_readonly",
        Profile::UtilityWrite => "utility_write",
    }
}

fn tier_gate(tool: &ToolEntry) -> String {
    if tool.profiles.contains(&Profile::Readonly) {
        "none (all tiers)".to_string()
    } else if tool.profiles.contains(&Profile::ApiReadonly) {
        "api_readonly+".to_string()
    } else if tool.profiles.contains(&Profile::Write) {
        "write+".to_string()
    } else {
        "admin only".to_string()
    }
}

fn generate_report() -> CoverageReport {
    let manifest = generate_manifest();
    let spec_names: BTreeSet<&str> = SPEC_ENDPOINTS.iter().map(|e| e.tool_name).collect();

    let mut tools_coverage: Vec<ToolCoverage> = Vec::new();
    let mut category_map: BTreeMap<String, (usize, usize, usize, usize, usize)> = BTreeMap::new();

    for tool in &manifest.tools {
        let layer = layer_for_tool(tool);
        let cat = category_str(tool.category).to_string();
        let has_kernel = KERNEL_CONFORMANCE_TESTED.contains(&tool.name.as_str());
        let has_contract = CONTRACT_TESTED.contains(&tool.name.as_str());
        let has_live = LIVE_TESTED.contains(&tool.name.as_str());

        tools_coverage.push(ToolCoverage {
            name: tool.name.clone(),
            category: cat.clone(),
            layer: layer.to_string(),
            mutation: tool.mutation,
            requires_x_client: tool.requires_x_client,
            requires_llm: tool.requires_llm,
            requires_db: tool.requires_db,
            requires_user_auth: tool.requires_user_auth,
            requires_elevated_access: tool.requires_elevated_access,
            scopes: tool.requires_scopes.clone(),
            profiles: tool
                .profiles
                .iter()
                .map(|p| profile_str(*p).to_string())
                .collect(),
            has_kernel_conformance_test: has_kernel,
            has_contract_test: has_contract,
            has_live_test: has_live,
            tier_gate: tier_gate(tool),
        });

        let entry = category_map.entry(cat).or_insert((0, 0, 0, 0, 0));
        entry.0 += 1; // total
        if !spec_names.contains(tool.name.as_str()) {
            entry.1 += 1; // curated
        } else {
            entry.2 += 1; // generated
        }
        if tool.mutation {
            entry.3 += 1; // mutation
        }
        if has_kernel || has_contract || has_live {
            entry.4 += 1; // tested
        }
    }

    let categories: Vec<CategoryBreakdown> = category_map
        .into_iter()
        .map(
            |(category, (total, curated, generated, mutation_count, tested_count))| {
                CategoryBreakdown {
                    category,
                    total,
                    curated,
                    generated,
                    mutation_count,
                    tested_count,
                }
            },
        )
        .collect();

    let profiles: Vec<ProfileBreakdown> = [
        Profile::Readonly,
        Profile::ApiReadonly,
        Profile::Write,
        Profile::Admin,
    ]
    .iter()
    .map(|p| {
        let matching: Vec<&ToolEntry> = manifest
            .tools
            .iter()
            .filter(|t| t.profiles.contains(p))
            .collect();
        ProfileBreakdown {
            profile: profile_str(*p).to_string(),
            tool_count: matching.len(),
            mutation_count: matching.iter().filter(|t| t.mutation).count(),
            read_count: matching.iter().filter(|t| !t.mutation).count(),
        }
    })
    .collect();

    let total_tools = manifest.tools.len();
    let curated = manifest
        .tools
        .iter()
        .filter(|t| !spec_names.contains(t.name.as_str()))
        .count();
    let generated = total_tools - curated;
    let mutation_tools = manifest.tools.iter().filter(|t| t.mutation).count();
    let kernel_tested = tools_coverage
        .iter()
        .filter(|t| t.has_kernel_conformance_test)
        .count();
    let contract_tested = tools_coverage
        .iter()
        .filter(|t| t.has_contract_test)
        .count();
    let live_tested = tools_coverage.iter().filter(|t| t.has_live_test).count();
    let untested = tools_coverage
        .iter()
        .filter(|t| !t.has_kernel_conformance_test && !t.has_contract_test && !t.has_live_test)
        .count();

    let coverage_gaps: Vec<String> = tools_coverage
        .iter()
        .filter(|t| !t.has_kernel_conformance_test && !t.has_contract_test && !t.has_live_test)
        .map(|t| format!("{} ({})", t.name, t.category))
        .collect();

    let tier_gated: Vec<String> = tools_coverage
        .iter()
        .filter(|t| t.tier_gate != "none (all tiers)")
        .map(|t| format!("{}: {}", t.name, t.tier_gate))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let credential_gated: Vec<String> = tools_coverage
        .iter()
        .filter(|t| t.requires_user_auth || t.requires_elevated_access || !t.scopes.is_empty())
        .map(|t| {
            let mut gates = Vec::new();
            if t.requires_user_auth {
                gates.push("user_auth");
            }
            if t.requires_elevated_access {
                gates.push("elevated_access");
            }
            if !t.scopes.is_empty() {
                gates.push("scoped");
            }
            format!("{}: [{}]", t.name, gates.join(", "))
        })
        .collect();

    CoverageReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        mcp_schema_version: crate::spec::MCP_SCHEMA_VERSION.to_string(),
        x_api_spec_version: crate::spec::X_API_SPEC_VERSION.to_string(),
        summary: CoverageSummary {
            total_tools,
            curated_tools: curated,
            generated_tools: generated,
            mutation_tools,
            readonly_tools: total_tools - mutation_tools,
            x_client_required: manifest
                .tools
                .iter()
                .filter(|t| t.requires_x_client)
                .count(),
            llm_required: manifest.tools.iter().filter(|t| t.requires_llm).count(),
            db_required: manifest.tools.iter().filter(|t| t.requires_db).count(),
            user_auth_required: manifest
                .tools
                .iter()
                .filter(|t| t.requires_user_auth)
                .count(),
            elevated_access_required: manifest
                .tools
                .iter()
                .filter(|t| t.requires_elevated_access)
                .count(),
            kernel_conformance_tested: kernel_tested,
            contract_tested,
            live_tested,
            untested,
        },
        categories,
        profiles,
        tools: tools_coverage,
        coverage_gaps,
        tier_gated_areas: tier_gated,
        credential_gated_areas: credential_gated,
    }
}

fn report_to_markdown(report: &CoverageReport) -> String {
    let mut md = String::new();
    md.push_str("# MCP Endpoint Coverage Report\n\n");
    md.push_str(&format!("**Generated:** {}\n\n", report.generated_at));
    md.push_str(&format!(
        "**MCP Schema:** {} | **X API Spec:** {}\n\n",
        report.mcp_schema_version, report.x_api_spec_version
    ));

    // Summary
    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Count |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!(
        "| Total tools | {} |\n",
        report.summary.total_tools
    ));
    md.push_str(&format!(
        "| Curated (L1) | {} |\n",
        report.summary.curated_tools
    ));
    md.push_str(&format!(
        "| Generated (L2) | {} |\n",
        report.summary.generated_tools
    ));
    md.push_str(&format!(
        "| Mutation tools | {} |\n",
        report.summary.mutation_tools
    ));
    md.push_str(&format!(
        "| Read-only tools | {} |\n",
        report.summary.readonly_tools
    ));
    md.push_str(&format!(
        "| Requires X client | {} |\n",
        report.summary.x_client_required
    ));
    md.push_str(&format!(
        "| Requires LLM | {} |\n",
        report.summary.llm_required
    ));
    md.push_str(&format!(
        "| Requires DB | {} |\n",
        report.summary.db_required
    ));
    md.push_str(&format!(
        "| Requires user auth | {} |\n",
        report.summary.user_auth_required
    ));
    md.push_str(&format!(
        "| Requires elevated access | {} |\n",
        report.summary.elevated_access_required
    ));

    // Test coverage
    md.push_str("\n## Test Coverage\n\n");
    let tested = report.summary.kernel_conformance_tested
        + report.summary.contract_tested
        + report.summary.live_tested;
    let unique_tested = report.summary.total_tools - report.summary.untested;
    md.push_str(&format!(
        "**{unique_tested}/{} tools have at least one test ({:.1}%)**\n\n",
        report.summary.total_tools,
        unique_tested as f64 / report.summary.total_tools as f64 * 100.0,
    ));
    md.push_str("| Test Type | Count |\n");
    md.push_str("|-----------|-------|\n");
    md.push_str(&format!(
        "| Kernel conformance | {} |\n",
        report.summary.kernel_conformance_tested
    ));
    md.push_str(&format!(
        "| Contract envelope | {} |\n",
        report.summary.contract_tested
    ));
    md.push_str(&format!(
        "| Live (sandbox) | {} |\n",
        report.summary.live_tested
    ));
    md.push_str(&format!("| Total test touches | {} |\n", tested));
    md.push_str(&format!("| Untested | {} |\n", report.summary.untested));

    // Categories
    md.push_str("\n## By Category\n\n");
    md.push_str("| Category | Total | Curated | Generated | Mutations | Tested |\n");
    md.push_str("|----------|-------|---------|-----------|-----------|--------|\n");
    for cat in &report.categories {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            cat.category,
            cat.total,
            cat.curated,
            cat.generated,
            cat.mutation_count,
            cat.tested_count,
        ));
    }

    // Profiles
    md.push_str("\n## By Profile\n\n");
    md.push_str("| Profile | Total | Mutations | Read-Only |\n");
    md.push_str("|---------|-------|-----------|-----------|\n");
    for p in &report.profiles {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            p.profile, p.tool_count, p.mutation_count, p.read_count,
        ));
    }

    // Tier-gated areas
    md.push_str("\n## Tier-Gated Areas\n\n");
    md.push_str("Tools restricted to specific profiles:\n\n");
    let mut tier_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &report.tools {
        *tier_counts
            .entry(match t.tier_gate.as_str() {
                "none (all tiers)" => "all tiers",
                "api_readonly+" => "api_readonly+",
                "write+" => "write+",
                "admin only" => "admin only",
                _ => "other",
            })
            .or_default() += 1;
    }
    for (tier, count) in &tier_counts {
        md.push_str(&format!("- **{tier}**: {count} tools\n"));
    }

    // Credential-gated areas
    md.push_str("\n## Credential-Gated Areas\n\n");
    md.push_str(&format!(
        "{} tools require specific credentials:\n\n",
        report.credential_gated_areas.len()
    ));
    for area in report.credential_gated_areas.iter().take(20) {
        md.push_str(&format!("- {area}\n"));
    }
    if report.credential_gated_areas.len() > 20 {
        md.push_str(&format!(
            "- ... and {} more\n",
            report.credential_gated_areas.len() - 20
        ));
    }

    // Coverage gaps
    md.push_str("\n## Coverage Gaps (Untested Tools)\n\n");
    if report.coverage_gaps.is_empty() {
        md.push_str("All tools have at least one test.\n");
    } else {
        md.push_str(&format!(
            "{} tools lack any test coverage:\n\n",
            report.coverage_gaps.len()
        ));
        for gap in &report.coverage_gaps {
            md.push_str(&format!("- {gap}\n"));
        }
    }

    md
}

#[tokio::test]
async fn generate_coverage_report() {
    let report = generate_report();

    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");

    // JSON report
    let json = serde_json::to_string_pretty(&report).expect("serialize report");
    std::fs::write(dir.join("session-09-coverage-report.json"), &json)
        .expect("write coverage JSON");

    // Markdown report
    let md = report_to_markdown(&report);
    std::fs::write(dir.join("session-09-coverage-report.md"), &md)
        .expect("write coverage markdown");

    // Also write to docs/generated/
    let docs_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/generated");
    std::fs::create_dir_all(&docs_dir).expect("create docs dir");
    std::fs::write(docs_dir.join("coverage-report.json"), &json).expect("write docs coverage JSON");
    std::fs::write(docs_dir.join("coverage-report.md"), &md).expect("write docs coverage markdown");

    // Assertions
    assert!(report.summary.total_tools > 0, "No tools found in manifest");
    assert!(
        report.summary.kernel_conformance_tested >= 27,
        "Expected at least 27 kernel conformance tests, got {}",
        report.summary.kernel_conformance_tested
    );
    assert!(
        report.summary.contract_tested >= 15,
        "Expected at least 15 contract tests, got {}",
        report.summary.contract_tested
    );
}
