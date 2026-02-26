//! MCP (Model Context Protocol) server for Tuitbot.
//!
//! Exposes Tuitbot operations as structured MCP tools over stdio transport,
//! allowing AI agents to natively discover and call analytics, approval queue,
//! content generation, scoring, and configuration operations.
//!
//! Three runtime profiles are available:
//! - **`full`** (default): full TuitBot growth features. All 60+ tools.
//! - **`readonly`**: read-only X tools. No DB, no LLM, no mutations.
//! - **`api-readonly`**: broader read-only X tools. No DB, no LLM, no mutations.

pub mod contract;
mod kernel;
mod provider;
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

use server::{ApiMcpServer, TuitbotMcpServer};
use state::{ApiState, AppState};
use tools::idempotency::IdempotencyStore;

pub use state::Profile;

/// Run the MCP server with the specified profile.
///
/// Dispatches to `run_stdio_server` (full) or `run_api_server` (readonly / api-readonly).
pub async fn run_server(config: Config, profile: Profile) -> anyhow::Result<()> {
    match profile {
        Profile::Full => run_stdio_server(config).await,
        Profile::Readonly | Profile::ApiReadonly => run_api_server(config, profile).await,
    }
}

/// Run the full-profile MCP server on stdio transport.
///
/// This is the main entry point called by the CLI `tuitbot mcp serve` subcommand
/// (or `--profile full`). It initializes the database, optionally creates an
/// LLM provider, and serves MCP tools over stdin/stdout.
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

    // Log provider backend selection.
    let backend = provider::parse_backend(&config.x_api.provider_backend);
    match backend {
        provider::ProviderBackend::XApi => {
            tracing::info!(backend = "x_api", "Provider backend: official X API");
        }
        provider::ProviderBackend::Scraper => {
            tracing::warn!(
                backend = "scraper",
                allow_mutations = config.x_api.scraper_allow_mutations,
                "Provider backend: scraper (elevated risk)"
            );
        }
    }

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config,
        llm_provider,
        x_client,
        authenticated_user_id,
        idempotency: Arc::new(IdempotencyStore::new()),
    });

    let server = TuitbotMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (full profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;

    // Clean shutdown
    pool.close().await;

    Ok(())
}

/// Run a readonly-profile MCP server on stdio transport.
///
/// Used for both `readonly` and `api-readonly` profiles. Fails fast if
/// X API tokens are missing or expired — a readonly profile with no X client
/// has zero usable tools.
pub async fn run_api_server(config: Config, profile: Profile) -> anyhow::Result<()> {
    // Load X API tokens (required for readonly profiles)
    let tokens = startup::load_tokens_from_file().map_err(|e| {
        anyhow::anyhow!(
            "{profile} profile requires X API tokens but they are not available: {e}. \
             Run `tuitbot auth` to authenticate."
        )
    })?;

    if tokens.is_expired() {
        anyhow::bail!(
            "{profile} profile requires valid X API tokens but they are expired. \
             Run `tuitbot auth` to re-authenticate."
        );
    }

    let client = XApiHttpClient::new(tokens.access_token);

    // Verify connectivity and get authenticated user ID
    let user = client.get_me().await.map_err(|e| {
        anyhow::anyhow!(
            "{profile} profile requires a working X API client but get_me() failed: {e}. \
             Check your network connection or re-authenticate with `tuitbot auth`."
        )
    })?;

    tracing::info!(
        username = %user.username,
        user_id = %user.id,
        profile = %profile,
        "X API client initialized ({profile} profile)"
    );

    // Log provider backend selection.
    let backend = provider::parse_backend(&config.x_api.provider_backend);
    match backend {
        provider::ProviderBackend::XApi => {
            tracing::info!(backend = "x_api", "Provider backend: official X API");
        }
        provider::ProviderBackend::Scraper => {
            tracing::warn!(
                backend = "scraper",
                allow_mutations = config.x_api.scraper_allow_mutations,
                "Provider backend: scraper (elevated risk)"
            );
        }
    }

    let state = Arc::new(ApiState {
        config,
        x_client: Box::new(client),
        authenticated_user_id: user.id,
        idempotency: Arc::new(IdempotencyStore::new()),
    });

    let server = ApiMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio ({profile} profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;

    Ok(())
}
