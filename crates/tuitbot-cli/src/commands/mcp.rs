//! Implementation of the `tuitbot mcp` command.
//!
//! Starts the MCP server on stdio transport for AI agent integration.

use tuitbot_core::config::Config;

/// Execute the `tuitbot mcp serve` subcommand.
pub async fn execute(config: &Config) -> anyhow::Result<()> {
    tuitbot_mcp::run_stdio_server(config.clone()).await
}
