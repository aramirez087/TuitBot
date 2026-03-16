//! MCP (Model Context Protocol) server for Tuitbot.
//!
//! Exposes Tuitbot operations as structured MCP tools over stdio transport,
//! allowing AI agents to natively discover and call analytics, approval queue,
//! content generation, scoring, and configuration operations.
//!
//! Six runtime profiles are available:
//! - **`readonly`**: minimal read-only X tools (10). No DB, no LLM, no mutations.
//! - **`api-readonly`**: broader read-only X tools (20). No DB, no LLM, no mutations.
//! - **`write`** (default): standard operating profile. All typed tools including mutations.
//! - **`admin`**: superset of write. Adds universal request tools. Explicit opt-in.
//! - **`utility-readonly`**: flat toolkit surface — stateless reads + scoring + config. No workflow.
//! - **`utility-write`**: flat toolkit surface — reads + writes + engages. No workflow, no policy gate.

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
use tuitbot_core::x_api::{LocalModeXClient, NullXApiClient, XApiClient, XApiHttpClient};

use server::{
    AdminMcpServer, ApiReadonlyMcpServer, ReadonlyMcpServer, UtilityReadonlyMcpServer,
    UtilityWriteMcpServer, WriteMcpServer,
};
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
        Profile::UtilityReadonly => run_utility_readonly_server(config).await,
        Profile::UtilityWrite => run_utility_write_server(config).await,
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

    // ── Scraper backend: use LocalModeXClient, skip OAuth ───────────
    if backend == provider::ProviderBackend::Scraper {
        let data_dir = tuitbot_core::startup::data_dir();
        let client =
            LocalModeXClient::with_session(config.x_api.scraper_allow_mutations, &data_dir).await;

        let (x_client, authenticated_user_id): (Option<Box<dyn XApiClient>>, Option<String>) =
            match client.get_me().await {
                Ok(user) => {
                    tracing::info!(
                        username = %user.username, user_id = %user.id,
                        "Scraper client initialized (write profile)"
                    );
                    (Some(Box::new(client)), Some(user.id))
                }
                Err(e) => {
                    tracing::warn!(
                        "Scraper get_me() unavailable: {e}. \
                         Direct X tools will be disabled."
                    );
                    (Some(Box::new(client)), None)
                }
            };

        return Ok(Arc::new(AppState {
            pool,
            config,
            llm_provider,
            x_client,
            authenticated_user_id,
            granted_scopes: vec![],
            idempotency: Arc::new(IdempotencyStore::new()),
        }));
    }

    // ── Official X API backend ──────────────────────────────────────
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
///
/// Gracefully degrades when tokens are missing, expired, or `get_me()` fails.
/// Non-X tools (config, scoring) remain functional in degraded mode.
async fn init_readonly_state(
    config: Config,
    profile: Profile,
) -> anyhow::Result<SharedReadonlyState> {
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

    // ── Scraper backend: use LocalModeXClient, skip OAuth ───────────
    if backend == provider::ProviderBackend::Scraper {
        let data_dir = tuitbot_core::startup::data_dir();
        let client =
            LocalModeXClient::with_session(config.x_api.scraper_allow_mutations, &data_dir).await;

        let (authenticated_user_id, x_available) = match client.get_me().await {
            Ok(user) => {
                tracing::info!(
                    username = %user.username, user_id = %user.id,
                    profile = %profile,
                    "Scraper client initialized ({profile} profile)"
                );
                (user.id, true)
            }
            Err(e) => {
                tracing::warn!(
                    "Scraper get_me() unavailable: {e}. \
                     Non-X tools (get_config, score_tweet) are still available."
                );
                (String::new(), false)
            }
        };

        return Ok(Arc::new(ReadonlyState {
            config,
            x_client: Box::new(client),
            authenticated_user_id,
            x_available,
        }));
    }

    // ── Official X API backend ──────────────────────────────────────
    let (x_client, authenticated_user_id, x_available): (Box<dyn XApiClient>, String, bool) =
        match startup::load_tokens_from_file() {
            Ok(tokens) if !tokens.is_expired() => {
                let client = XApiHttpClient::new(tokens.access_token);
                match client.get_me().await {
                    Ok(user) => {
                        tracing::info!(
                            username = %user.username,
                            user_id = %user.id,
                            profile = %profile,
                            "X API client initialized ({profile} profile)"
                        );
                        (Box::new(client) as Box<dyn XApiClient>, user.id, true)
                    }
                    Err(e) => {
                        tracing::warn!(
                            "X API get_me() failed: {e}. X tools will return errors. \
                             Non-X tools (get_config, score_tweet) are still available."
                        );
                        (
                            Box::new(client) as Box<dyn XApiClient>,
                            String::new(),
                            false,
                        )
                    }
                }
            }
            Ok(_) => {
                tracing::warn!(
                    "X API tokens expired for {profile} profile. X tools will be unavailable. \
                     Run `tuitbot auth` to re-authenticate."
                );
                (
                    Box::new(NullXApiClient) as Box<dyn XApiClient>,
                    String::new(),
                    false,
                )
            }
            Err(e) => {
                tracing::warn!(
                    "X API tokens not available for {profile} profile: {e}. \
                     Non-X tools (get_config, score_tweet) are still available."
                );
                (
                    Box::new(NullXApiClient) as Box<dyn XApiClient>,
                    String::new(),
                    false,
                )
            }
        };

    Ok(Arc::new(ReadonlyState {
        config,
        x_client,
        authenticated_user_id,
        x_available,
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

// ── Utility profile servers ─────────────────────────────────────────────

/// Run the utility-readonly MCP server on stdio transport (flat toolkit reads).
async fn run_utility_readonly_server(config: Config) -> anyhow::Result<()> {
    let state = init_readonly_state(config, Profile::UtilityReadonly).await?;
    let server = UtilityReadonlyMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (utility-readonly profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;
    Ok(())
}

/// Run the utility-write MCP server on stdio transport (flat toolkit reads + writes + engages).
async fn run_utility_write_server(config: Config) -> anyhow::Result<()> {
    let state = init_readonly_state(config, Profile::UtilityWrite).await?;
    let server = UtilityWriteMcpServer::new(state);

    tracing::info!("Starting Tuitbot MCP server on stdio (utility-write profile)");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {e}"))?;

    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: `init_readonly_state` must not bail when
    /// `provider_backend = "scraper"` and no browser session file exists.
    /// This was the root cause of the MCP readonly crash reported by users.
    #[tokio::test]
    async fn init_readonly_scraper_no_session_does_not_fail() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();

        let state = init_readonly_state(config, Profile::Readonly).await;
        assert!(
            state.is_ok(),
            "init_readonly_state should succeed with scraper backend even without a session: {:?}",
            state.err()
        );
    }

    /// Regression test: `init_readonly_state` must gracefully degrade when
    /// `provider_backend = "x_api"` (default) and no token file exists.
    #[tokio::test]
    async fn init_readonly_x_api_no_tokens_does_not_fail() {
        let config = Config::default(); // provider_backend defaults to ""  → x_api

        let state = init_readonly_state(config, Profile::Readonly).await;
        assert!(
            state.is_ok(),
            "init_readonly_state should succeed without tokens (graceful degradation): {:?}",
            state.err()
        );

        let state = state.unwrap();
        assert!(
            !state.x_available,
            "x_available should be false when no tokens are present"
        );
    }

    /// Same regression check for api-readonly profile with scraper backend.
    #[tokio::test]
    async fn init_readonly_scraper_api_readonly_profile() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();

        let state = init_readonly_state(config, Profile::ApiReadonly).await;
        assert!(
            state.is_ok(),
            "api-readonly + scraper should not crash: {:?}",
            state.err()
        );
    }

    /// Utility-readonly profile should degrade gracefully without tokens.
    #[tokio::test]
    async fn init_readonly_utility_readonly_no_tokens() {
        let config = Config::default();
        let state = init_readonly_state(config, Profile::UtilityReadonly).await;
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(!state.x_available);
    }

    /// Utility-write profile should degrade gracefully without tokens.
    #[tokio::test]
    async fn init_readonly_utility_write_no_tokens() {
        let config = Config::default();
        let state = init_readonly_state(config, Profile::UtilityWrite).await;
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(!state.x_available);
        assert!(state.authenticated_user_id.is_empty());
    }

    /// Scraper backend with utility-readonly profile.
    #[tokio::test]
    async fn init_readonly_scraper_utility_readonly() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();

        let state = init_readonly_state(config, Profile::UtilityReadonly).await;
        assert!(state.is_ok());
    }

    /// Scraper backend with utility-write profile.
    #[tokio::test]
    async fn init_readonly_scraper_utility_write() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();

        let state = init_readonly_state(config, Profile::UtilityWrite).await;
        assert!(state.is_ok());
    }

    /// Readonly state has config accessible.
    #[tokio::test]
    async fn readonly_state_config_accessible() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();

        let state = init_readonly_state(config, Profile::Readonly)
            .await
            .unwrap();
        // Config should be accessible
        assert_eq!(state.config.x_api.provider_backend, "scraper");
    }

    /// Profile display covers all variants.
    #[test]
    fn profile_display_all_variants() {
        assert_eq!(format!("{}", Profile::Readonly), "readonly");
        assert_eq!(format!("{}", Profile::ApiReadonly), "api-readonly");
        assert_eq!(format!("{}", Profile::Write), "write");
        assert_eq!(format!("{}", Profile::Admin), "admin");
        assert_eq!(format!("{}", Profile::UtilityReadonly), "utility-readonly");
        assert_eq!(format!("{}", Profile::UtilityWrite), "utility-write");
    }

    /// Profile parse from string.
    #[test]
    fn profile_parse_roundtrip() {
        for profile in [
            Profile::Readonly,
            Profile::ApiReadonly,
            Profile::Write,
            Profile::Admin,
            Profile::UtilityReadonly,
            Profile::UtilityWrite,
        ] {
            let s = profile.to_string();
            let parsed: Profile = s.parse().unwrap();
            assert_eq!(parsed, profile);
        }
    }

    /// Profile parse unknown string.
    #[test]
    fn profile_parse_unknown_error() {
        let result: Result<Profile, _> = "nonexistent".parse();
        assert!(result.is_err());
    }

    // ── Provider backend parsing ───────────────────────────────────

    #[test]
    fn parse_backend_scraper() {
        assert_eq!(
            provider::parse_backend("scraper"),
            provider::ProviderBackend::Scraper
        );
    }

    #[test]
    fn parse_backend_scraper_uppercase() {
        assert_eq!(
            provider::parse_backend("SCRAPER"),
            provider::ProviderBackend::Scraper
        );
    }

    #[test]
    fn parse_backend_x_api() {
        assert_eq!(
            provider::parse_backend("x_api"),
            provider::ProviderBackend::XApi
        );
    }

    #[test]
    fn parse_backend_empty_defaults_to_x_api() {
        assert_eq!(provider::parse_backend(""), provider::ProviderBackend::XApi);
    }

    #[test]
    fn parse_backend_unknown_defaults_to_x_api() {
        assert_eq!(
            provider::parse_backend("something_else"),
            provider::ProviderBackend::XApi
        );
    }

    // ── Provider backend display ───────────────────────────────────

    #[test]
    fn provider_backend_display_x_api() {
        assert_eq!(provider::ProviderBackend::XApi.to_string(), "x_api");
    }

    #[test]
    fn provider_backend_display_scraper() {
        assert_eq!(provider::ProviderBackend::Scraper.to_string(), "scraper");
    }

    // ── inject_provider_backend ────────────────────────────────────

    #[test]
    fn inject_backend_into_json_with_meta() {
        let input = r#"{"data":{},"meta":{"elapsed":5}}"#;
        let result = provider::inject_provider_backend(input, "scraper");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["meta"]["provider_backend"], "scraper");
        assert_eq!(v["meta"]["elapsed"], 5);
    }

    #[test]
    fn inject_backend_into_json_without_meta() {
        let input = r#"{"data":{}}"#;
        let result = provider::inject_provider_backend(input, "x_api");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["meta"]["provider_backend"], "x_api");
    }

    #[test]
    fn inject_backend_invalid_json_returns_input() {
        let input = "not valid json";
        let result = provider::inject_provider_backend(input, "x_api");
        assert_eq!(result, "not valid json");
    }

    // ── Profile equality and copy ──────────────────────────────────

    #[test]
    fn profile_equality() {
        assert_eq!(Profile::Write, Profile::Write);
        assert_ne!(Profile::Write, Profile::Admin);
        assert_ne!(Profile::Readonly, Profile::ApiReadonly);
    }

    #[test]
    fn profile_clone() {
        let p = Profile::UtilityWrite;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn profile_debug_format() {
        let debug = format!("{:?}", Profile::Admin);
        assert_eq!(debug, "Admin");
    }

    // ── Readonly state x_available checks ──────────────────────────

    #[tokio::test]
    async fn readonly_state_x_not_available_without_tokens() {
        let config = Config::default();
        let state = init_readonly_state(config, Profile::Readonly)
            .await
            .unwrap();
        assert!(!state.x_available);
        assert!(state.authenticated_user_id.is_empty());
    }

    #[tokio::test]
    async fn readonly_state_config_preserved() {
        let mut config = Config::default();
        config.x_api.provider_backend = "scraper".to_string();
        config.business.product_name = "TestBrand".to_string();
        let state = init_readonly_state(config, Profile::Readonly)
            .await
            .unwrap();
        assert_eq!(state.config.business.product_name, "TestBrand");
    }
}
