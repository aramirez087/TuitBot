//! Shared application state for the tuitbot server.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tuitbot_core::automation::circuit_breaker::CircuitBreaker;
use tuitbot_core::automation::Runtime;
use tuitbot_core::config::{ContentSourcesConfig, DeploymentMode};
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::storage::DbPool;

use crate::ws::WsEvent;

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
    /// Deployment mode (desktop, self_host, or cloud).
    pub deployment_mode: DeploymentMode,
}
