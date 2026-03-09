//! Profile inference engine for onboarding.
//!
//! Analyzes a connected X account's profile and recent tweets to produce
//! normalized onboarding suggestions with confidence scores and provenance.
//!
//! Two-pass architecture:
//! 1. **Heuristics** — deterministic extraction from bio, display name, and profile URL.
//! 2. **LLM enrichment** — optional semantic analysis for fields that benefit from it
//!    (target audience, keywords, topics, brand voice).

mod heuristics;
mod llm_enrichment;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::x_api::types::{Tweet, User};

// Re-export submodule entry points.
pub use heuristics::extract_heuristics;
pub use llm_enrichment::enrich_with_llm;

/// How confident the inference is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Where the inferred value came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Bio,
    Tweets,
    BioAndTweets,
    ProfileUrl,
    DisplayName,
    Default,
}

/// A single inferred field with confidence and provenance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredField<T: Serialize> {
    pub value: T,
    pub confidence: Confidence,
    pub provenance: Provenance,
}

/// The complete set of inferred onboarding fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredProfile {
    pub account_type: InferredField<String>,
    pub product_name: InferredField<String>,
    pub product_description: InferredField<String>,
    pub product_url: InferredField<Option<String>>,
    pub target_audience: InferredField<String>,
    pub product_keywords: InferredField<Vec<String>>,
    pub industry_topics: InferredField<Vec<String>>,
    pub brand_voice: InferredField<Option<String>>,
}

/// Raw input data for the inference pipeline.
pub struct ProfileInput {
    pub user: User,
    pub tweets: Vec<Tweet>,
}

/// Compute the base confidence level from available input data.
///
/// Per the inference contract:
/// - bio > 20 chars AND >= 10 tweets → High
/// - bio > 0 chars OR >= 5 tweets → Medium
/// - otherwise → Low
pub fn compute_base_confidence(bio_len: usize, tweet_count: usize) -> Confidence {
    if bio_len > 20 && tweet_count >= 10 {
        Confidence::High
    } else if bio_len > 0 || tweet_count >= 5 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}
