use std::path::Path;

use anyhow::{bail, Context, Result};
use console::Style;
use tuitbot_core::config::Config;

use super::helpers::{parse_bool, parse_csv, ChangeTracker};
use super::render::{validate_config, write_config_with_backup};

// ---------------------------------------------------------------------------
// Setter helpers — reduce 5-10 line match arms to one-liners
// ---------------------------------------------------------------------------

fn set_string(
    tracker: &mut ChangeTracker,
    field: &mut String,
    section: &str,
    name: &str,
    value: &str,
) {
    tracker.record(section, name, field, value);
    *field = value.to_string();
}

fn set_opt_string(
    tracker: &mut ChangeTracker,
    field: &mut Option<String>,
    section: &str,
    name: &str,
    value: &str,
) {
    let old = field.as_deref().unwrap_or("(none)").to_string();
    if value == "none" || value.is_empty() {
        *field = None;
        tracker.record(section, name, &old, "(none)");
    } else {
        *field = Some(value.to_string());
        tracker.record(section, name, &old, value);
    }
}

fn set_csv(
    tracker: &mut ChangeTracker,
    field: &mut Vec<String>,
    section: &str,
    name: &str,
    value: &str,
) {
    let old = field.join(", ");
    *field = parse_csv(value);
    tracker.record(section, name, &old, value);
}

fn set_u32(
    tracker: &mut ChangeTracker,
    field: &mut u32,
    section: &str,
    name: &str,
    value: &str,
) -> Result<()> {
    let v: u32 = value.parse().context("must be a positive number")?;
    tracker.record(section, name, &field.to_string(), value);
    *field = v;
    Ok(())
}

fn set_u64(
    tracker: &mut ChangeTracker,
    field: &mut u64,
    section: &str,
    name: &str,
    value: &str,
) -> Result<()> {
    let v: u64 = value.parse().context("must be a positive number")?;
    tracker.record(section, name, &field.to_string(), value);
    *field = v;
    Ok(())
}

fn set_f32(
    tracker: &mut ChangeTracker,
    field: &mut f32,
    section: &str,
    name: &str,
    value: &str,
) -> Result<()> {
    let v: f32 = value.parse().context("must be a number")?;
    tracker.record(section, name, &format!("{:.1}", *field), value);
    *field = v;
    Ok(())
}

fn set_f32_fmt2(
    tracker: &mut ChangeTracker,
    field: &mut f32,
    section: &str,
    name: &str,
    value: &str,
    ctx: &str,
) -> Result<()> {
    let v: f32 = value.parse().context(ctx.to_string())?;
    tracker.record(section, name, &format!("{:.2}", *field), value);
    *field = v;
    Ok(())
}

fn set_bool(
    tracker: &mut ChangeTracker,
    field: &mut bool,
    section: &str,
    name: &str,
    value: &str,
) -> Result<()> {
    let v: bool = parse_bool(value)?;
    tracker.record(section, name, &field.to_string(), value);
    *field = v;
    Ok(())
}

fn set_u8_range(
    tracker: &mut ChangeTracker,
    field: &mut u8,
    section: &str,
    name: &str,
    value: &str,
    max: u8,
) -> Result<()> {
    let v: u8 = value.parse().context("must be 0-23")?;
    if v > max {
        bail!("{name} must be 0-{max}");
    }
    tracker.record(section, name, &field.to_string(), value);
    *field = v;
    Ok(())
}

// ---------------------------------------------------------------------------
// --set direct mode
// ---------------------------------------------------------------------------

pub(super) fn set_direct(config: &mut Config, kv: &str, config_path: &Path) -> Result<()> {
    let (key, value) = kv.split_once('=').ok_or_else(|| {
        anyhow::anyhow!("Invalid format. Use --set KEY=VALUE (e.g., --set scoring.threshold=80)")
    })?;

    let key = key.trim();
    let value = value.trim();

    let mut tracker = ChangeTracker::new();

    match key {
        // Business — strings
        "business.product_name" => set_string(
            &mut tracker,
            &mut config.business.product_name,
            "business",
            "product_name",
            value,
        ),
        "business.product_description" => set_string(
            &mut tracker,
            &mut config.business.product_description,
            "business",
            "product_description",
            value,
        ),
        "business.target_audience" => set_string(
            &mut tracker,
            &mut config.business.target_audience,
            "business",
            "target_audience",
            value,
        ),

        // Business — optional strings
        "business.product_url" => set_opt_string(
            &mut tracker,
            &mut config.business.product_url,
            "business",
            "product_url",
            value,
        ),
        "business.brand_voice" => set_opt_string(
            &mut tracker,
            &mut config.business.brand_voice,
            "business",
            "brand_voice",
            value,
        ),
        "business.reply_style" => set_opt_string(
            &mut tracker,
            &mut config.business.reply_style,
            "business",
            "reply_style",
            value,
        ),
        "business.content_style" => set_opt_string(
            &mut tracker,
            &mut config.business.content_style,
            "business",
            "content_style",
            value,
        ),

        // Business — CSV lists
        "business.product_keywords" => set_csv(
            &mut tracker,
            &mut config.business.product_keywords,
            "business",
            "product_keywords",
            value,
        ),
        "business.competitor_keywords" => set_csv(
            &mut tracker,
            &mut config.business.competitor_keywords,
            "business",
            "competitor_keywords",
            value,
        ),
        "business.industry_topics" => set_csv(
            &mut tracker,
            &mut config.business.industry_topics,
            "business",
            "industry_topics",
            value,
        ),
        "business.persona_opinions" => set_csv(
            &mut tracker,
            &mut config.business.persona_opinions,
            "business",
            "persona_opinions",
            value,
        ),
        "business.persona_experiences" => set_csv(
            &mut tracker,
            &mut config.business.persona_experiences,
            "business",
            "persona_experiences",
            value,
        ),
        "business.content_pillars" => set_csv(
            &mut tracker,
            &mut config.business.content_pillars,
            "business",
            "content_pillars",
            value,
        ),

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
        "scoring.keyword_relevance_max" => set_f32(
            &mut tracker,
            &mut config.scoring.keyword_relevance_max,
            "scoring",
            "keyword_relevance_max",
            value,
        )?,
        "scoring.follower_count_max" => set_f32(
            &mut tracker,
            &mut config.scoring.follower_count_max,
            "scoring",
            "follower_count_max",
            value,
        )?,
        "scoring.recency_max" => set_f32(
            &mut tracker,
            &mut config.scoring.recency_max,
            "scoring",
            "recency_max",
            value,
        )?,
        "scoring.engagement_rate_max" => set_f32(
            &mut tracker,
            &mut config.scoring.engagement_rate_max,
            "scoring",
            "engagement_rate_max",
            value,
        )?,
        "scoring.reply_count_max" => set_f32(
            &mut tracker,
            &mut config.scoring.reply_count_max,
            "scoring",
            "reply_count_max",
            value,
        )?,
        "scoring.content_type_max" => set_f32(
            &mut tracker,
            &mut config.scoring.content_type_max,
            "scoring",
            "content_type_max",
            value,
        )?,

        // Limits
        "limits.max_replies_per_day" => set_u32(
            &mut tracker,
            &mut config.limits.max_replies_per_day,
            "limits",
            "max_replies_per_day",
            value,
        )?,
        "limits.max_tweets_per_day" => set_u32(
            &mut tracker,
            &mut config.limits.max_tweets_per_day,
            "limits",
            "max_tweets_per_day",
            value,
        )?,
        "limits.max_threads_per_week" => set_u32(
            &mut tracker,
            &mut config.limits.max_threads_per_week,
            "limits",
            "max_threads_per_week",
            value,
        )?,
        "limits.min_action_delay_seconds" => set_u64(
            &mut tracker,
            &mut config.limits.min_action_delay_seconds,
            "limits",
            "min_action_delay_seconds",
            value,
        )?,
        "limits.max_action_delay_seconds" => set_u64(
            &mut tracker,
            &mut config.limits.max_action_delay_seconds,
            "limits",
            "max_action_delay_seconds",
            value,
        )?,
        "limits.max_replies_per_author_per_day" => set_u32(
            &mut tracker,
            &mut config.limits.max_replies_per_author_per_day,
            "limits",
            "max_replies_per_author_per_day",
            value,
        )?,
        "limits.product_mention_ratio" => set_f32_fmt2(
            &mut tracker,
            &mut config.limits.product_mention_ratio,
            "limits",
            "product_mention_ratio",
            value,
            "must be a number 0.0-1.0",
        )?,
        "limits.banned_phrases" => set_csv(
            &mut tracker,
            &mut config.limits.banned_phrases,
            "limits",
            "banned_phrases",
            value,
        ),

        // Intervals
        "intervals.mentions_check_seconds" => set_u64(
            &mut tracker,
            &mut config.intervals.mentions_check_seconds,
            "intervals",
            "mentions_check_seconds",
            value,
        )?,
        "intervals.discovery_search_seconds" => set_u64(
            &mut tracker,
            &mut config.intervals.discovery_search_seconds,
            "intervals",
            "discovery_search_seconds",
            value,
        )?,
        "intervals.content_post_window_seconds" => set_u64(
            &mut tracker,
            &mut config.intervals.content_post_window_seconds,
            "intervals",
            "content_post_window_seconds",
            value,
        )?,
        "intervals.thread_interval_seconds" => set_u64(
            &mut tracker,
            &mut config.intervals.thread_interval_seconds,
            "intervals",
            "thread_interval_seconds",
            value,
        )?,

        // Targets
        "targets.accounts" => set_csv(
            &mut tracker,
            &mut config.targets.accounts,
            "targets",
            "accounts",
            value,
        ),
        "targets.max_target_replies_per_day" => set_u32(
            &mut tracker,
            &mut config.targets.max_target_replies_per_day,
            "targets",
            "max_target_replies_per_day",
            value,
        )?,

        // LLM
        "llm.provider" => set_string(
            &mut tracker,
            &mut config.llm.provider,
            "llm",
            "provider",
            value,
        ),
        "llm.api_key" => {
            // Special: mask secret in change tracking
            tracker.record("llm", "api_key", "(hidden)", "(updated)");
            config.llm.api_key = Some(value.to_string());
        }
        "llm.model" => set_string(&mut tracker, &mut config.llm.model, "llm", "model", value),
        "llm.base_url" => set_opt_string(
            &mut tracker,
            &mut config.llm.base_url,
            "llm",
            "base_url",
            value,
        ),

        // X API
        "x_api.client_id" => set_string(
            &mut tracker,
            &mut config.x_api.client_id,
            "x_api",
            "client_id",
            value,
        ),
        "x_api.client_secret" => {
            // Special: mask secret in change tracking
            tracker.record("x_api", "client_secret", "(hidden)", "(updated)");
            config.x_api.client_secret = Some(value.to_string());
        }

        // Approval mode
        "approval_mode" => set_bool(
            &mut tracker,
            &mut config.approval_mode,
            "",
            "approval_mode",
            value,
        )?,

        // Storage
        "storage.db_path" => set_string(
            &mut tracker,
            &mut config.storage.db_path,
            "storage",
            "db_path",
            value,
        ),
        "storage.retention_days" => set_u32(
            &mut tracker,
            &mut config.storage.retention_days,
            "storage",
            "retention_days",
            value,
        )?,

        // Logging
        "logging.status_interval_seconds" => set_u64(
            &mut tracker,
            &mut config.logging.status_interval_seconds,
            "logging",
            "status_interval_seconds",
            value,
        )?,

        // Schedule
        "schedule.timezone" => {
            // Special: chrono_tz validation
            value
                .trim()
                .parse::<chrono_tz::Tz>()
                .map_err(|_| anyhow::anyhow!("Unknown timezone: {value}"))?;
            tracker.record("schedule", "timezone", &config.schedule.timezone, value);
            config.schedule.timezone = value.to_string();
        }
        "schedule.active_hours_start" => set_u8_range(
            &mut tracker,
            &mut config.schedule.active_hours_start,
            "schedule",
            "active_hours_start",
            value,
            23,
        )?,
        "schedule.active_hours_end" => set_u8_range(
            &mut tracker,
            &mut config.schedule.active_hours_end,
            "schedule",
            "active_hours_end",
            value,
            23,
        )?,
        "schedule.active_days" => set_csv(
            &mut tracker,
            &mut config.schedule.active_days,
            "schedule",
            "active_days",
            value,
        ),
        "schedule.preferred_times" => set_csv(
            &mut tracker,
            &mut config.schedule.preferred_times,
            "schedule",
            "preferred_times",
            value,
        ),
        "schedule.thread_preferred_day" => set_opt_string(
            &mut tracker,
            &mut config.schedule.thread_preferred_day,
            "schedule",
            "thread_preferred_day",
            value,
        ),
        "schedule.thread_preferred_time" => set_string(
            &mut tracker,
            &mut config.schedule.thread_preferred_time,
            "schedule",
            "thread_preferred_time",
            value,
        ),

        _ => bail!(
            "Unknown setting: {key}\n\
             Use 'tuitbot settings --show' to see all available settings."
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
