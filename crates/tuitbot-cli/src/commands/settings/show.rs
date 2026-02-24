use anyhow::Result;
use console::Style;
use tuitbot_core::config::Config;

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
        mask_secret(&config.llm.api_key)
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
        mask_secret(&config.x_api.client_secret)
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
    println!("{}", serde_json::to_string(&config)?);
    Ok(())
}

pub(super) fn mask_secret(secret: &Option<String>) -> String {
    match secret {
        Some(s) if s.len() > 8 => {
            format!("{}...{}", &s[..4], &s[s.len() - 4..])
        }
        Some(s) if !s.is_empty() => "****".to_string(),
        Some(_) => "(empty)".to_string(),
        None => "(not set)".to_string(),
    }
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
