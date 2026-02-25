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
pub mod discovery;
pub mod health;
pub mod policy_gate;
pub mod rate_limits;
pub mod replies;
pub mod response;
pub mod scoring;
pub mod targets;
pub mod x_actions;

#[cfg(test)]
mod benchmark;
