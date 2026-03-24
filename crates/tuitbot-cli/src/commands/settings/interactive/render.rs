use anyhow::Result;
use dialoguer::{Input, Select};
use tuitbot_core::config::Config;

use super::{
    edit_and_record_bool, edit_and_record_duration, edit_and_record_f32, edit_and_record_list,
    edit_and_record_opt_string, edit_and_record_string, edit_and_record_u32, edit_and_record_u64,
    print_category_header,
};
use crate::commands::settings::helpers::{edit_u8, ChangeTracker};
use crate::commands::settings::show::{format_duration, format_list};

// ---------------------------------------------------------------------------
// Category editors: scoring / timing / approval / schedule / storage
// ---------------------------------------------------------------------------

pub(crate) fn edit_category_scoring(
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

pub(crate) fn edit_category_timing(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
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

pub(crate) fn edit_category_approval(
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

pub(crate) fn edit_category_schedule(
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

pub(crate) fn edit_category_storage(
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
