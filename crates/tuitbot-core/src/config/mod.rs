//! Configuration management for Tuitbot.
//!
//! Supports three-layer configuration loading:
//! 1. Built-in defaults
//! 2. TOML config file (`~/.tuitbot/config.toml`)
//! 3. Environment variable overrides (`TUITBOT_` prefix)
//!
//! CLI flag overrides are applied by the binary crate after loading.

mod defaults;

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn default_approval_mode() -> bool {
    true
}

/// Top-level configuration for the Tuitbot agent.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    /// X API credentials.
    #[serde(default)]
    pub x_api: XApiConfig,

    /// Authentication settings.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Business profile for content targeting.
    #[serde(default)]
    pub business: BusinessProfile,

    /// Scoring engine weights and threshold.
    #[serde(default)]
    pub scoring: ScoringConfig,

    /// Safety limits for API actions.
    #[serde(default)]
    pub limits: LimitsConfig,

    /// Automation interval settings.
    #[serde(default)]
    pub intervals: IntervalsConfig,

    /// LLM provider configuration.
    #[serde(default)]
    pub llm: LlmConfig,

    /// Target account monitoring configuration.
    #[serde(default)]
    pub targets: TargetsConfig,

    /// Enable approval mode: queue posts for human review instead of posting.
    #[serde(default = "default_approval_mode")]
    pub approval_mode: bool,

    /// Data storage configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Logging and observability settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Active hours schedule for posting.
    #[serde(default)]
    pub schedule: ScheduleConfig,
}

/// X API credentials.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct XApiConfig {
    /// OAuth 2.0 client ID.
    #[serde(default)]
    pub client_id: String,

    /// OAuth 2.0 client secret (optional for public clients).
    #[serde(default)]
    pub client_secret: Option<String>,
}

/// Authentication mode and callback settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Auth mode: "manual" or "local_callback".
    #[serde(default = "default_auth_mode")]
    pub mode: String,

    /// Host for local callback server.
    #[serde(default = "default_callback_host")]
    pub callback_host: String,

    /// Port for local callback server.
    #[serde(default = "default_callback_port")]
    pub callback_port: u16,
}

/// Business profile for content targeting and keyword matching.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BusinessProfile {
    /// Name of the user's product.
    #[serde(default)]
    pub product_name: String,

    /// One-line description of the product.
    #[serde(default)]
    pub product_description: String,

    /// URL to the product website.
    #[serde(default)]
    pub product_url: Option<String>,

    /// Description of the target audience.
    #[serde(default)]
    pub target_audience: String,

    /// Keywords for tweet discovery.
    #[serde(default)]
    pub product_keywords: Vec<String>,

    /// Competitor-related keywords for discovery.
    #[serde(default)]
    pub competitor_keywords: Vec<String>,

    /// Topics for content generation.
    #[serde(default)]
    pub industry_topics: Vec<String>,

    /// Brand voice / personality description for all generated content.
    #[serde(default)]
    pub brand_voice: Option<String>,

    /// Style guidelines specific to replies.
    #[serde(default)]
    pub reply_style: Option<String>,

    /// Style guidelines specific to original tweets and threads.
    #[serde(default)]
    pub content_style: Option<String>,

    /// Opinions the persona holds (used to add variety to generated content).
    #[serde(default)]
    pub persona_opinions: Vec<String>,

    /// Experiences the persona can reference (keeps content authentic).
    #[serde(default)]
    pub persona_experiences: Vec<String>,

    /// Core content pillars (broad themes the account focuses on).
    #[serde(default)]
    pub content_pillars: Vec<String>,
}

/// Scoring engine weights and threshold.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringConfig {
    /// Minimum score (0-100) to trigger a reply.
    #[serde(default = "default_threshold")]
    pub threshold: u32,

    /// Maximum points for keyword relevance.
    #[serde(default = "default_keyword_relevance_max")]
    pub keyword_relevance_max: f32,

    /// Maximum points for author follower count.
    #[serde(default = "default_follower_count_max")]
    pub follower_count_max: f32,

    /// Maximum points for tweet recency.
    #[serde(default = "default_recency_max")]
    pub recency_max: f32,

    /// Maximum points for engagement rate.
    #[serde(default = "default_engagement_rate_max")]
    pub engagement_rate_max: f32,

    /// Maximum points for reply count signal (fewer replies = higher score).
    #[serde(default = "default_reply_count_max")]
    pub reply_count_max: f32,

    /// Maximum points for content type signal (text-only originals score highest).
    #[serde(default = "default_content_type_max")]
    pub content_type_max: f32,
}

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

fn default_max_target_replies_per_day() -> u32 {
    3
}

/// LLM provider configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LlmConfig {
    /// LLM provider name: "openai", "anthropic", or "ollama".
    #[serde(default)]
    pub provider: String,

    /// API key for the LLM provider (not needed for ollama).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Provider-specific model name.
    #[serde(default)]
    pub model: String,

    /// Override URL for custom endpoints.
    #[serde(default)]
    pub base_url: Option<String>,
}

/// Data storage configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Path to the SQLite database file.
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// Number of days to retain data.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

/// Logging and observability settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Seconds between periodic status summaries (0 = disabled).
    #[serde(default)]
    pub status_interval_seconds: u64,
}

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

// --- Default value functions for serde ---

fn default_auth_mode() -> String {
    "manual".to_string()
}
fn default_callback_host() -> String {
    "127.0.0.1".to_string()
}
fn default_callback_port() -> u16 {
    8080
}
fn default_threshold() -> u32 {
    60
}
fn default_keyword_relevance_max() -> f32 {
    25.0
}
fn default_follower_count_max() -> f32 {
    15.0
}
fn default_recency_max() -> f32 {
    10.0
}
fn default_engagement_rate_max() -> f32 {
    15.0
}
fn default_reply_count_max() -> f32 {
    15.0
}
fn default_content_type_max() -> f32 {
    10.0
}
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
fn default_db_path() -> String {
    "~/.tuitbot/tuitbot.db".to_string()
}
fn default_retention_days() -> u32 {
    90
}

impl Config {
    /// Load configuration from a TOML file with environment variable overrides.
    ///
    /// The loading sequence:
    /// 1. Determine config file path (argument > `TUITBOT_CONFIG` env var > default)
    /// 2. Parse TOML file (or use defaults if default path doesn't exist)
    /// 3. Apply environment variable overrides
    pub fn load(config_path: Option<&str>) -> Result<Config, ConfigError> {
        let (path, explicit) = Self::resolve_config_path(config_path);

        let mut config = match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str::<Config>(&contents)
                .map_err(|e| ConfigError::ParseError { source: e })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if explicit {
                    return Err(ConfigError::FileNotFound {
                        path: path.display().to_string(),
                    });
                }
                Config::default()
            }
            Err(_) => {
                return Err(ConfigError::FileNotFound {
                    path: path.display().to_string(),
                });
            }
        };

        config.apply_env_overrides()?;

        Ok(config)
    }

    /// Load configuration and validate it, returning all validation errors at once.
    pub fn load_and_validate(config_path: Option<&str>) -> Result<Config, Vec<ConfigError>> {
        let config = Config::load(config_path).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration, returning all errors found (not just the first).
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Validate business profile
        if self.business.product_name.is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.product_name".to_string(),
            });
        }

        if self.business.product_keywords.is_empty() && self.business.competitor_keywords.is_empty()
        {
            errors.push(ConfigError::MissingField {
                field: "business.product_keywords or business.competitor_keywords".to_string(),
            });
        }

        // Validate LLM provider
        if !self.llm.provider.is_empty() {
            match self.llm.provider.as_str() {
                "openai" | "anthropic" | "ollama" => {}
                _ => {
                    errors.push(ConfigError::InvalidValue {
                        field: "llm.provider".to_string(),
                        message: "must be openai, anthropic, or ollama".to_string(),
                    });
                }
            }

            if matches!(self.llm.provider.as_str(), "openai" | "anthropic") {
                match &self.llm.api_key {
                    Some(key) if !key.is_empty() => {}
                    _ => {
                        errors.push(ConfigError::MissingField {
                            field: format!(
                                "llm.api_key (required for {} provider)",
                                self.llm.provider
                            ),
                        });
                    }
                }
            }
        }

        // Validate auth mode
        if !self.auth.mode.is_empty() {
            match self.auth.mode.as_str() {
                "manual" | "local_callback" => {}
                _ => {
                    errors.push(ConfigError::InvalidValue {
                        field: "auth.mode".to_string(),
                        message: "must be manual or local_callback".to_string(),
                    });
                }
            }
        }

        // Validate scoring threshold
        if self.scoring.threshold > 100 {
            errors.push(ConfigError::InvalidValue {
                field: "scoring.threshold".to_string(),
                message: "must be between 0 and 100".to_string(),
            });
        }

        // Validate limits
        if self.limits.max_replies_per_day == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_replies_per_day".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.max_tweets_per_day == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_tweets_per_day".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.max_threads_per_week == 0 {
            errors.push(ConfigError::InvalidValue {
                field: "limits.max_threads_per_week".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }

        if self.limits.min_action_delay_seconds > self.limits.max_action_delay_seconds {
            errors.push(ConfigError::InvalidValue {
                field: "limits.min_action_delay_seconds".to_string(),
                message: "must be less than or equal to max_action_delay_seconds".to_string(),
            });
        }

        // Validate schedule
        if self.schedule.active_hours_start > 23 {
            errors.push(ConfigError::InvalidValue {
                field: "schedule.active_hours_start".to_string(),
                message: "must be between 0 and 23".to_string(),
            });
        }
        if self.schedule.active_hours_end > 23 {
            errors.push(ConfigError::InvalidValue {
                field: "schedule.active_hours_end".to_string(),
                message: "must be between 0 and 23".to_string(),
            });
        }
        if !self.schedule.timezone.is_empty()
            && self.schedule.timezone.parse::<chrono_tz::Tz>().is_err()
        {
            errors.push(ConfigError::InvalidValue {
                field: "schedule.timezone".to_string(),
                message: format!(
                    "'{}' is not a valid IANA timezone name",
                    self.schedule.timezone
                ),
            });
        }
        let valid_days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        for day in &self.schedule.active_days {
            if !valid_days.contains(&day.as_str()) {
                errors.push(ConfigError::InvalidValue {
                    field: "schedule.active_days".to_string(),
                    message: format!(
                        "'{}' is not a valid day abbreviation (use Mon, Tue, Wed, Thu, Fri, Sat, Sun)",
                        day
                    ),
                });
                break;
            }
        }

        // Validate preferred_times
        for time_str in &self.schedule.preferred_times {
            if time_str != "auto" && !is_valid_hhmm(time_str) {
                errors.push(ConfigError::InvalidValue {
                    field: "schedule.preferred_times".to_string(),
                    message: format!(
                        "'{}' is not a valid time (use HH:MM 24h format or \"auto\")",
                        time_str
                    ),
                });
                break;
            }
        }

        // Validate preferred_times_override keys and values
        for (day, times) in &self.schedule.preferred_times_override {
            if !valid_days.contains(&day.as_str()) {
                errors.push(ConfigError::InvalidValue {
                    field: "schedule.preferred_times_override".to_string(),
                    message: format!(
                        "'{}' is not a valid day abbreviation (use Mon, Tue, Wed, Thu, Fri, Sat, Sun)",
                        day
                    ),
                });
                break;
            }
            for time_str in times {
                if !is_valid_hhmm(time_str) {
                    errors.push(ConfigError::InvalidValue {
                        field: "schedule.preferred_times_override".to_string(),
                        message: format!(
                            "'{}' is not a valid time for {} (use HH:MM 24h format)",
                            time_str, day
                        ),
                    });
                    break;
                }
            }
        }

        // Count effective slots per day vs max_tweets_per_day
        let effective_slots = if self.schedule.preferred_times.is_empty() {
            0
        } else {
            // "auto" expands to 3 slots
            let base_count: usize = self
                .schedule
                .preferred_times
                .iter()
                .map(|t| if t == "auto" { 3 } else { 1 })
                .sum();
            // Check max across all override days too
            let max_override = self
                .schedule
                .preferred_times_override
                .values()
                .map(|v| v.len())
                .max()
                .unwrap_or(0);
            base_count.max(max_override)
        };
        if effective_slots > self.limits.max_tweets_per_day as usize {
            errors.push(ConfigError::InvalidValue {
                field: "schedule.preferred_times".to_string(),
                message: format!(
                    "preferred_times has {} slots but limits.max_tweets_per_day is {} — \
                     increase the limit or reduce the number of time slots",
                    effective_slots, self.limits.max_tweets_per_day
                ),
            });
        }

        // Validate thread_preferred_day
        if let Some(day) = &self.schedule.thread_preferred_day {
            if !valid_days.contains(&day.as_str()) {
                errors.push(ConfigError::InvalidValue {
                    field: "schedule.thread_preferred_day".to_string(),
                    message: format!(
                        "'{}' is not a valid day abbreviation (use Mon, Tue, Wed, Thu, Fri, Sat, Sun)",
                        day
                    ),
                });
            }
        }

        // Validate thread_preferred_time
        if !is_valid_hhmm(&self.schedule.thread_preferred_time) {
            errors.push(ConfigError::InvalidValue {
                field: "schedule.thread_preferred_time".to_string(),
                message: format!(
                    "'{}' is not a valid time (use HH:MM 24h format)",
                    self.schedule.thread_preferred_time
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Resolve the config file path from arguments, env vars, or default.
    ///
    /// Returns `(path, explicit)` where `explicit` is true if the path was
    /// explicitly provided (via argument or env var) rather than using the default.
    fn resolve_config_path(config_path: Option<&str>) -> (PathBuf, bool) {
        if let Some(path) = config_path {
            return (expand_tilde(path), true);
        }

        if let Ok(env_path) = env::var("TUITBOT_CONFIG") {
            return (expand_tilde(&env_path), true);
        }

        (expand_tilde("~/.tuitbot/config.toml"), false)
    }

    /// Apply environment variable overrides to the configuration.
    ///
    /// Environment variables use the `TUITBOT_` prefix with double underscores
    /// separating nested keys (e.g., `TUITBOT_LLM__API_KEY`).
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // X API
        if let Ok(val) = env::var("TUITBOT_X_API__CLIENT_ID") {
            self.x_api.client_id = val;
        }
        if let Ok(val) = env::var("TUITBOT_X_API__CLIENT_SECRET") {
            self.x_api.client_secret = Some(val);
        }

        // Auth
        if let Ok(val) = env::var("TUITBOT_AUTH__MODE") {
            self.auth.mode = val;
        }
        if let Ok(val) = env::var("TUITBOT_AUTH__CALLBACK_HOST") {
            self.auth.callback_host = val;
        }
        if let Ok(val) = env::var("TUITBOT_AUTH__CALLBACK_PORT") {
            self.auth.callback_port = parse_env_u16("TUITBOT_AUTH__CALLBACK_PORT", &val)?;
        }

        // Business
        if let Ok(val) = env::var("TUITBOT_BUSINESS__PRODUCT_NAME") {
            self.business.product_name = val;
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__PRODUCT_DESCRIPTION") {
            self.business.product_description = val;
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__PRODUCT_URL") {
            self.business.product_url = Some(val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__TARGET_AUDIENCE") {
            self.business.target_audience = val;
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__PRODUCT_KEYWORDS") {
            self.business.product_keywords = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__COMPETITOR_KEYWORDS") {
            self.business.competitor_keywords = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__INDUSTRY_TOPICS") {
            self.business.industry_topics = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__BRAND_VOICE") {
            self.business.brand_voice = Some(val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__REPLY_STYLE") {
            self.business.reply_style = Some(val);
        }
        if let Ok(val) = env::var("TUITBOT_BUSINESS__CONTENT_STYLE") {
            self.business.content_style = Some(val);
        }

        // Scoring
        if let Ok(val) = env::var("TUITBOT_SCORING__THRESHOLD") {
            self.scoring.threshold = parse_env_u32("TUITBOT_SCORING__THRESHOLD", &val)?;
        }

        if let Ok(val) = env::var("TUITBOT_SCORING__REPLY_COUNT_MAX") {
            self.scoring.reply_count_max = parse_env_f32("TUITBOT_SCORING__REPLY_COUNT_MAX", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_SCORING__CONTENT_TYPE_MAX") {
            self.scoring.content_type_max =
                parse_env_f32("TUITBOT_SCORING__CONTENT_TYPE_MAX", &val)?;
        }

        // Limits
        if let Ok(val) = env::var("TUITBOT_LIMITS__MAX_REPLIES_PER_DAY") {
            self.limits.max_replies_per_day =
                parse_env_u32("TUITBOT_LIMITS__MAX_REPLIES_PER_DAY", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__MAX_TWEETS_PER_DAY") {
            self.limits.max_tweets_per_day =
                parse_env_u32("TUITBOT_LIMITS__MAX_TWEETS_PER_DAY", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__MAX_THREADS_PER_WEEK") {
            self.limits.max_threads_per_week =
                parse_env_u32("TUITBOT_LIMITS__MAX_THREADS_PER_WEEK", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__MIN_ACTION_DELAY_SECONDS") {
            self.limits.min_action_delay_seconds =
                parse_env_u64("TUITBOT_LIMITS__MIN_ACTION_DELAY_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__MAX_ACTION_DELAY_SECONDS") {
            self.limits.max_action_delay_seconds =
                parse_env_u64("TUITBOT_LIMITS__MAX_ACTION_DELAY_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__MAX_REPLIES_PER_AUTHOR_PER_DAY") {
            self.limits.max_replies_per_author_per_day =
                parse_env_u32("TUITBOT_LIMITS__MAX_REPLIES_PER_AUTHOR_PER_DAY", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__BANNED_PHRASES") {
            self.limits.banned_phrases = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_LIMITS__PRODUCT_MENTION_RATIO") {
            self.limits.product_mention_ratio =
                parse_env_f32("TUITBOT_LIMITS__PRODUCT_MENTION_RATIO", &val)?;
        }

        // Intervals
        if let Ok(val) = env::var("TUITBOT_INTERVALS__MENTIONS_CHECK_SECONDS") {
            self.intervals.mentions_check_seconds =
                parse_env_u64("TUITBOT_INTERVALS__MENTIONS_CHECK_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_INTERVALS__DISCOVERY_SEARCH_SECONDS") {
            self.intervals.discovery_search_seconds =
                parse_env_u64("TUITBOT_INTERVALS__DISCOVERY_SEARCH_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_INTERVALS__CONTENT_POST_WINDOW_SECONDS") {
            self.intervals.content_post_window_seconds =
                parse_env_u64("TUITBOT_INTERVALS__CONTENT_POST_WINDOW_SECONDS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_INTERVALS__THREAD_INTERVAL_SECONDS") {
            self.intervals.thread_interval_seconds =
                parse_env_u64("TUITBOT_INTERVALS__THREAD_INTERVAL_SECONDS", &val)?;
        }

        // Targets
        if let Ok(val) = env::var("TUITBOT_TARGETS__ACCOUNTS") {
            self.targets.accounts = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_TARGETS__MAX_TARGET_REPLIES_PER_DAY") {
            self.targets.max_target_replies_per_day =
                parse_env_u32("TUITBOT_TARGETS__MAX_TARGET_REPLIES_PER_DAY", &val)?;
        }

        // LLM
        if let Ok(val) = env::var("TUITBOT_LLM__PROVIDER") {
            self.llm.provider = val;
        }
        if let Ok(val) = env::var("TUITBOT_LLM__API_KEY") {
            self.llm.api_key = Some(val);
        }
        if let Ok(val) = env::var("TUITBOT_LLM__MODEL") {
            self.llm.model = val;
        }
        if let Ok(val) = env::var("TUITBOT_LLM__BASE_URL") {
            self.llm.base_url = Some(val);
        }

        // Storage
        if let Ok(val) = env::var("TUITBOT_STORAGE__DB_PATH") {
            self.storage.db_path = val;
        }
        if let Ok(val) = env::var("TUITBOT_STORAGE__RETENTION_DAYS") {
            self.storage.retention_days = parse_env_u32("TUITBOT_STORAGE__RETENTION_DAYS", &val)?;
        }

        // Logging
        if let Ok(val) = env::var("TUITBOT_LOGGING__STATUS_INTERVAL_SECONDS") {
            self.logging.status_interval_seconds =
                parse_env_u64("TUITBOT_LOGGING__STATUS_INTERVAL_SECONDS", &val)?;
        }

        // Schedule
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__TIMEZONE") {
            self.schedule.timezone = val;
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__ACTIVE_HOURS_START") {
            self.schedule.active_hours_start =
                parse_env_u8("TUITBOT_SCHEDULE__ACTIVE_HOURS_START", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__ACTIVE_HOURS_END") {
            self.schedule.active_hours_end =
                parse_env_u8("TUITBOT_SCHEDULE__ACTIVE_HOURS_END", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__ACTIVE_DAYS") {
            self.schedule.active_days = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__PREFERRED_TIMES") {
            self.schedule.preferred_times = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__THREAD_PREFERRED_DAY") {
            let val = val.trim().to_string();
            if val.is_empty() || val == "none" {
                self.schedule.thread_preferred_day = None;
            } else {
                self.schedule.thread_preferred_day = Some(val);
            }
        }
        if let Ok(val) = env::var("TUITBOT_SCHEDULE__THREAD_PREFERRED_TIME") {
            self.schedule.thread_preferred_time = val;
        }

        // Approval mode
        let explicit_approval = if let Ok(val) = env::var("TUITBOT_APPROVAL_MODE") {
            self.approval_mode = parse_env_bool("TUITBOT_APPROVAL_MODE", &val)?;
            true
        } else {
            false
        };

        // OpenClaw auto-detection: enable approval mode when running inside
        // OpenClaw unless the user explicitly set TUITBOT_APPROVAL_MODE.
        if !explicit_approval && env::vars().any(|(k, _)| k.starts_with("OPENCLAW_")) {
            self.approval_mode = true;
        }

        Ok(())
    }
}

/// Check if a string is a valid HH:MM time (24h format).
fn is_valid_hhmm(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let Ok(hour) = parts[0].parse::<u8>() else {
        return false;
    };
    let Ok(minute) = parts[1].parse::<u8>() else {
        return false;
    };
    hour <= 23 && minute <= 59
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Split a comma-separated string into trimmed, non-empty values.
fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Parse an environment variable value as `u16`.
fn parse_env_u16(var_name: &str, val: &str) -> Result<u16, ConfigError> {
    val.parse::<u16>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u16"),
    })
}

/// Parse an environment variable value as `u32`.
fn parse_env_u32(var_name: &str, val: &str) -> Result<u32, ConfigError> {
    val.parse::<u32>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u32"),
    })
}

/// Parse an environment variable value as `f32`.
fn parse_env_f32(var_name: &str, val: &str) -> Result<f32, ConfigError> {
    val.parse::<f32>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid f32"),
    })
}

/// Parse an environment variable value as `u64`.
fn parse_env_u64(var_name: &str, val: &str) -> Result<u64, ConfigError> {
    val.parse::<u64>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u64"),
    })
}

/// Parse an environment variable value as `u8`.
fn parse_env_u8(var_name: &str, val: &str) -> Result<u8, ConfigError> {
    val.parse::<u8>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u8"),
    })
}

/// Parse an environment variable value as a boolean.
///
/// Accepts: `true`, `false`, `1`, `0`, `yes`, `no` (case-insensitive).
fn parse_env_bool(var_name: &str, val: &str) -> Result<bool, ConfigError> {
    match val.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(ConfigError::InvalidValue {
            field: var_name.to_string(),
            message: format!("'{val}' is not a valid boolean (use true/false/1/0/yes/no)"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn load_valid_toml() {
        let toml_str = r#"
[x_api]
client_id = "test-client-id"

[business]
product_name = "TestProduct"
product_description = "A test product"
target_audience = "developers"
product_keywords = ["rust", "cli"]
industry_topics = ["rust", "development"]

[llm]
provider = "ollama"
model = "llama2"

[scoring]
threshold = 80
"#;
        let config: Config = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(config.x_api.client_id, "test-client-id");
        assert_eq!(config.business.product_name, "TestProduct");
        assert_eq!(config.scoring.threshold, 80);
        assert_eq!(config.llm.provider, "ollama");
    }

    #[test]
    fn missing_sections_use_defaults() {
        let toml_str = r#"
[x_api]
client_id = "test"
"#;
        let config: Config = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(config.auth.mode, "manual");
        assert_eq!(config.auth.callback_port, 8080);
        assert_eq!(config.scoring.threshold, 60);
        assert_eq!(config.limits.max_replies_per_day, 5);
        assert_eq!(config.intervals.mentions_check_seconds, 300);
        assert_eq!(config.storage.db_path, "~/.tuitbot/tuitbot.db");
        assert_eq!(config.storage.retention_days, 90);
        assert_eq!(config.logging.status_interval_seconds, 0);
    }

    #[test]
    fn env_var_override_string() {
        // Use a unique env var prefix to avoid test interference
        env::set_var("TUITBOT_LLM__PROVIDER", "anthropic");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.llm.provider, "anthropic");
        env::remove_var("TUITBOT_LLM__PROVIDER");
    }

    #[test]
    fn env_var_override_numeric() {
        env::set_var("TUITBOT_SCORING__THRESHOLD", "85");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(config.scoring.threshold, 85);
        env::remove_var("TUITBOT_SCORING__THRESHOLD");
    }

    #[test]
    fn env_var_override_csv() {
        env::set_var("TUITBOT_BUSINESS__PRODUCT_KEYWORDS", "rust, cli, tools");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert_eq!(
            config.business.product_keywords,
            vec!["rust", "cli", "tools"]
        );
        env::remove_var("TUITBOT_BUSINESS__PRODUCT_KEYWORDS");
    }

    #[test]
    fn env_var_invalid_numeric_returns_error() {
        // Test the parse function directly to avoid env var race conditions
        // with other tests that call apply_env_overrides()
        let result = parse_env_u32("TUITBOT_SCORING__THRESHOLD", "not_a_number");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(field, "TUITBOT_SCORING__THRESHOLD");
            }
            other => panic!("expected InvalidValue, got: {other}"),
        }
    }

    #[test]
    fn validate_missing_product_name() {
        let config = Config::default();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::MissingField { field } if field == "business.product_name")
        ));
    }

    #[test]
    fn validate_invalid_llm_provider() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "invalid_provider".to_string();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "llm.provider")
        ));
    }

    #[test]
    fn validate_threshold_over_100() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.scoring.threshold = 101;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "scoring.threshold")
        ));
    }

    #[test]
    fn validate_threshold_boundary_values() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();

        config.scoring.threshold = 0;
        assert!(config.validate().is_ok());

        config.scoring.threshold = 100;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_returns_multiple_errors() {
        let mut config = Config::default();
        // Missing product_name (default is empty)
        // Missing keywords (default is empty)
        config.llm.provider = "invalid".to_string();
        config.scoring.threshold = 101;
        config.limits.max_replies_per_day = 0;

        let errors = config.validate().unwrap_err();
        assert!(
            errors.len() >= 4,
            "expected at least 4 errors, got {}: {:?}",
            errors.len(),
            errors
        );
    }

    #[test]
    fn validate_valid_config_passes() {
        let mut config = Config::default();
        config.business.product_name = "TestProduct".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.llm.model = "llama2".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_openai_requires_api_key() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "openai".to_string();
        config.llm.api_key = None;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::MissingField { field } if field.contains("llm.api_key"))
        ));
    }

    #[test]
    fn validate_delay_ordering() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.limits.min_action_delay_seconds = 200;
        config.limits.max_action_delay_seconds = 100;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "limits.min_action_delay_seconds")));
    }

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("~/.tuitbot/config.toml");
        assert!(!expanded.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn split_csv_trims_and_filters() {
        let result = split_csv("  rust , cli ,, tools  ");
        assert_eq!(result, vec!["rust", "cli", "tools"]);
    }

    #[test]
    fn validate_preferred_times_valid() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.schedule.preferred_times = vec!["09:15".to_string(), "12:30".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_preferred_times_auto() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.schedule.preferred_times = vec!["auto".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_preferred_times_invalid_format() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.schedule.preferred_times = vec!["9:15".to_string(), "25:00".to_string()];
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times")
        ));
    }

    #[test]
    fn validate_preferred_times_exceeds_max_tweets() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.limits.max_tweets_per_day = 2;
        config.schedule.preferred_times = vec![
            "09:00".to_string(),
            "12:00".to_string(),
            "17:00".to_string(),
        ];
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, message } if field == "schedule.preferred_times" && message.contains("3 slots"))
        ));
    }

    #[test]
    fn validate_thread_preferred_day_invalid() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.schedule.thread_preferred_day = Some("Monday".to_string());
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.thread_preferred_day")
        ));
    }

    #[test]
    fn validate_thread_preferred_time_invalid() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config.schedule.thread_preferred_time = "25:00".to_string();
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.thread_preferred_time")
        ));
    }

    #[test]
    fn preferred_times_override_invalid_day() {
        let mut config = Config::default();
        config.business.product_name = "Test".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.llm.provider = "ollama".to_string();
        config
            .schedule
            .preferred_times_override
            .insert("Monday".to_string(), vec!["09:00".to_string()]);
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ConfigError::InvalidValue { field, .. } if field == "schedule.preferred_times_override")
        ));
    }

    #[test]
    fn preferred_times_toml_roundtrip() {
        let toml_str = r#"
[x_api]
client_id = "test"

[business]
product_name = "Test"
product_keywords = ["test"]

[llm]
provider = "ollama"
model = "llama2"

[schedule]
timezone = "America/New_York"
preferred_times = ["09:15", "12:30", "17:00"]
thread_preferred_day = "Tue"
thread_preferred_time = "10:00"
"#;
        let config: Config = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(
            config.schedule.preferred_times,
            vec!["09:15", "12:30", "17:00"]
        );
        assert_eq!(
            config.schedule.thread_preferred_day,
            Some("Tue".to_string())
        );
        assert_eq!(config.schedule.thread_preferred_time, "10:00");
    }

    #[test]
    fn config_file_not_found_explicit_path() {
        let result = Config::load(Some("/nonexistent/path/config.toml"));
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::FileNotFound { path } => {
                assert_eq!(path, "/nonexistent/path/config.toml");
            }
            other => panic!("expected FileNotFound, got: {other}"),
        }
    }

    #[test]
    fn parse_env_bool_values() {
        assert!(parse_env_bool("TEST", "true").unwrap());
        assert!(parse_env_bool("TEST", "True").unwrap());
        assert!(parse_env_bool("TEST", "1").unwrap());
        assert!(parse_env_bool("TEST", "yes").unwrap());
        assert!(parse_env_bool("TEST", "YES").unwrap());
        assert!(!parse_env_bool("TEST", "false").unwrap());
        assert!(!parse_env_bool("TEST", "False").unwrap());
        assert!(!parse_env_bool("TEST", "0").unwrap());
        assert!(!parse_env_bool("TEST", "no").unwrap());
        assert!(!parse_env_bool("TEST", "NO").unwrap());
        assert!(parse_env_bool("TEST", "maybe").is_err());
    }

    #[test]
    fn env_var_override_approval_mode() {
        env::set_var("TUITBOT_APPROVAL_MODE", "true");
        let mut config = Config::default();
        config.approval_mode = false;
        config.apply_env_overrides().expect("env override");
        assert!(config.approval_mode);
        env::remove_var("TUITBOT_APPROVAL_MODE");
    }

    #[test]
    fn env_var_override_approval_mode_false() {
        env::set_var("TUITBOT_APPROVAL_MODE", "false");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.approval_mode);
        env::remove_var("TUITBOT_APPROVAL_MODE");
    }

    #[test]
    fn openclaw_env_enables_approval_mode() {
        env::set_var("OPENCLAW_AGENT_ID", "test");
        let mut config = Config::default();
        config.approval_mode = false;
        config.apply_env_overrides().expect("env override");
        assert!(config.approval_mode);
        env::remove_var("OPENCLAW_AGENT_ID");
    }

    #[test]
    fn openclaw_env_respects_explicit_override() {
        env::set_var("OPENCLAW_AGENT_ID", "test");
        env::set_var("TUITBOT_APPROVAL_MODE", "false");
        let mut config = Config::default();
        config.apply_env_overrides().expect("env override");
        assert!(!config.approval_mode);
        env::remove_var("OPENCLAW_AGENT_ID");
        env::remove_var("TUITBOT_APPROVAL_MODE");
    }
}
