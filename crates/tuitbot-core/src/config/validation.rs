//! Configuration validation logic.

use super::Config;
use crate::error::ConfigError;

impl Config {
    /// Validate the minimum configuration required for progressive activation.
    ///
    /// Only checks business profile fields and structural requirements.
    /// Skips LLM API key, X API client_id, and other advanced fields that
    /// can be configured later via Settings.
    pub fn validate_minimum(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Business profile — required for tier 1
        if self.business.product_name.is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.product_name".to_string(),
            });
        }

        if self.business.product_description.trim().is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.product_description".to_string(),
            });
        }

        if self.business.product_keywords.is_empty() && self.business.competitor_keywords.is_empty()
        {
            errors.push(ConfigError::MissingField {
                field: "business.product_keywords or business.competitor_keywords".to_string(),
            });
        }

        // Validate LLM provider value if present (but don't require it)
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
        }

        // Validate provider_backend value if present
        let backend = self.x_api.provider_backend.as_str();
        if !backend.is_empty() && backend != "x_api" && backend != "scraper" {
            errors.push(ConfigError::InvalidValue {
                field: "x_api.provider_backend".to_string(),
                message: format!(
                    "must be 'x_api' or 'scraper', got '{}'",
                    self.x_api.provider_backend
                ),
            });
        }

        // Structural: db_path
        let db_path_trimmed = self.storage.db_path.trim();
        if db_path_trimmed.is_empty() {
            errors.push(ConfigError::InvalidValue {
                field: "storage.db_path".to_string(),
                message: "must not be empty or whitespace-only".to_string(),
            });
        } else {
            let expanded = crate::startup::expand_tilde(db_path_trimmed);
            if expanded.is_dir() {
                errors.push(ConfigError::InvalidValue {
                    field: "storage.db_path".to_string(),
                    message: format!("'{}' is a directory, must point to a file", db_path_trimmed),
                });
            }
        }

        // Validate content sources against deployment capabilities (if any)
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

        if self.business.product_description.trim().is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.product_description".to_string(),
            });
        }

        if self.business.industry_topics.is_empty() {
            errors.push(ConfigError::MissingField {
                field: "business.industry_topics".to_string(),
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

        // Validate provider_backend value
        let backend = self.x_api.provider_backend.as_str();
        if !backend.is_empty() && backend != "x_api" && backend != "scraper" {
            errors.push(ConfigError::InvalidValue {
                field: "x_api.provider_backend".to_string(),
                message: format!(
                    "must be 'x_api' or 'scraper', got '{}'",
                    self.x_api.provider_backend
                ),
            });
        }

        // Reject scraper mode in cloud deployment
        if self.deployment_mode == super::DeploymentMode::Cloud
            && self.x_api.provider_backend == "scraper"
        {
            errors.push(ConfigError::InvalidValue {
                field: "x_api.provider_backend".to_string(),
                message: "Local No-Key Mode is not available in cloud deployment. \
                          Use the Official X API (provider_backend = \"x_api\")."
                    .to_string(),
            });
        }

        // Require client_id when using official X API backend
        let is_x_api_backend = backend.is_empty() || backend == "x_api";
        if is_x_api_backend && self.x_api.client_id.trim().is_empty() {
            errors.push(ConfigError::MissingField {
                field: "x_api.client_id".to_string(),
            });
        }

        // Validate storage.db_path is not empty/whitespace and not a directory
        let db_path_trimmed = self.storage.db_path.trim();
        if db_path_trimmed.is_empty() {
            errors.push(ConfigError::InvalidValue {
                field: "storage.db_path".to_string(),
                message: "must not be empty or whitespace-only".to_string(),
            });
        } else {
            let expanded = crate::startup::expand_tilde(db_path_trimmed);
            if expanded.is_dir() {
                errors.push(ConfigError::InvalidValue {
                    field: "storage.db_path".to_string(),
                    message: format!("'{}' is a directory, must point to a file", db_path_trimmed),
                });
            }
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

            // Validate change_detection value.
            let valid_cd = [
                super::types::CHANGE_DETECTION_AUTO,
                super::types::CHANGE_DETECTION_POLL,
                super::types::CHANGE_DETECTION_NONE,
            ];
            if !valid_cd.contains(&source.change_detection.as_str()) {
                errors.push(ConfigError::InvalidValue {
                    field: format!("content_sources.sources[{}].change_detection", i),
                    message: format!(
                        "must be one of: auto, poll, none — got '{}'",
                        source.change_detection
                    ),
                });
            }

            // Validate poll_interval_seconds minimum.
            if let Some(interval) = source.poll_interval_seconds {
                if interval < super::types::MIN_POLL_INTERVAL_SECONDS {
                    errors.push(ConfigError::InvalidValue {
                        field: format!("content_sources.sources[{}].poll_interval_seconds", i),
                        message: format!(
                            "must be at least {} seconds, got {}",
                            super::types::MIN_POLL_INTERVAL_SECONDS,
                            interval
                        ),
                    });
                }
            }

            // Validate enabled sources have required fields.
            if source.is_enabled() {
                if source.source_type == "local_fs"
                    && source.path.as_ref().map_or(true, |p| p.is_empty())
                {
                    errors.push(ConfigError::MissingField {
                        field: format!(
                            "content_sources.sources[{}].path (required for enabled local_fs source)",
                            i
                        ),
                    });
                }
                if source.source_type == "google_drive"
                    && source.folder_id.as_ref().map_or(true, |f| f.is_empty())
                {
                    errors.push(ConfigError::MissingField {
                        field: format!(
                            "content_sources.sources[{}].folder_id (required for enabled google_drive source)",
                            i
                        ),
                    });
                }
            }

            // Warn if both connection_id and service_account_key are set.
            // Not a blocking error -- session 04 handles precedence.
            if source.source_type == "google_drive"
                && source.connection_id.is_some()
                && source.service_account_key.is_some()
            {
                tracing::warn!(
                    source_index = i,
                    "content_sources.sources[{}] has both connection_id and \
                     service_account_key; connection_id takes precedence",
                    i
                );
            }

            // Warn if a google_drive source has neither auth method configured.
            // The Watchtower will skip this source at runtime, but surface it
            // during validation so the user knows to connect via the dashboard.
            if source.source_type == "google_drive"
                && source.is_enabled()
                && source.connection_id.is_none()
                && source.service_account_key.is_none()
            {
                tracing::warn!(
                    source_index = i,
                    "content_sources.sources[{}] has no authentication configured \
                     (neither connection_id nor service_account_key); this source \
                     will be skipped at runtime -- connect via Settings > Content Sources",
                    i
                );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_valid_config() -> Config {
        let mut c = Config::default();
        c.business.product_name = "TestBot".to_string();
        c.business.product_description = "A test product for unit testing".to_string();
        c.business.product_keywords = vec!["test".to_string()];
        c
    }

    // ── validate_minimum ──────────────────────────────────────────────────

    #[test]
    fn validate_minimum_default_config_fails() {
        let c = Config::default();
        assert!(c.validate_minimum().is_err());
    }

    #[test]
    fn validate_minimum_populated_config_passes() {
        let c = minimal_valid_config();
        assert!(c.validate_minimum().is_ok(), "{:?}", c.validate_minimum());
    }

    #[test]
    fn validate_minimum_missing_product_name_fails() {
        let mut c = minimal_valid_config();
        c.business.product_name = String::new();
        let errs = c.validate_minimum().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| format!("{e:?}").contains("product_name")));
    }

    #[test]
    fn validate_minimum_missing_description_fails() {
        let mut c = minimal_valid_config();
        c.business.product_description = "   ".to_string(); // whitespace only
        let errs = c.validate_minimum().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| format!("{e:?}").contains("product_description")));
    }

    #[test]
    fn validate_minimum_missing_both_keyword_fields_fails() {
        let mut c = minimal_valid_config();
        c.business.product_keywords = vec![];
        c.business.competitor_keywords = vec![];
        let errs = c.validate_minimum().unwrap_err();
        assert!(errs.iter().any(|e| format!("{e:?}").contains("keywords")));
    }

    #[test]
    fn validate_minimum_competitor_keywords_satisfies_keyword_requirement() {
        let mut c = minimal_valid_config();
        c.business.product_keywords = vec![];
        c.business.competitor_keywords = vec!["competitor".to_string()];
        assert!(c.validate_minimum().is_ok(), "{:?}", c.validate_minimum());
    }

    #[test]
    fn validate_minimum_invalid_llm_provider_fails() {
        let mut c = minimal_valid_config();
        c.llm.provider = "invalid_provider".to_string();
        let errs = c.validate_minimum().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| format!("{e:?}").contains("llm.provider")));
    }

    #[test]
    fn validate_minimum_valid_llm_providers_pass() {
        for provider in &["openai", "anthropic", "ollama"] {
            let mut c = minimal_valid_config();
            c.llm.provider = provider.to_string();
            assert!(
                c.validate_minimum().is_ok(),
                "provider {provider} should pass"
            );
        }
    }

    #[test]
    fn validate_minimum_invalid_provider_backend_fails() {
        let mut c = minimal_valid_config();
        c.x_api.provider_backend = "invalid_backend".to_string();
        let errs = c.validate_minimum().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| format!("{e:?}").contains("provider_backend")));
    }

    #[test]
    fn validate_minimum_valid_provider_backends_pass() {
        for backend in &["x_api", "scraper"] {
            let mut c = minimal_valid_config();
            c.x_api.provider_backend = backend.to_string();
            assert!(
                c.validate_minimum().is_ok(),
                "backend {backend} should pass"
            );
        }
    }

    // ── validate (full) ───────────────────────────────────────────────────

    #[test]
    fn validate_default_config_fails() {
        let c = Config::default();
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_collects_multiple_errors() {
        let c = Config::default();
        let errs = c.validate().unwrap_err();
        assert!(errs.len() >= 2, "expected ≥2 errors, got {}", errs.len());
    }

    // ── is_valid_hhmm ────────────────────────────────────────────────────

    #[test]
    fn is_valid_hhmm_valid_times() {
        assert!(is_valid_hhmm("00:00"));
        assert!(is_valid_hhmm("09:30"));
        assert!(is_valid_hhmm("23:59"));
        assert!(is_valid_hhmm("12:00"));
    }

    #[test]
    fn is_valid_hhmm_invalid_times() {
        assert!(!is_valid_hhmm("24:00")); // hour out of range
        assert!(!is_valid_hhmm("12:60")); // minute out of range
        assert!(!is_valid_hhmm("noon")); // non-numeric
        assert!(!is_valid_hhmm("")); // empty
        assert!(!is_valid_hhmm("12:30:00")); // too many parts
        assert!(!is_valid_hhmm("1230")); // no colon
    }
}
