//! MCP mutation policy evaluator.
//!
//! Provides a centralized gate that sits between MCP tool invocation and
//! side effects, enforcing allow/deny/route-to-approval decisions based
//! on configuration, rate limits, and operating mode.

mod evaluator;

#[cfg(test)]
mod tests;

pub use evaluator::{McpPolicyEvaluator, PolicyAuditRecord, PolicyDecision, PolicyDenialReason};
