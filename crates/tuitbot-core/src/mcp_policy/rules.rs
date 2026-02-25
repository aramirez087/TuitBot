//! Rule matching engine for policy v2.
//!
//! Builds the effective rule set from hard rules + template + user rules + v1
//! compat rules, then evaluates conditions against an evaluation context.

use crate::config::{McpPolicyConfig, OperatingMode};

use super::migration::v1_to_v2_rules;
use super::templates::get_template;
use super::types::{
    tool_category, PolicyAction, PolicyRule, RuleConditions, ScheduleWindow, ToolCategory,
};

/// Context for evaluating a single tool invocation against policy rules.
pub struct EvalContext<'a> {
    pub tool_name: &'a str,
    pub category: ToolCategory,
    pub mode: &'a OperatingMode,
}

/// Check if all conditions in a rule match the evaluation context.
///
/// AND across dimensions, OR within each dimension.
/// Empty vectors mean "match any" for that dimension.
pub fn conditions_match(conditions: &RuleConditions, ctx: &EvalContext) -> bool {
    // Tool name filter (OR within)
    if !conditions.tools.is_empty() && !conditions.tools.iter().any(|t| t == ctx.tool_name) {
        return false;
    }

    // Category filter (OR within)
    if !conditions.categories.is_empty()
        && !conditions.categories.contains(&ctx.category)
    {
        return false;
    }

    // Mode filter (OR within)
    if !conditions.modes.is_empty() && !conditions.modes.iter().any(|m| m == ctx.mode) {
        return false;
    }

    // Schedule window filter
    if let Some(window) = &conditions.schedule_window {
        if !check_schedule_window(window) {
            return false;
        }
    }

    true
}

/// Check if the current time falls within a schedule window.
fn check_schedule_window(window: &ScheduleWindow) -> bool {
    let now = chrono::Utc::now();

    // Parse timezone; fall back to UTC on error
    let tz: chrono_tz::Tz = window.timezone.parse().unwrap_or(chrono_tz::UTC);
    let local = now.with_timezone(&tz);
    let hour = local.format("%H").to_string().parse::<u8>().unwrap_or(0);

    // Day-of-week check
    if !window.days.is_empty() {
        let day = local.format("%a").to_string().to_lowercase();
        let day_short = &day[..3.min(day.len())];
        if !window.days.iter().any(|d| d.to_lowercase() == day_short) {
            return false;
        }
    }

    // Hour window check (supports wrapping past midnight)
    if window.start_hour <= window.end_hour {
        hour >= window.start_hour && hour < window.end_hour
    } else {
        // Wraps past midnight: e.g. 22..06 means 22,23,0,1,2,3,4,5
        hour >= window.start_hour || hour < window.end_hour
    }
}

/// Build the sorted effective rule set for a given config and mode.
///
/// Order: hard rules (0-10) → template rules (100-199) → user rules (200+)
/// → v1 compat rules (300+, only when no explicit rules or template).
pub fn build_effective_rules(config: &McpPolicyConfig, mode: &OperatingMode) -> Vec<PolicyRule> {
    let mut rules: Vec<PolicyRule> = Vec::new();

    // --- Hard rules (always injected) ---
    // Priority 0: delete_tweet always requires approval
    rules.push(PolicyRule {
        id: "hard:delete_approval".into(),
        priority: 0,
        label: "Delete always requires approval".into(),
        enabled: true,
        conditions: RuleConditions {
            categories: vec![ToolCategory::Delete],
            ..Default::default()
        },
        action: PolicyAction::RequireApproval {
            reason: "delete actions always require approval".into(),
        },
    });

    // Priority 10: Composer mode forces approval for all mutations
    if *mode == OperatingMode::Composer {
        rules.push(PolicyRule {
            id: "hard:composer_approval".into(),
            priority: 10,
            label: "Composer mode requires approval".into(),
            enabled: true,
            conditions: RuleConditions::default(),
            action: PolicyAction::RequireApproval {
                reason: "composer mode requires approval for all mutations".into(),
            },
        });
    }

    // --- Template rules ---
    if let Some(template_name) = &config.template {
        let template = get_template(template_name);
        rules.extend(template.rules);
    }

    // --- User rules ---
    rules.extend(config.rules.iter().filter(|r| r.enabled).cloned());

    // --- v1 compat rules (only when no explicit v2 config) ---
    if config.template.is_none() && config.rules.is_empty() {
        rules.extend(v1_to_v2_rules(config));
    }

    // Sort by priority (stable sort preserves insertion order within same priority)
    rules.sort_by_key(|r| r.priority);
    rules
}

/// Resolve the first matching rule for a tool invocation.
pub fn find_matching_rule<'a>(
    rules: &'a [PolicyRule],
    ctx: &EvalContext,
) -> Option<&'a PolicyRule> {
    rules
        .iter()
        .filter(|r| r.enabled)
        .find(|r| conditions_match(&r.conditions, ctx))
}

/// Build an `EvalContext` from raw inputs.
pub fn make_eval_context<'a>(tool_name: &'a str, mode: &'a OperatingMode) -> EvalContext<'a> {
    EvalContext {
        tool_name,
        category: tool_category(tool_name),
        mode,
    }
}
