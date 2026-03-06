//! Shared application state for the tuitbot server.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tuitbot_core::automation::circuit_breaker::CircuitBreaker;
use tuitbot_core::automation::Runtime;
use tuitbot_core::config::{
    effective_config, Config, ConnectorConfig, ContentSourcesConfig, DeploymentMode,
};
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::storage::accounts::{self, DEFAULT_ACCOUNT_ID};
use tuitbot_core::storage::DbPool;
use tuitbot_core::x_api::auth::TokenManager;

use tuitbot_core::error::XApiError;
use tuitbot_core::x_api::auth;

use crate::ws::WsEvent;

/// Pending OAuth PKCE state for connector link flows.
pub struct PendingOAuth {
    /// The PKCE code verifier needed to complete the token exchange.
    pub code_verifier: String,
    /// When this entry was created (for 10-minute expiry).
    pub created_at: Instant,
    /// The account ID that initiated this OAuth flow (empty for connectors).
    pub account_id: String,
}

/// Shared application state accessible by all route handlers.
pub struct AppState {
    /// SQLite connection pool.
    pub db: DbPool,
    /// Path to the configuration file.
    pub config_path: PathBuf,
    /// Data directory for media storage (parent of config file).
    pub data_dir: PathBuf,
    /// Broadcast channel sender for real-time WebSocket events.
    pub event_tx: broadcast::Sender<WsEvent>,
    /// Local bearer token for API authentication.
    pub api_token: String,
    /// Bcrypt hash of the web login passphrase (None if not configured).
    pub passphrase_hash: RwLock<Option<String>>,
    /// Last-observed mtime of the `passphrase_hash` file (for detecting out-of-band resets).
    pub passphrase_hash_mtime: RwLock<Option<SystemTime>>,
    /// Host address the server is bound to.
    pub bind_host: String,
    /// Port the server is listening on.
    pub bind_port: u16,
    /// Per-IP login attempt tracking for rate limiting: (count, window_start).
    pub login_attempts: Mutex<HashMap<IpAddr, (u32, Instant)>>,
    /// Per-account automation runtimes (keyed by account_id).
    pub runtimes: Mutex<HashMap<String, Runtime>>,
    /// Per-account content generators for AI assist endpoints.
    pub content_generators: Mutex<HashMap<String, Arc<ContentGenerator>>>,
    /// Optional circuit breaker for X API rate-limit protection.
    pub circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Cancellation token for the Watchtower filesystem watcher (None if not running).
    pub watchtower_cancel: Option<CancellationToken>,
    /// Content sources configuration for the Watchtower.
    pub content_sources: ContentSourcesConfig,
    /// Connector configuration for remote source OAuth flows.
    pub connector_config: ConnectorConfig,
    /// Deployment mode (desktop, self_host, or cloud).
    pub deployment_mode: DeploymentMode,
    /// Pending OAuth PKCE challenges keyed by state parameter.
    pub pending_oauth: Mutex<HashMap<String, PendingOAuth>>,
    /// Per-account X API token managers for automatic token refresh.
    pub token_managers: Mutex<HashMap<String, Arc<TokenManager>>>,
    /// X API client ID from config (needed to create token managers).
    pub x_client_id: String,
}

impl AppState {
    /// Get a fresh X API access token for the given account.
    ///
    /// Lazily creates a `TokenManager` on first use (loading tokens from disk),
    /// then returns a token that is automatically refreshed before expiry.
    pub async fn get_x_access_token(
        &self,
        token_path: &std::path::Path,
        account_id: &str,
    ) -> Result<String, XApiError> {
        // Fast path: token manager already exists.
        {
            let managers = self.token_managers.lock().await;
            if let Some(tm) = managers.get(account_id) {
                return tm.get_access_token().await;
            }
        }

        // Load tokens from disk and create a new manager.
        let tokens = auth::load_tokens(token_path)?.ok_or(XApiError::AuthExpired)?;

        let tm = Arc::new(TokenManager::new(
            tokens,
            self.x_client_id.clone(),
            token_path.to_path_buf(),
        ));

        let access_token = tm.get_access_token().await?;

        self.token_managers
            .lock()
            .await
            .insert(account_id.to_string(), tm);

        Ok(access_token)
    }

    /// Load the effective config for a given account.
    ///
    /// Default account: reads config.toml directly (backward compat).
    /// Non-default: merges config.toml base with account's `config_overrides` from DB.
    pub async fn load_effective_config(&self, account_id: &str) -> Result<Config, String> {
        let contents = std::fs::read_to_string(&self.config_path).unwrap_or_default();
        let base: Config = toml::from_str(&contents).unwrap_or_default();

        if account_id == DEFAULT_ACCOUNT_ID {
            return Ok(base);
        }

        let account = accounts::get_account(&self.db, account_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("account not found: {account_id}"))?;

        effective_config(&base, &account.config_overrides)
            .map(|r| r.config)
            .map_err(|e| e.to_string())
    }

    /// Lazily create or return a cached `ContentGenerator` for the given account.
    ///
    /// Loads effective config, creates the LLM provider, and caches the generator.
    pub async fn get_or_create_content_generator(
        &self,
        account_id: &str,
    ) -> Result<Arc<ContentGenerator>, String> {
        // Fast path: already cached.
        {
            let generators = self.content_generators.lock().await;
            if let Some(gen) = generators.get(account_id) {
                return Ok(gen.clone());
            }
        }

        let config = self.load_effective_config(account_id).await?;

        let provider =
            create_provider(&config.llm).map_err(|e| format!("LLM not configured: {e}"))?;

        let gen = Arc::new(ContentGenerator::new(provider, config.business));

        self.content_generators
            .lock()
            .await
            .insert(account_id.to_string(), gen.clone());

        Ok(gen)
    }
}
