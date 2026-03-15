//! Schedule, MCP policy, and circuit breaker configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

/// Active hours schedule configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleConfig {
    /// IANA timezone name (e.g. "America/New_York", "UTC").
    #[serde(default = "default_timezone")]
    pub timezone: String,

    /// Hour of day (0-23) when active posting window starts.
    #[serde(default = "default_active_hours_start")]
    pub active_hours_start: u8,

    /// Hour of day (0-23) when active posting window ends.
    #[serde(default = "default_active_hours_end")]
    pub active_hours_end: u8,

    /// Days of the week when posting is active (e.g. ["Mon", "Tue", ...]).
    #[serde(default = "default_active_days")]
    pub active_days: Vec<String>,

    /// Preferred posting times for tweets (HH:MM in 24h format, in configured timezone).
    /// When set, the content loop posts at these specific times instead of using interval mode.
    /// Use "auto" for research-backed defaults: 09:15, 12:30, 17:00.
    #[serde(default)]
    pub preferred_times: Vec<String>,

    /// Per-day overrides for preferred posting times.
    /// Keys are day abbreviations (Mon-Sun), values are lists of "HH:MM" times.
    /// Days not listed use the base `preferred_times`. Empty list = no posts that day.
    #[serde(default)]
    pub preferred_times_override: HashMap<String, Vec<String>>,

    /// Preferred day for weekly thread posting (Mon-Sun). None = interval mode.
    #[serde(default)]
    pub thread_preferred_day: Option<String>,

    /// Preferred time for weekly thread posting (HH:MM, 24h format).
    #[serde(default = "default_thread_preferred_time")]
    pub thread_preferred_time: String,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            active_hours_start: default_active_hours_start(),
            active_hours_end: default_active_hours_end(),
            active_days: default_active_days(),
            preferred_times: Vec::new(),
            preferred_times_override: HashMap::new(),
            thread_preferred_day: None,
            thread_preferred_time: default_thread_preferred_time(),
        }
    }
}

fn default_timezone() -> String {
    "UTC".to_string()
}
fn default_active_hours_start() -> u8 {
    8
}
fn default_active_hours_end() -> u8 {
    22
}
fn default_active_days() -> Vec<String> {
    vec![
        "Mon".to_string(),
        "Tue".to_string(),
        "Wed".to_string(),
        "Thu".to_string(),
        "Fri".to_string(),
        "Sat".to_string(),
        "Sun".to_string(),
    ]
}
fn default_thread_preferred_time() -> String {
    "10:00".to_string()
}

// ---------------------------------------------------------------------------
// MCP Policy
// ---------------------------------------------------------------------------

/// MCP mutation policy configuration.
///
/// Controls whether MCP mutation tools (post, reply, like, follow, etc.)
/// are gated by policy checks before execution.
///
/// v2 fields (`template`, `rules`, `rate_limits`) are additive — existing
/// v1 configs deserialize without changes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpPolicyConfig {
    // --- v1 fields (unchanged) ---
    /// Master switch: when false, all mutations are allowed without checks.
    #[serde(default = "default_true")]
    pub enforce_for_mutations: bool,

    /// Tool names that require routing through the approval queue.
    #[serde(default = "default_require_approval_for")]
    pub require_approval_for: Vec<String>,

    /// Tool names that are completely blocked from execution.
    #[serde(default)]
    pub blocked_tools: Vec<String>,

    /// When true, mutations return a dry-run response without executing.
    #[serde(default)]
    pub dry_run_mutations: bool,

    /// Maximum MCP mutations allowed per hour (aggregate across all tools).
    #[serde(default = "default_max_mutations_per_hour")]
    pub max_mutations_per_hour: u32,

    // --- v2 fields ---
    /// Optional named template to apply as the baseline rule set.
    #[serde(default)]
    pub template: Option<crate::mcp_policy::types::PolicyTemplateName>,

    /// Explicit policy rules (user-defined). Evaluated by priority order.
    #[serde(default)]
    pub rules: Vec<crate::mcp_policy::types::PolicyRule>,

    /// Per-dimension rate limits (beyond the global `max_mutations_per_hour`).
    #[serde(default)]
    pub rate_limits: Vec<crate::mcp_policy::types::PolicyRateLimit>,
}

fn default_true() -> bool {
    true
}

fn default_require_approval_for() -> Vec<String> {
    vec![
        "post_tweet".to_string(),
        "reply_to_tweet".to_string(),
        "follow_user".to_string(),
        "like_tweet".to_string(),
    ]
}

fn default_max_mutations_per_hour() -> u32 {
    20
}

// ---------------------------------------------------------------------------
// Circuit Breaker
// ---------------------------------------------------------------------------

/// Circuit breaker configuration for X API rate-limit protection.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    /// Number of errors within the window to trip the breaker.
    #[serde(default = "default_cb_error_threshold")]
    pub error_threshold: u32,

    /// Sliding window duration in seconds for counting errors.
    #[serde(default = "default_cb_window_seconds")]
    pub window_seconds: u64,

    /// How long (seconds) to stay Open before allowing a probe mutation.
    #[serde(default = "default_cb_cooldown_seconds")]
    pub cooldown_seconds: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            error_threshold: default_cb_error_threshold(),
            window_seconds: default_cb_window_seconds(),
            cooldown_seconds: default_cb_cooldown_seconds(),
        }
    }
}

fn default_cb_error_threshold() -> u32 {
    5
}
fn default_cb_window_seconds() -> u64 {
    300
}
fn default_cb_cooldown_seconds() -> u64 {
    600
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ScheduleConfig ---

    #[test]
    fn schedule_config_default() {
        let sc = ScheduleConfig::default();
        assert_eq!(sc.timezone, "UTC");
        assert_eq!(sc.active_hours_start, 8);
        assert_eq!(sc.active_hours_end, 22);
        assert_eq!(sc.active_days.len(), 7);
        assert!(sc.preferred_times.is_empty());
        assert!(sc.preferred_times_override.is_empty());
        assert!(sc.thread_preferred_day.is_none());
        assert_eq!(sc.thread_preferred_time, "10:00");
    }

    #[test]
    fn schedule_config_serde_roundtrip() {
        let sc = ScheduleConfig {
            timezone: "America/New_York".into(),
            active_hours_start: 9,
            active_hours_end: 21,
            active_days: vec!["Mon".into(), "Wed".into(), "Fri".into()],
            preferred_times: vec!["09:15".into(), "12:30".into()],
            preferred_times_override: HashMap::from([("Mon".into(), vec!["08:00".into()])]),
            thread_preferred_day: Some("Tue".into()),
            thread_preferred_time: "14:00".into(),
        };
        let json = serde_json::to_string(&sc).unwrap();
        let back: ScheduleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timezone, "America/New_York");
        assert_eq!(back.active_hours_start, 9);
        assert_eq!(back.active_days.len(), 3);
        assert_eq!(back.preferred_times.len(), 2);
        assert_eq!(back.preferred_times_override.len(), 1);
        assert_eq!(back.thread_preferred_day.as_deref(), Some("Tue"));
        assert_eq!(back.thread_preferred_time, "14:00");
    }

    #[test]
    fn schedule_config_deserialize_with_defaults() {
        let json = r#"{}"#;
        let sc: ScheduleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(sc.timezone, "UTC");
        assert_eq!(sc.active_hours_start, 8);
        assert_eq!(sc.active_hours_end, 22);
        assert_eq!(sc.active_days.len(), 7);
        assert_eq!(sc.thread_preferred_time, "10:00");
    }

    // --- CircuitBreakerConfig ---

    #[test]
    fn circuit_breaker_config_default() {
        let cb = CircuitBreakerConfig::default();
        assert_eq!(cb.error_threshold, 5);
        assert_eq!(cb.window_seconds, 300);
        assert_eq!(cb.cooldown_seconds, 600);
    }

    #[test]
    fn circuit_breaker_config_serde_roundtrip() {
        let cb = CircuitBreakerConfig {
            error_threshold: 10,
            window_seconds: 600,
            cooldown_seconds: 1200,
        };
        let json = serde_json::to_string(&cb).unwrap();
        let back: CircuitBreakerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.error_threshold, 10);
        assert_eq!(back.window_seconds, 600);
        assert_eq!(back.cooldown_seconds, 1200);
    }

    #[test]
    fn circuit_breaker_config_deserialize_with_defaults() {
        let json = r#"{}"#;
        let cb: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cb.error_threshold, 5);
        assert_eq!(cb.window_seconds, 300);
        assert_eq!(cb.cooldown_seconds, 600);
    }

    // --- McpPolicyConfig ---

    #[test]
    fn mcp_policy_config_deserialize_v1_minimal() {
        let json = r#"{}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert!(pc.enforce_for_mutations);
        assert!(!pc.require_approval_for.is_empty());
        assert!(pc.blocked_tools.is_empty());
        assert!(!pc.dry_run_mutations);
        assert_eq!(pc.max_mutations_per_hour, 20);
        assert!(pc.template.is_none());
        assert!(pc.rules.is_empty());
        assert!(pc.rate_limits.is_empty());
    }

    #[test]
    fn mcp_policy_config_serde_roundtrip() {
        let pc = McpPolicyConfig {
            enforce_for_mutations: false,
            require_approval_for: vec!["post_tweet".into()],
            blocked_tools: vec!["delete_tweet".into()],
            dry_run_mutations: true,
            max_mutations_per_hour: 50,
            template: Some(crate::mcp_policy::types::PolicyTemplateName::SafeDefault),
            rules: vec![],
            rate_limits: vec![],
        };
        let json = serde_json::to_string(&pc).unwrap();
        let back: McpPolicyConfig = serde_json::from_str(&json).unwrap();
        assert!(!back.enforce_for_mutations);
        assert_eq!(back.require_approval_for, vec!["post_tweet"]);
        assert_eq!(back.blocked_tools, vec!["delete_tweet"]);
        assert!(back.dry_run_mutations);
        assert_eq!(back.max_mutations_per_hour, 50);
        assert!(back.template.is_some());
    }

    #[test]
    fn mcp_policy_config_default_approval_list() {
        let json = r#"{}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert!(pc.require_approval_for.contains(&"post_tweet".to_string()));
        assert!(pc
            .require_approval_for
            .contains(&"reply_to_tweet".to_string()));
        assert!(pc.require_approval_for.contains(&"follow_user".to_string()));
        assert!(pc.require_approval_for.contains(&"like_tweet".to_string()));
    }

    // =========================================================================
    // Additional edge case tests for coverage push
    // =========================================================================

    // --- ScheduleConfig edge cases ---

    #[test]
    fn schedule_config_with_overrides_serde() {
        let sc = ScheduleConfig {
            preferred_times: vec!["09:00".into(), "15:00".into()],
            preferred_times_override: HashMap::from([
                ("Mon".into(), vec!["08:00".into(), "12:00".into()]),
                ("Fri".into(), vec![]),
            ]),
            ..Default::default()
        };
        let json = serde_json::to_string(&sc).unwrap();
        let back: ScheduleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.preferred_times.len(), 2);
        assert_eq!(back.preferred_times_override.len(), 2);
        assert!(back.preferred_times_override.contains_key("Mon"));
        assert_eq!(back.preferred_times_override["Mon"].len(), 2);
        assert!(back.preferred_times_override["Fri"].is_empty());
    }

    #[test]
    fn schedule_config_thread_preferred_day_none() {
        let sc = ScheduleConfig::default();
        assert!(sc.thread_preferred_day.is_none());
    }

    #[test]
    fn schedule_config_active_days_all_week() {
        let sc = ScheduleConfig::default();
        assert_eq!(sc.active_days.len(), 7);
        assert!(sc.active_days.contains(&"Mon".to_string()));
        assert!(sc.active_days.contains(&"Sun".to_string()));
    }

    #[test]
    fn schedule_config_partial_deserialize() {
        let json = r#"{"timezone":"America/Chicago","active_hours_start":10}"#;
        let sc: ScheduleConfig = serde_json::from_str(json).unwrap();
        assert_eq!(sc.timezone, "America/Chicago");
        assert_eq!(sc.active_hours_start, 10);
        assert_eq!(sc.active_hours_end, 22); // default
        assert_eq!(sc.active_days.len(), 7); // default
    }

    #[test]
    fn schedule_config_empty_preferred_times() {
        let sc = ScheduleConfig::default();
        assert!(sc.preferred_times.is_empty());
        assert!(sc.preferred_times_override.is_empty());
    }

    // --- CircuitBreakerConfig edge cases ---

    #[test]
    fn circuit_breaker_config_custom_values() {
        let cb = CircuitBreakerConfig {
            error_threshold: 1,
            window_seconds: 60,
            cooldown_seconds: 30,
        };
        assert_eq!(cb.error_threshold, 1);
        assert_eq!(cb.window_seconds, 60);
        assert_eq!(cb.cooldown_seconds, 30);
    }

    #[test]
    fn circuit_breaker_config_partial_deserialize() {
        let json = r#"{"error_threshold":20}"#;
        let cb: CircuitBreakerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cb.error_threshold, 20);
        assert_eq!(cb.window_seconds, 300); // default
        assert_eq!(cb.cooldown_seconds, 600); // default
    }

    #[test]
    fn circuit_breaker_config_large_values() {
        let cb = CircuitBreakerConfig {
            error_threshold: 1000,
            window_seconds: 86400,
            cooldown_seconds: 3600,
        };
        let json = serde_json::to_string(&cb).unwrap();
        let back: CircuitBreakerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.error_threshold, 1000);
        assert_eq!(back.window_seconds, 86400);
        assert_eq!(back.cooldown_seconds, 3600);
    }

    // --- McpPolicyConfig edge cases ---

    #[test]
    fn mcp_policy_config_all_tools_blocked() {
        let pc = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: vec![],
            blocked_tools: vec![
                "post_tweet".into(),
                "reply_to_tweet".into(),
                "follow_user".into(),
                "like_tweet".into(),
            ],
            dry_run_mutations: false,
            max_mutations_per_hour: 0,
            template: None,
            rules: vec![],
            rate_limits: vec![],
        };
        let json = serde_json::to_string(&pc).unwrap();
        let back: McpPolicyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.blocked_tools.len(), 4);
        assert!(back.require_approval_for.is_empty());
        assert_eq!(back.max_mutations_per_hour, 0);
    }

    #[test]
    fn mcp_policy_config_dry_run_mode() {
        let json = r#"{"dry_run_mutations":true}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert!(pc.dry_run_mutations);
        assert!(pc.enforce_for_mutations); // default true
    }

    #[test]
    fn mcp_policy_config_enforcement_disabled() {
        let json = r#"{"enforce_for_mutations":false}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert!(!pc.enforce_for_mutations);
    }

    #[test]
    fn mcp_policy_config_with_rules_and_rate_limits() {
        let pc = McpPolicyConfig {
            enforce_for_mutations: true,
            require_approval_for: vec![],
            blocked_tools: vec![],
            dry_run_mutations: false,
            max_mutations_per_hour: 100,
            template: None,
            rules: vec![crate::mcp_policy::types::PolicyRule {
                id: "test_rule".into(),
                priority: 10,
                label: "Test rule".into(),
                enabled: true,
                conditions: crate::mcp_policy::types::RuleConditions {
                    tools: vec!["post_tweet".into()],
                    ..Default::default()
                },
                action: crate::mcp_policy::types::PolicyAction::RequireApproval {
                    reason: "needs review".into(),
                },
            }],
            rate_limits: vec![crate::mcp_policy::types::PolicyRateLimit {
                key: "test:hourly".into(),
                dimension: crate::mcp_policy::types::RateLimitDimension::Global,
                match_value: String::new(),
                max_count: 10,
                period_seconds: 3600,
            }],
        };
        let json = serde_json::to_string(&pc).unwrap();
        let back: McpPolicyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rules.len(), 1);
        assert_eq!(back.rules[0].id, "test_rule");
        assert_eq!(back.rate_limits.len(), 1);
        assert_eq!(back.rate_limits[0].key, "test:hourly");
    }

    #[test]
    fn mcp_policy_config_custom_max_mutations() {
        let json = r#"{"max_mutations_per_hour":500}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(pc.max_mutations_per_hour, 500);
    }

    #[test]
    fn mcp_policy_config_with_template() {
        let json = r#"{"template":"safe_default"}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        assert!(pc.template.is_some());
    }

    #[test]
    fn schedule_config_debug_format() {
        let sc = ScheduleConfig::default();
        let debug = format!("{sc:?}");
        assert!(debug.contains("ScheduleConfig"));
        assert!(debug.contains("UTC"));
    }

    #[test]
    fn circuit_breaker_config_debug_format() {
        let cb = CircuitBreakerConfig::default();
        let debug = format!("{cb:?}");
        assert!(debug.contains("CircuitBreakerConfig"));
        assert!(debug.contains("5"));
    }

    #[test]
    fn mcp_policy_config_debug_format() {
        let json = r#"{}"#;
        let pc: McpPolicyConfig = serde_json::from_str(json).unwrap();
        let debug = format!("{pc:?}");
        assert!(debug.contains("McpPolicyConfig"));
        assert!(debug.contains("enforce_for_mutations"));
    }
}
