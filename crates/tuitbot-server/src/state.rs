//! Shared application state for the tuitbot server.

use std::path::PathBuf;

use tokio::sync::{broadcast, Mutex};
use tuitbot_core::automation::Runtime;
use tuitbot_core::storage::DbPool;

use crate::ws::WsEvent;

/// Shared application state accessible by all route handlers.
pub struct AppState {
    /// SQLite connection pool.
    pub db: DbPool,
    /// Path to the configuration file.
    pub config_path: PathBuf,
    /// Broadcast channel sender for real-time WebSocket events.
    pub event_tx: broadcast::Sender<WsEvent>,
    /// Local bearer token for API authentication.
    pub api_token: String,
    /// Optional automation runtime handle for start/stop control.
    pub runtime: Mutex<Option<Runtime>>,
}
