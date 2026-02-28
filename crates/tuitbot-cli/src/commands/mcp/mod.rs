//! `tuitbot mcp` subcommands: serve, manifest, and setup.
//!
//! - `serve`    — loads config (file + env overrides, no file required) and starts the MCP server.
//! - `manifest` — prints the profile-specific tool manifest as JSON.
//! - `setup`    — interactive wizard for MCP-only users (client ID → auth → register).
mod detect;
mod setup;

use tuitbot_core::config::Config;
use tuitbot_mcp::Profile;

use crate::output::write_stdout;

/// Execute the `tuitbot mcp serve` subcommand.
///
/// Loads config itself with `Config::load(None)` so that env-var-only
/// operation works without a config file on disk.
pub async fn execute_serve(profile_str: &str) -> anyhow::Result<()> {
    let profile: Profile = profile_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let config = Config::load(None).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load configuration: {e}\n\
             Hint: Run 'tuitbot mcp setup' or set TUITBOT_X_API__CLIENT_ID as an env var."
        )
    })?;

    tuitbot_mcp::run_server(config, profile).await
}

/// Print the profile-specific tool manifest as JSON to stdout.
pub fn print_manifest(profile_str: &str) -> anyhow::Result<()> {
    let profile: Profile = profile_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let manifest = tuitbot_mcp::generate_profile_manifest(profile);
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| anyhow::anyhow!("Failed to serialize manifest: {e}"))?;
    write_stdout(&json)
}

/// Execute the `tuitbot mcp setup` interactive wizard.
pub async fn execute_setup() -> anyhow::Result<()> {
    setup::run_setup().await
}
