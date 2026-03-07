/// `tuitbot upgrade` -- detect and configure new features in an existing config.
///
/// Parses the raw TOML file to find missing feature groups, then offers an
/// interactive mini-wizard to configure only the missing features. Uses
/// `toml_edit` to patch the file in-place, preserving user comments and
/// formatting.
mod content_sources;
mod defaults;
mod detect;
mod group;
mod patch;
mod wizard;

#[cfg(test)]
mod tests;

use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{bail, Result};
use console::Style;

pub(crate) use defaults::apply_defaults;
pub use detect::detect_missing_features;
#[allow(unused_imports)] // Used by tests via `super::*`
pub(crate) use group::UpgradeGroup;
pub(crate) use wizard::run_upgrade_wizard;

/// Collected answers from the upgrade wizard.
struct UpgradeAnswers {
    persona: Option<(Vec<String>, Vec<String>, Vec<String>)>,
    targets: Option<Vec<String>>,
    approval_mode: Option<bool>,
    enhanced_limits: Option<(u32, Vec<String>, f32)>,
    deployment_mode: Option<String>,
    connectors: Option<Option<(String, String)>>,
    content_sources_noticed: bool,
}

/// Run the upgrade command explicitly.
///
/// **Deprecated:** Use `tuitbot update --config-only` instead. This command
/// will be removed in a future release.
pub async fn execute(non_interactive: bool, config_path_str: &str) -> Result<()> {
    let dim = Style::new().dim();
    eprintln!(
        "{}",
        Style::new()
            .yellow()
            .bold()
            .apply_to("Note: 'tuitbot upgrade' is deprecated. Use 'tuitbot update' instead.")
    );
    eprintln!(
        "{}",
        dim.apply_to("  'tuitbot update' also checks for new binary releases.")
    );
    eprintln!();

    let config_path = expand_tilde(config_path_str);

    if !config_path.exists() {
        bail!(
            "Config file not found: {}\nRun 'tuitbot init' first.",
            config_path.display()
        );
    }

    let missing = detect_missing_features(&config_path)?;
    if missing.is_empty() {
        // Still check for legacy SA-key notice even when all groups present.
        let content = fs::read_to_string(&config_path)?;
        content_sources::print_legacy_sa_key_notice(&content);

        eprintln!("Config is up to date -- no new features to configure.");
        return Ok(());
    }

    if non_interactive {
        return defaults::apply_defaults(&config_path, &missing);
    }

    if !std::io::stdin().is_terminal() {
        bail!(
            "Interactive upgrade requires a terminal.\n\
             Use --non-interactive to apply default values for new features."
        );
    }

    wizard::run_upgrade_wizard(&config_path, &missing)
}

/// Expand `~` at the start of a path to the user's home directory.
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}
