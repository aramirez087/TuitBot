//! MCP tool implementations for Tuitbot.
//!
//! Each sub-module groups related tools by domain.

pub mod actions;
pub mod analytics;
pub mod approval;
pub mod capabilities;
pub mod composite;
pub mod config;
pub mod content;
pub mod context;
pub mod discovery;
pub mod health;
pub mod policy_gate;
pub mod rate_limits;
pub mod replies;
pub mod response;
pub mod scoring;
pub mod targets;
pub mod telemetry;
pub mod x_actions;

#[cfg(test)]
mod benchmark;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod eval_harness;
