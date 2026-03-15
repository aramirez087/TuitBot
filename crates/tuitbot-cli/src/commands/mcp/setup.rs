//! `tuitbot mcp setup` — interactive wizard for MCP-only users.
//!
//! Streamlined alternative to `tuitbot init` that skips LLM and business
//! profile configuration. Two prompts (Client ID + profile), OAuth, and
//! auto-registration with Claude Code.

use std::fs;
use std::io::IsTerminal;

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{Confirm, Input, Select};
use toml_edit::DocumentMut;
use tuitbot_core::config::Config;
use tuitbot_core::startup::data_dir;

use super::detect;
use crate::commands::auth;

/// Run the interactive MCP setup wizard.
pub async fn run_setup(out: crate::output::CliOutput) -> Result<()> {
    // 1. TTY guard
    if !std::io::stdin().is_terminal() {
        if out.is_json() {
            out.error(
                "Interactive setup requires a terminal. \
                 For non-interactive MCP usage, set environment variables.",
            )?;
            std::process::exit(1);
        }
        if out.quiet {
            bail!("Interactive setup requires a terminal.");
        }
        bail!(
            "Interactive setup requires a terminal.\n\n\
             For non-interactive MCP usage, set environment variables:\n  \
             claude mcp add -s user \\\n    \
             -e TUITBOT_X_API__CLIENT_ID=your_client_id \\\n    \
             tuitbot -- tuitbot mcp serve"
        );
    }

    let bold = Style::new().bold();
    let dim = Style::new().dim();
    let green = Style::new().green();

    // 2. Banner
    eprintln!();
    eprintln!(
        "{}",
        bold.apply_to("Tuitbot MCP Setup — connect X to your AI coding assistant.")
    );
    eprintln!();

    // 3. X API guide
    print_x_api_guide();

    // 4. Client ID prompt
    let client_id: String = Input::new()
        .with_prompt("X API Client ID")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Client ID cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let client_id = client_id.trim().to_string();

    // 5. Write minimal config (merge if exists)
    let dir = data_dir();
    let config_path = dir.join("config.toml");

    fs::create_dir_all(&dir)?;
    write_or_merge_config(&config_path, &client_id)?;

    eprintln!(
        "{}",
        green.apply_to(format!("  Config written to {}", config_path.display()))
    );
    eprintln!();

    // 6. Auth flow
    let do_auth = Confirm::new()
        .with_prompt("Connect your X account now?")
        .default(true)
        .interact()?;

    if do_auth {
        let config_str = config_path.display().to_string();
        let config =
            Config::load(Some(&config_str)).context("Failed to load config after writing")?;

        if let Err(e) = auth::execute(&config, None).await {
            eprintln!("\nAuth failed: {e:#}");
            eprintln!("{}", dim.apply_to("You can retry later with: tuitbot auth"));
            eprintln!();
        }
    } else {
        eprintln!(
            "{}",
            dim.apply_to("Skipped. Run `tuitbot auth` when ready.")
        );
        eprintln!();
    }

    // 7. Profile selection
    let profiles = ["write", "readonly", "admin"];
    let profile_descriptions = [
        "write    — full growth co-pilot (default, recommended)",
        "readonly — read-only tools, no mutations",
        "admin    — all tools including Ads, Compliance, Stream Rules",
    ];

    let selection = Select::new()
        .with_prompt("MCP profile")
        .items(&profile_descriptions)
        .default(0)
        .interact()?;

    let profile = profiles[selection];

    eprintln!();

    // 8. Auto-register with Claude Code
    if detect::detect_claude_code() {
        let do_register = Confirm::new()
            .with_prompt("Register with Claude Code automatically?")
            .default(true)
            .interact()?;

        if do_register {
            match detect::register_with_claude_code(profile) {
                Ok(()) => {
                    eprintln!("{}", green.apply_to("  Registered with Claude Code."));
                }
                Err(e) => {
                    eprintln!("  Registration failed: {e:#}");
                    eprintln!();
                    print_manual_snippet(profile);
                }
            }
        } else {
            print_manual_snippet(profile);
        }
    } else {
        eprintln!(
            "{}",
            dim.apply_to("Claude Code not found on PATH. Add this to your MCP config:")
        );
        eprintln!();
        eprintln!("{}", detect::mcp_json_snippet(profile));
    }

    eprintln!();
    eprintln!("{}", bold.apply_to("Setup complete."));
    eprintln!();

    Ok(())
}

/// Print the inline X API guide (same content as init/display.rs).
fn print_x_api_guide() {
    let dim = Style::new().dim();

    eprintln!(
        "{}",
        dim.apply_to("To get your Client ID (takes ~2 minutes):")
    );
    eprintln!(
        "{}",
        dim.apply_to("  1. Go to https://developer.x.com/en/portal/dashboard")
    );
    eprintln!(
        "{}",
        dim.apply_to("  2. Create a Project and App (or select an existing one)")
    );
    eprintln!(
        "{}",
        dim.apply_to("  3. Under \"User authentication settings\", enable OAuth 2.0:")
    );
    eprintln!(
        "{}",
        dim.apply_to("     Type: \"Web App\" — Callback URL: http://127.0.0.1:8080/callback")
    );
    eprintln!(
        "{}",
        dim.apply_to("  4. Copy the Client ID from the \"Keys and tokens\" tab")
    );
    eprintln!();
}

/// Write a minimal config or merge client_id into an existing one.
fn write_or_merge_config(config_path: &std::path::Path, client_id: &str) -> Result<()> {
    if config_path.exists() {
        // Merge into existing config
        let contents = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        let mut doc = contents
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse {}", config_path.display()))?;

        // Ensure [x_api] table exists
        if !doc.contains_key("x_api") {
            doc["x_api"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        doc["x_api"]["client_id"] = toml_edit::value(client_id);

        fs::write(config_path, doc.to_string())
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
    } else {
        // Write minimal config
        let content = format!(
            "\
[x_api]
client_id = \"{client_id}\"

# MCP-only setup: approval mode ensures all posts are reviewed.
approval_mode = true
"
        );
        fs::write(config_path, content)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── write_or_merge_config ─────────────────────────────────────────

    #[test]
    fn write_or_merge_config_creates_new_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");

        write_or_merge_config(&config_path, "test-client-id").unwrap();

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("client_id = \"test-client-id\""));
        assert!(contents.contains("[x_api]"));
        assert!(contents.contains("approval_mode = true"));
    }

    #[test]
    fn write_or_merge_config_merges_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Create an existing config
        fs::write(&config_path, "[business]\nproduct_name = \"MyApp\"\n").unwrap();

        write_or_merge_config(&config_path, "new-client-id").unwrap();

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("client_id = \"new-client-id\""));
        // Original content should be preserved
        assert!(contents.contains("product_name = \"MyApp\""));
    }

    #[test]
    fn write_or_merge_config_updates_existing_client_id() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Create an existing config with old client_id
        fs::write(&config_path, "[x_api]\nclient_id = \"old-id\"\n").unwrap();

        write_or_merge_config(&config_path, "new-id").unwrap();

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("client_id = \"new-id\""));
        assert!(!contents.contains("old-id"));
    }

    #[test]
    fn write_or_merge_config_creates_x_api_section_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Config without [x_api] section
        fs::write(&config_path, "approval_mode = true\n").unwrap();

        write_or_merge_config(&config_path, "my-client").unwrap();

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("[x_api]"));
        assert!(contents.contains("client_id = \"my-client\""));
        assert!(contents.contains("approval_mode = true"));
    }

    // ── print_x_api_guide ─────────────────────────────────────────────

    #[test]
    fn print_x_api_guide_does_not_panic() {
        print_x_api_guide();
    }

    // ── Profile selection ─────────────────────────────────────────────

    #[test]
    fn profiles_array_has_expected_entries() {
        let profiles = ["write", "readonly", "admin"];
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0], "write");
        assert_eq!(profiles[1], "readonly");
        assert_eq!(profiles[2], "admin");
    }

    #[test]
    fn profile_descriptions_match_profiles() {
        let profiles = ["write", "readonly", "admin"];
        let descriptions = [
            "write    — full growth co-pilot (default, recommended)",
            "readonly — read-only tools, no mutations",
            "admin    — all tools including Ads, Compliance, Stream Rules",
        ];
        assert_eq!(profiles.len(), descriptions.len());
        for (profile, desc) in profiles.iter().zip(descriptions.iter()) {
            assert!(desc.starts_with(profile));
        }
    }
}

fn print_manual_snippet(profile: &str) {
    let dim = Style::new().dim();
    eprintln!("{}", dim.apply_to("Add this to your MCP config manually:"));
    eprintln!();
    eprintln!("{}", detect::mcp_json_snippet(profile));
}
