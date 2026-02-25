//! Core policy evaluator and audit types.
//!
//! v2 evaluation order:
//! 1. `enforce_for_mutations` disabled → Allow (master kill switch)
//! 2. Build effective rule set via `build_effective_rules()`
//! 3. Walk rules by priority: first match → mapped PolicyDecision
//! 4. Check per-dimension rate limits from `config.rate_limits`
//! 5. Check legacy global rate limit (`mcp_mutation`)
//! 6. Default → Allow

use serde::Serialize;

use crate::config::{McpPolicyConfig, OperatingMode};
use crate::error::StorageError;
use crate::storage::rate_limits;
use crate::storage::DbPool;

use super::rules::{build_effective_rules, find_matching_rule, make_eval_context};
use super::types::{tool_category, PolicyAction, PolicyAuditRecordV2};

/// The outcome of a policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The mutation is allowed to proceed.
    Allow,
    /// The mutation should be routed to the approval queue.
    RouteToApproval {
        reason: String,
        rule_id: Option<String>,
    },
    /// The mutation is denied.
    Deny {
        reason: PolicyDenialReason,
        rule_id: Option<String>,
    },
    /// Dry-run mode: return what would happen without executing.
    DryRun { rule_id: Option<String> },
}

/// Reason a mutation was denied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PolicyDenialReason {
    /// The tool is explicitly blocked in configuration.
    ToolBlocked,
    /// The hourly MCP mutation rate limit has been exceeded.
    RateLimited,
    /// A hard rule denied the request.
    HardRule,
    /// A user-defined rule denied the request.
    UserRule,
}

impl std::fmt::Display for PolicyDenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyDenialReason::ToolBlocked => write!(f, "tool_blocked"),
            PolicyDenialReason::RateLimited => write!(f, "rate_limited"),
            PolicyDenialReason::HardRule => write!(f, "hard_rule"),
            PolicyDenialReason::UserRule => write!(f, "user_rule"),
        }
    }
}

/// Audit record logged for every policy evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyAuditRecord {
    pub tool_name: String,
    pub decision: String,
    pub reason: Option<String>,
    pub matched_rule_id: Option<String>,
    pub matched_rule_label: Option<String>,
    pub rate_limit_key: Option<String>,
}

/// Centralized MCP mutation policy evaluator.
pub struct McpPolicyEvaluator;

impl McpPolicyEvaluator {
    /// Evaluate whether a tool invocation should proceed.
    ///
    /// Accepts `params_json` for future keyword/author matching;
    /// currently unused but threaded through for v2 extensibility.
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

        // 2. Build effective rule set
        let rules = build_effective_rules(config, mode);
        let ctx = make_eval_context(tool_name, mode);

        // 3. Walk rules by priority: first match wins
        if let Some(rule) = find_matching_rule(&rules, &ctx) {
            let rule_id = Some(rule.id.clone());
            match &rule.action {
                PolicyAction::Allow => {
                    // Rule explicitly allows — still check rate limits below
                }
                PolicyAction::Deny { reason } => {
                    let denial = if rule.id.starts_with("v1:blocked:") {
                        PolicyDenialReason::ToolBlocked
                    } else if rule.id.starts_with("hard:") {
                        PolicyDenialReason::HardRule
                    } else {
                        PolicyDenialReason::UserRule
                    };
                    return Ok(PolicyDecision::Deny {
                        reason: denial,
                        rule_id: Some(format!("{} ({})", rule.id, reason)),
                    });
                }
                PolicyAction::RequireApproval { reason } => {
                    return Ok(PolicyDecision::RouteToApproval {
                        reason: reason.clone(),
                        rule_id,
                    });
                }
                PolicyAction::DryRun => {
                    return Ok(PolicyDecision::DryRun { rule_id });
                }
            }
        }

        // 4. Check per-dimension rate limits
        if let Some(exceeded_key) = rate_limits::check_policy_rate_limits(
            pool,
            tool_name,
            &ctx.category.to_string(),
            &config.rate_limits,
        )
        .await?
        {
            return Ok(PolicyDecision::Deny {
                reason: PolicyDenialReason::RateLimited,
                rule_id: Some(exceeded_key),
            });
        }

        // 5. Legacy global rate limit
        let allowed = rate_limits::check_rate_limit(pool, "mcp_mutation").await?;
        if !allowed {
            return Ok(PolicyDecision::Deny {
                reason: PolicyDenialReason::RateLimited,
                rule_id: None,
            });
        }

        // 6. Default: allow
        Ok(PolicyDecision::Allow)
    }

    /// Log a policy decision to the action log.
    pub async fn log_decision(
        pool: &DbPool,
        tool_name: &str,
        decision: &PolicyDecision,
    ) -> Result<(), StorageError> {
        let (status, reason_str, rule_id) = match decision {
            PolicyDecision::Allow => ("allowed", None, None),
            PolicyDecision::RouteToApproval { reason, rule_id } => {
                ("routed_to_approval", Some(reason.clone()), rule_id.clone())
            }
            PolicyDecision::Deny { reason, rule_id } => {
                ("denied", Some(reason.to_string()), rule_id.clone())
            }
            PolicyDecision::DryRun { rule_id } => ("dry_run", None, rule_id.clone()),
        };

        let category = tool_category(tool_name);

        let audit = PolicyAuditRecordV2 {
            tool_name: tool_name.to_string(),
            category: category.to_string(),
            decision: status.to_string(),
            reason: reason_str,
            matched_rule_id: rule_id,
            matched_rule_label: None,
            rate_limit_key: None,
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

    /// Record a successful mutation against rate limit counters.
    ///
    /// Increments both the legacy global counter and any applicable
    /// per-dimension counters.
    pub async fn record_mutation(
        pool: &DbPool,
        tool_name: &str,
        rate_limit_configs: &[super::types::PolicyRateLimit],
    ) -> Result<(), StorageError> {
        // Legacy global counter
        rate_limits::increment_rate_limit(pool, "mcp_mutation").await?;

        // Per-dimension counters
        let category = tool_category(tool_name).to_string();
        rate_limits::record_policy_rate_limits(pool, tool_name, &category, rate_limit_configs).await
    }
}
