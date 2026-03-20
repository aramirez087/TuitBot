//! Tests for winning_dna split into focused modules.
//!
//! - `unit`        — core unit tests: classification, scoring, helpers, formatting
//! - `edge_cases`  — additional edge case tests for coverage
//! - `structs`     — struct debug/clone/constants tests
//! - `integration` — async DB integration tests

mod edge_cases;
mod integration;
mod structs;
mod unit;
