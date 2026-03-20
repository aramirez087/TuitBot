//! Winning DNA classification, scoring, and retrieval.
//!
//! Classifies historical tweet output into archetypes, computes
//! engagement-weighted success scores, and retrieves ranked ancestors
//! for use as RAG context in new draft generation.
//!
//! ## Module layout
//! - `scoring`  — pure engagement score + recency-decay functions (no I/O)
//! - `analysis` — classification, retrieval, context builder, prompt formatting
//! - `tests`    — unit and DB integration tests

pub mod analysis;
pub mod scoring;

#[cfg(test)]
mod tests;
// tests/ is a submodule directory: tests/mod.rs → unit, edge_cases, structs, integration

// Re-export public API so callers use `winning_dna::*` unchanged.
pub use analysis::{
    build_draft_context, build_draft_context_with_selection, classify_reply_archetype,
    classify_tweet_format, retrieve_ancestors, retrieve_cold_start_seeds,
};
pub use scoring::{compute_engagement_score, compute_retrieval_weight};

use crate::context::retrieval::VaultCitation;

// ============================================================================
// Thresholds (documented in rag-ranking.md)
// ============================================================================

/// Exponential decay half-life for retrieval weight (days).
/// Content success patterns change; older hits contribute less.
pub const RECENCY_HALF_LIFE_DAYS: f64 = 14.0;

/// Maximum number of winning ancestors to include in a draft context.
pub const MAX_ANCESTORS: u32 = 5;

/// Default engagement weight for unscored content (cold-start baseline).
pub const COLD_START_WEIGHT: f64 = 0.5;

/// Minimum engagement score to include an ancestor in retrieval.
/// Filters out bottom ~10% performers.
pub const MIN_ENGAGEMENT_SCORE: f64 = 0.1;

/// Maximum character count for the formatted RAG prompt block.
/// Conservative estimate at ~500 tokens (4 chars/token).
pub const RAG_MAX_CHARS: usize = 2000;

/// Maximum character count for the ancestor prompt section when combined with fragments.
pub const MAX_ANCESTOR_CHARS: usize = 800;

/// Maximum number of cold-start seeds to retrieve as fallback.
pub const MAX_COLD_START_SEEDS: u32 = 5;

// ============================================================================
// Structs
// ============================================================================

/// A historically successful tweet classified with engagement data.
#[derive(Debug, Clone)]
pub struct WinningAncestor {
    /// Tweet or reply ID.
    pub tweet_id: String,
    /// Truncated content preview (up to 120 chars).
    pub content_preview: String,
    /// "reply" or "tweet".
    pub content_type: String,
    /// Classified archetype/format name.
    pub archetype_vibe: String,
    /// Normalized engagement score (0.0-1.0).
    pub engagement_score: f64,
    /// Engagement score weighted by recency decay.
    pub retrieval_weight: f64,
    /// When the content was posted (ISO-8601).
    pub posted_at: String,
}

/// Context block ready for injection into LLM prompts.
#[derive(Debug, Clone)]
pub struct DraftContext {
    /// High-performing historical content for reference.
    pub winning_ancestors: Vec<WinningAncestor>,
    /// Content seeds from ingested notes (cold-start fallback).
    pub content_seeds: Vec<ContentSeedContext>,
    /// Structured citations for vault fragments used in the prompt.
    pub vault_citations: Vec<VaultCitation>,
    /// Formatted text block for LLM prompt injection.
    pub prompt_block: String,
}

/// A content seed from an ingested note.
#[derive(Debug, Clone)]
pub struct ContentSeedContext {
    /// The seed hook text.
    pub seed_text: String,
    /// Title from the originating note.
    pub source_title: Option<String>,
    /// Suggested archetype for the seed.
    pub archetype_suggestion: Option<String>,
    /// Engagement weight for ranking.
    pub engagement_weight: f64,
}
