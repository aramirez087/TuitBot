//! Pre-built policy template definitions.
//!
//! Each template provides a complete set of rules and rate limits
//! that can be applied as a baseline policy profile.

use super::types::{
    PolicyAction, PolicyRateLimit, PolicyRule, PolicyTemplateName, RateLimitDimension,
    RuleConditions, ToolCategory,
};

/// A complete policy template with rules and rate limits.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PolicyTemplate {
    pub name: PolicyTemplateName,
    pub description: &'static str,
    pub rules: Vec<PolicyRule>,
    pub rate_limits: Vec<PolicyRateLimit>,
}

/// Get a specific template by name.
pub fn get_template(name: &PolicyTemplateName) -> PolicyTemplate {
    match name {
        PolicyTemplateName::SafeDefault => safe_default(),
        PolicyTemplateName::GrowthAggressive => growth_aggressive(),
        PolicyTemplateName::AgencyMode => agency_mode(),
    }
}

/// List all available templates.
pub fn list_templates() -> Vec<PolicyTemplate> {
    vec![safe_default(), growth_aggressive(), agency_mode()]
}

fn safe_default() -> PolicyTemplate {
    PolicyTemplate {
        name: PolicyTemplateName::SafeDefault,
        description: "Conservative defaults. Deletes and posts require approval, \
                       engagement actions are allowed. 20/hour global limit.",
        rules: vec![
            PolicyRule {
                id: "tpl:safe:delete_approval".into(),
                priority: 100,
                label: "Delete requires approval".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Delete],
                    ..Default::default()
                },
                action: PolicyAction::RequireApproval {
                    reason: "delete actions require approval (safe_default)".into(),
                },
            },
            PolicyRule {
                id: "tpl:safe:write_approval".into(),
                priority: 101,
                label: "Posts and replies require approval".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Write],
                    ..Default::default()
                },
                action: PolicyAction::RequireApproval {
                    reason: "write actions require approval (safe_default)".into(),
                },
            },
            PolicyRule {
                id: "tpl:safe:thread_approval".into(),
                priority: 102,
                label: "Threads require approval".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Thread],
                    ..Default::default()
                },
                action: PolicyAction::RequireApproval {
                    reason: "thread actions require approval (safe_default)".into(),
                },
            },
            PolicyRule {
                id: "tpl:safe:engage_allow".into(),
                priority: 103,
                label: "Engagement actions allowed".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Engage],
                    ..Default::default()
                },
                action: PolicyAction::Allow,
            },
        ],
        rate_limits: vec![
            PolicyRateLimit {
                key: "mcp:global:hourly".into(),
                dimension: RateLimitDimension::Global,
                match_value: "*".into(),
                max_count: 20,
                period_seconds: 3600,
            },
            PolicyRateLimit {
                key: "mcp:category:engage:daily".into(),
                dimension: RateLimitDimension::Category,
                match_value: "engage".into(),
                max_count: 50,
                period_seconds: 86400,
            },
        ],
    }
}

fn growth_aggressive() -> PolicyTemplate {
    PolicyTemplate {
        name: PolicyTemplateName::GrowthAggressive,
        description: "Higher limits for growth-focused accounts. Deletes still require \
                       approval, all other mutations allowed. 50/hour global limit.",
        rules: vec![
            PolicyRule {
                id: "tpl:growth:delete_approval".into(),
                priority: 100,
                label: "Delete requires approval".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Delete],
                    ..Default::default()
                },
                action: PolicyAction::RequireApproval {
                    reason: "delete actions require approval (growth_aggressive)".into(),
                },
            },
            PolicyRule {
                id: "tpl:growth:allow_all".into(),
                priority: 150,
                label: "All other mutations allowed".into(),
                enabled: true,
                conditions: RuleConditions::default(),
                action: PolicyAction::Allow,
            },
        ],
        rate_limits: vec![
            PolicyRateLimit {
                key: "mcp:global:hourly".into(),
                dimension: RateLimitDimension::Global,
                match_value: "*".into(),
                max_count: 50,
                period_seconds: 3600,
            },
            PolicyRateLimit {
                key: "mcp:category:engage:daily".into(),
                dimension: RateLimitDimension::Category,
                match_value: "engage".into(),
                max_count: 100,
                period_seconds: 86400,
            },
        ],
    }
}

fn agency_mode() -> PolicyTemplate {
    PolicyTemplate {
        name: PolicyTemplateName::AgencyMode,
        description: "Maximum autonomy for managed accounts. Only deletes require \
                       approval. 200/day global limit with per-tool daily caps.",
        rules: vec![
            PolicyRule {
                id: "tpl:agency:delete_approval".into(),
                priority: 100,
                label: "Delete requires approval".into(),
                enabled: true,
                conditions: RuleConditions {
                    categories: vec![ToolCategory::Delete],
                    ..Default::default()
                },
                action: PolicyAction::RequireApproval {
                    reason: "delete actions require approval (agency_mode)".into(),
                },
            },
            PolicyRule {
                id: "tpl:agency:allow_all".into(),
                priority: 150,
                label: "All mutations allowed".into(),
                enabled: true,
                conditions: RuleConditions::default(),
                action: PolicyAction::Allow,
            },
        ],
        rate_limits: vec![
            PolicyRateLimit {
                key: "mcp:global:daily".into(),
                dimension: RateLimitDimension::Global,
                match_value: "*".into(),
                max_count: 200,
                period_seconds: 86400,
            },
            PolicyRateLimit {
                key: "mcp:tool:post_tweet:daily".into(),
                dimension: RateLimitDimension::Tool,
                match_value: "post_tweet".into(),
                max_count: 30,
                period_seconds: 86400,
            },
            PolicyRateLimit {
                key: "mcp:tool:like_tweet:daily".into(),
                dimension: RateLimitDimension::Tool,
                match_value: "like_tweet".into(),
                max_count: 50,
                period_seconds: 86400,
            },
            PolicyRateLimit {
                key: "mcp:tool:follow_user:daily".into(),
                dimension: RateLimitDimension::Tool,
                match_value: "follow_user".into(),
                max_count: 20,
                period_seconds: 86400,
            },
        ],
    }
}
