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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_templates_returns_three() {
        let templates = list_templates();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn list_templates_has_unique_names() {
        let templates = list_templates();
        let names: Vec<_> = templates.iter().map(|t| format!("{:?}", t.name)).collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len(), "Template names must be unique");
    }

    #[test]
    fn safe_default_has_rules_and_limits() {
        let t = get_template(&PolicyTemplateName::SafeDefault);
        assert!(!t.rules.is_empty(), "SafeDefault must have rules");
        assert!(
            !t.rate_limits.is_empty(),
            "SafeDefault must have rate limits"
        );
        assert!(!t.description.is_empty());
    }

    #[test]
    fn growth_aggressive_has_rules_and_limits() {
        let t = get_template(&PolicyTemplateName::GrowthAggressive);
        assert!(!t.rules.is_empty());
        assert!(!t.rate_limits.is_empty());
    }

    #[test]
    fn agency_mode_has_rules_and_limits() {
        let t = get_template(&PolicyTemplateName::AgencyMode);
        assert!(!t.rules.is_empty());
        assert!(!t.rate_limits.is_empty());
    }

    #[test]
    fn safe_default_requires_approval_for_deletes() {
        let t = get_template(&PolicyTemplateName::SafeDefault);
        let delete_rule = t
            .rules
            .iter()
            .find(|r| r.conditions.categories.contains(&ToolCategory::Delete));
        assert!(delete_rule.is_some(), "SafeDefault must have a delete rule");
        if let Some(rule) = delete_rule {
            assert!(matches!(rule.action, PolicyAction::RequireApproval { .. }));
        }
    }

    #[test]
    fn safe_default_requires_approval_for_writes() {
        let t = get_template(&PolicyTemplateName::SafeDefault);
        let write_rule = t
            .rules
            .iter()
            .find(|r| r.conditions.categories.contains(&ToolCategory::Write));
        assert!(write_rule.is_some());
        if let Some(rule) = write_rule {
            assert!(matches!(rule.action, PolicyAction::RequireApproval { .. }));
        }
    }

    #[test]
    fn all_templates_have_nonempty_descriptions() {
        for t in list_templates() {
            assert!(
                !t.description.is_empty(),
                "{:?} has empty description",
                t.name
            );
        }
    }

    #[test]
    fn all_templates_have_enabled_rules() {
        for t in list_templates() {
            assert!(
                t.rules.iter().all(|r| r.enabled),
                "{:?} has disabled rules in template",
                t.name
            );
        }
    }

    #[test]
    fn all_templates_rate_limits_have_positive_max() {
        for t in list_templates() {
            for rl in &t.rate_limits {
                assert!(
                    rl.max_count > 0,
                    "{:?} rate limit {} has max_count=0",
                    t.name,
                    rl.key
                );
                assert!(
                    rl.period_seconds > 0,
                    "{:?} rate limit {} has period=0",
                    t.name,
                    rl.key
                );
            }
        }
    }

    #[test]
    fn all_templates_rules_have_unique_ids() {
        for t in list_templates() {
            let ids: Vec<_> = t.rules.iter().map(|r| &r.id).collect();
            let unique: std::collections::HashSet<_> = ids.iter().collect();
            assert_eq!(
                ids.len(),
                unique.len(),
                "{:?} has duplicate rule ids",
                t.name
            );
        }
    }

    #[test]
    fn get_template_roundtrip() {
        // get_template should return same data as list_templates for each name
        for t in list_templates() {
            let fetched = get_template(&t.name);
            assert_eq!(fetched.rules.len(), t.rules.len());
            assert_eq!(fetched.rate_limits.len(), t.rate_limits.len());
        }
    }

    #[test]
    fn template_serializes_to_json() {
        for t in list_templates() {
            let json = serde_json::to_string(&t);
            assert!(
                json.is_ok(),
                "{:?} failed to serialize: {:?}",
                t.name,
                json.err()
            );
        }
    }
}
