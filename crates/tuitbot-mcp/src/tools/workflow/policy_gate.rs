//! MCP-side mutation gateway adapter.
//!
//! Translates the core [`MutationGateway`] decisions into MCP JSON responses.
//!
//! Provides two entry points:
//! - [`run_gateway`]: Unified pre-mutation check (policy + idempotency + audit).
//!   Returns either a [`MutationTicket`] to continue with, or an early-return JSON.
//! - [`check_policy`]: Policy-only check (no idempotency/audit). Used by dry-run
//!   validation tools that don't actually execute mutations.

use std::time::Instant;

use tuitbot_core::mcp_policy::PolicyDenialReason;
use tuitbot_core::mutation_gateway::{
    DuplicateInfo, GatewayDecision, GatewayDenial, MutationGateway, MutationRequest, MutationTicket,
};
use tuitbot_core::storage::rate_limits;

use crate::state::SharedState;

use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

/// Result of the unified gateway check.
pub enum GatewayResult {
    /// The mutation may proceed. Carries the audit ticket.
    Proceed(MutationTicket),
    /// The mutation was intercepted; return this JSON to the caller.
    EarlyReturn(String),
}

/// Run the unified mutation gateway: policy + idempotency + audit.
///
/// This is the single entry point for all MCP mutation tools. It replaces
/// the previous 3-step sequence of check_policy + begin_audited_mutation +
/// record_mutation with a single call.
///
/// On success, returns `GatewayResult::Proceed(ticket)`. The caller executes
/// the mutation, then calls [`complete_gateway_success`] or
/// [`complete_gateway_failure`].
pub async fn run_gateway(
    state: &SharedState,
    tool_name: &str,
    params_json: &str,
    start: Instant,
) -> GatewayResult {
    // In-memory dedup (fast path, 30s window) — transport-specific.
    if let Some(err) = state.idempotency.check_and_record(tool_name, params_json) {
        return GatewayResult::EarlyReturn(err);
    }

    let req = MutationRequest {
        pool: &state.pool,
        policy_config: &state.config.mcp_policy,
        mode: &state.config.mode,
        tool_name,
        params_json,
    };

    let decision = match MutationGateway::evaluate(&req).await {
        Ok(d) => d,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let json = ToolResponse::error(
                ErrorCode::PolicyError,
                format!("Gateway evaluation failed: {e}"),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            return GatewayResult::EarlyReturn(json);
        }
    };

    match decision {
        GatewayDecision::Proceed(ticket) => GatewayResult::Proceed(ticket),

        GatewayDecision::Denied(denial) => {
            let json = format_denial(state, &denial, tool_name, start).await;
            GatewayResult::EarlyReturn(json)
        }

        GatewayDecision::RoutedToApproval {
            queue_id,
            reason,
            rule_id,
        } => {
            let elapsed = start.elapsed().as_millis() as u64;
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                true,
                None,
                Some("route_to_approval"),
                None,
            )
            .await;
            let json = ToolResponse::success(serde_json::json!({
                "routed_to_approval": true,
                "approval_queue_id": queue_id,
                "reason": reason,
                "matched_rule_id": rule_id,
            }))
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            GatewayResult::EarlyReturn(json)
        }

        GatewayDecision::DryRun { rule_id } => {
            let elapsed = start.elapsed().as_millis() as u64;
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                true,
                None,
                Some("dry_run"),
                None,
            )
            .await;
            let json = ToolResponse::success(serde_json::json!({
                "dry_run": true,
                "would_execute": tool_name,
                "params": params_json,
                "matched_rule_id": rule_id,
            }))
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            GatewayResult::EarlyReturn(json)
        }

        GatewayDecision::Duplicate(info) => {
            let json = format_duplicate(&info, tool_name, start);
            GatewayResult::EarlyReturn(json)
        }
    }
}

/// Complete a successful mutation through the gateway.
pub async fn complete_gateway_success(
    state: &SharedState,
    ticket: &MutationTicket,
    result_data: &serde_json::Value,
    start: Instant,
) -> ToolMeta {
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let result_json = serde_json::to_string(result_data).unwrap_or_default();
    let guidance = crate::tools::rollback::guidance_for(&ticket.tool_name, result_data);
    let rollback_json = crate::tools::rollback::guidance_to_json(&guidance);

    let _ = MutationGateway::complete_success(
        &state.pool,
        ticket,
        &result_json,
        rollback_json.as_deref(),
        elapsed_ms,
        &state.config.mcp_policy.rate_limits,
    )
    .await;

    let rollback_value = serde_json::to_value(&guidance).unwrap_or_default();

    ToolMeta::new(elapsed_ms)
        .with_correlation_id(&ticket.correlation_id)
        .with_rollback(rollback_value)
}

/// Record a failed mutation through the gateway and return metadata.
pub async fn complete_gateway_failure(
    state: &SharedState,
    ticket: &MutationTicket,
    error_msg: &str,
    start: Instant,
) -> ToolMeta {
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let _ = MutationGateway::complete_failure(&state.pool, ticket, error_msg, elapsed_ms).await;

    ToolMeta::new(elapsed_ms).with_correlation_id(&ticket.correlation_id)
}

// ── Legacy: policy-only check (used by dry-run tools) ──────────────────

/// Result of a policy-only gate check.
pub enum GateResult {
    /// The mutation may proceed.
    Proceed,
    /// The mutation was intercepted; return this JSON to the caller.
    EarlyReturn(String),
}

/// Check the MCP policy for a mutation tool (policy only, no idempotency/audit).
///
/// Used by dry-run validation tools that inspect policy without executing.
pub async fn check_policy(
    state: &SharedState,
    tool_name: &str,
    _mutation_params_json: &str,
    start: Instant,
) -> GateResult {
    let decision = match tuitbot_core::mcp_policy::McpPolicyEvaluator::evaluate(
        &state.pool,
        &state.config.mcp_policy,
        &state.config.mode,
        tool_name,
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let json = ToolResponse::error(
                ErrorCode::PolicyError,
                format!("Policy evaluation failed: {e}"),
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            return GateResult::EarlyReturn(json);
        }
    };

    match decision {
        tuitbot_core::mcp_policy::PolicyDecision::Allow => GateResult::Proceed,
        tuitbot_core::mcp_policy::PolicyDecision::Deny { reason, .. } => {
            let elapsed = start.elapsed().as_millis() as u64;
            let code = match &reason {
                PolicyDenialReason::ToolBlocked => ErrorCode::PolicyDeniedBlocked,
                PolicyDenialReason::RateLimited => ErrorCode::PolicyDeniedRateLimited,
                PolicyDenialReason::HardRule => ErrorCode::PolicyDeniedHardRule,
                PolicyDenialReason::UserRule => ErrorCode::PolicyDeniedUserRule,
            };
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                false,
                Some(code.as_str()),
                Some("deny"),
                None,
            )
            .await;
            let json = ToolResponse::error(code, format!("Policy denied: {reason}"))
                .with_policy_decision("denied")
                .with_meta(ToolMeta::new(elapsed))
                .to_json();
            GateResult::EarlyReturn(json)
        }
        tuitbot_core::mcp_policy::PolicyDecision::RouteToApproval { reason, rule_id } => {
            let elapsed = start.elapsed().as_millis() as u64;
            let json = ToolResponse::success(serde_json::json!({
                "routed_to_approval": true,
                "reason": reason,
                "matched_rule_id": rule_id,
            }))
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            GateResult::EarlyReturn(json)
        }
        tuitbot_core::mcp_policy::PolicyDecision::DryRun { rule_id } => {
            let elapsed = start.elapsed().as_millis() as u64;
            let json = ToolResponse::success(serde_json::json!({
                "dry_run": true,
                "would_execute": tool_name,
                "matched_rule_id": rule_id,
            }))
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            GateResult::EarlyReturn(json)
        }
    }
}

// ── Policy status (read-only) ──────────────────────────────────────────

/// Get the current MCP policy status: config + rate limit usage + v2 fields.
pub async fn get_policy_status(state: &SharedState) -> String {
    let start = Instant::now();

    let rate_limit_info = match rate_limits::get_all_rate_limits(&state.pool).await {
        Ok(limits) => {
            let mcp = limits.iter().find(|l| l.action_type == "mcp_mutation");
            match mcp {
                Some(rl) => serde_json::json!({
                    "used": rl.request_count,
                    "max": rl.max_requests,
                    "period_seconds": rl.period_seconds,
                    "period_start": rl.period_start,
                }),
                None => serde_json::json!({"error": "mcp_mutation rate limit not initialized"}),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}),
    };

    let elapsed = start.elapsed().as_millis() as u64;

    ToolResponse::success(serde_json::json!({
        "enforce_for_mutations": state.config.mcp_policy.enforce_for_mutations,
        "require_approval_for": state.config.mcp_policy.require_approval_for,
        "blocked_tools": state.config.mcp_policy.blocked_tools,
        "dry_run_mutations": state.config.mcp_policy.dry_run_mutations,
        "max_mutations_per_hour": state.config.mcp_policy.max_mutations_per_hour,
        "mode": state.config.mode.to_string(),
        "rate_limit": rate_limit_info,
        "template": state.config.mcp_policy.template,
        "rules": state.config.mcp_policy.rules,
        "rate_limits": state.config.mcp_policy.rate_limits,
    }))
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}

// ── Formatting helpers ─────────────────────────────────────────────────

/// Format a policy denial into a JSON error response.
async fn format_denial(
    state: &SharedState,
    denial: &GatewayDenial,
    tool_name: &str,
    start: Instant,
) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    let code = match &denial.reason {
        PolicyDenialReason::ToolBlocked => ErrorCode::PolicyDeniedBlocked,
        PolicyDenialReason::RateLimited => ErrorCode::PolicyDeniedRateLimited,
        PolicyDenialReason::HardRule => ErrorCode::PolicyDeniedHardRule,
        PolicyDenialReason::UserRule => ErrorCode::PolicyDeniedUserRule,
    };
    super::telemetry::record(
        &state.pool,
        tool_name,
        "mutation",
        elapsed,
        false,
        Some(code.as_str()),
        Some("deny"),
        None,
    )
    .await;
    let mut resp = ToolResponse::error(code, format!("Policy denied: {}", denial.reason))
        .with_policy_decision("denied")
        .with_meta(ToolMeta::new(elapsed));

    // For rate-limited denials, attach the reset timestamp.
    if matches!(denial.reason, PolicyDenialReason::RateLimited) {
        let rl_key = denial.rule_id.as_deref().unwrap_or("mcp_mutation");
        if let Ok(limits) = rate_limits::get_all_rate_limits(&state.pool).await {
            if let Some(rl) = limits.iter().find(|l| l.action_type == rl_key) {
                if let Ok(start_ts) =
                    chrono::NaiveDateTime::parse_from_str(&rl.period_start, "%Y-%m-%dT%H:%M:%SZ")
                {
                    let reset = start_ts + chrono::Duration::seconds(rl.period_seconds);
                    resp =
                        resp.with_rate_limit_reset(reset.format("%Y-%m-%dT%H:%M:%SZ").to_string());
                }
            }
        }
    }

    resp.to_json()
}

/// Format a duplicate detection into a JSON response.
fn format_duplicate(info: &DuplicateInfo, tool_name: &str, start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    let cached_result = info.cached_result.as_deref().unwrap_or("{}");
    ToolResponse::success(serde_json::json!({
        "duplicate": true,
        "original_correlation_id": info.original_correlation_id,
        "cached_result": serde_json::from_str::<serde_json::Value>(cached_result)
            .unwrap_or(serde_json::Value::String(cached_result.to_string())),
        "message": format!(
            "Identical {} was already executed successfully. Returning cached result.",
            tool_name
        ),
    }))
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}
