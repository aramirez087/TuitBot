//! Shared application state for the tuitbot server.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, Mutex};
use tuitbot_core::automation::circuit_breaker::CircuitBreaker;
use tuitbot_core::automation::Runtime;
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
    pub passphrase_hash: Option<String>,
    /// Per-IP login attempt tracking for rate limiting: (count, window_start).
    pub login_attempts: Mutex<HashMap<IpAddr, (u32, Instant)>>,
    /// Per-account automation runtimes (keyed by account_id).
    pub runtimes: Mutex<HashMap<String, Runtime>>,
    /// Per-account content generators for AI assist endpoints.
    pub content_generators: Mutex<HashMap<String, Arc<ContentGenerator>>>,
    /// Optional circuit breaker for X API rate-limit protection.
    pub circuit_breaker: Option<Arc<CircuitBreaker>>,
}
