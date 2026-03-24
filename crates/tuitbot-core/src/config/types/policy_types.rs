//! Rate limit, interval, target, and content source configuration types.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

/// Safety limits for API actions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    /// Maximum replies per day.
    #[serde(default = "default_max_replies_per_day")]
    pub max_replies_per_day: u32,

    /// Maximum original tweets per day.
    #[serde(default = "default_max_tweets_per_day")]
    pub max_tweets_per_day: u32,

    /// Maximum threads per week.
    #[serde(default = "default_max_threads_per_week")]
    pub max_threads_per_week: u32,

    /// Minimum delay between actions in seconds.
    #[serde(default = "default_min_action_delay_seconds")]
    pub min_action_delay_seconds: u64,

    /// Maximum delay between actions in seconds.
    #[serde(default = "default_max_action_delay_seconds")]
    pub max_action_delay_seconds: u64,

    /// Maximum replies to the same author per day.
    #[serde(default = "default_max_replies_per_author_per_day")]
    pub max_replies_per_author_per_day: u32,

    /// Phrases that should never appear in generated replies.
    #[serde(default = "default_banned_phrases")]
    pub banned_phrases: Vec<String>,

    /// Fraction of replies that may mention the product (0.0 - 1.0).
    #[serde(default = "default_product_mention_ratio")]
    pub product_mention_ratio: f32,
}

// ---------------------------------------------------------------------------
// Intervals
// ---------------------------------------------------------------------------

/// Automation interval settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntervalsConfig {
    /// Seconds between mention checks.
    #[serde(default = "default_mentions_check_seconds")]
    pub mentions_check_seconds: u64,

    /// Seconds between discovery searches.
    #[serde(default = "default_discovery_search_seconds")]
    pub discovery_search_seconds: u64,

    /// Seconds for content post window.
    #[serde(default = "default_content_post_window_seconds")]
    pub content_post_window_seconds: u64,

    /// Seconds between thread posts.
    #[serde(default = "default_thread_interval_seconds")]
    pub thread_interval_seconds: u64,
}

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

/// Target account monitoring configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetsConfig {
    /// Target account usernames to monitor (without @).
    #[serde(default)]
    pub accounts: Vec<String>,

    /// Maximum target account replies per day (separate from general limit).
    #[serde(default = "default_max_target_replies_per_day")]
    pub max_target_replies_per_day: u32,
}

// ---------------------------------------------------------------------------
// Content Sources
// ---------------------------------------------------------------------------

/// Content source configuration for the Watchtower.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ContentSourcesConfig {
    /// Configured content sources.
    #[serde(default)]
    pub sources: Vec<ContentSourceEntry>,
}

/// A single content source entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentSourceEntry {
    /// Source type: `"local_fs"` or `"google_drive"`.
    #[serde(default = "default_source_type")]
    pub source_type: String,

    /// Filesystem path (for local_fs sources). Supports ~ expansion.
    #[serde(default)]
    pub path: Option<String>,

    /// Google Drive folder ID (for google_drive sources).
    #[serde(default)]
    pub folder_id: Option<String>,

    /// Path to a Google service-account JSON key file (for google_drive sources).
    /// Legacy field -- new installs use `connection_id` with OAuth 2.0 instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account_key: Option<String>,

    /// Reference to a row in the `connections` table for remote sources.
    /// When set, the Watchtower uses the linked account's credentials
    /// instead of `service_account_key`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<i64>,

    /// Whether to watch for changes in real-time.
    ///
    /// **Deprecated:** Use `enabled` and `change_detection` instead.
    /// Kept for backward compatibility — when `enabled` is `None`, the
    /// value of `watch` is used as the fallback.
    #[serde(default = "default_watch")]
    pub watch: bool,

    /// Whether this source participates in ingestion at all.
    ///
    /// When `None`, falls back to `watch` for backward compatibility.
    /// When `Some(false)`, the source is completely skipped.
    #[serde(default)]
    pub enabled: Option<bool>,

    /// How changes are detected for this source.
    ///
    /// - `"auto"` (default) — local_fs: notify watcher + fallback poll;
    ///   google_drive: interval poll.
    /// - `"poll"` — poll only (useful when notify is unreliable, e.g. NFS).
    /// - `"none"` — initial scan only, no ongoing monitoring.
    #[serde(default = "default_change_detection")]
    pub change_detection: String,

    /// File patterns to include.
    #[serde(default = "default_file_patterns")]
    pub file_patterns: Vec<String>,

    /// Whether to write metadata back to source files.
    #[serde(default = "default_loop_back")]
    pub loop_back_enabled: bool,

    /// Whether to sync analytics data (impressions, engagement, performance
    /// score) back into source file frontmatter on a periodic schedule.
    ///
    /// Only supported for `local_fs` sources.
    /// Default: false.
    #[serde(default)]
    pub analytics_sync_enabled: bool,

    /// Polling interval in seconds for remote sources (default: 300 = 5 min).
    #[serde(default)]
    pub poll_interval_seconds: Option<u64>,
}

/// Valid values for `ContentSourceEntry::change_detection`.
pub const CHANGE_DETECTION_AUTO: &str = "auto";
pub const CHANGE_DETECTION_POLL: &str = "poll";
pub const CHANGE_DETECTION_NONE: &str = "none";

/// Minimum allowed poll interval in seconds.
pub const MIN_POLL_INTERVAL_SECONDS: u64 = 30;

impl ContentSourceEntry {
    /// Whether this source should participate in ingestion.
    ///
    /// Prefers `enabled` when explicitly set; otherwise falls back to
    /// the legacy `watch` field for backward compatibility.
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(self.watch)
    }

    /// The effective change detection mode for this source.
    ///
    /// Returns `"none"` when the source is disabled (short-circuit).
    pub fn effective_change_detection(&self) -> &str {
        if !self.is_enabled() {
            return CHANGE_DETECTION_NONE;
        }
        &self.change_detection
    }

    /// Whether this source uses poll-only change detection.
    pub fn is_poll_only(&self) -> bool {
        self.effective_change_detection() == CHANGE_DETECTION_POLL
    }

    /// Whether this source should only do an initial scan with no ongoing monitoring.
    pub fn is_scan_only(&self) -> bool {
        self.effective_change_detection() == CHANGE_DETECTION_NONE
    }
}

// ---------------------------------------------------------------------------
// Default value functions
// ---------------------------------------------------------------------------

fn default_max_replies_per_day() -> u32 {
    5
}

fn default_max_tweets_per_day() -> u32 {
    6
}

fn default_max_threads_per_week() -> u32 {
    1
}

fn default_min_action_delay_seconds() -> u64 {
    45
}

fn default_max_action_delay_seconds() -> u64 {
    180
}

fn default_mentions_check_seconds() -> u64 {
    300
}

fn default_discovery_search_seconds() -> u64 {
    900
}

fn default_content_post_window_seconds() -> u64 {
    10800
}

fn default_thread_interval_seconds() -> u64 {
    604800
}

fn default_max_replies_per_author_per_day() -> u32 {
    1
}

fn default_banned_phrases() -> Vec<String> {
    vec![
        "check out".to_string(),
        "you should try".to_string(),
        "I recommend".to_string(),
        "link in bio".to_string(),
    ]
}

fn default_product_mention_ratio() -> f32 {
    0.2
}

fn default_max_target_replies_per_day() -> u32 {
    3
}

fn default_source_type() -> String {
    "local_fs".to_string()
}

fn default_watch() -> bool {
    true
}

fn default_change_detection() -> String {
    CHANGE_DETECTION_AUTO.to_string()
}

fn default_file_patterns() -> Vec<String> {
    vec!["*.md".to_string(), "*.txt".to_string()]
}

fn default_loop_back() -> bool {
    true
}
