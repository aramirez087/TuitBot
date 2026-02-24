use std::path::Path;

use anyhow::Result;
use console::Style;
use dialoguer::{Input, Select};
use tuitbot_core::config::Config;

use super::helpers::*;
use super::render::save_flow;
use super::show::{format_duration, format_list, mask_secret};

// ---------------------------------------------------------------------------
// edit_and_record helpers — reduce 6-8 line blocks to one-liners
// ---------------------------------------------------------------------------

fn print_category_header(title: &str) {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let underline = "─".repeat(title.len());
    eprintln!();
    eprintln!("{}", bold.apply_to(title));
    eprintln!("{}", dim.apply_to(underline));
}

fn edit_and_record_string(
    tracker: &mut ChangeTracker,
    field: &mut String,
    section: &str,
    name: &str,
    label: &str,
) -> Result<()> {
    let new_val = edit_string(label, field)?;
    tracker.record(section, name, field, &new_val);
    *field = new_val;
    Ok(())
}

fn edit_and_record_opt_string(
    tracker: &mut ChangeTracker,
    field: &mut Option<String>,
    section: &str,
    name: &str,
    label: &str,
) -> Result<()> {
    let new_val = edit_optional_string(label, field)?;
    let old_display = field.as_deref().unwrap_or("(none)");
    let new_display = new_val.as_deref().unwrap_or("(none)");
    tracker.record(section, name, old_display, new_display);
    *field = new_val;
    Ok(())
}

fn edit_and_record_list(
    tracker: &mut ChangeTracker,
    field: &mut Vec<String>,
    section: &str,
    name: &str,
    label: &str,
) -> Result<()> {
    let new_val = edit_list(label, field)?;
    let old_display = field.join(", ");
    let new_display = new_val.join(", ");
    tracker.record(section, name, &old_display, &new_display);
    *field = new_val;
    Ok(())
}

fn edit_and_record_u32(
    tracker: &mut ChangeTracker,
    field: &mut u32,
    section: &str,
    name: &str,
    label: &str,
    help: Option<&str>,
) -> Result<()> {
    let new_val = edit_u32(label, *field, help)?;
    tracker.record(section, name, &field.to_string(), &new_val.to_string());
    *field = new_val;
    Ok(())
}

fn edit_and_record_u64(
    tracker: &mut ChangeTracker,
    field: &mut u64,
    section: &str,
    name: &str,
    label: &str,
    help: Option<&str>,
) -> Result<()> {
    let new_val = edit_u64(label, *field, help)?;
    tracker.record(section, name, &field.to_string(), &new_val.to_string());
    *field = new_val;
    Ok(())
}

fn edit_and_record_f32(
    tracker: &mut ChangeTracker,
    field: &mut f32,
    section: &str,
    name: &str,
    label: &str,
    help: Option<&str>,
) -> Result<()> {
    let new_val = edit_f32(label, *field, help)?;
    tracker.record(
        section,
        name,
        &format!("{:.1}", *field),
        &format!("{:.1}", new_val),
    );
    *field = new_val;
    Ok(())
}

fn edit_and_record_bool(
    tracker: &mut ChangeTracker,
    field: &mut bool,
    section: &str,
    name: &str,
    label: &str,
) -> Result<()> {
    let new_val = edit_bool(label, *field)?;
    if new_val != *field {
        tracker.record(section, name, &field.to_string(), &new_val.to_string());
        *field = new_val;
    }
    Ok(())
}

fn edit_and_record_duration(
    tracker: &mut ChangeTracker,
    field: &mut u64,
    section: &str,
    name: &str,
    label: &str,
) -> Result<()> {
    let new_val = edit_duration_minutes(label, *field)?;
    tracker.record(section, name, &field.to_string(), &new_val.to_string());
    *field = new_val;
    Ok(())
}

// ---------------------------------------------------------------------------
// Interactive menu
// ---------------------------------------------------------------------------

pub(super) fn interactive_menu(config: &mut Config, config_path: &Path) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let mut tracker = ChangeTracker::new();

    loop {
        eprintln!();
        eprintln!("{}", bold.apply_to("Tuitbot Settings"));
        eprintln!("{}", dim.apply_to("─────────────────"));
        eprintln!();

        let categories = &[
            "Your Product        — name, description, keywords, audience",
            "Brand Voice         — personality, reply style, content style",
            "Persona             — opinions, experiences, content pillars",
            "AI Provider         — LLM provider, API key, model",
            "X API Credentials   — client ID, client secret",
            "Target Accounts     — accounts to monitor and engage with",
            "Posting Limits      — replies/tweets/threads per day",
            "Scoring             — how picky the bot is about which tweets to reply to",
            "Timing              — how often the bot checks for new tweets",
            "Approval Mode       — review posts before they go live",
            "Schedule            — timezone, active hours, active days",
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
            10 => edit_category_schedule(config, &mut tracker)?,
            11 => edit_category_storage(config, &mut tracker)?,
            12 => break, // Save & Exit
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

pub(super) fn edit_category_product(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Your Product");

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
        0 => edit_and_record_string(
            tracker,
            &mut config.business.product_name,
            "business",
            "product_name",
            "Product name",
        )?,
        1 => edit_and_record_string(
            tracker,
            &mut config.business.product_description,
            "business",
            "product_description",
            "Description",
        )?,
        2 => edit_and_record_opt_string(
            tracker,
            &mut config.business.product_url,
            "business",
            "product_url",
            "Product URL",
        )?,
        3 => edit_and_record_string(
            tracker,
            &mut config.business.target_audience,
            "business",
            "target_audience",
            "Target audience",
        )?,
        4 => edit_and_record_list(
            tracker,
            &mut config.business.product_keywords,
            "business",
            "product_keywords",
            "Discovery keywords",
        )?,
        5 => edit_and_record_list(
            tracker,
            &mut config.business.competitor_keywords,
            "business",
            "competitor_keywords",
            "Competitor keywords",
        )?,
        6 => edit_and_record_list(
            tracker,
            &mut config.business.industry_topics,
            "business",
            "industry_topics",
            "Content topics",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_voice(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("Brand Voice");

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
        0 => edit_and_record_opt_string(
            tracker,
            &mut config.business.brand_voice,
            "business",
            "brand_voice",
            "Personality",
        )?,
        1 => edit_and_record_opt_string(
            tracker,
            &mut config.business.reply_style,
            "business",
            "reply_style",
            "Reply style",
        )?,
        2 => edit_and_record_opt_string(
            tracker,
            &mut config.business.content_style,
            "business",
            "content_style",
            "Content style",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_persona(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Persona");

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
        0 => edit_and_record_list(
            tracker,
            &mut config.business.persona_opinions,
            "business",
            "persona_opinions",
            "Opinions",
        )?,
        1 => edit_and_record_list(
            tracker,
            &mut config.business.persona_experiences,
            "business",
            "persona_experiences",
            "Experiences",
        )?,
        2 => edit_and_record_list(
            tracker,
            &mut config.business.content_pillars,
            "business",
            "content_pillars",
            "Content pillars",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_llm(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("AI Provider");

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
        2 => edit_and_record_string(tracker, &mut config.llm.model, "llm", "model", "Model")?,
        3 => edit_and_record_opt_string(
            tracker,
            &mut config.llm.base_url,
            "llm",
            "base_url",
            "Base URL",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_xapi(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("X API Credentials");

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
        0 => edit_and_record_string(
            tracker,
            &mut config.x_api.client_id,
            "x_api",
            "client_id",
            "Client ID",
        )?,
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

pub(super) fn edit_category_targets(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Target Accounts");

    let fields = &[
        format!(
            "Accounts:          {}",
            format_list(&config.targets.accounts)
        ),
        format!(
            "Max replies/day:   {}",
            config.targets.max_target_replies_per_day
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
        1 => edit_and_record_u32(
            tracker,
            &mut config.targets.max_target_replies_per_day,
            "targets",
            "max_target_replies_per_day",
            "Max target replies per day",
            None,
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_limits(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("Posting Limits");

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
        0 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_replies_per_day,
            "limits",
            "max_replies_per_day",
            "Max replies per day",
            Some("How many tweets to reply to per day"),
        )?,
        1 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_tweets_per_day,
            "limits",
            "max_tweets_per_day",
            "Max tweets per day",
            Some("How many original tweets to post per day"),
        )?,
        2 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_threads_per_week,
            "limits",
            "max_threads_per_week",
            "Max threads per week",
            Some("How many educational threads to post per week"),
        )?,
        3 => edit_and_record_u64(
            tracker,
            &mut config.limits.min_action_delay_seconds,
            "limits",
            "min_action_delay_seconds",
            "Min action delay (seconds)",
            Some("Minimum wait between actions"),
        )?,
        4 => edit_and_record_u64(
            tracker,
            &mut config.limits.max_action_delay_seconds,
            "limits",
            "max_action_delay_seconds",
            "Max action delay (seconds)",
            Some("Maximum wait between actions"),
        )?,
        5 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_replies_per_author_per_day,
            "limits",
            "max_replies_per_author_per_day",
            "Max replies per author per day",
            Some("Avoid spamming the same person"),
        )?,
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
        7 => edit_and_record_list(
            tracker,
            &mut config.limits.banned_phrases,
            "limits",
            "banned_phrases",
            "Banned phrases",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_scoring(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Scoring");

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
        0 => edit_and_record_u32(
            tracker,
            &mut config.scoring.threshold,
            "scoring",
            "threshold",
            "Scoring threshold (0-100)",
            Some("Higher = replies only to very relevant tweets"),
        )?,
        1 => edit_and_record_f32(
            tracker,
            &mut config.scoring.keyword_relevance_max,
            "scoring",
            "keyword_relevance_max",
            "Keyword relevance max pts",
            Some("Points for matching discovery keywords"),
        )?,
        2 => edit_and_record_f32(
            tracker,
            &mut config.scoring.follower_count_max,
            "scoring",
            "follower_count_max",
            "Follower count max pts",
            Some("Points for author follower count (bell curve)"),
        )?,
        3 => edit_and_record_f32(
            tracker,
            &mut config.scoring.recency_max,
            "scoring",
            "recency_max",
            "Recency max pts",
            Some("Points for how recently the tweet was posted"),
        )?,
        4 => edit_and_record_f32(
            tracker,
            &mut config.scoring.engagement_rate_max,
            "scoring",
            "engagement_rate_max",
            "Engagement rate max pts",
            Some("Points for tweet engagement level"),
        )?,
        5 => edit_and_record_f32(
            tracker,
            &mut config.scoring.reply_count_max,
            "scoring",
            "reply_count_max",
            "Reply count max pts",
            Some("Points for few existing replies (less competition)"),
        )?,
        6 => edit_and_record_f32(
            tracker,
            &mut config.scoring.content_type_max,
            "scoring",
            "content_type_max",
            "Content type max pts",
            Some("Points for text-only original tweets"),
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_timing(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("Timing");

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
        0 => edit_and_record_duration(
            tracker,
            &mut config.intervals.mentions_check_seconds,
            "intervals",
            "mentions_check_seconds",
            "How often to check mentions",
        )?,
        1 => edit_and_record_duration(
            tracker,
            &mut config.intervals.discovery_search_seconds,
            "intervals",
            "discovery_search_seconds",
            "How often to search for tweets",
        )?,
        2 => edit_and_record_duration(
            tracker,
            &mut config.intervals.content_post_window_seconds,
            "intervals",
            "content_post_window_seconds",
            "Content post window",
        )?,
        3 => edit_and_record_duration(
            tracker,
            &mut config.intervals.thread_interval_seconds,
            "intervals",
            "thread_interval_seconds",
            "Thread interval",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_approval(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Approval Mode");

    eprintln!(
        "  Currently: {}",
        if config.approval_mode {
            "enabled (posts are queued for review)"
        } else {
            "disabled (posts go live immediately)"
        }
    );

    edit_and_record_bool(
        tracker,
        &mut config.approval_mode,
        "",
        "approval_mode",
        "Queue posts for review before posting?",
    )
}

pub(super) fn edit_category_schedule(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Schedule");

    let fields = &[
        format!("Timezone:      {}", config.schedule.timezone),
        format!("Hours start:   {}:00", config.schedule.active_hours_start),
        format!("Hours end:     {}:00", config.schedule.active_hours_end),
        format!(
            "Active days:   {}",
            format_list(&config.schedule.active_days)
        ),
        format!(
            "Tweet times:   {}",
            if config.schedule.preferred_times.is_empty() {
                "(interval mode)".to_string()
            } else {
                format_list(&config.schedule.preferred_times)
            }
        ),
        format!(
            "Thread day:    {}",
            config
                .schedule
                .thread_preferred_day
                .as_deref()
                .unwrap_or("(interval mode)")
        ),
        format!("Thread time:   {}", config.schedule.thread_preferred_time),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val: String = Input::new()
                .with_prompt("Timezone (IANA name)")
                .default(config.schedule.timezone.clone())
                .validate_with(|input: &String| -> std::result::Result<(), String> {
                    input
                        .trim()
                        .parse::<chrono_tz::Tz>()
                        .map(|_| ())
                        .map_err(|_| format!("Unknown timezone: {input}"))
                })
                .interact_text()?;
            let new_val = new_val.trim().to_string();
            tracker.record("schedule", "timezone", &config.schedule.timezone, &new_val);
            config.schedule.timezone = new_val;
        }
        1 => {
            let new_val = edit_u8(
                "Active hours start (0-23)",
                config.schedule.active_hours_start,
            )?;
            tracker.record(
                "schedule",
                "active_hours_start",
                &config.schedule.active_hours_start.to_string(),
                &new_val.to_string(),
            );
            config.schedule.active_hours_start = new_val;
        }
        2 => {
            let new_val = edit_u8("Active hours end (0-23)", config.schedule.active_hours_end)?;
            tracker.record(
                "schedule",
                "active_hours_end",
                &config.schedule.active_hours_end.to_string(),
                &new_val.to_string(),
            );
            config.schedule.active_hours_end = new_val;
        }
        3 => edit_and_record_list(
            tracker,
            &mut config.schedule.active_days,
            "schedule",
            "active_days",
            "Active days",
        )?,
        4 => edit_and_record_list(
            tracker,
            &mut config.schedule.preferred_times,
            "schedule",
            "preferred_times",
            "Tweet times (HH:MM, or \"auto\" for defaults)",
        )?,
        5 => edit_and_record_opt_string(
            tracker,
            &mut config.schedule.thread_preferred_day,
            "schedule",
            "thread_preferred_day",
            "Thread day (Mon-Sun, empty for interval mode)",
        )?,
        6 => edit_and_record_string(
            tracker,
            &mut config.schedule.thread_preferred_time,
            "schedule",
            "thread_preferred_time",
            "Thread time (HH:MM)",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(super) fn edit_category_storage(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Storage & Logging");

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
        0 => edit_and_record_string(
            tracker,
            &mut config.storage.db_path,
            "storage",
            "db_path",
            "Database path",
        )?,
        1 => edit_and_record_u32(
            tracker,
            &mut config.storage.retention_days,
            "storage",
            "retention_days",
            "Data retention (days)",
            Some("How many days of data to keep"),
        )?,
        2 => edit_and_record_u64(
            tracker,
            &mut config.logging.status_interval_seconds,
            "logging",
            "status_interval_seconds",
            "Status interval (seconds, 0 = disabled)",
            Some("How often to print status summaries (0 = disabled)"),
        )?,
        _ => {} // Back
    }

    Ok(())
}
