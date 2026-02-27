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
