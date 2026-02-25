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
