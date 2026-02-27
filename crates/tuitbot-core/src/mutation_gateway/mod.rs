//! Unified mutation governance gateway.
//!
//! Every mutation path — MCP tools, autopilot loops, HTTP API — routes
//! through [`MutationGateway`] before executing side effects.
//!
//! The gateway enforces a strict sequence:
//! 1. **Policy evaluation** — block rules, rate limits, approval routing, dry-run
//! 2. **Idempotency** — DB-backed dedup within a 5-minute window
//! 3. **Audit record** — pending entry before execution
//! 4. **Post-execution recording** — rate-limit increment + audit completion
//!
//! This single path replaces the scattered policy/idempotency/audit logic
//! that was previously duplicated across MCP tool handlers.

#[cfg(test)]
mod tests;

use crate::config::{McpPolicyConfig, OperatingMode};
use crate::error::StorageError;
use crate::mcp_policy::types::PolicyRateLimit;
use crate::mcp_policy::{McpPolicyEvaluator, PolicyDecision, PolicyDenialReason};
use crate::storage::mutation_audit;
use crate::storage::DbPool;

/// DB-backed idempotency window in seconds (5 minutes).
const IDEMPOTENCY_WINDOW_SECS: u32 = 300;

/// Unified mutation governance gateway.
///
/// Stateless: all dependencies are passed per-call via [`MutationRequest`].
/// This allows any consumer (MCP, autopilot, HTTP server) to use the gateway.
pub struct MutationGateway;

/// Input parameters for a gateway evaluation.
pub struct MutationRequest<'a> {
    pub pool: &'a DbPool,
    pub policy_config: &'a McpPolicyConfig,
    pub mode: &'a OperatingMode,
    pub tool_name: &'a str,
    pub params_json: &'a str,
}

/// The gateway's decision for a mutation request.
#[derive(Debug)]
pub enum GatewayDecision {
    /// The mutation may proceed. Carry the ticket through execution
    /// and call [`MutationGateway::complete_success`] or
    /// [`MutationGateway::complete_failure`] afterward.
    Proceed(MutationTicket),

    /// The mutation was denied by policy.
    Denied(GatewayDenial),

    /// The mutation was routed to the approval queue.
    RoutedToApproval {
        queue_id: i64,
        reason: String,
        rule_id: Option<String>,
    },

    /// Dry-run mode: the mutation would have executed but was intercepted.
    DryRun { rule_id: Option<String> },

    /// Idempotency hit: an identical recent mutation was already successful.
    Duplicate(DuplicateInfo),
}

/// Details of a policy denial.
#[derive(Debug, Clone)]
pub struct GatewayDenial {
    pub reason: PolicyDenialReason,
    pub rule_id: Option<String>,
}

/// Information about a detected duplicate mutation.
#[derive(Debug, Clone)]
pub struct DuplicateInfo {
    pub original_correlation_id: String,
    pub cached_result: Option<String>,
    pub audit_id: i64,
}

/// Handle for a mutation in progress. Created by the gateway on `Proceed`.
///
/// Carries the audit trail ID and correlation ID needed to complete
/// the audit record after execution.
#[derive(Debug)]
pub struct MutationTicket {
    pub audit_id: i64,
    pub correlation_id: String,
    pub tool_name: String,
}

impl MutationGateway {
    /// Evaluate a mutation request through the full governance pipeline.
    ///
    /// Sequence:
    /// 1. Policy evaluation (block rules, rate limits, approval routing)
    /// 2. DB-backed idempotency check (5-minute window)
    /// 3. Pending audit record creation
    ///
    /// Returns a [`GatewayDecision`] indicating whether the mutation may
    /// proceed, was denied, routed to approval, or is a duplicate.
    pub async fn evaluate(req: &MutationRequest<'_>) -> Result<GatewayDecision, StorageError> {
        // ── Step 1: Policy evaluation ──────────────────────────────────
        let decision =
            McpPolicyEvaluator::evaluate(req.pool, req.policy_config, req.mode, req.tool_name)
                .await?;

        // Log the policy decision (best-effort).
        let _ = McpPolicyEvaluator::log_decision(req.pool, req.tool_name, &decision).await;

        match decision {
            PolicyDecision::Deny { reason, rule_id } => {
                return Ok(GatewayDecision::Denied(GatewayDenial { reason, rule_id }));
            }
            PolicyDecision::RouteToApproval { reason, rule_id } => {
                // Enqueue into approval queue.
                let queue_id = crate::storage::approval_queue::enqueue_with_context(
                    req.pool,
                    req.tool_name,
                    "",
                    "",
                    req.params_json,
                    "mcp_policy",
                    req.tool_name,
                    0.0,
                    "[]",
                    Some(&reason),
                    Some(&match &rule_id {
                        Some(rid) => format!(r#"["policy_rule:{rid}"]"#),
                        None => "[]".to_string(),
                    }),
                )
                .await
                .map_err(|e| StorageError::Query {
                    source: sqlx::Error::Protocol(format!("Failed to enqueue for approval: {e}")),
                })?;

                return Ok(GatewayDecision::RoutedToApproval {
                    queue_id,
                    reason,
                    rule_id,
                });
            }
            PolicyDecision::DryRun { rule_id } => {
                return Ok(GatewayDecision::DryRun { rule_id });
            }
            PolicyDecision::Allow => { /* continue to idempotency check */ }
        }

        // ── Step 2: DB-backed idempotency (5-minute window) ────────────
        let params_hash = mutation_audit::compute_params_hash(req.tool_name, req.params_json);
        let params_summary = mutation_audit::truncate_summary(req.params_json, 500);

        if let Some(existing) = mutation_audit::find_recent_duplicate(
            req.pool,
            req.tool_name,
            &params_hash,
            IDEMPOTENCY_WINDOW_SECS,
        )
        .await?
        {
            // Record the duplicate attempt in audit trail.
            let dup_corr = generate_correlation_id();
            let dup_id = mutation_audit::insert_pending(
                req.pool,
                &dup_corr,
                None,
                req.tool_name,
                &params_hash,
                &params_summary,
            )
            .await?;
            let _ =
                mutation_audit::mark_duplicate(req.pool, dup_id, &existing.correlation_id).await;

            return Ok(GatewayDecision::Duplicate(DuplicateInfo {
                original_correlation_id: existing.correlation_id,
                cached_result: existing.result_summary,
                audit_id: dup_id,
            }));
        }

        // ── Step 3: Create pending audit record ────────────────────────
        let correlation_id = generate_correlation_id();
        let audit_id = mutation_audit::insert_pending(
            req.pool,
            &correlation_id,
            None,
            req.tool_name,
            &params_hash,
            &params_summary,
        )
        .await?;

        Ok(GatewayDecision::Proceed(MutationTicket {
            audit_id,
            correlation_id,
            tool_name: req.tool_name.to_string(),
        }))
    }

    /// Record a successful mutation: complete audit + increment rate counters.
    pub async fn complete_success(
        pool: &DbPool,
        ticket: &MutationTicket,
        result_summary: &str,
        rollback_action: Option<&str>,
        elapsed_ms: u64,
        rate_limit_configs: &[PolicyRateLimit],
    ) -> Result<(), StorageError> {
        let summary = mutation_audit::truncate_summary(result_summary, 500);
        mutation_audit::complete_success(
            pool,
            ticket.audit_id,
            &summary,
            rollback_action,
            elapsed_ms,
        )
        .await?;

        McpPolicyEvaluator::record_mutation(pool, &ticket.tool_name, rate_limit_configs).await?;

        Ok(())
    }

    /// Record a failed mutation in the audit trail.
    pub async fn complete_failure(
        pool: &DbPool,
        ticket: &MutationTicket,
        error_message: &str,
        elapsed_ms: u64,
    ) -> Result<(), StorageError> {
        mutation_audit::complete_failure(pool, ticket.audit_id, error_message, elapsed_ms).await
    }
}

/// Generate a UUID v4-like correlation ID.
fn generate_correlation_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::SystemTime;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut hasher = DefaultHasher::new();
    nanos.hash(&mut hasher);
    count.hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let h1 = hasher.finish();
    count.wrapping_add(1).hash(&mut hasher);
    let h2 = hasher.finish();

    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (h1 >> 32) as u32,
        (h1 >> 16) as u16,
        h1 as u16 & 0x0fff,
        (h2 >> 48) as u16 & 0x3fff | 0x8000,
        h2 & 0xffffffffffff,
    )
}
