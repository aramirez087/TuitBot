//! Tests for the unified mutation gateway.
//!
//! Covers: allowed, blocked (tool blocked, hard rule), rate-limited,
//! approval routing, dry-run, idempotency (duplicate), and post-execution
//! recording.

use super::*;
use crate::config::{McpPolicyConfig, OperatingMode};
use crate::mcp_policy::types::{
    PolicyAction, PolicyRateLimit, PolicyRule, RateLimitDimension, RuleConditions,
};
use crate::storage::{init_test_db, rate_limits};

fn default_policy_config() -> McpPolicyConfig {
    McpPolicyConfig {
        enforce_for_mutations: true,
        max_mutations_per_hour: 10,
        blocked_tools: vec![],
        require_approval_for: vec![],
        dry_run_mutations: false,
        template: None,
        rules: vec![],
        rate_limits: vec![],
    }
}

fn make_request<'a>(
    pool: &'a DbPool,
    config: &'a McpPolicyConfig,
    mode: &'a OperatingMode,
    tool_name: &'a str,
    params_json: &'a str,
) -> MutationRequest<'a> {
    MutationRequest {
        pool,
        policy_config: config,
        mode,
        tool_name,
        params_json,
    }
}

// ── Allowed scenario ───────────────────────────────────────────────────

#[tokio::test]
async fn gateway_allows_valid_mutation() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 10)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"hi"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::Proceed(ticket) => {
            assert!(!ticket.correlation_id.is_empty());
            assert_eq!(ticket.tool_name, "post_tweet");
        }
        other => panic!("expected Proceed, got {other:?}"),
    }
}

// ── Enforcement disabled → always allow ────────────────────────────────

#[tokio::test]
async fn gateway_allows_when_enforcement_disabled() {
    let pool = init_test_db().await.expect("init db");
    let mut config = default_policy_config();
    config.enforce_for_mutations = false;
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"hello"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    assert!(
        matches!(decision, GatewayDecision::Proceed(_)),
        "should allow when enforcement is disabled"
    );
}

// ── Blocked tool ───────────────────────────────────────────────────────

#[tokio::test]
async fn gateway_denies_blocked_tool() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 10)
        .await
        .expect("init rl");
    let mut config = default_policy_config();
    config.blocked_tools = vec!["post_tweet".to_string()];
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"hi"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::Denied(denial) => {
            assert_eq!(denial.reason, PolicyDenialReason::ToolBlocked);
        }
        other => panic!("expected Denied(ToolBlocked), got {other:?}"),
    }
}

// ── Rate limited ───────────────────────────────────────────────────────

#[tokio::test]
async fn gateway_denies_when_rate_limited() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 2)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    // Exhaust the rate limit.
    rate_limits::increment_rate_limit(&pool, "mcp_mutation")
        .await
        .expect("inc");
    rate_limits::increment_rate_limit(&pool, "mcp_mutation")
        .await
        .expect("inc");

    let req = make_request(&pool, &config, &mode, "like_tweet", r#"{"tweet_id":"1"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::Denied(denial) => {
            assert_eq!(denial.reason, PolicyDenialReason::RateLimited);
        }
        other => panic!("expected Denied(RateLimited), got {other:?}"),
    }
}

// ── Per-dimension rate limit ───────────────────────────────────────────

#[tokio::test]
async fn gateway_denies_per_dimension_rate_limit() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init global rl");

    let rl = PolicyRateLimit {
        key: "mcp:like_tweet:hourly".to_string(),
        dimension: RateLimitDimension::Tool,
        match_value: "like_tweet".to_string(),
        max_count: 1,
        period_seconds: 3600,
    };

    rate_limits::init_policy_rate_limits(&pool, &[rl.clone()])
        .await
        .expect("init policy rl");
    rate_limits::increment_rate_limit(&pool, "mcp:like_tweet:hourly")
        .await
        .expect("inc");

    let mut config = default_policy_config();
    config.rate_limits = vec![rl];
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "like_tweet", r#"{"tweet_id":"1"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::Denied(denial) => {
            assert_eq!(denial.reason, PolicyDenialReason::RateLimited);
        }
        other => panic!("expected Denied(RateLimited), got {other:?}"),
    }
}

// ── Approval routing ───────────────────────────────────────────────────

#[tokio::test]
async fn gateway_routes_to_approval() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 10)
        .await
        .expect("init rl");

    let mut config = default_policy_config();
    config.rules = vec![PolicyRule {
        id: "user:approve-all-writes".to_string(),
        priority: 200,
        label: "Approve all writes".to_string(),
        enabled: true,
        conditions: RuleConditions {
            tools: vec!["post_tweet".to_string()],
            ..Default::default()
        },
        action: PolicyAction::RequireApproval {
            reason: "Manual approval required".to_string(),
        },
    }];
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"hi"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::RoutedToApproval {
            queue_id,
            reason,
            rule_id,
        } => {
            assert!(queue_id > 0);
            assert_eq!(reason, "Manual approval required");
            assert!(rule_id.is_some());
        }
        other => panic!("expected RoutedToApproval, got {other:?}"),
    }
}

// ── Dry-run ────────────────────────────────────────────────────────────

#[tokio::test]
async fn gateway_returns_dry_run() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 10)
        .await
        .expect("init rl");

    let mut config = default_policy_config();
    config.rules = vec![PolicyRule {
        id: "user:dry-run-all".to_string(),
        priority: 200,
        label: "Dry run everything".to_string(),
        enabled: true,
        conditions: RuleConditions::default(),
        action: PolicyAction::DryRun,
    }];
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"hi"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::DryRun { rule_id } => {
            assert!(rule_id.is_some());
        }
        other => panic!("expected DryRun, got {other:?}"),
    }
}

// ── Idempotency (duplicate detection) ──────────────────────────────────

#[tokio::test]
async fn gateway_detects_duplicate() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    // First call: should proceed.
    let req1 = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"dup"}"#);
    let d1 = MutationGateway::evaluate(&req1).await.expect("eval 1");
    let ticket = match d1 {
        GatewayDecision::Proceed(t) => t,
        other => panic!("first call should proceed, got {other:?}"),
    };

    // Complete it successfully.
    MutationGateway::complete_success(&pool, &ticket, r#"{"tweet_id":"999"}"#, None, 100, &[])
        .await
        .expect("complete");

    // Second call with same params: should be duplicate.
    let req2 = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"dup"}"#);
    let d2 = MutationGateway::evaluate(&req2).await.expect("eval 2");

    match d2 {
        GatewayDecision::Duplicate(info) => {
            assert_eq!(info.original_correlation_id, ticket.correlation_id);
            assert!(info.cached_result.as_deref().unwrap_or("").contains("999"));
        }
        other => panic!("expected Duplicate, got {other:?}"),
    }
}

// ── Post-execution: success recording ──────────────────────────────────

#[tokio::test]
async fn gateway_records_success() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"ok"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("eval");
    let ticket = match decision {
        GatewayDecision::Proceed(t) => t,
        other => panic!("expected Proceed, got {other:?}"),
    };

    MutationGateway::complete_success(
        &pool,
        &ticket,
        r#"{"tweet_id":"123"}"#,
        Some(r#"{"tool":"x_delete_tweet","params":{"tweet_id":"123"}}"#),
        150,
        &config.rate_limits,
    )
    .await
    .expect("complete");

    // Verify audit trail.
    let entry = mutation_audit::get_by_correlation_id(&pool, &ticket.correlation_id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(entry.status, "success");
    assert_eq!(entry.elapsed_ms, Some(150));
    assert!(entry.rollback_action.is_some());
}

// ── Post-execution: failure recording ──────────────────────────────────

#[tokio::test]
async fn gateway_records_failure() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "like_tweet", r#"{"id":"1"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("eval");
    let ticket = match decision {
        GatewayDecision::Proceed(t) => t,
        other => panic!("expected Proceed, got {other:?}"),
    };

    MutationGateway::complete_failure(&pool, &ticket, "X API rate limit", 50)
        .await
        .expect("fail");

    let entry = mutation_audit::get_by_correlation_id(&pool, &ticket.correlation_id)
        .await
        .expect("get")
        .expect("found");
    assert_eq!(entry.status, "failure");
    assert_eq!(entry.error_message.as_deref(), Some("X API rate limit"));
}

// ── Retry after failure is allowed ─────────────────────────────────────

#[tokio::test]
async fn gateway_allows_retry_after_failure() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    // First attempt: proceed then fail.
    let req1 = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"retry"}"#);
    let d1 = MutationGateway::evaluate(&req1).await.expect("eval");
    let ticket = match d1 {
        GatewayDecision::Proceed(t) => t,
        other => panic!("first call should proceed, got {other:?}"),
    };
    MutationGateway::complete_failure(&pool, &ticket, "error", 10)
        .await
        .expect("fail");

    // Retry: should proceed (not duplicate, since first attempt failed).
    let req2 = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"retry"}"#);
    let d2 = MutationGateway::evaluate(&req2).await.expect("eval 2");
    assert!(
        matches!(d2, GatewayDecision::Proceed(_)),
        "retry after failure should proceed"
    );
}

// ── Hard rule denial ───────────────────────────────────────────────────

#[tokio::test]
async fn gateway_denies_hard_rule() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");

    let mut config = default_policy_config();
    // Use post_tweet (no built-in hard rule) with a hard:-prefixed deny rule
    // to test the HardRule denial path. delete_tweet has a built-in
    // hard:delete_approval at priority 0 that would match first.
    config.rules = vec![PolicyRule {
        id: "hard:no-posting".to_string(),
        priority: 10,
        label: "No posting".to_string(),
        enabled: true,
        conditions: RuleConditions {
            tools: vec!["post_tweet".to_string()],
            ..Default::default()
        },
        action: PolicyAction::Deny {
            reason: "Posting is prohibited".to_string(),
        },
    }];
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "post_tweet", r#"{"text":"blocked"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::Denied(denial) => {
            assert_eq!(denial.reason, PolicyDenialReason::HardRule);
        }
        other => panic!("expected Denied(HardRule), got {other:?}"),
    }
}

// ── Delete always routes to approval (built-in hard rule) ──────────────

#[tokio::test]
async fn gateway_routes_delete_to_approval() {
    let pool = init_test_db().await.expect("init db");
    rate_limits::init_mcp_rate_limit(&pool, 100)
        .await
        .expect("init rl");
    let config = default_policy_config();
    let mode = OperatingMode::Autopilot;

    let req = make_request(&pool, &config, &mode, "delete_tweet", r#"{"tweet_id":"1"}"#);
    let decision = MutationGateway::evaluate(&req).await.expect("evaluate");

    match decision {
        GatewayDecision::RoutedToApproval { rule_id, .. } => {
            assert_eq!(rule_id, Some("hard:delete_approval".to_string()));
        }
        other => panic!("expected RoutedToApproval for delete, got {other:?}"),
    }
}

// ── Correlation ID format ──────────────────────────────────────────────

#[test]
fn correlation_id_is_uuid_v4_format() {
    let id = generate_correlation_id();
    assert_eq!(id.len(), 36);
    assert_eq!(&id[8..9], "-");
    assert_eq!(&id[13..14], "-");
    assert_eq!(&id[14..15], "4"); // version nibble
    assert_eq!(&id[18..19], "-");
    assert_eq!(&id[23..24], "-");
}

#[test]
fn correlation_ids_are_unique() {
    let ids: Vec<String> = (0..100).map(|_| generate_correlation_id()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
    assert_eq!(ids.len(), unique.len());
}
