//! Core policy evaluator and audit types.
//!
//! Evaluation order (safest wins):
//! 1. `enforce_for_mutations` disabled → Allow
//! 2. Tool in `blocked_tools` → Deny(ToolBlocked)
//! 3. `dry_run_mutations` enabled → DryRun
//! 4. MCP mutation rate limit exceeded → Deny(RateLimited)
//! 5. Composer mode → RouteToApproval (all mutations)
//! 6. Tool in `require_approval_for` → RouteToApproval
//! 7. Otherwise → Allow

use serde::Serialize;

use crate::config::{McpPolicyConfig, OperatingMode};
use crate::error::StorageError;
use crate::storage::rate_limits;
use crate::storage::DbPool;

/// The outcome of a policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The mutation is allowed to proceed.
    Allow,
    /// The mutation should be routed to the approval queue.
    RouteToApproval { reason: String },
    /// The mutation is denied.
    Deny { reason: PolicyDenialReason },
    /// Dry-run mode: return what would happen without executing.
    DryRun,
}

/// Reason a mutation was denied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PolicyDenialReason {
    /// The tool is explicitly blocked in configuration.
    ToolBlocked,
    /// The hourly MCP mutation rate limit has been exceeded.
    RateLimited,
}

impl std::fmt::Display for PolicyDenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyDenialReason::ToolBlocked => write!(f, "tool_blocked"),
            PolicyDenialReason::RateLimited => write!(f, "rate_limited"),
        }
    }
}

/// Audit record logged for every policy evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyAuditRecord {
    pub tool_name: String,
    pub decision: String,
    pub reason: Option<String>,
}

/// Centralized MCP mutation policy evaluator.
pub struct McpPolicyEvaluator;

impl McpPolicyEvaluator {
    /// Evaluate whether a tool invocation should proceed.
    ///
    /// Follows the ordered evaluation chain described in the module docs.
    pub async fn evaluate(
        pool: &DbPool,
        config: &McpPolicyConfig,
        mode: &OperatingMode,
        tool_name: &str,
    ) -> Result<PolicyDecision, StorageError> {
        // 1. Enforcement disabled → allow all
        if !config.enforce_for_mutations {
            return Ok(PolicyDecision::Allow);
        }

        // 2. Blocked tool → deny
        if config.blocked_tools.iter().any(|t| t == tool_name) {
            return Ok(PolicyDecision::Deny {
                reason: PolicyDenialReason::ToolBlocked,
            });
        }

        // 3. Dry-run mode → return DryRun
        if config.dry_run_mutations {
            return Ok(PolicyDecision::DryRun);
        }

        // 4. Rate limit check
        let allowed = rate_limits::check_rate_limit(pool, "mcp_mutation").await?;
        if !allowed {
            return Ok(PolicyDecision::Deny {
                reason: PolicyDenialReason::RateLimited,
            });
        }

        // 5. Composer mode forces approval for all mutations
        if *mode == OperatingMode::Composer {
            return Ok(PolicyDecision::RouteToApproval {
                reason: "composer mode requires approval for all mutations".to_string(),
            });
        }

        // 6. Tool in require_approval_for list
        if config.require_approval_for.iter().any(|t| t == tool_name) {
            return Ok(PolicyDecision::RouteToApproval {
                reason: format!("tool '{tool_name}' requires approval"),
            });
        }

        // 7. Default: allow
        Ok(PolicyDecision::Allow)
    }

    /// Log a policy decision to the action log.
    pub async fn log_decision(
        pool: &DbPool,
        tool_name: &str,
        decision: &PolicyDecision,
    ) -> Result<(), StorageError> {
        let (status, reason_str) = match decision {
            PolicyDecision::Allow => ("allowed", None),
            PolicyDecision::RouteToApproval { reason } => {
                ("routed_to_approval", Some(reason.clone()))
            }
            PolicyDecision::Deny { reason } => ("denied", Some(reason.to_string())),
            PolicyDecision::DryRun => ("dry_run", None),
        };

        let audit = PolicyAuditRecord {
            tool_name: tool_name.to_string(),
            decision: status.to_string(),
            reason: reason_str,
        };

        let metadata = serde_json::to_string(&audit).ok();

        crate::storage::action_log::log_action(
            pool,
            "mcp_policy",
            status,
            Some(tool_name),
            metadata.as_deref(),
        )
        .await
    }

    /// Record a successful mutation against the rate limit counter.
    pub async fn record_mutation(pool: &DbPool) -> Result<(), StorageError> {
        rate_limits::increment_rate_limit(pool, "mcp_mutation").await
    }
}
