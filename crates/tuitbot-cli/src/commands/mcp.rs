//! Implementation of the `tuitbot mcp` command.
//!
//! Starts the MCP server on stdio transport for AI agent integration.
//! Supports three runtime profiles: `full` (default), `readonly`, and `api-readonly`.

use tuitbot_core::config::Config;
use tuitbot_mcp::Profile;

/// Execute the `tuitbot mcp serve` subcommand.
pub async fn execute(config: &Config, profile_str: &str) -> anyhow::Result<()> {
    let profile: Profile = profile_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    tuitbot_mcp::run_server(config.clone(), profile).await
}
