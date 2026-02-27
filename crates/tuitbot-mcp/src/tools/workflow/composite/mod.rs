//! Composite goal-oriented MCP tools.
//!
//! Each tool delegates to `tuitbot_core::workflow` steps, adding only
//! MCP-specific concerns: parameter parsing, response envelope wrapping,
//! and telemetry recording.
//!
//! Shared IO types (`ScoredCandidate`, `DraftResult`, `ProposeResult`,
//! `ScoreBreakdown`) live in `tuitbot_core::workflow` and are re-exported
//! here for backward compatibility.

pub mod draft_replies;
pub mod find_opportunities;
pub mod propose_queue;
pub mod thread_plan;

#[cfg(test)]
mod tests;

// Re-export shared IO types from core workflow layer.
// These are the canonical definitions — no local copies.
// Allowed unused: these are public API re-exports for external consumers.
#[allow(unused_imports)]
pub use tuitbot_core::workflow::{DraftResult, ProposeResult, ScoreBreakdown, ScoredCandidate};
