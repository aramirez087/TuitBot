//! Contract layer: protocol-level types reusable by any MCP consumer.
//!
//! Defines the response envelope, error taxonomy, and provider error mapping.
//! These types carry no TuitBot workflow assumptions.

pub mod envelope;
pub mod error;

pub use envelope::{ToolError, ToolMeta, ToolResponse};
pub use error::ProviderError;
