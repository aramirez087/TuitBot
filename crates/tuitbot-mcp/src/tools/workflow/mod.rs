//! Workflow-only MCP tool implementations.
//!
//! These tools require the full workflow stack (DB, LLM, policy gate)
//! and are NOT available in the API profile.

pub mod actions;
pub mod analytics;
pub mod approval;
pub mod capabilities;
pub mod composite;
pub mod content;
pub mod context;
pub mod discovery;
pub mod health;
pub mod mutation_audit;
pub mod policy_gate;
pub mod rate_limits;
pub mod replies;
pub mod targets;
pub mod telemetry;
pub mod x_actions;
