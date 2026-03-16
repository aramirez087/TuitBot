//! Policy engine v2 types.
//!
//! Defines multi-dimensional rules, per-dimension rate limits, template names,
//! and enriched audit records for the v2 policy engine.

use serde::{Deserialize, Serialize};

use crate::config::OperatingMode;

/// Tool category for grouping related MCP tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Read,
    Write,
    Engage,
    Media,
    Thread,
    Delete,
    /// Universal request mutations via x_post/x_put/x_delete.
    UniversalRequest,
    /// Enterprise admin mutations (compliance jobs, stream rules).
    EnterpriseAdmin,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::Read => write!(f, "read"),
            ToolCategory::Write => write!(f, "write"),
            ToolCategory::Engage => write!(f, "engage"),
            ToolCategory::Media => write!(f, "media"),
            ToolCategory::Thread => write!(f, "thread"),
            ToolCategory::Delete => write!(f, "delete"),
            ToolCategory::UniversalRequest => write!(f, "universal_request"),
            ToolCategory::EnterpriseAdmin => write!(f, "enterprise_admin"),
        }
    }
}

/// Map a tool name to its canonical category.
pub fn tool_category(name: &str) -> ToolCategory {
    match name {
        // Read tools
        "get_tweet" | "search_tweets" | "get_user_profile" | "get_timeline" | "get_mentions"
        | "get_followers" | "get_following" | "get_policy_status" => ToolCategory::Read,
        // Write tools
        "post_tweet" | "compose_tweet" | "reply_to_tweet" | "quote_tweet" | "x_post_tweet"
        | "x_reply_to_tweet" | "x_quote_tweet" => ToolCategory::Write,
        // Read tools (new)
        "x_get_home_timeline" | "get_x_usage" => ToolCategory::Read,
        // Engage tools
        "like_tweet" | "x_like_tweet" | "follow_user" | "x_follow_user" | "unfollow_user"
        | "x_unfollow_user" | "retweet" | "x_retweet" | "unretweet" | "x_unretweet" => {
            ToolCategory::Engage
        }
        // Media tools
        "upload_media" | "x_upload_media" | "post_tweet_with_media" => ToolCategory::Media,
        // Thread tools
        "post_thread" | "x_post_thread" | "compose_thread" | "propose_and_queue_replies" => {
            ToolCategory::Thread
        }
        // Delete tools
        "delete_tweet" | "x_delete_tweet" => ToolCategory::Delete,
        // Universal request mutations (admin-only)
        "x_post" | "x_put" | "x_delete" => ToolCategory::UniversalRequest,
        // Enterprise admin mutations (compliance jobs, stream rules)
        "x_v2_compliance_job_create" | "x_v2_stream_rules_add" | "x_v2_stream_rules_delete" => {
            ToolCategory::EnterpriseAdmin
        }
        // Default unknown tools to Write (safest default for mutations)
        _ => ToolCategory::Write,
    }
}

/// A policy rule with multi-dimensional conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Unique identifier for this rule.
    pub id: String,
    /// Priority: 0-99 = hard rules, 100-199 = template, 200+ = user.
    pub priority: u32,
    /// Human-readable label.
    pub label: String,
    /// Whether this rule is active.
    pub enabled: bool,
    /// Conditions that must all match (AND across dimensions).
    pub conditions: RuleConditions,
    /// Action to take when conditions match.
    pub action: PolicyAction,
}

/// Conditions for a policy rule. AND across dimensions, OR within each dimension.
///
/// Empty vectors mean "match any" for that dimension.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleConditions {
    /// Tool names to match (OR). Empty = match all tools.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Categories to match (OR). Empty = match all categories.
    #[serde(default)]
    pub categories: Vec<ToolCategory>,
    /// Operating modes to match (OR). Empty = match all modes.
    #[serde(default)]
    pub modes: Vec<OperatingMode>,
    /// Optional schedule window constraint.
    #[serde(default)]
    pub schedule_window: Option<ScheduleWindow>,
}

/// A time-based schedule window for rule conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleWindow {
    /// Start hour (0-23).
    pub start_hour: u8,
    /// End hour (0-23). If < start_hour, wraps past midnight.
    pub end_hour: u8,
    /// IANA timezone string (e.g. "America/New_York").
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Days of week (e.g. ["mon", "tue"]). Empty = all days.
    #[serde(default)]
    pub days: Vec<String>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Action to take when a policy rule matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyAction {
    /// Allow the mutation to proceed.
    Allow,
    /// Deny the mutation with a reason.
    Deny { reason: String },
    /// Route to the approval queue.
    RequireApproval { reason: String },
    /// Dry-run mode: return what would happen without executing.
    DryRun,
}

/// Per-dimension rate limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRateLimit {
    /// Composite key, e.g. "mcp:like_tweet:hourly".
    pub key: String,
    /// Which dimension to rate-limit on.
    pub dimension: RateLimitDimension,
    /// Value to match within the dimension (tool name, category name, etc.).
    pub match_value: String,
    /// Maximum count within the period.
    pub max_count: u32,
    /// Period length in seconds.
    pub period_seconds: u64,
}

/// Dimension for per-dimension rate limiting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitDimension {
    /// Rate limit per individual tool.
    Tool,
    /// Rate limit per tool category.
    Category,
    /// Rate limit per engagement type (like, follow, etc.).
    EngagementType,
    /// Global rate limit across all tools.
    Global,
}

/// Named policy template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTemplateName {
    SafeDefault,
    GrowthAggressive,
    AgencyMode,
}

impl std::fmt::Display for PolicyTemplateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyTemplateName::SafeDefault => write!(f, "safe_default"),
            PolicyTemplateName::GrowthAggressive => write!(f, "growth_aggressive"),
            PolicyTemplateName::AgencyMode => write!(f, "agency_mode"),
        }
    }
}

impl std::str::FromStr for PolicyTemplateName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "safe_default" => Ok(PolicyTemplateName::SafeDefault),
            "growth_aggressive" => Ok(PolicyTemplateName::GrowthAggressive),
            "agency_mode" => Ok(PolicyTemplateName::AgencyMode),
            _ => Err(format!("unknown template: {s}")),
        }
    }
}

/// Enriched audit record for v2 policy evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyAuditRecordV2 {
    pub tool_name: String,
    pub category: String,
    pub decision: String,
    pub reason: Option<String>,
    pub matched_rule_id: Option<String>,
    pub matched_rule_label: Option<String>,
    pub rate_limit_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_category_universal_request_tools() {
        assert_eq!(tool_category("x_post"), ToolCategory::UniversalRequest);
        assert_eq!(tool_category("x_put"), ToolCategory::UniversalRequest);
        assert_eq!(tool_category("x_delete"), ToolCategory::UniversalRequest);
    }

    #[test]
    fn tool_category_standard_tools() {
        assert_eq!(tool_category("post_tweet"), ToolCategory::Write);
        assert_eq!(tool_category("like_tweet"), ToolCategory::Engage);
        assert_eq!(tool_category("get_tweet"), ToolCategory::Read);
        assert_eq!(tool_category("delete_tweet"), ToolCategory::Delete);
        assert_eq!(tool_category("upload_media"), ToolCategory::Media);
        assert_eq!(tool_category("post_thread"), ToolCategory::Thread);
    }

    #[test]
    fn tool_category_unknown_defaults_to_write() {
        assert_eq!(tool_category("some_future_tool"), ToolCategory::Write);
    }

    #[test]
    fn tool_category_enterprise_admin_tools() {
        assert_eq!(
            tool_category("x_v2_compliance_job_create"),
            ToolCategory::EnterpriseAdmin
        );
        assert_eq!(
            tool_category("x_v2_stream_rules_add"),
            ToolCategory::EnterpriseAdmin
        );
        assert_eq!(
            tool_category("x_v2_stream_rules_delete"),
            ToolCategory::EnterpriseAdmin
        );
    }

    #[test]
    fn tool_category_display() {
        assert_eq!(
            ToolCategory::UniversalRequest.to_string(),
            "universal_request"
        );
        assert_eq!(
            ToolCategory::EnterpriseAdmin.to_string(),
            "enterprise_admin"
        );
        assert_eq!(ToolCategory::Read.to_string(), "read");
        assert_eq!(ToolCategory::Write.to_string(), "write");
    }

    #[test]
    fn tool_category_serde_roundtrip() {
        let json = serde_json::to_string(&ToolCategory::UniversalRequest).unwrap();
        assert_eq!(json, "\"universal_request\"");
        let parsed: ToolCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ToolCategory::UniversalRequest);
    }

    #[test]
    fn tool_category_enterprise_admin_serde_roundtrip() {
        let json = serde_json::to_string(&ToolCategory::EnterpriseAdmin).unwrap();
        assert_eq!(json, "\"enterprise_admin\"");
        let parsed: ToolCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ToolCategory::EnterpriseAdmin);
    }

    // --- PolicyAction serde ---

    #[test]
    fn policy_action_allow_serde() {
        let json = serde_json::to_string(&PolicyAction::Allow).unwrap();
        let back: PolicyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PolicyAction::Allow);
    }

    #[test]
    fn policy_action_deny_serde() {
        let action = PolicyAction::Deny {
            reason: "blocked".into(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let back: PolicyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back,
            PolicyAction::Deny {
                reason: "blocked".into()
            }
        );
    }

    #[test]
    fn policy_action_require_approval_serde() {
        let action = PolicyAction::RequireApproval {
            reason: "needs review".into(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let back: PolicyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back,
            PolicyAction::RequireApproval {
                reason: "needs review".into()
            }
        );
    }

    #[test]
    fn policy_action_dry_run_serde() {
        let json = serde_json::to_string(&PolicyAction::DryRun).unwrap();
        let back: PolicyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PolicyAction::DryRun);
    }

    // --- RuleConditions ---

    #[test]
    fn rule_conditions_default() {
        let rc = RuleConditions::default();
        assert!(rc.tools.is_empty());
        assert!(rc.categories.is_empty());
        assert!(rc.modes.is_empty());
        assert!(rc.schedule_window.is_none());
    }

    #[test]
    fn rule_conditions_serde_roundtrip() {
        let rc = RuleConditions {
            tools: vec!["post_tweet".into()],
            categories: vec![ToolCategory::Write],
            modes: vec![OperatingMode::Autopilot],
            schedule_window: Some(ScheduleWindow {
                start_hour: 9,
                end_hour: 17,
                timezone: "UTC".into(),
                days: vec!["mon".into(), "fri".into()],
            }),
        };
        let json = serde_json::to_string(&rc).unwrap();
        let back: RuleConditions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tools, vec!["post_tweet"]);
        assert_eq!(back.categories, vec![ToolCategory::Write]);
        let sw = back.schedule_window.unwrap();
        assert_eq!(sw.start_hour, 9);
        assert_eq!(sw.end_hour, 17);
        assert_eq!(sw.days.len(), 2);
    }

    // --- PolicyRule ---

    #[test]
    fn policy_rule_serde_roundtrip() {
        let rule = PolicyRule {
            id: "rule1".into(),
            priority: 100,
            label: "Test rule".into(),
            enabled: true,
            conditions: RuleConditions::default(),
            action: PolicyAction::Allow,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: PolicyRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "rule1");
        assert_eq!(back.priority, 100);
        assert_eq!(back.label, "Test rule");
        assert!(back.enabled);
    }

    // --- PolicyRateLimit ---

    #[test]
    fn policy_rate_limit_serde_roundtrip() {
        let rl = PolicyRateLimit {
            key: "mcp:like_tweet:hourly".into(),
            dimension: RateLimitDimension::Tool,
            match_value: "like_tweet".into(),
            max_count: 10,
            period_seconds: 3600,
        };
        let json = serde_json::to_string(&rl).unwrap();
        let back: PolicyRateLimit = serde_json::from_str(&json).unwrap();
        assert_eq!(back.key, "mcp:like_tweet:hourly");
        assert_eq!(back.dimension, RateLimitDimension::Tool);
        assert_eq!(back.max_count, 10);
        assert_eq!(back.period_seconds, 3600);
    }

    // --- RateLimitDimension ---

    #[test]
    fn rate_limit_dimension_serde_roundtrip() {
        for dim in [
            RateLimitDimension::Tool,
            RateLimitDimension::Category,
            RateLimitDimension::EngagementType,
            RateLimitDimension::Global,
        ] {
            let json = serde_json::to_string(&dim).unwrap();
            let back: RateLimitDimension = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dim);
        }
    }

    // --- PolicyTemplateName ---

    #[test]
    fn policy_template_name_display() {
        assert_eq!(PolicyTemplateName::SafeDefault.to_string(), "safe_default");
        assert_eq!(
            PolicyTemplateName::GrowthAggressive.to_string(),
            "growth_aggressive"
        );
        assert_eq!(PolicyTemplateName::AgencyMode.to_string(), "agency_mode");
    }

    #[test]
    fn policy_template_name_from_str() {
        assert_eq!(
            "safe_default".parse::<PolicyTemplateName>().unwrap(),
            PolicyTemplateName::SafeDefault
        );
        assert_eq!(
            "growth_aggressive".parse::<PolicyTemplateName>().unwrap(),
            PolicyTemplateName::GrowthAggressive
        );
        assert_eq!(
            "agency_mode".parse::<PolicyTemplateName>().unwrap(),
            PolicyTemplateName::AgencyMode
        );
        assert!("invalid".parse::<PolicyTemplateName>().is_err());
    }

    #[test]
    fn policy_template_name_serde_roundtrip() {
        for name in [
            PolicyTemplateName::SafeDefault,
            PolicyTemplateName::GrowthAggressive,
            PolicyTemplateName::AgencyMode,
        ] {
            let json = serde_json::to_string(&name).unwrap();
            let back: PolicyTemplateName = serde_json::from_str(&json).unwrap();
            assert_eq!(back, name);
        }
    }

    // --- ScheduleWindow ---

    #[test]
    fn schedule_window_serde_roundtrip() {
        let sw = ScheduleWindow {
            start_hour: 0,
            end_hour: 23,
            timezone: "America/Chicago".into(),
            days: vec!["mon".into(), "tue".into(), "wed".into()],
        };
        let json = serde_json::to_string(&sw).unwrap();
        let back: ScheduleWindow = serde_json::from_str(&json).unwrap();
        assert_eq!(back.start_hour, 0);
        assert_eq!(back.end_hour, 23);
        assert_eq!(back.timezone, "America/Chicago");
        assert_eq!(back.days.len(), 3);
    }

    #[test]
    fn schedule_window_default_timezone() {
        let json = r#"{"start_hour": 9, "end_hour": 17}"#;
        let sw: ScheduleWindow = serde_json::from_str(json).unwrap();
        assert_eq!(sw.timezone, "UTC");
        assert!(sw.days.is_empty());
    }

    // --- ToolCategory Display all variants ---

    #[test]
    fn tool_category_display_all_variants() {
        assert_eq!(ToolCategory::Engage.to_string(), "engage");
        assert_eq!(ToolCategory::Media.to_string(), "media");
        assert_eq!(ToolCategory::Thread.to_string(), "thread");
        assert_eq!(ToolCategory::Delete.to_string(), "delete");
    }

    // --- ToolCategory serde all variants ---

    #[test]
    fn tool_category_serde_all_variants() {
        for cat in [
            ToolCategory::Read,
            ToolCategory::Write,
            ToolCategory::Engage,
            ToolCategory::Media,
            ToolCategory::Thread,
            ToolCategory::Delete,
            ToolCategory::UniversalRequest,
            ToolCategory::EnterpriseAdmin,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: ToolCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cat, "roundtrip failed for {json}");
        }
    }

    // --- PolicyAuditRecordV2 ---

    #[test]
    fn policy_audit_record_v2_serialize() {
        let rec = PolicyAuditRecordV2 {
            tool_name: "post_tweet".into(),
            category: "write".into(),
            decision: "allow".into(),
            reason: Some("matched rule".into()),
            matched_rule_id: Some("r1".into()),
            matched_rule_label: Some("allow writes".into()),
            rate_limit_key: None,
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("post_tweet"));
        assert!(json.contains("allow"));
    }

    // --- tool_category additional coverage ---

    #[test]
    fn tool_category_new_read_tools() {
        assert_eq!(tool_category("x_get_home_timeline"), ToolCategory::Read);
        assert_eq!(tool_category("get_x_usage"), ToolCategory::Read);
        assert_eq!(tool_category("get_policy_status"), ToolCategory::Read);
    }

    #[test]
    fn tool_category_engage_variants() {
        assert_eq!(tool_category("x_like_tweet"), ToolCategory::Engage);
        assert_eq!(tool_category("x_follow_user"), ToolCategory::Engage);
        assert_eq!(tool_category("x_unfollow_user"), ToolCategory::Engage);
        assert_eq!(tool_category("x_retweet"), ToolCategory::Engage);
        assert_eq!(tool_category("x_unretweet"), ToolCategory::Engage);
        assert_eq!(tool_category("unfollow_user"), ToolCategory::Engage);
        assert_eq!(tool_category("unretweet"), ToolCategory::Engage);
    }

    #[test]
    fn tool_category_write_variants() {
        assert_eq!(tool_category("x_post_tweet"), ToolCategory::Write);
        assert_eq!(tool_category("x_reply_to_tweet"), ToolCategory::Write);
        assert_eq!(tool_category("x_quote_tweet"), ToolCategory::Write);
        assert_eq!(tool_category("compose_tweet"), ToolCategory::Write);
    }

    #[test]
    fn tool_category_media_and_thread() {
        assert_eq!(tool_category("x_upload_media"), ToolCategory::Media);
        assert_eq!(tool_category("post_tweet_with_media"), ToolCategory::Media);
        assert_eq!(tool_category("x_post_thread"), ToolCategory::Thread);
        assert_eq!(tool_category("compose_thread"), ToolCategory::Thread);
        assert_eq!(
            tool_category("propose_and_queue_replies"),
            ToolCategory::Thread
        );
    }

    #[test]
    fn tool_category_delete_variants() {
        assert_eq!(tool_category("x_delete_tweet"), ToolCategory::Delete);
    }
}
