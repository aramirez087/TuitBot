mod render;
mod wizard;

#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::Result;
use console::Style;
use dialoguer::Select;
use tuitbot_core::config::Config;

use super::helpers::*;
use super::render::save_flow;

// Re-export category editors so settings/mod.rs can call interactive::edit_category_*
pub(crate) use self::render::{
    edit_category_approval, edit_category_schedule, edit_category_scoring, edit_category_storage,
    edit_category_timing,
};
pub(crate) use wizard::{
    edit_category_limits, edit_category_llm, edit_category_persona, edit_category_product,
    edit_category_targets, edit_category_voice, edit_category_xapi,
};

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
            "Enrich Profile      — guided setup for voice, persona, targeting",
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
            0 => super::enrich::run_enrichment(config, &mut tracker)?,
            1 => edit_category_product(config, &mut tracker)?,
            2 => edit_category_voice(config, &mut tracker)?,
            3 => edit_category_persona(config, &mut tracker)?,
            4 => edit_category_llm(config, &mut tracker)?,
            5 => edit_category_xapi(config, &mut tracker)?,
            6 => edit_category_targets(config, &mut tracker)?,
            7 => edit_category_limits(config, &mut tracker)?,
            8 => edit_category_scoring(config, &mut tracker)?,
            9 => edit_category_timing(config, &mut tracker)?,
            10 => edit_category_approval(config, &mut tracker)?,
            11 => edit_category_schedule(config, &mut tracker)?,
            12 => edit_category_storage(config, &mut tracker)?,
            13 => break, // Save & Exit
            _ => unreachable!(),
        }
    }

    if tracker.changes.is_empty() {
        eprintln!("No changes made.");
        return Ok(());
    }

    save_flow(config, config_path, &tracker)
}
