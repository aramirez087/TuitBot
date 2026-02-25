//! v1 → v2 policy migration.
//!
//! Converts flat v1 config fields into v2 policy rules for backward
//! compatibility. Only used when `config.rules` is empty AND
//! `config.template` is None.

use crate::config::McpPolicyConfig;

use super::types::{PolicyAction, PolicyRule, RuleConditions};

/// Convert v1 flat config fields into v2 rules.
///
/// Synthesized rules use priority 300+ so they sort after hard rules,
/// template rules, and user rules.
pub fn v1_to_v2_rules(config: &McpPolicyConfig) -> Vec<PolicyRule> {
    let mut rules = Vec::new();
    let mut priority = 300u32;

    // blocked_tools → Deny rules
    for tool in &config.blocked_tools {
        rules.push(PolicyRule {
            id: format!("v1:blocked:{tool}"),
            priority,
            label: format!("Blocked: {tool}"),
            enabled: true,
            conditions: RuleConditions {
                tools: vec![tool.clone()],
                ..Default::default()
            },
            action: PolicyAction::Deny {
                reason: format!("tool '{tool}' is blocked"),
            },
        });
        priority += 1;
    }

    // dry_run_mutations → global DryRun rule
    if config.dry_run_mutations {
        rules.push(PolicyRule {
            id: "v1:dry_run".into(),
            priority,
            label: "Dry-run mode".into(),
            enabled: true,
            conditions: RuleConditions::default(),
            action: PolicyAction::DryRun,
        });
        priority += 1;
    }

    // require_approval_for → RequireApproval rules
    for tool in &config.require_approval_for {
        rules.push(PolicyRule {
            id: format!("v1:approval:{tool}"),
            priority,
            label: format!("Requires approval: {tool}"),
            enabled: true,
            conditions: RuleConditions {
                tools: vec![tool.clone()],
                ..Default::default()
            },
            action: PolicyAction::RequireApproval {
                reason: format!("tool '{tool}' requires approval"),
            },
        });
        priority += 1;
    }

    rules
}
