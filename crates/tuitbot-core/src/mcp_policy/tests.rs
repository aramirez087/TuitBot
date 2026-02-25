//! Tests for the MCP policy evaluator.

use super::evaluator::{McpPolicyEvaluator, PolicyDecision, PolicyDenialReason};
use super::rules::{build_effective_rules, conditions_match, make_eval_context};
use super::templates::{get_template, list_templates};
use super::types::{
    PolicyAction, PolicyRateLimit, PolicyRule, PolicyTemplateName, RateLimitDimension,
    RuleConditions, ToolCategory,
};
use crate::config::{McpPolicyConfig, OperatingMode};
use crate::storage::{self, rate_limits};

fn default_policy() -> McpPolicyConfig {
    McpPolicyConfig::default()
}

fn enforcement_disabled() -> McpPolicyConfig {
    McpPolicyConfig {
        enforce_for_mutations: false,
        ..default_policy()
    }
}

fn with_blocked(tools: Vec<&str>) -> McpPolicyConfig {
    McpPolicyConfig {
        blocked_tools: tools.into_iter().map(String::from).collect(),
        ..default_policy()
    }
}

fn dry_run_policy() -> McpPolicyConfig {
    McpPolicyConfig {
        dry_run_mutations: true,
        ..default_policy()
    }
}

fn no_approval_policy() -> McpPolicyConfig {
    McpPolicyConfig {
        require_approval_for: Vec::new(),
        ..default_policy()
    }
}

// =========================================================================
// Backward compatibility tests (v1 behavior preserved)
// =========================================================================

#[tokio::test]
async fn enforcement_disabled_allows_all() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = enforcement_disabled();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert_eq!(decision, PolicyDecision::Allow);
}

#[tokio::test]
async fn blocked_tool_denied() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = with_blocked(vec!["post_tweet"]);
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::ToolBlocked,
            ..
        }
    ));
}

#[tokio::test]
async fn dry_run_returns_dry_run() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = dry_run_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert!(matches!(decision, PolicyDecision::DryRun { .. }));
}

#[tokio::test]
async fn rate_limit_exceeded_denies() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 2)
        .await
        .expect("init rate limit");

    // Exhaust the limit
    rate_limits::increment_rate_limit(&pool, "mcp_mutation")
        .await
        .expect("inc");
    rate_limits::increment_rate_limit(&pool, "mcp_mutation")
        .await
        .expect("inc");

    let config = no_approval_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::RateLimited,
            ..
        }
    ));
}

#[tokio::test]
async fn approval_required_routes() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = default_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert!(matches!(decision, PolicyDecision::RouteToApproval { .. }));
}

#[tokio::test]
async fn non_approval_tool_allowed() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = default_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "unfollow_user")
            .await
            .expect("evaluate");

    assert_eq!(decision, PolicyDecision::Allow);
}

#[tokio::test]
async fn composer_mode_forces_approval() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = no_approval_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Composer, "unfollow_user")
            .await
            .expect("evaluate");

    assert!(matches!(decision, PolicyDecision::RouteToApproval { .. }));
}

#[tokio::test]
async fn audit_record_logged() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = default_policy();
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    McpPolicyEvaluator::log_decision(&pool, "post_tweet", &decision)
        .await
        .expect("log");

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT action_type, status FROM action_log WHERE action_type = 'mcp_policy'",
    )
    .fetch_all(&pool)
    .await
    .expect("query");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "mcp_policy");
    assert_eq!(rows[0].1, "routed_to_approval");
}

#[tokio::test]
async fn record_mutation_increments_counter() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    McpPolicyEvaluator::record_mutation(&pool, "post_tweet", &[])
        .await
        .expect("record");

    let limits = rate_limits::get_all_rate_limits(&pool)
        .await
        .expect("get limits");
    let mcp = limits
        .iter()
        .find(|l| l.action_type == "mcp_mutation")
        .expect("mcp_mutation row");
    assert_eq!(mcp.request_count, 1);
}

#[tokio::test]
async fn blocked_takes_priority_over_dry_run() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = McpPolicyConfig {
        blocked_tools: vec!["post_tweet".to_string()],
        dry_run_mutations: true,
        ..default_policy()
    };
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::ToolBlocked,
            ..
        }
    ));
}

// =========================================================================
// Rule matching tests
// =========================================================================

#[test]
fn tool_name_match_hit() {
    let conditions = RuleConditions {
        tools: vec!["post_tweet".into()],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(conditions_match(&conditions, &ctx));
}

#[test]
fn tool_name_match_miss() {
    let conditions = RuleConditions {
        tools: vec!["like_tweet".into()],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(!conditions_match(&conditions, &ctx));
}

#[test]
fn category_match_hit() {
    let conditions = RuleConditions {
        categories: vec![ToolCategory::Write],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(conditions_match(&conditions, &ctx));
}

#[test]
fn category_match_miss() {
    let conditions = RuleConditions {
        categories: vec![ToolCategory::Engage],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(!conditions_match(&conditions, &ctx));
}

#[test]
fn mode_match_hit() {
    let conditions = RuleConditions {
        modes: vec![OperatingMode::Autopilot],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(conditions_match(&conditions, &ctx));
}

#[test]
fn mode_match_miss() {
    let conditions = RuleConditions {
        modes: vec![OperatingMode::Composer],
        ..Default::default()
    };
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(!conditions_match(&conditions, &ctx));
}

#[test]
fn empty_conditions_match_all() {
    let conditions = RuleConditions::default();
    let ctx = make_eval_context("any_tool", &OperatingMode::Autopilot);
    assert!(conditions_match(&conditions, &ctx));
}

#[test]
fn multi_condition_and_logic() {
    // Requires both category=Write AND mode=Autopilot
    let conditions = RuleConditions {
        categories: vec![ToolCategory::Write],
        modes: vec![OperatingMode::Autopilot],
        ..Default::default()
    };

    // Match: Write + Autopilot
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    assert!(conditions_match(&conditions, &ctx));

    // No match: Write + Composer
    let ctx = make_eval_context("post_tweet", &OperatingMode::Composer);
    assert!(!conditions_match(&conditions, &ctx));

    // No match: Engage + Autopilot
    let ctx = make_eval_context("like_tweet", &OperatingMode::Autopilot);
    assert!(!conditions_match(&conditions, &ctx));
}

// =========================================================================
// Template tests
// =========================================================================

#[test]
fn list_templates_returns_three() {
    let templates = list_templates();
    assert_eq!(templates.len(), 3);
}

#[test]
fn safe_default_template_has_expected_rules() {
    let template = get_template(&PolicyTemplateName::SafeDefault);
    assert!(template.rules.len() >= 3);
    // Should have a delete approval rule
    assert!(template
        .rules
        .iter()
        .any(|r| r.id.contains("delete_approval")));
}

#[tokio::test]
async fn template_applied_produces_expected_decisions() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 50)
        .await
        .expect("init rate limit");

    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::GrowthAggressive),
        require_approval_for: Vec::new(),
        ..default_policy()
    };

    // Growth aggressive allows most mutations
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");
    assert_eq!(decision, PolicyDecision::Allow);

    // But delete still requires approval (hard rule)
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "delete_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(decision, PolicyDecision::RouteToApproval { .. }));
}

#[tokio::test]
async fn template_plus_user_rules_coexist() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 50)
        .await
        .expect("init rate limit");

    // User rule at priority 99 fires before template's allow_all at 150
    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::GrowthAggressive),
        rules: vec![PolicyRule {
            id: "user:block_likes".into(),
            priority: 99, // Before template rules
            label: "Block likes".into(),
            enabled: true,
            conditions: RuleConditions {
                tools: vec!["like_tweet".into()],
                ..Default::default()
            },
            action: PolicyAction::Deny {
                reason: "user blocked likes".into(),
            },
        }],
        require_approval_for: Vec::new(),
        ..default_policy()
    };

    // like_tweet denied at priority 99 (before template allow at 150)
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "like_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::UserRule,
            ..
        }
    ));

    // post_tweet still allowed by template
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");
    assert_eq!(decision, PolicyDecision::Allow);
}

// =========================================================================
// Hard rule tests
// =========================================================================

#[tokio::test]
async fn delete_tweet_always_routes_to_approval() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 200)
        .await
        .expect("init rate limit");

    // Even with agency_mode template (maximum autonomy)
    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::AgencyMode),
        require_approval_for: Vec::new(),
        ..default_policy()
    };

    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "delete_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(decision, PolicyDecision::RouteToApproval { .. }));
}

#[tokio::test]
async fn composer_mode_always_routes_to_approval() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 200)
        .await
        .expect("init rate limit");

    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::AgencyMode),
        require_approval_for: Vec::new(),
        ..default_policy()
    };

    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Composer, "like_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(decision, PolicyDecision::RouteToApproval { .. }));
}

// =========================================================================
// Rate limit per-dimension tests
// =========================================================================

#[tokio::test]
async fn per_tool_rate_limit_enforced() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rate limit");

    let per_tool_limits = vec![PolicyRateLimit {
        key: "mcp:tool:like_tweet:hourly".into(),
        dimension: RateLimitDimension::Tool,
        match_value: "like_tweet".into(),
        max_count: 2,
        period_seconds: 3600,
    }];

    rate_limits::init_policy_rate_limits(&pool, &per_tool_limits)
        .await
        .expect("init policy rate limits");

    // Exhaust per-tool limit
    rate_limits::increment_rate_limit(&pool, "mcp:tool:like_tweet:hourly")
        .await
        .expect("inc");
    rate_limits::increment_rate_limit(&pool, "mcp:tool:like_tweet:hourly")
        .await
        .expect("inc");

    let config = McpPolicyConfig {
        require_approval_for: Vec::new(),
        rate_limits: per_tool_limits,
        ..default_policy()
    };

    // like_tweet should be rate limited
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "like_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::RateLimited,
            ..
        }
    ));

    // post_tweet should still be allowed (different tool)
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");
    assert_eq!(decision, PolicyDecision::Allow);
}

#[tokio::test]
async fn per_category_rate_limit_enforced() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rate limit");

    let per_cat_limits = vec![PolicyRateLimit {
        key: "mcp:category:engage:daily".into(),
        dimension: RateLimitDimension::Category,
        match_value: "engage".into(),
        max_count: 1,
        period_seconds: 86400,
    }];

    rate_limits::init_policy_rate_limits(&pool, &per_cat_limits)
        .await
        .expect("init");

    // Exhaust category limit
    rate_limits::increment_rate_limit(&pool, "mcp:category:engage:daily")
        .await
        .expect("inc");

    let config = McpPolicyConfig {
        require_approval_for: Vec::new(),
        rate_limits: per_cat_limits,
        ..default_policy()
    };

    // like_tweet (engage category) should be rate limited
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "like_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::RateLimited,
            ..
        }
    ));
}

#[tokio::test]
async fn global_limit_still_works_alongside_per_dimension() {
    let pool = storage::init_test_db().await.expect("init db");
    // Set global limit to 1
    rate_limits::init_mcp_rate_limit(&pool, 1)
        .await
        .expect("init rate limit");

    let per_tool_limits = vec![PolicyRateLimit {
        key: "mcp:tool:post_tweet:hourly".into(),
        dimension: RateLimitDimension::Tool,
        match_value: "post_tweet".into(),
        max_count: 100, // Very high per-tool limit
        period_seconds: 3600,
    }];

    rate_limits::init_policy_rate_limits(&pool, &per_tool_limits)
        .await
        .expect("init");

    // Exhaust global limit
    rate_limits::increment_rate_limit(&pool, "mcp_mutation")
        .await
        .expect("inc");

    let config = McpPolicyConfig {
        require_approval_for: Vec::new(),
        rate_limits: per_tool_limits,
        ..default_policy()
    };

    // Should be denied by global limit even though per-tool limit is fine
    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");
    assert!(matches!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::RateLimited,
            ..
        }
    ));
}

// =========================================================================
// Effective rules construction tests
// =========================================================================

#[test]
fn effective_rules_sorted_by_priority() {
    let config = McpPolicyConfig {
        rules: vec![
            PolicyRule {
                id: "user:z".into(),
                priority: 250,
                label: "Z".into(),
                enabled: true,
                conditions: RuleConditions::default(),
                action: PolicyAction::Allow,
            },
            PolicyRule {
                id: "user:a".into(),
                priority: 200,
                label: "A".into(),
                enabled: true,
                conditions: RuleConditions::default(),
                action: PolicyAction::Allow,
            },
        ],
        ..default_policy()
    };

    let rules = build_effective_rules(&config, &OperatingMode::Autopilot);
    // Hard rules first (priority 0), then user rules in priority order
    assert!(rules[0].priority <= rules[1].priority);
    for i in 1..rules.len() {
        assert!(rules[i - 1].priority <= rules[i].priority);
    }
}

#[test]
fn v1_compat_rules_generated_when_no_v2_config() {
    let config = McpPolicyConfig {
        blocked_tools: vec!["bad_tool".into()],
        require_approval_for: vec!["post_tweet".into()],
        ..default_policy()
    };

    let rules = build_effective_rules(&config, &OperatingMode::Autopilot);
    // Should have hard rules + v1 compat rules
    assert!(rules.iter().any(|r| r.id.starts_with("v1:blocked:")));
    assert!(rules.iter().any(|r| r.id.starts_with("v1:approval:")));
}

#[test]
fn v1_compat_rules_not_generated_with_template() {
    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::SafeDefault),
        blocked_tools: vec!["bad_tool".into()],
        ..default_policy()
    };

    let rules = build_effective_rules(&config, &OperatingMode::Autopilot);
    // Should NOT have v1 compat rules when template is set
    assert!(!rules.iter().any(|r| r.id.starts_with("v1:")));
}

#[test]
fn disabled_rules_excluded_from_matching() {
    let config = McpPolicyConfig {
        rules: vec![PolicyRule {
            id: "user:disabled".into(),
            priority: 200,
            label: "Disabled".into(),
            enabled: false,
            conditions: RuleConditions::default(),
            action: PolicyAction::Deny {
                reason: "should not fire".into(),
            },
        }],
        ..default_policy()
    };

    let rules = build_effective_rules(&config, &OperatingMode::Autopilot);
    // The disabled rule should still be in the list (for display)
    // but find_matching_rule should skip it
    let ctx = make_eval_context("post_tweet", &OperatingMode::Autopilot);
    let matching = super::rules::find_matching_rule(&rules, &ctx);
    // Should match a hard rule or v1 rule, not the disabled one
    if let Some(rule) = matching {
        assert_ne!(rule.id, "user:disabled");
    }
}

// =========================================================================
// Audit record tests
// =========================================================================

#[tokio::test]
async fn audit_record_includes_rule_metadata() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = McpPolicyConfig {
        template: Some(PolicyTemplateName::SafeDefault),
        require_approval_for: Vec::new(),
        ..default_policy()
    };

    let decision =
        McpPolicyEvaluator::evaluate(&pool, &config, &OperatingMode::Autopilot, "post_tweet")
            .await
            .expect("evaluate");

    McpPolicyEvaluator::log_decision(&pool, "post_tweet", &decision)
        .await
        .expect("log");

    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT action_type, metadata FROM action_log WHERE action_type = 'mcp_policy'",
    )
    .fetch_all(&pool)
    .await
    .expect("query");

    assert_eq!(rows.len(), 1);
    // Metadata should include v2 audit fields
    let metadata = rows[0].1.as_ref().expect("metadata should exist");
    assert!(metadata.contains("category"));
    assert!(metadata.contains("matched_rule_id"));
}

#[tokio::test]
async fn record_mutation_increments_per_dimension_counters() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rate limit");

    let per_tool_limits = vec![PolicyRateLimit {
        key: "mcp:tool:like_tweet:hourly".into(),
        dimension: RateLimitDimension::Tool,
        match_value: "like_tweet".into(),
        max_count: 10,
        period_seconds: 3600,
    }];

    rate_limits::init_policy_rate_limits(&pool, &per_tool_limits)
        .await
        .expect("init");

    McpPolicyEvaluator::record_mutation(&pool, "like_tweet", &per_tool_limits)
        .await
        .expect("record");

    let limits = rate_limits::get_all_rate_limits(&pool)
        .await
        .expect("get limits");

    // Global counter incremented
    let mcp = limits
        .iter()
        .find(|l| l.action_type == "mcp_mutation")
        .expect("mcp_mutation row");
    assert_eq!(mcp.request_count, 1);

    // Per-tool counter incremented
    let per_tool = limits
        .iter()
        .find(|l| l.action_type == "mcp:tool:like_tweet:hourly")
        .expect("per-tool row");
    assert_eq!(per_tool.request_count, 1);
}
