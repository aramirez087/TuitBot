//! Environment variable overrides for configuration.

use super::{Config, OperatingMode};
use crate::error::ConfigError;
use std::env;

impl Config {
    /// Apply environment variable overrides to the configuration.
    ///
    /// Environment variables use the `TUITBOT_` prefix with double underscores
    /// separating nested keys (e.g., `TUITBOT_LLM__API_KEY`).
    pub(super) fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Operating mode
        if let Ok(val) = env::var("TUITBOT_MODE") {
            match val.to_lowercase().as_str() {
                "autopilot" => self.mode = OperatingMode::Autopilot,
                "composer" => self.mode = OperatingMode::Composer,
                other => {
                    return Err(ConfigError::InvalidValue {
                        field: "mode".to_string(),
                        message: format!(
                            "invalid mode '{other}', expected 'autopilot' or 'composer'"
                        ),
                    });
                }
            }
        }

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

        // MCP Policy
        if let Ok(val) = env::var("TUITBOT_MCP_POLICY__ENFORCE_FOR_MUTATIONS") {
            self.mcp_policy.enforce_for_mutations =
                parse_env_bool("TUITBOT_MCP_POLICY__ENFORCE_FOR_MUTATIONS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_MCP_POLICY__REQUIRE_APPROVAL_FOR") {
            self.mcp_policy.require_approval_for = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_MCP_POLICY__BLOCKED_TOOLS") {
            self.mcp_policy.blocked_tools = split_csv(&val);
        }
        if let Ok(val) = env::var("TUITBOT_MCP_POLICY__DRY_RUN_MUTATIONS") {
            self.mcp_policy.dry_run_mutations =
                parse_env_bool("TUITBOT_MCP_POLICY__DRY_RUN_MUTATIONS", &val)?;
        }
        if let Ok(val) = env::var("TUITBOT_MCP_POLICY__MAX_MUTATIONS_PER_HOUR") {
            self.mcp_policy.max_mutations_per_hour =
                parse_env_u32("TUITBOT_MCP_POLICY__MAX_MUTATIONS_PER_HOUR", &val)?;
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

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Split a comma-separated string into trimmed, non-empty values.
pub(super) fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Parse an environment variable value as `u16`.
pub(super) fn parse_env_u16(var_name: &str, val: &str) -> Result<u16, ConfigError> {
    val.parse::<u16>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u16"),
    })
}

/// Parse an environment variable value as `u32`.
pub(super) fn parse_env_u32(var_name: &str, val: &str) -> Result<u32, ConfigError> {
    val.parse::<u32>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u32"),
    })
}

/// Parse an environment variable value as `f32`.
pub(super) fn parse_env_f32(var_name: &str, val: &str) -> Result<f32, ConfigError> {
    val.parse::<f32>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid f32"),
    })
}

/// Parse an environment variable value as `u64`.
pub(super) fn parse_env_u64(var_name: &str, val: &str) -> Result<u64, ConfigError> {
    val.parse::<u64>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u64"),
    })
}

/// Parse an environment variable value as `u8`.
pub(super) fn parse_env_u8(var_name: &str, val: &str) -> Result<u8, ConfigError> {
    val.parse::<u8>().map_err(|_| ConfigError::InvalidValue {
        field: var_name.to_string(),
        message: format!("'{val}' is not a valid u8"),
    })
}

/// Parse an environment variable value as a boolean.
///
/// Accepts: `true`, `false`, `1`, `0`, `yes`, `no` (case-insensitive).
pub(super) fn parse_env_bool(var_name: &str, val: &str) -> Result<bool, ConfigError> {
    match val.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(ConfigError::InvalidValue {
            field: var_name.to_string(),
            message: format!("'{val}' is not a valid boolean (use true/false/1/0/yes/no)"),
        }),
    }
}
