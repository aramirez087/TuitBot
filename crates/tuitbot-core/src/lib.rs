/// Core library for the Tuitbot autonomous X growth assistant.
///
/// This crate contains all business logic including configuration management,
/// error types, startup helpers, and shared types used by the CLI binary.
pub mod auth;
pub mod automation;
pub mod config;
pub mod content;
pub mod context;
pub mod error;
pub mod llm;
pub mod mcp_policy;
pub mod mutation_gateway;
pub mod net;
pub mod safety;
pub mod scoring;
pub mod startup;
pub mod storage;
pub mod strategy;
pub mod toolkit;
pub mod workflow;
pub mod x_api;

pub use error::*;

/// Returns the version of the tuitbot-core library.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
