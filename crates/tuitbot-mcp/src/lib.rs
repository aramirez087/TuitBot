//! MCP (Model Context Protocol) server for Tuitbot.
//!
//! Exposes Tuitbot operations as structured MCP tools over stdio transport,
//! allowing AI agents to natively discover and call analytics, approval queue,
//! content generation, scoring, and configuration operations.

mod requests;
mod server;
mod state;
mod tools;

use std::sync::Arc;

use rmcp::transport::stdio;
use rmcp::ServiceExt;

use tuitbot_core::config::Config;
use tuitbot_core::llm;
use tuitbot_core::startup;
use tuitbot_core::storage;
use tuitbot_core::x_api::{XApiClient, XApiHttpClient};

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

    // Initialize MCP mutation rate limit
    storage::rate_limits::init_mcp_rate_limit(&pool, config.mcp_policy.max_mutations_per_hour)
        .await?;

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

    // Try to initialize X API client (optional — direct X tools won't work without it)
    let (x_client, authenticated_user_id): (Option<Box<dyn XApiClient>>, Option<String>) =
        match startup::load_tokens_from_file() {
            Ok(tokens) if !tokens.is_expired() => {
                let client = XApiHttpClient::new(tokens.access_token);
                client.set_pool(pool.clone()).await;
                match client.get_me().await {
                    Ok(user) => {
                        tracing::info!(
                            username = %user.username,
                            user_id = %user.id,
                            "X API client initialized"
                        );
                        (Some(Box::new(client)), Some(user.id))
                    }
                    Err(e) => {
                        tracing::warn!(
                            "X API client created but get_me() failed: {e}. \
                             Direct X tools will be disabled."
                        );
                        (Some(Box::new(client)), None)
                    }
                }
            }
            Ok(_) => {
                tracing::warn!(
                    "X API tokens expired. Direct X tools will be disabled. \
                     Run `tuitbot auth` to re-authenticate."
                );
                (None, None)
            }
            Err(e) => {
                tracing::warn!("X API tokens not available: {e}. Direct X tools will be disabled.");
                (None, None)
            }
        };

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config,
        llm_provider,
        x_client,
        authenticated_user_id,
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
