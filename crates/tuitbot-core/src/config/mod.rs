//! Configuration management for Tuitbot.
//!
//! Supports three-layer configuration loading:
//! 1. Built-in defaults
//! 2. TOML config file (`~/.tuitbot/config.toml`)
//! 3. Environment variable overrides (`TUITBOT_` prefix)
//!
//! CLI flag overrides are applied by the binary crate after loading.

mod defaults;
mod enrichment;
mod env_overrides;
mod types;
mod types_policy;
mod validation;

#[cfg(test)]
mod tests;

pub use enrichment::{EnrichmentStage, ProfileCompleteness};
pub use types::{
    AuthConfig, BusinessProfile, IntervalsConfig, LimitsConfig, LlmConfig, LoggingConfig,
    ScoringConfig, ServerConfig, StorageConfig, TargetsConfig, XApiConfig,
};
pub use types_policy::{CircuitBreakerConfig, McpPolicyConfig, ScheduleConfig};

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

fn default_approval_mode() -> bool {
    true
}

fn default_max_batch_approve() -> usize {
    25
}

/// Operating mode controlling how autonomous Tuitbot is.
///
/// - **Autopilot**: Full autonomous operation — discovers, generates, and posts content.
/// - **Composer**: User-controlled posting with on-demand AI intelligence.
///   In composer mode, `approval_mode` is implicitly `true` and autonomous
///   posting loops (content, threads, discovery replies) are disabled.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    /// Full autonomous operation.
    #[default]
    Autopilot,
    /// User-controlled posting with on-demand AI assist.
    Composer,
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingMode::Autopilot => write!(f, "autopilot"),
            OperatingMode::Composer => write!(f, "composer"),
        }
    }
}

/// Top-level configuration for the Tuitbot agent.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    /// Operating mode: "autopilot" (default) or "composer".
    #[serde(default)]
    pub mode: OperatingMode,

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

    /// Maximum items that can be batch-approved at once.
    #[serde(default = "default_max_batch_approve")]
    pub max_batch_approve: usize,

    /// Server binding configuration for LAN access.
    #[serde(default)]
    pub server: ServerConfig,

    /// Data storage configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Logging and observability settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Active hours schedule for posting.
    #[serde(default)]
    pub schedule: ScheduleConfig,

    /// MCP mutation policy enforcement.
    #[serde(default)]
    pub mcp_policy: McpPolicyConfig,

    /// Circuit breaker for X API rate-limit protection.
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
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

    /// Returns `true` if approval mode is effectively enabled.
    ///
    /// In composer mode, approval mode is always implicitly enabled so
    /// the user controls all posting.
    pub fn effective_approval_mode(&self) -> bool {
        self.approval_mode || self.mode == OperatingMode::Composer
    }

    /// Returns `true` if the agent is in composer mode.
    pub fn is_composer_mode(&self) -> bool {
        self.mode == OperatingMode::Composer
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
