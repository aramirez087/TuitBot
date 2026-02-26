//! Implementation of the `tuitbot mcp` command.
//!
//! Starts the MCP server on stdio transport for AI agent integration.
//! Supports four runtime profiles: `write` (default), `readonly`, `api-readonly`, and `admin`.

use tuitbot_core::config::Config;
use tuitbot_mcp::Profile;

use crate::output::write_stdout;

/// Execute the `tuitbot mcp serve` subcommand.
pub async fn execute(config: &Config, profile_str: &str) -> anyhow::Result<()> {
    let profile: Profile = profile_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    tuitbot_mcp::run_server(config.clone(), profile).await
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
