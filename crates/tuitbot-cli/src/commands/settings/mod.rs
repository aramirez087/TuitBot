/// `tuitbot settings` — interactive configuration editor.
///
/// Modes:
/// - `tuitbot settings`              — interactive category menu
/// - `tuitbot settings --show`       — pretty-print current config
/// - `tuitbot settings --set K=V`    — direct one-shot set
/// - `tuitbot settings <category>`   — jump to a specific category
mod enrich;
mod helpers;
mod interactive;
mod render;
mod set;
mod show;

#[cfg(test)]
mod tests;

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{bail, Result};
use tuitbot_core::config::Config;

use super::{OutputFormat, SettingsArgs};

/// Entry point for the settings command.
pub async fn execute(args: SettingsArgs, config_path: &str, output: OutputFormat) -> Result<()> {
    let expanded = expand_tilde(config_path);
    if !expanded.exists() {
        bail!(
            "Config file not found: {}\nRun 'tuitbot init' first.",
            expanded.display()
        );
    }

    let config = Config::load(Some(config_path)).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load configuration: {e}\n\
             Hint: Run 'tuitbot init' to create a default configuration file."
        )
    })?;

    if args.show && args.set.is_some() {
        bail!(
            "--show and --set are mutually exclusive.\n\
             Use --show to view configuration, or --set to change a value."
        );
    }

    if args.show && args.category.is_some() {
        bail!(
            "--show does not support category filtering.\n\
             Use `tuitbot settings --show` to view the full configuration."
        );
    }

    if args.show {
        if output.is_json() {
            show::show_config_json(&config)?;
        } else {
            show::show_config(&config);
        }
        return Ok(());
    }

    if let Some(kv) = &args.set {
        let mut config = config;
        return set::set_direct(&mut config, kv, &expanded);
    }

    if !std::io::stdin().is_terminal() {
        bail!(
            "Interactive settings editor requires a terminal.\n\
             Use --show to view or --set KEY=VALUE to change settings."
        );
    }

    let mut config = config;

    if let Some(category) = &args.category {
        let tracker = &mut helpers::ChangeTracker::new();
        match category.to_lowercase().as_str() {
            "product" | "business" => interactive::edit_category_product(&mut config, tracker)?,
            "voice" | "brand" => interactive::edit_category_voice(&mut config, tracker)?,
            "persona" => interactive::edit_category_persona(&mut config, tracker)?,
            "ai" | "llm" | "provider" => interactive::edit_category_llm(&mut config, tracker)?,
            "x" | "xapi" | "credentials" => {
                interactive::edit_category_xapi(&mut config, tracker)?
            }
            "targets" | "accounts" => {
                interactive::edit_category_targets(&mut config, tracker)?
            }
            "limits" | "posting" => interactive::edit_category_limits(&mut config, tracker)?,
            "scoring" => interactive::edit_category_scoring(&mut config, tracker)?,
            "timing" | "intervals" => interactive::edit_category_timing(&mut config, tracker)?,
            "approval" => interactive::edit_category_approval(&mut config, tracker)?,
            "schedule" | "hours" => interactive::edit_category_schedule(&mut config, tracker)?,
            "storage" | "logging" => interactive::edit_category_storage(&mut config, tracker)?,
            "enrich" | "enrichment" | "profile" => {
                enrich::run_enrichment(&mut config, tracker)?
            }
            other => bail!(
                "Unknown category: {other}\n\
                 Valid categories: product, voice, persona, ai, x, targets, limits, scoring, timing, approval, schedule, storage, enrich"
            ),
        }
        if !tracker.changes.is_empty() {
            render::save_flow(&config, &expanded, tracker)?;
        } else {
            eprintln!("No changes made.");
        }
        return Ok(());
    }

    interactive::interactive_menu(&mut config, &expanded)?;

    Ok(())
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}
