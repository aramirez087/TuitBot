//! Tests for the MCP policy evaluator.

use super::evaluator::{McpPolicyEvaluator, PolicyDecision, PolicyDenialReason};
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

    assert_eq!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::ToolBlocked
        }
    );
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

    assert_eq!(decision, PolicyDecision::DryRun);
}

#[tokio::test]
async fn rate_limit_exceeded_denies() {
    let pool = storage::init_test_db().await.expect("init db");
    // Set max to 2
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

    assert_eq!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::RateLimited
        }
    );
}

#[tokio::test]
async fn approval_required_routes() {
    let pool = storage::init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 20)
        .await
        .expect("init rate limit");

    let config = default_policy(); // post_tweet is in require_approval_for by default
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
    // unfollow_user is NOT in the default require_approval_for list
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

    // Even with empty require_approval_for, Composer mode forces approval
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

    // Verify action_log has an entry
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

    McpPolicyEvaluator::record_mutation(&pool)
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

    assert_eq!(
        decision,
        PolicyDecision::Deny {
            reason: PolicyDenialReason::ToolBlocked
        }
    );
}
