//! Request structs for MCP tool parameters.
//!
//! Extracted from `server.rs` to keep the tool router focused on routing
//! and to share request types across primitive and composite tools.

mod query_requests;
mod tool_requests;
mod write_requests;

pub use query_requests::*;
pub use tool_requests::*;
pub use write_requests::*;
