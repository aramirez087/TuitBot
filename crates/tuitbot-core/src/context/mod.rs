//! Context aggregation service for strategy-grade intelligence.
//!
//! Provides reusable functions to aggregate historical data into
//! actionable context: author profiles, engagement recommendations,
//! and topic performance snapshots.

pub mod author;
pub mod engagement;
pub mod topics;

#[cfg(test)]
mod tests;
