/// Core library for the ReplyGuy autonomous X growth assistant.
///
/// This crate contains all business logic including configuration management,
/// error types, and shared types used by the CLI binary.
pub mod config;
pub mod content;
pub mod error;
pub mod llm;
pub mod safety;
pub mod storage;
pub mod x_api;

pub use error::*;

/// Returns the version of the replyguy-core library.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
