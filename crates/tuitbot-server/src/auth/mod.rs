//! Authentication layer for the tuitbot API server.
//!
//! Supports two authentication strategies:
//! - **Bearer token**: File-based token for Tauri desktop and API/MCP clients
//! - **Session cookie**: Passphrase-based login for web/LAN access

pub mod middleware;
pub mod routes;
pub mod token;

pub use middleware::auth_middleware;
pub use token::ensure_api_token;
