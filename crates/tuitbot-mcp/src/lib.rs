//! MCP (Model Context Protocol) server for Tuitbot.
//!
//! Exposes Tuitbot operations as structured MCP tools over stdio transport,
//! allowing AI agents to natively discover and call analytics, approval queue,
//! content generation, scoring, and configuration operations.
//!
//! Four runtime profiles are available:
//! - **`readonly`**: minimal read-only X tools (10). No DB, no LLM, no mutations.
//! - **`api-readonly`**: broader read-only X tools (20). No DB, no LLM, no mutations.
//! - **`write`** (default): standard operating profile. All typed tools including mutations.
//! - **`admin`**: superset of write. Adds universal request tools. Explicit opt-in.

pub mod contract;
mod kernel;
mod provider;
mod requests;
mod server;
pub mod spec;
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

use server::{AdminMcpServer, ApiReadonlyMcpServer, ReadonlyMcpServer, WriteMcpServer};
use state::{AppState, ReadonlyState, SharedReadonlyState};
use tools::idempotency::IdempotencyStore;

pub use state::Profile;
pub use tools::manifest::{generate_profile_manifest, ProfileManifest};

/// Run the MCP server with the specified profile.
///
/// Dispatches to the appropriate server implementation based on profile.
pub async fn run_server(config: Config, profile: Profile) -> anyhow::Result<()> {
    match profile {
        Profile::Readonly => run_readonly_server(config).await,
        Profile::ApiReadonly => run_api_readonly_server(config).await,
        Profile::Write => run_write_server(config).await,
        Profile::Admin => run_admin_server(config).await,
    }
}

// ── Shared init for write/admin profiles ────────────────────────────────

/// Initialize shared state for write / admin profiles: DB, LLM, X client.
async fn init_write_state(config: Config) -> anyhow::Result<Arc<AppState>> {
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
    let (x_client, authenticated_user_id, granted_scopes): (
        Option<Box<dyn XApiClient>>,
        Option<String>,
        Vec<String>,
    ) = match startup::load_tokens_from_file() {
        Ok(tokens) if !tokens.is_expired() => {
            let scopes = tokens.scopes.clone();
            let client = XApiHttpClient::new(tokens.access_token);
            client.set_pool(pool.clone()).await;
            match client.get_me().await {
                Ok(user) => {
                    tracing::info!(
                        username = %user.username,
                        user_id = %user.id,
                        scopes = ?scopes,
                        "X API client initialized"
                    );
                    (Some(Box::new(client)), Some(user.id), scopes)
                }
                Err(e) => {
                    tracing::warn!(
                        "X API client created but get_me() failed: {e}. \
                         Direct X tools will be disabled."
                    );
                    (Some(Box::new(client)), None, scopes)
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "X API tokens expired. Direct X tools will be disabled. \
                 Run `tuitbot auth` to re-authenticate."
            );
            (None, None, vec![])
        }
        Err(e) => {
            tracing::warn!("X API tokens not available: {e}. Direct X tools will be disabled.");
            (None, None, vec![])
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

    Ok(Arc::new(AppState {
        pool,
        config,
        llm_provider,
        x_client,
        authenticated_user_id,
        granted_scopes,
        idempotency: Arc::new(IdempotencyStore::new()),
    }))
}

/// Run the write-profile MCP server on stdio transport (standard operating profile).
async fn run_write_server(config: Config) -> anyhow::Result<()> {
    let state = init_write_state(config).await?;
    let pool = state.pool.clone();
    let server = WriteMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (write profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;

    // Clean shutdown
    pool.close().await;

    Ok(())
}

/// Run the admin-profile MCP server on stdio transport (write + universal requests).
async fn run_admin_server(config: Config) -> anyhow::Result<()> {
    let state = init_write_state(config).await?;
    let pool = state.pool.clone();
    let server = AdminMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (admin profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;

    // Clean shutdown
    pool.close().await;

    Ok(())
}

// ── Shared init for read-only profiles ──────────────────────────────────

/// Initialize shared readonly state: load tokens, create X client, verify get_me.
async fn init_readonly_state(
    config: Config,
    profile: Profile,
) -> anyhow::Result<SharedReadonlyState> {
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

    Ok(Arc::new(ReadonlyState {
        config,
        x_client: Box::new(client),
        authenticated_user_id: user.id,
    }))
}

/// Run the readonly-profile MCP server on stdio transport (10 tools).
async fn run_readonly_server(config: Config) -> anyhow::Result<()> {
    let state = init_readonly_state(config, Profile::Readonly).await?;
    let server = ReadonlyMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (readonly profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;
    Ok(())
}

/// Run the api-readonly-profile MCP server on stdio transport (20 tools).
async fn run_api_readonly_server(config: Config) -> anyhow::Result<()> {
    let state = init_readonly_state(config, Profile::ApiReadonly).await?;
    let server = ApiReadonlyMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (api-readonly profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;
    Ok(())
}
