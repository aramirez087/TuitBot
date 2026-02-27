//! Authentication module for Tuitbot.
//!
//! Provides passphrase-based authentication and session management for
//! web/LAN access. Bearer tokens remain the primary auth method for
//! Tauri desktop and API clients.

pub mod error;
pub mod passphrase;
pub mod session;
