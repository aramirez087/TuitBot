//! Context aggregation service for strategy-grade intelligence.
//!
//! Provides reusable functions to aggregate historical data into
//! actionable context: author profiles, engagement recommendations,
//! and topic performance snapshots.

pub mod author;
pub mod engagement;
pub mod graph_expansion;
pub mod hybrid_retrieval;
pub mod retrieval;
pub mod semantic_index;
pub mod semantic_search;
pub mod topics;
pub mod winning_dna;

#[cfg(test)]
mod tests;
