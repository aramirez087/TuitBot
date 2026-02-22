//! MCP (Model Context Protocol) server for Tuitbot.
//!
//! Exposes Tuitbot operations as structured MCP tools over stdio transport,
//! allowing AI agents to natively discover and call analytics, approval queue,
//! content generation, scoring, and configuration operations.

mod server;
mod state;
mod tools;

use std::sync::Arc;

use rmcp::transport::stdio;
use rmcp::ServiceExt;

use tuitbot_core::config::Config;
use tuitbot_core::llm;
use tuitbot_core::storage;

use server::TuitbotMcpServer;
use state::AppState;

/// Run the MCP server on stdio transport.
///
/// This is the main entry point called by the CLI `tuitbot mcp serve` subcommand.
/// It initializes the database, optionally creates an LLM provider, and serves
/// MCP tools over stdin/stdout.
pub async fn run_stdio_server(config: Config) -> anyhow::Result<()> {
    // Initialize database
    let pool = storage::init_db(&config.storage.db_path).await?;

    // Try to create LLM provider (optional — content tools won't work without it)
    let llm_provider = match llm::factory::create_provider(&config.llm) {
        Ok(provider) => {
            tracing::info!(provider = provider.name(), "LLM provider initialized");
            Some(provider)
        }
        Err(e) => {
            tracing::warn!(
                "LLM provider not available: {e}. Content generation tools will be disabled."
            );
            None
        }
    };

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config,
        llm_provider,
    });

    let server = TuitbotMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;

    // Clean shutdown
    pool.close().await;

    Ok(())
}
