//! MCP mutation policy evaluator.
//!
//! Provides a centralized gate that sits between MCP tool invocation and
//! side effects, enforcing allow/deny/route-to-approval decisions based
//! on configuration, rate limits, and operating mode.
//!
//! v2 adds multi-dimensional rules, templates, and per-dimension rate limits.

mod evaluator;
pub mod migration;
pub mod rules;
pub mod templates;
pub mod types;

#[cfg(test)]
mod tests;

pub use evaluator::{McpPolicyEvaluator, PolicyAuditRecord, PolicyDecision, PolicyDenialReason};
