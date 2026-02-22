//! Shared application state for the MCP server.
//!
//! Bundles the database pool, configuration, and optional LLM provider
//! so that all tool handlers can access them through the server struct.

use std::sync::Arc;

use tuitbot_core::config::Config;
use tuitbot_core::llm::LlmProvider;
use tuitbot_core::storage::DbPool;

/// Shared state accessible by all MCP tool handlers.
pub struct AppState {
    /// SQLite connection pool.
    pub pool: DbPool,
    /// Loaded and validated configuration.
    pub config: Config,
    /// Optional LLM provider (None if not configured or creation failed).
    pub llm_provider: Option<Box<dyn LlmProvider>>,
}

/// Thread-safe reference to shared state.
pub type SharedState = Arc<AppState>;
