//! MCP tool implementations for Tuitbot.
//!
//! Shared tools live at this level; workflow-only tools are gated
//! behind the `workflow` submodule.

pub mod config;
pub mod idempotency;
#[allow(dead_code)]
pub mod manifest;
pub mod response;
pub mod rollback;
pub mod scoring;

pub mod workflow;

#[cfg(test)]
pub(crate) mod test_mocks;

#[cfg(test)]
mod benchmark;
#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod conformance_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod eval_harness;
#[cfg(test)]
mod eval_session09;
#[cfg(test)]
mod golden_fixtures;
