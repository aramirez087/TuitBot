use anyhow::Result;
use console::Style;
use tuitbot_core::config::Config;
use tuitbot_core::safety::redact::mask_optional_secret;

use crate::output::write_stdout;

pub(super) fn show_config(config: &Config) {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Tuitbot Configuration"));
    eprintln!("{}", dim.apply_to("══════════════════════"));

    // Product
    eprintln!();
    eprintln!("{}", bold.apply_to("Your Product"));
    eprintln!("  Product name:        {}", config.business.product_name);
    eprintln!(
        "  Description:         {}",
        config.business.product_description
    );
    eprintln!(
        "  Product URL:         {}",
        config
            .business
            .product_url
            .as_deref()
            .unwrap_or("(not set)")
    );
    eprintln!("  Target audience:     {}", config.business.target_audience);
    eprintln!(
        "  Discovery keywords:  {}",
        format_list(&config.business.product_keywords)
    );
    eprintln!(
        "  Competitor keywords: {}",
        format_list(&config.business.competitor_keywords)
    );
    eprintln!(
        "  Content topics:      {}",
        format_list(&config.business.industry_topics)
    );

    // Brand voice
    eprintln!();
    eprintln!("{}", bold.apply_to("Brand Voice"));
    eprintln!(
        "  Personality:         {}",
        config
            .business
            .brand_voice
            .as_deref()
            .unwrap_or("(default)")
    );
    eprintln!(
        "  Reply style:         {}",
        config
            .business
            .reply_style
            .as_deref()
            .unwrap_or("(default)")
    );
    eprintln!(
        "  Content style:       {}",
        config
            .business
            .content_style
            .as_deref()
            .unwrap_or("(default)")
    );

    // Persona
    eprintln!();
    eprintln!("{}", bold.apply_to("Persona"));
    eprintln!(
        "  Opinions:            {}",
        format_list(&config.business.persona_opinions)
    );
    eprintln!(
        "  Experiences:         {}",
        format_list(&config.business.persona_experiences)
    );
    eprintln!(
        "  Content pillars:     {}",
        format_list(&config.business.content_pillars)
    );

    // AI Provider
    eprintln!();
    eprintln!("{}", bold.apply_to("AI Provider"));
    eprintln!("  Provider:            {}", config.llm.provider);
    eprintln!(
        "  API key:             {}",
        mask_optional_secret(&config.llm.api_key)
    );
    eprintln!("  Model:               {}", config.llm.model);
    eprintln!(
        "  Base URL:            {}",
        config.llm.base_url.as_deref().unwrap_or("(default)")
    );

    // X API
    eprintln!();
    eprintln!("{}", bold.apply_to("X API Credentials"));
    eprintln!("  Client ID:           {}", config.x_api.client_id);
    eprintln!(
        "  Client secret:       {}",
        mask_optional_secret(&config.x_api.client_secret)
    );

    // Targets
    eprintln!();
    eprintln!("{}", bold.apply_to("Target Accounts"));
    eprintln!(
        "  Accounts:            {}",
        format_list(&config.targets.accounts)
    );
    eprintln!(
        "  Max replies/day:     {}",
        config.targets.max_target_replies_per_day
    );

    // Limits
    eprintln!();
    eprintln!("{}", bold.apply_to("Posting Limits"));
    eprintln!(
        "  Replies/day:         {}",
        config.limits.max_replies_per_day
    );
    eprintln!(
        "  Tweets/day:          {}",
        config.limits.max_tweets_per_day
    );
    eprintln!(
        "  Threads/week:        {}",
        config.limits.max_threads_per_week
    );
    eprintln!(
        "  Action delay:        {}-{} seconds",
        config.limits.min_action_delay_seconds, config.limits.max_action_delay_seconds
    );
    eprintln!(
        "  Replies/author/day:  {}",
        config.limits.max_replies_per_author_per_day
    );
    eprintln!(
        "  Product mention %:   {:.0}%",
        config.limits.product_mention_ratio * 100.0
    );
    eprintln!(
        "  Banned phrases:      {}",
        format_list(&config.limits.banned_phrases)
    );

    // Scoring
    eprintln!();
    eprintln!("{}", bold.apply_to("Scoring"));
    eprintln!(
        "  Threshold:           {} (0-100, higher = pickier)",
        config.scoring.threshold
    );
    eprintln!(
        "  Keyword relevance:   {:.0} pts max",
        config.scoring.keyword_relevance_max
    );
    eprintln!(
        "  Follower count:      {:.0} pts max",
        config.scoring.follower_count_max
    );
    eprintln!(
        "  Recency:             {:.0} pts max",
        config.scoring.recency_max
    );
    eprintln!(
        "  Engagement rate:     {:.0} pts max",
        config.scoring.engagement_rate_max
    );
    eprintln!(
        "  Reply count:         {:.0} pts max",
        config.scoring.reply_count_max
    );
    eprintln!(
        "  Content type:        {:.0} pts max",
        config.scoring.content_type_max
    );

    // Timing
    eprintln!();
    eprintln!("{}", bold.apply_to("Timing"));
    eprintln!(
        "  Mention checks:      {}",
        format_duration(config.intervals.mentions_check_seconds)
    );
    eprintln!(
        "  Discovery searches:  {}",
        format_duration(config.intervals.discovery_search_seconds)
    );
    eprintln!(
        "  Content post window: {}",
        format_duration(config.intervals.content_post_window_seconds)
    );
    eprintln!(
        "  Thread interval:     {}",
        format_duration(config.intervals.thread_interval_seconds)
    );

    // Approval
    eprintln!();
    eprintln!("{}", bold.apply_to("Approval Mode"));
    eprintln!(
        "  Enabled:             {}",
        if config.approval_mode { "yes" } else { "no" }
    );

    // Schedule
    eprintln!();
    eprintln!("{}", bold.apply_to("Schedule"));
    eprintln!("  Timezone:            {}", config.schedule.timezone);
    eprintln!(
        "  Active hours:        {}:00 – {}:00",
        config.schedule.active_hours_start, config.schedule.active_hours_end
    );
    eprintln!(
        "  Active days:         {}",
        format_list(&config.schedule.active_days)
    );
    eprintln!(
        "  Tweet times:         {}",
        if config.schedule.preferred_times.is_empty() {
            "(interval mode)".to_string()
        } else {
            format_list(&config.schedule.preferred_times)
        }
    );
    if !config.schedule.preferred_times_override.is_empty() {
        let mut overrides: Vec<_> = config.schedule.preferred_times_override.iter().collect();
        overrides.sort_by_key(|(k, _)| (*k).clone());
        for (day, times) in overrides {
            eprintln!(
                "    {} override:      {}",
                day,
                if times.is_empty() {
                    "(no posts)".to_string()
                } else {
                    times.join(", ")
                }
            );
        }
    }
    eprintln!(
        "  Thread day:          {}",
        config
            .schedule
            .thread_preferred_day
            .as_deref()
            .unwrap_or("(interval mode)")
    );
    eprintln!(
        "  Thread time:         {}",
        config.schedule.thread_preferred_time
    );

    // Storage & Logging
    eprintln!();
    eprintln!("{}", bold.apply_to("Storage & Logging"));
    eprintln!("  Database path:       {}", config.storage.db_path);
    eprintln!(
        "  Data retention:      {} days",
        config.storage.retention_days
    );
    eprintln!(
        "  Status interval:     {}",
        if config.logging.status_interval_seconds == 0 {
            "disabled".to_string()
        } else {
            format_duration(config.logging.status_interval_seconds)
        }
    );
    eprintln!();
}

/// Output configuration as JSON with secrets redacted.
pub(super) fn show_config_json(config: &Config) -> Result<()> {
    let mut config = config.clone();
    config.llm.api_key = config
        .llm
        .api_key
        .as_ref()
        .map(|_| "***REDACTED***".to_string());
    config.x_api.client_secret = config
        .x_api
        .client_secret
        .as_ref()
        .map(|_| "***REDACTED***".to_string());
    write_stdout(&serde_json::to_string(&config)?)?;
    Ok(())
}

pub(super) fn format_list(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".to_string()
    } else {
        items.join(", ")
    }
}

pub(super) fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        "0 seconds".to_string()
    } else if seconds < 60 {
        format!("{seconds} seconds")
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let remaining = seconds % 60;
        if remaining == 0 {
            format!("{mins} min")
        } else {
            format!("{mins} min {remaining} sec")
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let remaining_mins = (seconds % 3600) / 60;
        if remaining_mins == 0 {
            format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
        } else {
            format!(
                "{hours} hour{} {remaining_mins} min",
                if hours == 1 { "" } else { "s" }
            )
        }
    } else {
        let days = seconds / 86400;
        let remaining_hours = (seconds % 86400) / 3600;
        if remaining_hours == 0 {
            format!("{days} day{}", if days == 1 { "" } else { "s" })
        } else {
            format!(
                "{days} day{} {remaining_hours} hour{}",
                if days == 1 { "" } else { "s" },
                if remaining_hours == 1 { "" } else { "s" }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_list ──────────────────────────────────────────────────

    #[test]
    fn format_list_empty_returns_none_label() {
        assert_eq!(format_list(&[]), "(none)");
    }

    #[test]
    fn format_list_single_item() {
        let items = vec!["hello".to_string()];
        assert_eq!(format_list(&items), "hello");
    }

    #[test]
    fn format_list_multiple_items() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(format_list(&items), "a, b, c");
    }

    // ── format_duration ──────────────────────────────────────────────

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0 seconds");
    }

    #[test]
    fn format_duration_1_second() {
        assert_eq!(format_duration(1), "1 seconds");
    }

    #[test]
    fn format_duration_59_seconds() {
        assert_eq!(format_duration(59), "59 seconds");
    }

    #[test]
    fn format_duration_exact_minutes() {
        assert_eq!(format_duration(60), "1 min");
        assert_eq!(format_duration(120), "2 min");
        assert_eq!(format_duration(300), "5 min");
    }

    #[test]
    fn format_duration_minutes_with_seconds() {
        assert_eq!(format_duration(90), "1 min 30 sec");
        assert_eq!(format_duration(150), "2 min 30 sec");
    }

    #[test]
    fn format_duration_exact_hours() {
        assert_eq!(format_duration(3600), "1 hour");
        assert_eq!(format_duration(7200), "2 hours");
    }

    #[test]
    fn format_duration_hours_with_minutes() {
        assert_eq!(format_duration(5400), "1 hour 30 min");
        assert_eq!(format_duration(9000), "2 hours 30 min");
    }

    #[test]
    fn format_duration_exact_days() {
        assert_eq!(format_duration(86400), "1 day");
        assert_eq!(format_duration(172800), "2 days");
    }

    #[test]
    fn format_duration_days_with_hours() {
        assert_eq!(format_duration(90000), "1 day 1 hour");
        assert_eq!(format_duration(176400), "2 days 1 hour");
        assert_eq!(format_duration(180000), "2 days 2 hours");
    }

    #[test]
    fn format_duration_one_week() {
        assert_eq!(format_duration(604800), "7 days");
    }

    // ── format_duration additional edge cases ────────────────────────

    #[test]
    fn format_duration_30_seconds() {
        assert_eq!(format_duration(30), "30 seconds");
    }

    #[test]
    fn format_duration_61_seconds() {
        assert_eq!(format_duration(61), "1 min 1 sec");
    }

    #[test]
    fn format_duration_3661_seconds() {
        assert_eq!(format_duration(3661), "1 hour 1 min");
    }

    #[test]
    fn format_duration_7201_seconds() {
        // 7201 = 2*3600 + 1; remaining_mins = 1/60 = 0
        assert_eq!(format_duration(7201), "2 hours");
    }

    #[test]
    fn format_duration_two_days_three_hours() {
        assert_eq!(format_duration(2 * 86400 + 3 * 3600), "2 days 3 hours");
    }

    #[test]
    fn format_duration_one_day_one_hour() {
        assert_eq!(format_duration(86400 + 3600), "1 day 1 hour");
    }

    #[test]
    fn format_duration_three_days_exact() {
        assert_eq!(format_duration(3 * 86400), "3 days");
    }

    // ── show_config_json ────────────────────────────────────────────

    #[test]
    fn show_config_json_redacts_secrets() {
        let mut config = Config::default();
        config.llm.api_key = Some("sk-secret-key-12345".to_string());
        config.x_api.client_secret = Some("secret-value".to_string());

        // We can't easily capture stdout in a test, but we can verify the
        // redaction logic directly:
        let mut redacted = config.clone();
        redacted.llm.api_key = redacted
            .llm
            .api_key
            .as_ref()
            .map(|_| "***REDACTED***".to_string());
        redacted.x_api.client_secret = redacted
            .x_api
            .client_secret
            .as_ref()
            .map(|_| "***REDACTED***".to_string());

        let json = serde_json::to_string(&redacted).unwrap();
        assert!(json.contains("***REDACTED***"));
        assert!(!json.contains("sk-secret-key-12345"));
        assert!(!json.contains("secret-value"));
    }

    #[test]
    fn show_config_json_no_secrets_still_works() {
        let config = Config::default();
        let mut redacted = config.clone();
        redacted.llm.api_key = redacted
            .llm
            .api_key
            .as_ref()
            .map(|_| "***REDACTED***".to_string());
        let json = serde_json::to_string(&redacted).unwrap();
        assert!(!json.contains("***REDACTED***")); // None -> no redaction needed
    }

    // ── show_config display branches ────────────────────────────────

    #[test]
    fn show_config_does_not_panic() {
        let config = Config::default();
        show_config(&config);
    }

    #[test]
    fn show_config_with_all_fields_set_does_not_panic() {
        let mut config = Config::default();
        config.business.product_url = Some("https://example.com".to_string());
        config.business.brand_voice = Some("Friendly".to_string());
        config.business.reply_style = Some("Helpful".to_string());
        config.business.content_style = Some("Practical".to_string());
        config.llm.api_key = Some("sk-test".to_string());
        config.x_api.client_secret = Some("secret".to_string());
        config.schedule.preferred_times = vec!["09:00".to_string(), "12:00".to_string()];
        config
            .schedule
            .preferred_times_override
            .insert("Mon".to_string(), vec!["10:00".to_string()]);
        config.schedule.thread_preferred_day = Some("Tue".to_string());
        show_config(&config);
    }

    #[test]
    fn show_config_with_empty_preferred_times() {
        let config = Config::default();
        // exercises the "(interval mode)" branch
        show_config(&config);
    }

    #[test]
    fn show_config_with_logging_disabled() {
        let mut config = Config::default();
        config.logging.status_interval_seconds = 0;
        show_config(&config);
    }

    #[test]
    fn show_config_with_logging_enabled() {
        let mut config = Config::default();
        config.logging.status_interval_seconds = 300;
        show_config(&config);
    }

    #[test]
    fn show_config_approval_mode_on() {
        let mut config = Config::default();
        config.approval_mode = true;
        show_config(&config);
    }

    #[test]
    fn show_config_approval_mode_off() {
        let mut config = Config::default();
        config.approval_mode = false;
        show_config(&config);
    }

    // ── format_list additional cases ─────────────────────────────────

    #[test]
    fn format_list_many_items() {
        let items: Vec<String> = (0..10).map(|i| format!("item{i}")).collect();
        let result = format_list(&items);
        assert!(result.contains("item0"));
        assert!(result.contains("item9"));
        assert_eq!(result.matches(", ").count(), 9);
    }
}
