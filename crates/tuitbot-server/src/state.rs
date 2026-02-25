//! Shared application state for the tuitbot server.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
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
    /// Optional automation runtime handle for start/stop control.
    pub runtime: Mutex<Option<Runtime>>,
    /// Optional content generator for AI assist endpoints.
    pub content_generator: Option<Arc<ContentGenerator>>,
}
