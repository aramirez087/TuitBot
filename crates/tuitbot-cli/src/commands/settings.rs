/// `replyguy settings` — interactive configuration editor.
///
/// Modes:
/// - `replyguy settings`              — interactive category menu
/// - `replyguy settings --show`       — pretty-print current config
/// - `replyguy settings --set K=V`    — direct one-shot set
/// - `replyguy settings <category>`   — jump to a specific category
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use replyguy_core::config::Config;

use super::SettingsArgs;

/// Entry point for the settings command.
pub async fn execute(args: SettingsArgs, config_path: &str) -> Result<()> {
    let expanded = expand_tilde(config_path);
    if !expanded.exists() {
        bail!(
            "Config file not found: {}\nRun 'replyguy init' first.",
            expanded.display()
        );
    }

    let config = Config::load(Some(config_path)).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load configuration: {e}\n\
             Hint: Run 'replyguy init' to create a default configuration file."
        )
    })?;

    if args.show {
        show_config(&config);
        return Ok(());
    }

    if let Some(kv) = &args.set {
        let mut config = config;
        return set_direct(&mut config, kv, &expanded);
    }

    if !std::io::stdin().is_terminal() {
        bail!(
            "Interactive settings editor requires a terminal.\n\
             Use --show to view or --set KEY=VALUE to change settings."
        );
    }

    let mut config = config;

    if let Some(category) = &args.category {
        let tracker = &mut ChangeTracker::new();
        match category.to_lowercase().as_str() {
            "product" | "business" => edit_category_product(&mut config, tracker)?,
            "voice" | "brand" => edit_category_voice(&mut config, tracker)?,
            "persona" => edit_category_persona(&mut config, tracker)?,
            "ai" | "llm" | "provider" => edit_category_llm(&mut config, tracker)?,
            "x" | "xapi" | "credentials" => edit_category_xapi(&mut config, tracker)?,
            "targets" | "accounts" => edit_category_targets(&mut config, tracker)?,
            "limits" | "posting" => edit_category_limits(&mut config, tracker)?,
            "scoring" => edit_category_scoring(&mut config, tracker)?,
            "timing" | "intervals" => edit_category_timing(&mut config, tracker)?,
            "approval" => edit_category_approval(&mut config, tracker)?,
            "storage" | "logging" => edit_category_storage(&mut config, tracker)?,
            other => bail!(
                "Unknown category: {other}\n\
                 Valid categories: product, voice, persona, ai, x, targets, limits, scoring, timing, approval, storage"
            ),
        }
        if !tracker.changes.is_empty() {
            save_flow(&config, &expanded, tracker)?;
        } else {
            eprintln!("No changes made.");
        }
        return Ok(());
    }

    interactive_menu(&mut config, &expanded)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// --show mode
// ---------------------------------------------------------------------------

fn show_config(config: &Config) {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("ReplyGuy Configuration"));
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
    eprintln!(
        "  Auto-follow:         {}",
        if config.targets.auto_follow {
            "yes"
        } else {
            "no"
        }
    );
    eprintln!(
        "  Follow warmup days:  {}",
        config.targets.follow_warmup_days
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

// ---------------------------------------------------------------------------
// --set direct mode
// ---------------------------------------------------------------------------

fn set_direct(config: &mut Config, kv: &str, config_path: &Path) -> Result<()> {
    let (key, value) = kv.split_once('=').ok_or_else(|| {
        anyhow::anyhow!("Invalid format. Use --set KEY=VALUE (e.g., --set scoring.threshold=80)")
    })?;

    let key = key.trim();
    let value = value.trim();

    let mut tracker = ChangeTracker::new();

    match key {
        // Business
        "business.product_name" => {
            tracker.record(
                "business",
                "product_name",
                &config.business.product_name,
                value,
            );
            config.business.product_name = value.to_string();
        }
        "business.product_description" => {
            tracker.record(
                "business",
                "product_description",
                &config.business.product_description,
                value,
            );
            config.business.product_description = value.to_string();
        }
        "business.product_url" => {
            let old = config
                .business
                .product_url
                .as_deref()
                .unwrap_or("(none)")
                .to_string();
            if value == "none" || value.is_empty() {
                config.business.product_url = None;
                tracker.record("business", "product_url", &old, "(none)");
            } else {
                config.business.product_url = Some(value.to_string());
                tracker.record("business", "product_url", &old, value);
            }
        }
        "business.target_audience" => {
            tracker.record(
                "business",
                "target_audience",
                &config.business.target_audience,
                value,
            );
            config.business.target_audience = value.to_string();
        }
        "business.product_keywords" => {
            let old = config.business.product_keywords.join(", ");
            config.business.product_keywords = parse_csv(value);
            tracker.record("business", "product_keywords", &old, value);
        }
        "business.competitor_keywords" => {
            let old = config.business.competitor_keywords.join(", ");
            config.business.competitor_keywords = parse_csv(value);
            tracker.record("business", "competitor_keywords", &old, value);
        }
        "business.industry_topics" => {
            let old = config.business.industry_topics.join(", ");
            config.business.industry_topics = parse_csv(value);
            tracker.record("business", "industry_topics", &old, value);
        }
        "business.brand_voice" => {
            let old = config
                .business
                .brand_voice
                .as_deref()
                .unwrap_or("(none)")
                .to_string();
            if value == "none" || value.is_empty() {
                config.business.brand_voice = None;
                tracker.record("business", "brand_voice", &old, "(none)");
            } else {
                config.business.brand_voice = Some(value.to_string());
                tracker.record("business", "brand_voice", &old, value);
            }
        }
        "business.reply_style" => {
            let old = config
                .business
                .reply_style
                .as_deref()
                .unwrap_or("(none)")
                .to_string();
            if value == "none" || value.is_empty() {
                config.business.reply_style = None;
                tracker.record("business", "reply_style", &old, "(none)");
            } else {
                config.business.reply_style = Some(value.to_string());
                tracker.record("business", "reply_style", &old, value);
            }
        }
        "business.content_style" => {
            let old = config
                .business
                .content_style
                .as_deref()
                .unwrap_or("(none)")
                .to_string();
            if value == "none" || value.is_empty() {
                config.business.content_style = None;
                tracker.record("business", "content_style", &old, "(none)");
            } else {
                config.business.content_style = Some(value.to_string());
                tracker.record("business", "content_style", &old, value);
            }
        }
        "business.persona_opinions" => {
            let old = config.business.persona_opinions.join(", ");
            config.business.persona_opinions = parse_csv(value);
            tracker.record("business", "persona_opinions", &old, value);
        }
        "business.persona_experiences" => {
            let old = config.business.persona_experiences.join(", ");
            config.business.persona_experiences = parse_csv(value);
            tracker.record("business", "persona_experiences", &old, value);
        }
        "business.content_pillars" => {
            let old = config.business.content_pillars.join(", ");
            config.business.content_pillars = parse_csv(value);
            tracker.record("business", "content_pillars", &old, value);
        }

        // Scoring
        "scoring.threshold" => {
            let v: u32 = value.parse().context("threshold must be a number 0-100")?;
            tracker.record(
                "scoring",
                "threshold",
                &config.scoring.threshold.to_string(),
                value,
            );
            config.scoring.threshold = v;
        }
        "scoring.keyword_relevance_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "keyword_relevance_max",
                &format!("{:.1}", config.scoring.keyword_relevance_max),
                value,
            );
            config.scoring.keyword_relevance_max = v;
        }
        "scoring.follower_count_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "follower_count_max",
                &format!("{:.1}", config.scoring.follower_count_max),
                value,
            );
            config.scoring.follower_count_max = v;
        }
        "scoring.recency_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "recency_max",
                &format!("{:.1}", config.scoring.recency_max),
                value,
            );
            config.scoring.recency_max = v;
        }
        "scoring.engagement_rate_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "engagement_rate_max",
                &format!("{:.1}", config.scoring.engagement_rate_max),
                value,
            );
            config.scoring.engagement_rate_max = v;
        }
        "scoring.reply_count_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "reply_count_max",
                &format!("{:.1}", config.scoring.reply_count_max),
                value,
            );
            config.scoring.reply_count_max = v;
        }
        "scoring.content_type_max" => {
            let v: f32 = value.parse().context("must be a number")?;
            tracker.record(
                "scoring",
                "content_type_max",
                &format!("{:.1}", config.scoring.content_type_max),
                value,
            );
            config.scoring.content_type_max = v;
        }

        // Limits
        "limits.max_replies_per_day" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "max_replies_per_day",
                &config.limits.max_replies_per_day.to_string(),
                value,
            );
            config.limits.max_replies_per_day = v;
        }
        "limits.max_tweets_per_day" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "max_tweets_per_day",
                &config.limits.max_tweets_per_day.to_string(),
                value,
            );
            config.limits.max_tweets_per_day = v;
        }
        "limits.max_threads_per_week" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "max_threads_per_week",
                &config.limits.max_threads_per_week.to_string(),
                value,
            );
            config.limits.max_threads_per_week = v;
        }
        "limits.min_action_delay_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "min_action_delay_seconds",
                &config.limits.min_action_delay_seconds.to_string(),
                value,
            );
            config.limits.min_action_delay_seconds = v;
        }
        "limits.max_action_delay_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "max_action_delay_seconds",
                &config.limits.max_action_delay_seconds.to_string(),
                value,
            );
            config.limits.max_action_delay_seconds = v;
        }
        "limits.max_replies_per_author_per_day" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "limits",
                "max_replies_per_author_per_day",
                &config.limits.max_replies_per_author_per_day.to_string(),
                value,
            );
            config.limits.max_replies_per_author_per_day = v;
        }
        "limits.product_mention_ratio" => {
            let v: f32 = value.parse().context("must be a number 0.0-1.0")?;
            tracker.record(
                "limits",
                "product_mention_ratio",
                &format!("{:.2}", config.limits.product_mention_ratio),
                value,
            );
            config.limits.product_mention_ratio = v;
        }
        "limits.banned_phrases" => {
            let old = config.limits.banned_phrases.join(", ");
            config.limits.banned_phrases = parse_csv(value);
            tracker.record("limits", "banned_phrases", &old, value);
        }

        // Intervals
        "intervals.mentions_check_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "intervals",
                "mentions_check_seconds",
                &config.intervals.mentions_check_seconds.to_string(),
                value,
            );
            config.intervals.mentions_check_seconds = v;
        }
        "intervals.discovery_search_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "intervals",
                "discovery_search_seconds",
                &config.intervals.discovery_search_seconds.to_string(),
                value,
            );
            config.intervals.discovery_search_seconds = v;
        }
        "intervals.content_post_window_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "intervals",
                "content_post_window_seconds",
                &config.intervals.content_post_window_seconds.to_string(),
                value,
            );
            config.intervals.content_post_window_seconds = v;
        }
        "intervals.thread_interval_seconds" => {
            let v: u64 = value.parse().context("must be a positive number")?;
            tracker.record(
                "intervals",
                "thread_interval_seconds",
                &config.intervals.thread_interval_seconds.to_string(),
                value,
            );
            config.intervals.thread_interval_seconds = v;
        }

        // Targets
        "targets.accounts" => {
            let old = config.targets.accounts.join(", ");
            config.targets.accounts = parse_csv(value);
            tracker.record("targets", "accounts", &old, value);
        }
        "targets.max_target_replies_per_day" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "targets",
                "max_target_replies_per_day",
                &config.targets.max_target_replies_per_day.to_string(),
                value,
            );
            config.targets.max_target_replies_per_day = v;
        }
        "targets.auto_follow" => {
            let v: bool = parse_bool(value)?;
            tracker.record(
                "targets",
                "auto_follow",
                &config.targets.auto_follow.to_string(),
                value,
            );
            config.targets.auto_follow = v;
        }
        "targets.follow_warmup_days" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "targets",
                "follow_warmup_days",
                &config.targets.follow_warmup_days.to_string(),
                value,
            );
            config.targets.follow_warmup_days = v;
        }

        // LLM
        "llm.provider" => {
            tracker.record("llm", "provider", &config.llm.provider, value);
            config.llm.provider = value.to_string();
        }
        "llm.api_key" => {
            tracker.record("llm", "api_key", "(hidden)", "(updated)");
            config.llm.api_key = Some(value.to_string());
        }
        "llm.model" => {
            tracker.record("llm", "model", &config.llm.model, value);
            config.llm.model = value.to_string();
        }
        "llm.base_url" => {
            let old = config
                .llm
                .base_url
                .as_deref()
                .unwrap_or("(none)")
                .to_string();
            if value == "none" || value.is_empty() {
                config.llm.base_url = None;
                tracker.record("llm", "base_url", &old, "(none)");
            } else {
                config.llm.base_url = Some(value.to_string());
                tracker.record("llm", "base_url", &old, value);
            }
        }

        // X API
        "x_api.client_id" => {
            tracker.record("x_api", "client_id", &config.x_api.client_id, value);
            config.x_api.client_id = value.to_string();
        }
        "x_api.client_secret" => {
            tracker.record("x_api", "client_secret", "(hidden)", "(updated)");
            config.x_api.client_secret = Some(value.to_string());
        }

        // Approval mode
        "approval_mode" => {
            let v: bool = parse_bool(value)?;
            tracker.record(
                "",
                "approval_mode",
                &config.approval_mode.to_string(),
                value,
            );
            config.approval_mode = v;
        }

        // Storage
        "storage.db_path" => {
            tracker.record("storage", "db_path", &config.storage.db_path, value);
            config.storage.db_path = value.to_string();
        }
        "storage.retention_days" => {
            let v: u32 = value.parse().context("must be a positive number")?;
            tracker.record(
                "storage",
                "retention_days",
                &config.storage.retention_days.to_string(),
                value,
            );
            config.storage.retention_days = v;
        }

        // Logging
        "logging.status_interval_seconds" => {
            let v: u64 = value.parse().context("must be a number")?;
            tracker.record(
                "logging",
                "status_interval_seconds",
                &config.logging.status_interval_seconds.to_string(),
                value,
            );
            config.logging.status_interval_seconds = v;
        }

        _ => bail!(
            "Unknown setting: {key}\n\
             Use 'replyguy settings --show' to see all available settings."
        ),
    }

    validate_config(config)?;
    let path_str = config_path.display().to_string();
    write_config_with_backup(config, &path_str)?;

    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to("Updated:"));
    for change in &tracker.changes {
        let section_prefix = if change.section.is_empty() {
            String::new()
        } else {
            format!("{}.", change.section)
        };
        eprintln!(
            "  {}{}: \"{}\" -> \"{}\"",
            section_prefix, change.field, change.old_value, change.new_value
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Interactive menu
// ---------------------------------------------------------------------------

fn interactive_menu(config: &mut Config, config_path: &Path) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let mut tracker = ChangeTracker::new();

    loop {
        eprintln!();
        eprintln!("{}", bold.apply_to("ReplyGuy Settings"));
        eprintln!("{}", dim.apply_to("─────────────────"));
        eprintln!();

        let categories = &[
            "Your Product        — name, description, keywords, audience",
            "Brand Voice         — personality, reply style, content style",
            "Persona             — opinions, experiences, content pillars",
            "AI Provider         — LLM provider, API key, model",
            "X API Credentials   — client ID, client secret",
            "Target Accounts     — accounts to monitor, auto-follow",
            "Posting Limits      — replies/tweets/threads per day",
            "Scoring             — how picky the bot is about which tweets to reply to",
            "Timing              — how often the bot checks for new tweets",
            "Approval Mode       — review posts before they go live",
            "Storage & Logging   — database path, data retention",
            "Save & Exit",
        ];

        let selection = Select::new()
            .with_prompt("Pick a category to edit")
            .items(categories)
            .default(0)
            .interact()?;

        match selection {
            0 => edit_category_product(config, &mut tracker)?,
            1 => edit_category_voice(config, &mut tracker)?,
            2 => edit_category_persona(config, &mut tracker)?,
            3 => edit_category_llm(config, &mut tracker)?,
            4 => edit_category_xapi(config, &mut tracker)?,
            5 => edit_category_targets(config, &mut tracker)?,
            6 => edit_category_limits(config, &mut tracker)?,
            7 => edit_category_scoring(config, &mut tracker)?,
            8 => edit_category_timing(config, &mut tracker)?,
            9 => edit_category_approval(config, &mut tracker)?,
            10 => edit_category_storage(config, &mut tracker)?,
            11 => break, // Save & Exit
            _ => unreachable!(),
        }
    }

    if tracker.changes.is_empty() {
        eprintln!("No changes made.");
        return Ok(());
    }

    save_flow(config, config_path, &tracker)
}

// ---------------------------------------------------------------------------
// Category editors
// ---------------------------------------------------------------------------

fn edit_category_product(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Your Product"));
    eprintln!("{}", dim.apply_to("────────────"));

    let fields = &[
        format!("Product name:        {}", config.business.product_name),
        format!(
            "Description:         {}",
            config.business.product_description
        ),
        format!(
            "Product URL:         {}",
            config
                .business
                .product_url
                .as_deref()
                .unwrap_or("(not set)")
        ),
        format!("Target audience:     {}", config.business.target_audience),
        format!(
            "Discovery keywords:  {}",
            format_list(&config.business.product_keywords)
        ),
        format!(
            "Competitor keywords: {}",
            format_list(&config.business.competitor_keywords)
        ),
        format!(
            "Content topics:      {}",
            format_list(&config.business.industry_topics)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_string("Product name", &config.business.product_name)?;
            tracker.record(
                "business",
                "product_name",
                &config.business.product_name,
                &new_val,
            );
            config.business.product_name = new_val;
        }
        1 => {
            let new_val = edit_string("Description", &config.business.product_description)?;
            tracker.record(
                "business",
                "product_description",
                &config.business.product_description,
                &new_val,
            );
            config.business.product_description = new_val;
        }
        2 => {
            let new_val = edit_optional_string("Product URL", &config.business.product_url)?;
            let old_display = config.business.product_url.as_deref().unwrap_or("(none)");
            let new_display = new_val.as_deref().unwrap_or("(none)");
            tracker.record("business", "product_url", old_display, new_display);
            config.business.product_url = new_val;
        }
        3 => {
            let new_val = edit_string("Target audience", &config.business.target_audience)?;
            tracker.record(
                "business",
                "target_audience",
                &config.business.target_audience,
                &new_val,
            );
            config.business.target_audience = new_val;
        }
        4 => {
            let new_val = edit_list("Discovery keywords", &config.business.product_keywords)?;
            let old_display = config.business.product_keywords.join(", ");
            let new_display = new_val.join(", ");
            tracker.record("business", "product_keywords", &old_display, &new_display);
            config.business.product_keywords = new_val;
        }
        5 => {
            let new_val = edit_list("Competitor keywords", &config.business.competitor_keywords)?;
            let old_display = config.business.competitor_keywords.join(", ");
            let new_display = new_val.join(", ");
            tracker.record(
                "business",
                "competitor_keywords",
                &old_display,
                &new_display,
            );
            config.business.competitor_keywords = new_val;
        }
        6 => {
            let new_val = edit_list("Content topics", &config.business.industry_topics)?;
            let old_display = config.business.industry_topics.join(", ");
            let new_display = new_val.join(", ");
            tracker.record("business", "industry_topics", &old_display, &new_display);
            config.business.industry_topics = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_voice(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Brand Voice"));
    eprintln!("{}", dim.apply_to("───────────"));

    let fields = &[
        format!(
            "Personality:   {}",
            config
                .business
                .brand_voice
                .as_deref()
                .unwrap_or("(default)")
        ),
        format!(
            "Reply style:   {}",
            config
                .business
                .reply_style
                .as_deref()
                .unwrap_or("(default)")
        ),
        format!(
            "Content style: {}",
            config
                .business
                .content_style
                .as_deref()
                .unwrap_or("(default)")
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_optional_string("Personality", &config.business.brand_voice)?;
            let old_display = config.business.brand_voice.as_deref().unwrap_or("(none)");
            let new_display = new_val.as_deref().unwrap_or("(none)");
            tracker.record("business", "brand_voice", old_display, new_display);
            config.business.brand_voice = new_val;
        }
        1 => {
            let new_val = edit_optional_string("Reply style", &config.business.reply_style)?;
            let old_display = config.business.reply_style.as_deref().unwrap_or("(none)");
            let new_display = new_val.as_deref().unwrap_or("(none)");
            tracker.record("business", "reply_style", old_display, new_display);
            config.business.reply_style = new_val;
        }
        2 => {
            let new_val = edit_optional_string("Content style", &config.business.content_style)?;
            let old_display = config.business.content_style.as_deref().unwrap_or("(none)");
            let new_display = new_val.as_deref().unwrap_or("(none)");
            tracker.record("business", "content_style", old_display, new_display);
            config.business.content_style = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_persona(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Persona"));
    eprintln!("{}", dim.apply_to("───────"));

    let fields = &[
        format!(
            "Opinions:        {}",
            format_list(&config.business.persona_opinions)
        ),
        format!(
            "Experiences:     {}",
            format_list(&config.business.persona_experiences)
        ),
        format!(
            "Content pillars: {}",
            format_list(&config.business.content_pillars)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_list("Opinions", &config.business.persona_opinions)?;
            let old_display = config.business.persona_opinions.join(", ");
            let new_display = new_val.join(", ");
            tracker.record("business", "persona_opinions", &old_display, &new_display);
            config.business.persona_opinions = new_val;
        }
        1 => {
            let new_val = edit_list("Experiences", &config.business.persona_experiences)?;
            let old_display = config.business.persona_experiences.join(", ");
            let new_display = new_val.join(", ");
            tracker.record(
                "business",
                "persona_experiences",
                &old_display,
                &new_display,
            );
            config.business.persona_experiences = new_val;
        }
        2 => {
            let new_val = edit_list("Content pillars", &config.business.content_pillars)?;
            let old_display = config.business.content_pillars.join(", ");
            let new_display = new_val.join(", ");
            tracker.record("business", "content_pillars", &old_display, &new_display);
            config.business.content_pillars = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_llm(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("AI Provider"));
    eprintln!("{}", dim.apply_to("───────────"));

    let fields = &[
        format!("Provider:  {}", config.llm.provider),
        format!("API key:   {}", mask_secret(&config.llm.api_key)),
        format!("Model:     {}", config.llm.model),
        format!(
            "Base URL:  {}",
            config.llm.base_url.as_deref().unwrap_or("(default)")
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let providers = &["openai", "anthropic", "ollama"];
            let current = providers
                .iter()
                .position(|p| *p == config.llm.provider)
                .unwrap_or(0);
            let idx = Select::new()
                .with_prompt("LLM provider")
                .items(providers)
                .default(current)
                .interact()?;
            let new_val = providers[idx].to_string();
            tracker.record("llm", "provider", &config.llm.provider, &new_val);
            config.llm.provider = new_val;
        }
        1 => {
            let new_val: String = Input::new()
                .with_prompt("API key")
                .allow_empty(true)
                .interact_text()?;
            if new_val.trim().is_empty() {
                tracker.record("llm", "api_key", "(hidden)", "(cleared)");
                config.llm.api_key = None;
            } else {
                tracker.record("llm", "api_key", "(hidden)", "(updated)");
                config.llm.api_key = Some(new_val.trim().to_string());
            }
        }
        2 => {
            let new_val = edit_string("Model", &config.llm.model)?;
            tracker.record("llm", "model", &config.llm.model, &new_val);
            config.llm.model = new_val;
        }
        3 => {
            let new_val = edit_optional_string("Base URL", &config.llm.base_url)?;
            let old_display = config.llm.base_url.as_deref().unwrap_or("(none)");
            let new_display = new_val.as_deref().unwrap_or("(none)");
            tracker.record("llm", "base_url", old_display, new_display);
            config.llm.base_url = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_xapi(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("X API Credentials"));
    eprintln!("{}", dim.apply_to("─────────────────"));

    let fields = &[
        format!("Client ID:     {}", config.x_api.client_id),
        format!(
            "Client secret: {}",
            mask_secret(&config.x_api.client_secret)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_string("Client ID", &config.x_api.client_id)?;
            tracker.record("x_api", "client_id", &config.x_api.client_id, &new_val);
            config.x_api.client_id = new_val;
        }
        1 => {
            let new_val: String = Input::new()
                .with_prompt("Client secret (Enter to clear)")
                .allow_empty(true)
                .interact_text()?;
            if new_val.trim().is_empty() {
                tracker.record("x_api", "client_secret", "(hidden)", "(cleared)");
                config.x_api.client_secret = None;
            } else {
                tracker.record("x_api", "client_secret", "(hidden)", "(updated)");
                config.x_api.client_secret = Some(new_val.trim().to_string());
            }
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_targets(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Target Accounts"));
    eprintln!("{}", dim.apply_to("───────────────"));

    let fields = &[
        format!(
            "Accounts:          {}",
            format_list(&config.targets.accounts)
        ),
        format!(
            "Max replies/day:   {}",
            config.targets.max_target_replies_per_day
        ),
        format!(
            "Auto-follow:       {}",
            if config.targets.auto_follow {
                "yes"
            } else {
                "no"
            }
        ),
        format!(
            "Follow warmup:     {} days",
            config.targets.follow_warmup_days
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_list("Target accounts (without @)", &config.targets.accounts)?;
            let cleaned: Vec<String> = new_val
                .into_iter()
                .map(|a| a.trim_start_matches('@').to_string())
                .collect();
            let old_display = config.targets.accounts.join(", ");
            let new_display = cleaned.join(", ");
            tracker.record("targets", "accounts", &old_display, &new_display);
            config.targets.accounts = cleaned;
        }
        1 => {
            let new_val = edit_u32(
                "Max target replies per day",
                config.targets.max_target_replies_per_day,
                None,
            )?;
            tracker.record(
                "targets",
                "max_target_replies_per_day",
                &config.targets.max_target_replies_per_day.to_string(),
                &new_val.to_string(),
            );
            config.targets.max_target_replies_per_day = new_val;
        }
        2 => {
            let new_val = edit_bool("Auto-follow target accounts?", config.targets.auto_follow)?;
            tracker.record(
                "targets",
                "auto_follow",
                &config.targets.auto_follow.to_string(),
                &new_val.to_string(),
            );
            config.targets.auto_follow = new_val;
        }
        3 => {
            let new_val = edit_u32(
                "Follow warmup days",
                config.targets.follow_warmup_days,
                None,
            )?;
            tracker.record(
                "targets",
                "follow_warmup_days",
                &config.targets.follow_warmup_days.to_string(),
                &new_val.to_string(),
            );
            config.targets.follow_warmup_days = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_limits(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Posting Limits"));
    eprintln!("{}", dim.apply_to("──────────────"));

    let fields = &[
        format!("Replies/day:         {}", config.limits.max_replies_per_day),
        format!("Tweets/day:          {}", config.limits.max_tweets_per_day),
        format!(
            "Threads/week:        {}",
            config.limits.max_threads_per_week
        ),
        format!(
            "Min action delay:    {} sec",
            config.limits.min_action_delay_seconds
        ),
        format!(
            "Max action delay:    {} sec",
            config.limits.max_action_delay_seconds
        ),
        format!(
            "Replies/author/day:  {}",
            config.limits.max_replies_per_author_per_day
        ),
        format!(
            "Product mention %:   {:.0}%",
            config.limits.product_mention_ratio * 100.0
        ),
        format!(
            "Banned phrases:      {}",
            format_list(&config.limits.banned_phrases)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_u32(
                "Max replies per day",
                config.limits.max_replies_per_day,
                Some("How many tweets to reply to per day"),
            )?;
            tracker.record(
                "limits",
                "max_replies_per_day",
                &config.limits.max_replies_per_day.to_string(),
                &new_val.to_string(),
            );
            config.limits.max_replies_per_day = new_val;
        }
        1 => {
            let new_val = edit_u32(
                "Max tweets per day",
                config.limits.max_tweets_per_day,
                Some("How many original tweets to post per day"),
            )?;
            tracker.record(
                "limits",
                "max_tweets_per_day",
                &config.limits.max_tweets_per_day.to_string(),
                &new_val.to_string(),
            );
            config.limits.max_tweets_per_day = new_val;
        }
        2 => {
            let new_val = edit_u32(
                "Max threads per week",
                config.limits.max_threads_per_week,
                Some("How many educational threads to post per week"),
            )?;
            tracker.record(
                "limits",
                "max_threads_per_week",
                &config.limits.max_threads_per_week.to_string(),
                &new_val.to_string(),
            );
            config.limits.max_threads_per_week = new_val;
        }
        3 => {
            let new_val = edit_u64(
                "Min action delay (seconds)",
                config.limits.min_action_delay_seconds,
                Some("Minimum wait between actions"),
            )?;
            tracker.record(
                "limits",
                "min_action_delay_seconds",
                &config.limits.min_action_delay_seconds.to_string(),
                &new_val.to_string(),
            );
            config.limits.min_action_delay_seconds = new_val;
        }
        4 => {
            let new_val = edit_u64(
                "Max action delay (seconds)",
                config.limits.max_action_delay_seconds,
                Some("Maximum wait between actions"),
            )?;
            tracker.record(
                "limits",
                "max_action_delay_seconds",
                &config.limits.max_action_delay_seconds.to_string(),
                &new_val.to_string(),
            );
            config.limits.max_action_delay_seconds = new_val;
        }
        5 => {
            let new_val = edit_u32(
                "Max replies per author per day",
                config.limits.max_replies_per_author_per_day,
                Some("Avoid spamming the same person"),
            )?;
            tracker.record(
                "limits",
                "max_replies_per_author_per_day",
                &config.limits.max_replies_per_author_per_day.to_string(),
                &new_val.to_string(),
            );
            config.limits.max_replies_per_author_per_day = new_val;
        }
        6 => {
            let new_val = edit_f32(
                "Product mention ratio (0.0-1.0)",
                config.limits.product_mention_ratio,
                Some("What fraction of replies may mention your product (e.g., 0.2 = 20%)"),
            )?;
            tracker.record(
                "limits",
                "product_mention_ratio",
                &format!("{:.2}", config.limits.product_mention_ratio),
                &format!("{:.2}", new_val),
            );
            config.limits.product_mention_ratio = new_val;
        }
        7 => {
            let new_val = edit_list("Banned phrases", &config.limits.banned_phrases)?;
            let old_display = config.limits.banned_phrases.join(", ");
            let new_display = new_val.join(", ");
            tracker.record("limits", "banned_phrases", &old_display, &new_display);
            config.limits.banned_phrases = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_scoring(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Scoring"));
    eprintln!("{}", dim.apply_to("───────"));

    let fields = &[
        format!(
            "Threshold:         {} (0-100, higher = pickier)",
            config.scoring.threshold
        ),
        format!(
            "Keyword relevance: {:.0} pts max",
            config.scoring.keyword_relevance_max
        ),
        format!(
            "Follower count:    {:.0} pts max",
            config.scoring.follower_count_max
        ),
        format!(
            "Recency:           {:.0} pts max",
            config.scoring.recency_max
        ),
        format!(
            "Engagement rate:   {:.0} pts max",
            config.scoring.engagement_rate_max
        ),
        format!(
            "Reply count:       {:.0} pts max",
            config.scoring.reply_count_max
        ),
        format!(
            "Content type:      {:.0} pts max",
            config.scoring.content_type_max
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_u32(
                "Scoring threshold (0-100)",
                config.scoring.threshold,
                Some("Higher = replies only to very relevant tweets"),
            )?;
            tracker.record(
                "scoring",
                "threshold",
                &config.scoring.threshold.to_string(),
                &new_val.to_string(),
            );
            config.scoring.threshold = new_val;
        }
        1 => {
            let new_val = edit_f32(
                "Keyword relevance max pts",
                config.scoring.keyword_relevance_max,
                Some("Points for matching discovery keywords"),
            )?;
            tracker.record(
                "scoring",
                "keyword_relevance_max",
                &format!("{:.1}", config.scoring.keyword_relevance_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.keyword_relevance_max = new_val;
        }
        2 => {
            let new_val = edit_f32(
                "Follower count max pts",
                config.scoring.follower_count_max,
                Some("Points for author follower count (bell curve)"),
            )?;
            tracker.record(
                "scoring",
                "follower_count_max",
                &format!("{:.1}", config.scoring.follower_count_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.follower_count_max = new_val;
        }
        3 => {
            let new_val = edit_f32(
                "Recency max pts",
                config.scoring.recency_max,
                Some("Points for how recently the tweet was posted"),
            )?;
            tracker.record(
                "scoring",
                "recency_max",
                &format!("{:.1}", config.scoring.recency_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.recency_max = new_val;
        }
        4 => {
            let new_val = edit_f32(
                "Engagement rate max pts",
                config.scoring.engagement_rate_max,
                Some("Points for tweet engagement level"),
            )?;
            tracker.record(
                "scoring",
                "engagement_rate_max",
                &format!("{:.1}", config.scoring.engagement_rate_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.engagement_rate_max = new_val;
        }
        5 => {
            let new_val = edit_f32(
                "Reply count max pts",
                config.scoring.reply_count_max,
                Some("Points for few existing replies (less competition)"),
            )?;
            tracker.record(
                "scoring",
                "reply_count_max",
                &format!("{:.1}", config.scoring.reply_count_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.reply_count_max = new_val;
        }
        6 => {
            let new_val = edit_f32(
                "Content type max pts",
                config.scoring.content_type_max,
                Some("Points for text-only original tweets"),
            )?;
            tracker.record(
                "scoring",
                "content_type_max",
                &format!("{:.1}", config.scoring.content_type_max),
                &format!("{:.1}", new_val),
            );
            config.scoring.content_type_max = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_timing(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Timing"));
    eprintln!("{}", dim.apply_to("──────"));

    let fields = &[
        format!(
            "Mention checks:      {}",
            format_duration(config.intervals.mentions_check_seconds)
        ),
        format!(
            "Discovery searches:  {}",
            format_duration(config.intervals.discovery_search_seconds)
        ),
        format!(
            "Content post window: {}",
            format_duration(config.intervals.content_post_window_seconds)
        ),
        format!(
            "Thread interval:     {}",
            format_duration(config.intervals.thread_interval_seconds)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_duration_minutes(
                "How often to check mentions",
                config.intervals.mentions_check_seconds,
            )?;
            tracker.record(
                "intervals",
                "mentions_check_seconds",
                &config.intervals.mentions_check_seconds.to_string(),
                &new_val.to_string(),
            );
            config.intervals.mentions_check_seconds = new_val;
        }
        1 => {
            let new_val = edit_duration_minutes(
                "How often to search for tweets",
                config.intervals.discovery_search_seconds,
            )?;
            tracker.record(
                "intervals",
                "discovery_search_seconds",
                &config.intervals.discovery_search_seconds.to_string(),
                &new_val.to_string(),
            );
            config.intervals.discovery_search_seconds = new_val;
        }
        2 => {
            let new_val = edit_duration_minutes(
                "Content post window",
                config.intervals.content_post_window_seconds,
            )?;
            tracker.record(
                "intervals",
                "content_post_window_seconds",
                &config.intervals.content_post_window_seconds.to_string(),
                &new_val.to_string(),
            );
            config.intervals.content_post_window_seconds = new_val;
        }
        3 => {
            let new_val =
                edit_duration_minutes("Thread interval", config.intervals.thread_interval_seconds)?;
            tracker.record(
                "intervals",
                "thread_interval_seconds",
                &config.intervals.thread_interval_seconds.to_string(),
                &new_val.to_string(),
            );
            config.intervals.thread_interval_seconds = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

fn edit_category_approval(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Approval Mode"));
    eprintln!("{}", dim.apply_to("─────────────"));
    eprintln!(
        "  Currently: {}",
        if config.approval_mode {
            "enabled (posts are queued for review)"
        } else {
            "disabled (posts go live immediately)"
        }
    );

    let new_val = edit_bool(
        "Queue posts for review before posting?",
        config.approval_mode,
    )?;
    if new_val != config.approval_mode {
        tracker.record(
            "",
            "approval_mode",
            &config.approval_mode.to_string(),
            &new_val.to_string(),
        );
        config.approval_mode = new_val;
    }

    Ok(())
}

fn edit_category_storage(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Storage & Logging"));
    eprintln!("{}", dim.apply_to("─────────────────"));

    let fields = &[
        format!("Database path:     {}", config.storage.db_path),
        format!("Data retention:    {} days", config.storage.retention_days),
        format!(
            "Status interval:   {}",
            if config.logging.status_interval_seconds == 0 {
                "disabled".to_string()
            } else {
                format_duration(config.logging.status_interval_seconds)
            }
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_string("Database path", &config.storage.db_path)?;
            tracker.record("storage", "db_path", &config.storage.db_path, &new_val);
            config.storage.db_path = new_val;
        }
        1 => {
            let new_val = edit_u32(
                "Data retention (days)",
                config.storage.retention_days,
                Some("How many days of data to keep"),
            )?;
            tracker.record(
                "storage",
                "retention_days",
                &config.storage.retention_days.to_string(),
                &new_val.to_string(),
            );
            config.storage.retention_days = new_val;
        }
        2 => {
            let new_val = edit_u64(
                "Status interval (seconds, 0 = disabled)",
                config.logging.status_interval_seconds,
                Some("How often to print status summaries (0 = disabled)"),
            )?;
            tracker.record(
                "logging",
                "status_interval_seconds",
                &config.logging.status_interval_seconds.to_string(),
                &new_val.to_string(),
            );
            config.logging.status_interval_seconds = new_val;
        }
        _ => {} // Back
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Save flow
// ---------------------------------------------------------------------------

fn save_flow(config: &Config, config_path: &Path, tracker: &ChangeTracker) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Changes to save:"));
    eprintln!("{}", dim.apply_to("────────────────"));

    for change in &tracker.changes {
        let section_prefix = if change.section.is_empty() {
            String::new()
        } else {
            format!("[{}] ", change.section)
        };
        eprintln!(
            "  {}{}: \"{}\" -> \"{}\"",
            section_prefix, change.field, change.old_value, change.new_value
        );
    }
    eprintln!();

    // Validate before writing
    if let Err(errors) = config.validate() {
        eprintln!("Validation errors:");
        for e in &errors {
            eprintln!("  - {e}");
        }
        eprintln!();
        let proceed = Confirm::new()
            .with_prompt("Save anyway? (config may not work correctly)")
            .default(false)
            .interact()?;
        if !proceed {
            eprintln!("Aborted. No changes written.");
            return Ok(());
        }
    }

    let confirm = Confirm::new()
        .with_prompt(format!("Write changes to {}?", config_path.display()))
        .default(true)
        .interact()?;

    if !confirm {
        eprintln!("Aborted. No changes written.");
        return Ok(());
    }

    let path_str = config_path.display().to_string();
    write_config_with_backup(config, &path_str)?;

    eprintln!("Saved to {}", config_path.display());

    Ok(())
}

// ---------------------------------------------------------------------------
// Field editing helpers
// ---------------------------------------------------------------------------

fn edit_string(label: &str, current: &str) -> Result<String> {
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .interact_text()?;
    Ok(val.trim().to_string())
}

fn edit_optional_string(label: &str, current: &Option<String>) -> Result<Option<String>> {
    let default = current.as_deref().unwrap_or("").to_string();
    let prompt = if current.is_some() {
        format!("{label} (type \"none\" to clear)")
    } else {
        format!("{label} (Enter to skip)")
    };
    let val: String = Input::new()
        .with_prompt(prompt)
        .default(default)
        .allow_empty(true)
        .interact_text()?;
    let trimmed = val.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn edit_bool(label: &str, current: bool) -> Result<bool> {
    let val = Confirm::new()
        .with_prompt(label)
        .default(current)
        .interact()?;
    Ok(val)
}

fn edit_u32(label: &str, current: u32, help: Option<&str>) -> Result<u32> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<u32>()
                .map(|_| ())
                .map_err(|_| "Must be a positive number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

fn edit_u64(label: &str, current: u64, help: Option<&str>) -> Result<u64> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(current.to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<u64>()
                .map(|_| ())
                .map_err(|_| "Must be a positive number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

fn edit_f32(label: &str, current: f32, help: Option<&str>) -> Result<f32> {
    if let Some(h) = help {
        let dim = Style::new().dim();
        eprintln!("  {}", dim.apply_to(h));
    }
    let val: String = Input::new()
        .with_prompt(label)
        .default(format!("{current:.2}"))
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<f32>()
                .map(|_| ())
                .map_err(|_| "Must be a number".to_string())
        })
        .interact_text()?;
    Ok(val.trim().parse().unwrap())
}

fn edit_list(label: &str, current: &[String]) -> Result<Vec<String>> {
    let actions = if current.is_empty() {
        vec!["Add items", "Replace all"]
    } else {
        vec!["Add items", "Remove items", "Replace all"]
    };

    let selection = Select::new()
        .with_prompt(format!("{label} — what do you want to do?"))
        .items(&actions)
        .default(0)
        .interact()?;

    let action = actions[selection];

    match action {
        "Add items" => {
            let raw: String = Input::new()
                .with_prompt("Items to add (comma-separated)")
                .interact_text()?;
            let new_items = parse_csv(&raw);
            let mut result = current.to_vec();
            result.extend(new_items);
            Ok(result)
        }
        "Remove items" => {
            if current.is_empty() {
                eprintln!("Nothing to remove.");
                return Ok(current.to_vec());
            }
            let items: Vec<&str> = current.iter().map(|s| s.as_str()).collect();
            let selections = MultiSelect::new()
                .with_prompt("Select items to remove (Space to toggle, Enter to confirm)")
                .items(&items)
                .interact()?;
            let result: Vec<String> = current
                .iter()
                .enumerate()
                .filter(|(i, _)| !selections.contains(i))
                .map(|(_, s)| s.clone())
                .collect();
            Ok(result)
        }
        "Replace all" => {
            let raw: String = Input::new()
                .with_prompt("New items (comma-separated)")
                .allow_empty(true)
                .interact_text()?;
            Ok(parse_csv(&raw))
        }
        _ => Ok(current.to_vec()),
    }
}

fn edit_duration_minutes(label: &str, current_seconds: u64) -> Result<u64> {
    let dim = Style::new().dim();
    eprintln!(
        "  {}",
        dim.apply_to(format!("Currently: {}", format_duration(current_seconds)))
    );
    eprintln!(
        "  {}",
        dim.apply_to("Enter value in minutes (e.g., 15) or hours (e.g., 3h)")
    );

    let default_display = if current_seconds >= 3600 && current_seconds % 3600 == 0 {
        format!("{}h", current_seconds / 3600)
    } else {
        format!("{}", current_seconds / 60)
    };

    let val: String = Input::new()
        .with_prompt(format!("{label} (minutes, or Nh for hours)"))
        .default(default_display)
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            parse_duration_input(input.trim())
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .interact_text()?;

    parse_duration_input(val.trim())
}

// ---------------------------------------------------------------------------
// Config rendering & writing
// ---------------------------------------------------------------------------

fn render_config(config: &Config) -> String {
    let client_secret_line = match &config.x_api.client_secret {
        Some(secret) => format!("client_secret = \"{}\"", escape_toml(secret)),
        None => "# client_secret = \"your-client-secret-here\"".to_string(),
    };

    let product_url_line = match &config.business.product_url {
        Some(url) => format!("product_url = \"{}\"", escape_toml(url)),
        None => "# product_url = \"https://example.com\"".to_string(),
    };

    let brand_voice_line = match &config.business.brand_voice {
        Some(v) => format!("brand_voice = \"{}\"", escape_toml(v)),
        None => {
            "# brand_voice = \"Friendly technical expert. Casual, occasionally witty.\"".to_string()
        }
    };

    let reply_style_line = match &config.business.reply_style {
        Some(s) => format!("reply_style = \"{}\"", escape_toml(s)),
        None => "# reply_style = \"Lead with genuine help. Only mention our product if relevant.\""
            .to_string(),
    };

    let content_style_line = match &config.business.content_style {
        Some(s) => format!("content_style = \"{}\"", escape_toml(s)),
        None => "# content_style = \"Share practical tips with real examples.\"".to_string(),
    };

    let persona_opinions_line = if config.business.persona_opinions.is_empty() {
        "# persona_opinions = [\"Your strong opinion here\"]".to_string()
    } else {
        format!(
            "persona_opinions = {}",
            format_toml_array(&config.business.persona_opinions)
        )
    };

    let persona_experiences_line = if config.business.persona_experiences.is_empty() {
        "# persona_experiences = [\"Your personal experience here\"]".to_string()
    } else {
        format!(
            "persona_experiences = {}",
            format_toml_array(&config.business.persona_experiences)
        )
    };

    let content_pillars_line = if config.business.content_pillars.is_empty() {
        "# content_pillars = [\"Your core topic here\"]".to_string()
    } else {
        format!(
            "content_pillars = {}",
            format_toml_array(&config.business.content_pillars)
        )
    };

    let targets_section = if config.targets.accounts.is_empty() {
        "# --- Target Accounts ---\n\
         # Monitor specific accounts and reply to their conversations.\n\
         [targets]\n\
         # accounts = [\"elonmusk\", \"levelsio\"]\n\
         accounts = []\n\
         # auto_follow = false"
            .to_string()
    } else {
        format!(
            "# --- Target Accounts ---\n\
             # Monitor specific accounts and reply to their conversations.\n\
             [targets]\n\
             accounts = {accounts}\n\
             max_target_replies_per_day = {max_target}\n\
             auto_follow = {auto_follow}\n\
             follow_warmup_days = {warmup}",
            accounts = format_toml_array(&config.targets.accounts),
            max_target = config.targets.max_target_replies_per_day,
            auto_follow = config.targets.auto_follow,
            warmup = config.targets.follow_warmup_days,
        )
    };

    let api_key_line = match &config.llm.api_key {
        Some(key) => format!("api_key = \"{}\"", escape_toml(key)),
        None => "# api_key = \"your-api-key-here\"".to_string(),
    };

    let base_url_line = match &config.llm.base_url {
        Some(url) => format!("base_url = \"{}\"", escape_toml(url)),
        None => "# base_url = \"http://localhost:11434/v1\"".to_string(),
    };

    format!(
        r#"# =============================================================================
# ReplyGuy Configuration
# =============================================================================
# Generated by `replyguy settings`.
# Edit this file to tune scoring, limits, and intervals.
# Docs: https://github.com/your-org/replyguy
# =============================================================================

# Queue posts for review before posting (use `replyguy approve` to review).
approval_mode = {approval_mode}

# --- X API Credentials ---
# Get your credentials from https://developer.x.com/en/portal/dashboard
[x_api]
client_id = "{client_id}"
{client_secret_line}

# --- Authentication Settings ---
[auth]
# Auth mode: "local_callback" (auto-catch via local server) or "manual" (paste code from browser).
mode = "{auth_mode}"
callback_host = "{callback_host}"
callback_port = {callback_port}

# --- Business Profile ---
# Describe your product so ReplyGuy can find relevant conversations
# and generate on-brand content.
[business]
product_name = "{product_name}"
product_description = "{product_description}"
{product_url_line}
target_audience = "{target_audience}"

# Keywords for tweet discovery (ReplyGuy searches for tweets containing these).
product_keywords = {product_keywords}

# Optional: competitor keywords for discovery.
competitor_keywords = {competitor_keywords}

# Topics for original content generation (tweets and threads).
industry_topics = {industry_topics}

# Brand voice and style — shapes how the bot sounds in all generated content.
{brand_voice_line}
{reply_style_line}
{content_style_line}

# Persona — strong opinions, experiences, and pillars make content more authentic.
{persona_opinions_line}
{persona_experiences_line}
{content_pillars_line}

# --- Scoring Engine ---
# Controls how tweets are scored for reply-worthiness (0-100 scale).
# Weights should sum to ~100 for balanced scoring.
[scoring]
threshold = {threshold}
keyword_relevance_max = {keyword_relevance_max:.1}
follower_count_max = {follower_count_max:.1}
recency_max = {recency_max:.1}
engagement_rate_max = {engagement_rate_max:.1}
reply_count_max = {reply_count_max:.1}
content_type_max = {content_type_max:.1}

# --- Safety Limits ---
# Prevent aggressive posting that could trigger account restrictions.
[limits]
max_replies_per_day = {max_replies_per_day}
max_tweets_per_day = {max_tweets_per_day}
max_threads_per_week = {max_threads_per_week}
min_action_delay_seconds = {min_action_delay_seconds}
max_action_delay_seconds = {max_action_delay_seconds}
max_replies_per_author_per_day = {max_replies_per_author_per_day}
product_mention_ratio = {product_mention_ratio}
banned_phrases = {banned_phrases}

# --- Automation Intervals ---
# How often each loop runs. Shorter intervals use more API quota.
[intervals]
mentions_check_seconds = {mentions_check_seconds}
discovery_search_seconds = {discovery_search_seconds}
content_post_window_seconds = {content_post_window_seconds}
thread_interval_seconds = {thread_interval_seconds}

{targets_section}

# --- LLM Provider ---
# Supported: "openai", "anthropic", "ollama"
[llm]
provider = "{llm_provider}"
{api_key_line}
model = "{llm_model}"
{base_url_line}

# --- Data Storage ---
[storage]
db_path = "{db_path}"
retention_days = {retention_days}

# --- Logging ---
[logging]
# Seconds between periodic status summaries (0 = disabled).
status_interval_seconds = {status_interval_seconds}
"#,
        approval_mode = config.approval_mode,
        client_id = escape_toml(&config.x_api.client_id),
        client_secret_line = client_secret_line,
        auth_mode = escape_toml(&config.auth.mode),
        callback_host = escape_toml(&config.auth.callback_host),
        callback_port = config.auth.callback_port,
        product_name = escape_toml(&config.business.product_name),
        product_description = escape_toml(&config.business.product_description),
        product_url_line = product_url_line,
        target_audience = escape_toml(&config.business.target_audience),
        product_keywords = format_toml_array(&config.business.product_keywords),
        competitor_keywords = format_toml_array(&config.business.competitor_keywords),
        industry_topics = format_toml_array(&config.business.industry_topics),
        brand_voice_line = brand_voice_line,
        reply_style_line = reply_style_line,
        content_style_line = content_style_line,
        persona_opinions_line = persona_opinions_line,
        persona_experiences_line = persona_experiences_line,
        content_pillars_line = content_pillars_line,
        threshold = config.scoring.threshold,
        keyword_relevance_max = config.scoring.keyword_relevance_max,
        follower_count_max = config.scoring.follower_count_max,
        recency_max = config.scoring.recency_max,
        engagement_rate_max = config.scoring.engagement_rate_max,
        reply_count_max = config.scoring.reply_count_max,
        content_type_max = config.scoring.content_type_max,
        max_replies_per_day = config.limits.max_replies_per_day,
        max_tweets_per_day = config.limits.max_tweets_per_day,
        max_threads_per_week = config.limits.max_threads_per_week,
        min_action_delay_seconds = config.limits.min_action_delay_seconds,
        max_action_delay_seconds = config.limits.max_action_delay_seconds,
        max_replies_per_author_per_day = config.limits.max_replies_per_author_per_day,
        product_mention_ratio = config.limits.product_mention_ratio,
        banned_phrases = format_toml_array(&config.limits.banned_phrases),
        mentions_check_seconds = config.intervals.mentions_check_seconds,
        discovery_search_seconds = config.intervals.discovery_search_seconds,
        content_post_window_seconds = config.intervals.content_post_window_seconds,
        thread_interval_seconds = config.intervals.thread_interval_seconds,
        targets_section = targets_section,
        llm_provider = escape_toml(&config.llm.provider),
        api_key_line = api_key_line,
        llm_model = escape_toml(&config.llm.model),
        base_url_line = base_url_line,
        db_path = escape_toml(&config.storage.db_path),
        retention_days = config.storage.retention_days,
        status_interval_seconds = config.logging.status_interval_seconds,
    )
}

fn write_config_with_backup(config: &Config, config_path: &str) -> Result<()> {
    let path = expand_tilde(config_path);

    // Create backup
    if path.exists() {
        let backup_path = path.with_extension("toml.bak");
        fs::copy(&path, &backup_path)
            .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;
    }

    let toml_str = render_config(config);
    fs::write(&path, toml_str)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    Ok(())
}

fn validate_config(config: &Config) -> Result<()> {
    if let Err(errors) = config.validate() {
        let messages: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
        bail!("Configuration validation failed:\n{}", messages.join("\n"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Change tracking
// ---------------------------------------------------------------------------

struct Change {
    section: String,
    field: String,
    old_value: String,
    new_value: String,
}

struct ChangeTracker {
    changes: Vec<Change>,
}

impl ChangeTracker {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn record(&mut self, section: &str, field: &str, old_value: &str, new_value: &str) {
        if old_value != new_value {
            self.changes.push(Change {
                section: section.to_string(),
                field: field.to_string(),
                old_value: old_value.to_string(),
                new_value: new_value.to_string(),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

fn mask_secret(secret: &Option<String>) -> String {
    match secret {
        Some(s) if s.len() > 8 => {
            format!("{}...{}", &s[..4], &s[s.len() - 4..])
        }
        Some(s) if !s.is_empty() => "****".to_string(),
        Some(_) => "(empty)".to_string(),
        None => "(not set)".to_string(),
    }
}

fn format_list(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".to_string()
    } else {
        items.join(", ")
    }
}

fn format_duration(seconds: u64) -> String {
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

fn parse_duration_input(input: &str) -> Result<u64> {
    let input = input.trim().to_lowercase();
    if let Some(hours) = input.strip_suffix('h') {
        let h: u64 = hours.trim().parse().context("Invalid number of hours")?;
        Ok(h * 3600)
    } else if let Some(days) = input.strip_suffix('d') {
        let d: u64 = days.trim().parse().context("Invalid number of days")?;
        Ok(d * 86400)
    } else {
        let mins: u64 = input
            .parse()
            .context("Enter a number (minutes), or Nh for hours, Nd for days")?;
        Ok(mins * 60)
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => bail!("Invalid boolean value: {value} (use true/false, yes/no, 1/0)"),
    }
}

fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn format_toml_array(items: &[String]) -> String {
    let inner: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", escape_toml(s)))
        .collect();
    format!("[{}]", inner.join(", "))
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_secret_long() {
        let s = Some("sk-1234567890abcdef".to_string());
        assert_eq!(mask_secret(&s), "sk-1...cdef");
    }

    #[test]
    fn mask_secret_short() {
        let s = Some("abc".to_string());
        assert_eq!(mask_secret(&s), "****");
    }

    #[test]
    fn mask_secret_none() {
        assert_eq!(mask_secret(&None), "(not set)");
    }

    #[test]
    fn format_list_empty() {
        assert_eq!(format_list(&[]), "(none)");
    }

    #[test]
    fn format_list_items() {
        let items = vec!["a".to_string(), "b".to_string()];
        assert_eq!(format_list(&items), "a, b");
    }

    #[test]
    fn format_duration_seconds() {
        assert_eq!(format_duration(30), "30 seconds");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(300), "5 min");
        assert_eq!(format_duration(330), "5 min 30 sec");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(3600), "1 hour");
        assert_eq!(format_duration(10800), "3 hours");
        assert_eq!(format_duration(5400), "1 hour 30 min");
    }

    #[test]
    fn format_duration_days() {
        assert_eq!(format_duration(86400), "1 day");
        assert_eq!(format_duration(604800), "7 days");
        assert_eq!(format_duration(90000), "1 day 1 hour");
    }

    #[test]
    fn parse_duration_input_minutes() {
        assert_eq!(parse_duration_input("15").unwrap(), 900);
    }

    #[test]
    fn parse_duration_input_hours() {
        assert_eq!(parse_duration_input("3h").unwrap(), 10800);
        assert_eq!(parse_duration_input("3H").unwrap(), 10800);
    }

    #[test]
    fn parse_duration_input_days() {
        assert_eq!(parse_duration_input("7d").unwrap(), 604800);
    }

    #[test]
    fn parse_bool_values() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("yes").unwrap());
        assert!(parse_bool("1").unwrap());
        assert!(parse_bool("on").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("no").unwrap());
        assert!(!parse_bool("0").unwrap());
        assert!(!parse_bool("off").unwrap());
        assert!(parse_bool("maybe").is_err());
    }

    #[test]
    fn change_tracker_only_records_changes() {
        let mut tracker = ChangeTracker::new();
        tracker.record("section", "field", "old", "new");
        tracker.record("section", "field", "same", "same");
        assert_eq!(tracker.changes.len(), 1);
    }

    #[test]
    fn render_config_roundtrip() {
        let mut config = Config::default();
        config.business.product_name = "TestProduct".to_string();
        config.business.product_description = "A test product".to_string();
        config.business.target_audience = "developers".to_string();
        config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];
        config.business.industry_topics = vec!["Rust dev".to_string()];
        config.x_api.client_id = "test-client-id".to_string();
        config.llm.provider = "ollama".to_string();
        config.llm.model = "llama3.2".to_string();

        let toml_str = render_config(&config);
        let parsed: Config = toml::from_str(&toml_str).expect("rendered config should parse");

        assert_eq!(parsed.business.product_name, "TestProduct");
        assert_eq!(parsed.x_api.client_id, "test-client-id");
        assert_eq!(parsed.llm.provider, "ollama");
        assert_eq!(parsed.scoring.threshold, config.scoring.threshold);
        assert_eq!(
            parsed.limits.max_replies_per_day,
            config.limits.max_replies_per_day
        );
    }

    #[test]
    fn render_config_with_all_fields() {
        let mut config = Config::default();
        config.business.product_name = "FullApp".to_string();
        config.business.product_description = "Complete app".to_string();
        config.business.product_url = Some("https://example.com".to_string());
        config.business.target_audience = "everyone".to_string();
        config.business.product_keywords = vec!["test".to_string()];
        config.business.competitor_keywords = vec!["rival".to_string()];
        config.business.industry_topics = vec!["topic".to_string()];
        config.business.brand_voice = Some("Friendly".to_string());
        config.business.reply_style = Some("Helpful".to_string());
        config.business.content_style = Some("Practical".to_string());
        config.business.persona_opinions = vec!["Strong opinion".to_string()];
        config.business.persona_experiences = vec!["Built stuff".to_string()];
        config.business.content_pillars = vec!["Dev tools".to_string()];
        config.x_api.client_id = "cid".to_string();
        config.x_api.client_secret = Some("secret".to_string());
        config.llm.provider = "openai".to_string();
        config.llm.api_key = Some("sk-test".to_string());
        config.llm.model = "gpt-4o-mini".to_string();
        config.llm.base_url = Some("https://api.openai.com".to_string());
        config.targets.accounts = vec!["user1".to_string()];
        config.targets.auto_follow = true;
        config.approval_mode = true;

        let toml_str = render_config(&config);
        let parsed: Config = toml::from_str(&toml_str).expect("rendered config should parse");

        assert_eq!(
            parsed.business.product_url,
            Some("https://example.com".to_string())
        );
        assert_eq!(parsed.business.brand_voice, Some("Friendly".to_string()));
        assert_eq!(parsed.targets.accounts, vec!["user1"]);
        assert!(parsed.targets.auto_follow);
        assert!(parsed.approval_mode);
        assert_eq!(parsed.llm.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn render_config_escapes_special_chars() {
        let mut config = Config::default();
        config.business.product_name = "My \"App\"".to_string();
        config.business.product_description = "line\\break".to_string();
        config.business.target_audience = "devs".to_string();
        config.business.product_keywords = vec!["say \"hi\"".to_string()];
        config.business.industry_topics = vec!["topic".to_string()];
        config.x_api.client_id = "id-\"test\"".to_string();
        config.llm.provider = "ollama".to_string();
        config.llm.model = "llama3.2".to_string();

        let toml_str = render_config(&config);
        let parsed: Config =
            toml::from_str(&toml_str).expect("config with special chars should parse");

        assert_eq!(parsed.business.product_name, "My \"App\"");
        assert_eq!(parsed.business.product_description, "line\\break");
        assert_eq!(parsed.x_api.client_id, "id-\"test\"");
    }
}
