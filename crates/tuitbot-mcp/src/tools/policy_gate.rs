//! MCP-side policy gate that translates policy decisions to tool responses.
//!
//! Each mutation tool calls [`check_policy`] at the top; on non-Allow
//! decisions the function returns an `EarlyReturn` with a pre-formatted
//! JSON response so the tool can bail immediately.

use std::time::Instant;

use tuitbot_core::mcp_policy::{McpPolicyEvaluator, PolicyDecision, PolicyDenialReason};
use tuitbot_core::storage::rate_limits;

use crate::state::SharedState;

use super::response::{ToolMeta, ToolResponse};

/// Result of a policy gate check.
pub enum GateResult {
    /// The mutation may proceed.
    Proceed,
    /// The mutation was intercepted; return this JSON to the caller.
    EarlyReturn(String),
}

/// Check the MCP policy for a mutation tool.
///
/// On `Allow`, returns `GateResult::Proceed` so the caller continues.
/// On any other decision, returns `GateResult::EarlyReturn` with a
/// structured JSON response.
pub async fn check_policy(
    state: &SharedState,
    tool_name: &str,
    mutation_params_json: &str,
    start: Instant,
) -> GateResult {
    let decision = match McpPolicyEvaluator::evaluate(
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
                "policy_error",
                format!("Policy evaluation failed: {e}"),
                true,
            )
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            return GateResult::EarlyReturn(json);
        }
    };

    // Log the decision (best-effort, don't fail the request)
    let _ = McpPolicyEvaluator::log_decision(&state.pool, tool_name, &decision).await;

    match decision {
        PolicyDecision::Allow => GateResult::Proceed,

        PolicyDecision::RouteToApproval { ref reason } => {
            let reason = reason.clone();
            // Enqueue into approval queue
            let enqueue_result = tuitbot_core::storage::approval_queue::enqueue(
                &state.pool,
                tool_name,
                "", // no target tweet ID for generic mutations
                "", // no target author
                mutation_params_json,
                "mcp_policy",
                tool_name,
                0.0,
                "[]",
            )
            .await;

            let elapsed = start.elapsed().as_millis() as u64;
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                true,
                None,
                Some("route_to_approval"),
            )
            .await;
            let json = match enqueue_result {
                Ok(id) => ToolResponse::success(serde_json::json!({
                    "routed_to_approval": true,
                    "approval_queue_id": id,
                    "reason": reason,
                }))
                .with_meta(ToolMeta::new(elapsed))
                .to_json(),
                Err(e) => ToolResponse::error(
                    "policy_error",
                    format!("Failed to enqueue for approval: {e}"),
                    true,
                )
                .with_meta(ToolMeta::new(elapsed))
                .to_json(),
            };
            GateResult::EarlyReturn(json)
        }

        PolicyDecision::Deny { reason } => {
            let elapsed = start.elapsed().as_millis() as u64;
            let code = match &reason {
                PolicyDenialReason::ToolBlocked => "policy_denied_blocked",
                PolicyDenialReason::RateLimited => "policy_denied_rate_limited",
            };
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                false,
                Some(code),
                Some("deny"),
            )
            .await;
            let mut resp = ToolResponse::error(code, format!("Policy denied: {reason}"), false)
                .with_policy_decision("denied")
                .with_meta(ToolMeta::new(elapsed));

            // For rate-limited denials, attach the reset timestamp.
            if matches!(reason, PolicyDenialReason::RateLimited) {
                if let Ok(limits) = rate_limits::get_all_rate_limits(&state.pool).await {
                    if let Some(rl) = limits.iter().find(|l| l.action_type == "mcp_mutation") {
                        if let Ok(start_ts) = chrono::NaiveDateTime::parse_from_str(
                            &rl.period_start,
                            "%Y-%m-%dT%H:%M:%SZ",
                        ) {
                            let reset = start_ts + chrono::Duration::seconds(rl.period_seconds);
                            resp = resp.with_rate_limit_reset(
                                reset.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            );
                        }
                    }
                }
            }

            GateResult::EarlyReturn(resp.to_json())
        }

        PolicyDecision::DryRun => {
            let elapsed = start.elapsed().as_millis() as u64;
            super::telemetry::record(
                &state.pool,
                tool_name,
                "mutation",
                elapsed,
                true,
                None,
                Some("dry_run"),
            )
            .await;
            let json = ToolResponse::success(serde_json::json!({
                "dry_run": true,
                "would_execute": tool_name,
                "params": mutation_params_json,
            }))
            .with_meta(ToolMeta::new(elapsed))
            .to_json();
            GateResult::EarlyReturn(json)
        }
    }
}

/// Get the current MCP policy status: config + rate limit usage.
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
    }))
    .with_meta(ToolMeta::new(elapsed))
    .to_json()
}
