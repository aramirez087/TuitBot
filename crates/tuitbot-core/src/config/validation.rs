//! Configuration validation logic.

use super::Config;
use crate::error::ConfigError;

impl Config {
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

        // Validate MCP policy: tools can't be in both blocked_tools and require_approval_for
        for tool in &self.mcp_policy.blocked_tools {
            if self.mcp_policy.require_approval_for.contains(tool) {
                errors.push(ConfigError::InvalidValue {
                    field: "mcp_policy.blocked_tools".to_string(),
                    message: format!(
                        "tool '{tool}' cannot be in both blocked_tools and require_approval_for"
                    ),
                });
                break;
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

        // Validate content sources against deployment capabilities
        for (i, source) in self.content_sources.sources.iter().enumerate() {
            if !self.deployment_mode.allows_source_type(&source.source_type) {
                errors.push(ConfigError::InvalidValue {
                    field: format!("content_sources.sources[{}].source_type", i),
                    message: format!(
                        "source type '{}' is not available in {} deployment mode",
                        source.source_type, self.deployment_mode
                    ),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
