/// `tuitbot update` — unified binary self-update + config upgrade.
///
/// Phase 1a: Check GitHub releases for a newer CLI binary, download, verify
///           SHA256, and atomically replace the current binary.
/// Phase 1b: Independently check whether `tuitbot-server` (if on PATH) is
///           behind the latest release that ships server assets. This runs
///           regardless of whether the CLI itself needed an update, fixing
///           the bootstrapping bug where a newly-updated CLI skips the server
///           because it's "already up to date."
/// Phase 2:  Run config upgrade (reuses `upgrade.rs` logic) to patch missing
///           feature groups into the user's `config.toml`.
mod binary;
mod github;
mod platform;
mod version;

#[cfg(test)]
mod tests;

use std::io::IsTerminal;

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::Confirm;
use semver::Version;

use super::upgrade;

use binary::{detect_server_path, detect_server_version, update_cli_binary, update_target_binary};
use github::{available_asset_names, check_recent_releases, GitHubRelease};
use platform::{asset_name_for_binary, platform_asset_name};
use version::{
    is_newer, latest_compatible_release, latest_known_release, latest_release_with_server_asset,
};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_TARGET_ENV: &str = "TUITBOT_UPDATE_TARGET";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Execute the `update` command.
pub async fn execute(
    non_interactive: bool,
    check_only: bool,
    config_only: bool,
    config_path_str: &str,
) -> Result<()> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let green = Style::new().green().bold();
    let current = Version::parse(CURRENT_VERSION).context("Failed to parse current version")?;

    // Phase 1: Binary update
    if !config_only {
        eprintln!("{}", bold.apply_to("Checking for updates..."));
        eprintln!();

        let fetched_releases = check_recent_releases().await;

        match &fetched_releases {
            Ok(releases) => match latest_known_release(releases) {
                Some((latest_release, latest)) if is_newer(&latest, &current) => {
                    eprintln!(
                        "  {} {} → {}",
                        green.apply_to("New version available:"),
                        current,
                        latest
                    );

                    if check_only {
                        eprintln!();
                        eprintln!(
                            "{}",
                            dim.apply_to("Run 'tuitbot update' to install the update.")
                        );
                        return Ok(());
                    }

                    // Confirm with user in interactive mode
                    if !non_interactive && std::io::stdin().is_terminal() {
                        eprintln!();
                        let proceed = Confirm::new()
                            .with_prompt(format!("Update tuitbot to v{latest}?"))
                            .default(true)
                            .interact()?;

                        if !proceed {
                            eprintln!("{}", dim.apply_to("Update skipped."));
                            eprintln!();
                            // Fall through to config upgrade
                            return run_config_upgrade(
                                non_interactive,
                                config_path_str,
                                &bold,
                                &dim,
                            );
                        }
                    }

                    let asset_name = match platform_asset_name()
                        .context("Unsupported platform for binary self-update")
                    {
                        Ok(name) => name,
                        Err(e) => {
                            eprintln!();
                            eprintln!(
                                "  {} Binary update skipped: {e}",
                                Style::new().yellow().bold().apply_to("⚠"),
                            );
                            eprintln!(
                                    "  {}",
                                    dim.apply_to(
                                        "No prebuilt binary is published for this platform. Build from source or install manually."
                                    )
                                );
                            eprintln!(
                                    "  {}",
                                    dim.apply_to(
                                        "Manual downloads: https://github.com/aramirez087/TuitBot/releases"
                                    )
                                );
                            eprintln!();
                            return run_config_upgrade(
                                non_interactive,
                                config_path_str,
                                &bold,
                                &dim,
                            );
                        }
                    };

                    let (release_for_update, release_version) = match latest_compatible_release(
                        releases,
                        &current,
                        &asset_name,
                    ) {
                        Some(found) => found,
                        None => {
                            eprintln!();
                            eprintln!(
                                "  {} Binary update skipped: no compatible asset found for '{}'",
                                Style::new().yellow().bold().apply_to("⚠"),
                                asset_name,
                            );
                            eprintln!(
                                "  {} Latest release checked: {}",
                                dim.apply_to("Tag:"),
                                latest_release.tag_name,
                            );
                            eprintln!(
                                "  {} {}",
                                dim.apply_to("Available assets:"),
                                available_asset_names(latest_release),
                            );
                            eprintln!(
                                    "  {}",
                                    dim.apply_to(
                                        "Manual downloads: https://github.com/aramirez087/TuitBot/releases"
                                    )
                                );
                            eprintln!();
                            return run_config_upgrade(
                                non_interactive,
                                config_path_str,
                                &bold,
                                &dim,
                            );
                        }
                    };

                    if release_version != latest {
                        eprintln!(
                            "  {} Latest version v{} has no '{}' asset; installing newest compatible v{}.",
                            Style::new().yellow().bold().apply_to("⚠"),
                            latest,
                            asset_name,
                            release_version
                        );
                    }

                    // Phase 1a: Update CLI binary
                    match update_cli_binary(release_for_update).await {
                        Ok(()) => {
                            eprintln!();
                            eprintln!(
                                "  {} Updated tuitbot to v{}",
                                green.apply_to("✓"),
                                release_version
                            );
                        }
                        Err(e) => {
                            eprintln!();
                            eprintln!(
                                "  {} CLI binary update failed: {e}",
                                Style::new().red().bold().apply_to("✗"),
                            );
                            eprintln!(
                                "  {}",
                                dim.apply_to(
                                    "You can download manually from: https://github.com/aramirez087/TuitBot/releases"
                                )
                            );
                        }
                    }
                }
                Some((_, latest)) => {
                    eprintln!("  Already up to date (v{current}).");
                    if latest != current {
                        eprintln!("  {}", dim.apply_to(format!("(latest release: v{latest})")));
                    }

                    if check_only {
                        return Ok(());
                    }
                }
                None => {
                    eprintln!(
                        "  {} Could not find a parseable CLI release tag",
                        Style::new().yellow().bold().apply_to("⚠"),
                    );

                    if check_only {
                        return Ok(());
                    }
                }
            },
            Err(e) => {
                eprintln!(
                    "  {} Could not check for updates: {e}",
                    Style::new().yellow().bold().apply_to("⚠"),
                );
                eprintln!(
                    "  {}",
                    dim.apply_to("Skipping binary update, continuing with config upgrade...")
                );
            }
        }

        // Phase 1b: Standalone server update check
        // Runs independently of the CLI update — fixes the bootstrapping bug where
        // the server stays stuck at an old version when the CLI is already current.
        if !check_only {
            if let Ok(releases) = &fetched_releases {
                check_and_update_server(releases, &green, &dim).await;
            }
        }

        eprintln!();
    } else if check_only {
        bail!("--check and --config-only cannot be used together.");
    }

    // Phase 2: Config upgrade
    run_config_upgrade(non_interactive, config_path_str, &bold, &dim)
}

/// Pre-run check: hint about `tuitbot update` when config has missing features.
pub async fn check_before_run(config_path_str: &str) -> Result<()> {
    let config_path = upgrade::expand_tilde(config_path_str);

    if !config_path.exists() {
        return Ok(());
    }

    let missing = upgrade::detect_missing_features(&config_path)?;
    if missing.is_empty() {
        return Ok(());
    }

    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!(
        "{}",
        bold.apply_to("New features available in your config:")
    );
    for group in &missing {
        eprintln!("  • {} — {}", group.display_name(), group.description());
    }
    eprintln!();

    let configure_now = Confirm::new()
        .with_prompt("Configure new features now?")
        .default(false)
        .interact()?;

    if !configure_now {
        eprintln!(
            "{}",
            dim.apply_to("Tip: Run 'tuitbot update' any time to configure new features.")
        );
        eprintln!();
        return Ok(());
    }

    upgrade::run_upgrade_wizard(&config_path, &missing)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Server update (Phase 1b) — standalone, non-fatal
// ---------------------------------------------------------------------------

/// Check whether `tuitbot-server` is installed, behind, and update it.
///
/// This runs independently of the CLI update check so the server can be updated
/// even when the CLI is already at the latest version (bootstrapping fix).
async fn check_and_update_server(releases: &[GitHubRelease], green: &Style, dim: &Style) {
    let server_exe = match detect_server_path() {
        Some(path) => path,
        None => return, // server not installed — nothing to do
    };

    let server_asset = match asset_name_for_binary("tuitbot-server") {
        Some(name) => name,
        None => {
            eprintln!(
                "  {} Server update skipped: unsupported platform",
                dim.apply_to("ℹ"),
            );
            return;
        }
    };

    // Find the newest release that ships a server binary
    let (release, release_version) = match latest_release_with_server_asset(releases, &server_asset)
    {
        Some(found) => found,
        None => return, // no release has server assets — skip silently
    };

    // Compare against the installed server version (if detectable)
    if let Some(server_version) = detect_server_version(&server_exe) {
        if server_version >= release_version {
            eprintln!(
                "  {} tuitbot-server is up to date (v{server_version}).",
                dim.apply_to("ℹ"),
            );
            return;
        }
        eprintln!(
            "  {} tuitbot-server v{server_version} → v{release_version}",
            green.apply_to("Server update available:"),
        );
    } else {
        eprintln!(
            "  {} Could not detect server version; attempting update to v{release_version}.",
            dim.apply_to("ℹ"),
        );
    }

    match update_target_binary(release, "tuitbot-server", &server_asset, &server_exe).await {
        Ok(()) => {
            eprintln!(
                "  {} Updated tuitbot-server at {}",
                green.apply_to("✓"),
                server_exe.display()
            );
            eprintln!(
                "  {}",
                dim.apply_to(
                    "Restart the server to use the new version (e.g., sudo systemctl restart tuitbot)."
                )
            );
        }
        Err(e) => {
            eprintln!(
                "  {} Server update failed: {e}",
                Style::new().yellow().bold().apply_to("⚠"),
            );
            let hint = if cfg!(unix) && server_exe.starts_with("/usr") {
                "Hint: You may need to run with sudo to update the server binary."
            } else {
                "Hint: Make sure tuitbot-server is not running, then try again."
            };
            eprintln!("  {}", dim.apply_to(hint));
        }
    }
}

// ---------------------------------------------------------------------------
// Config upgrade (Phase 2)
// ---------------------------------------------------------------------------

fn run_config_upgrade(
    non_interactive: bool,
    config_path_str: &str,
    bold: &Style,
    dim: &Style,
) -> Result<()> {
    let config_path = upgrade::expand_tilde(config_path_str);

    if !config_path.exists() {
        eprintln!(
            "  {}",
            dim.apply_to("No config file found — run 'tuitbot init' to create one.")
        );
        return Ok(());
    }

    eprintln!("{}", bold.apply_to("Checking configuration..."));

    let missing = upgrade::detect_missing_features(&config_path)?;

    if missing.is_empty() {
        eprintln!("  Config is up to date.");
        return Ok(());
    }

    eprintln!("  New feature groups to configure:");
    for group in &missing {
        eprintln!("    • {} — {}", group.display_name(), group.description());
    }
    eprintln!();

    if non_interactive {
        upgrade::apply_defaults(&config_path, &missing)?;
    } else if std::io::stdin().is_terminal() {
        upgrade::run_upgrade_wizard(&config_path, &missing)?;
    } else {
        eprintln!(
            "  {}",
            dim.apply_to(
                "Non-interactive terminal detected. Use --non-interactive to apply defaults."
            )
        );
    }

    Ok(())
}
