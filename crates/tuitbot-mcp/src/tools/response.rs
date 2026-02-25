//! Re-exports from the contract layer.
//!
//! All envelope types now live in [`crate::contract::envelope`].
//! This module exists so existing `tools::*` imports continue to resolve.

#[allow(unused_imports)]
pub use crate::contract::envelope::{ToolError, ToolMeta, ToolResponse};
