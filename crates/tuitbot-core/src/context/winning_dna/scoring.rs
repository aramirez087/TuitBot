//! Engagement scoring and retrieval weight computation.
//!
//! Pure functions — no I/O, no DB. Safe to unit-test in isolation.

use crate::context::winning_dna::COLD_START_WEIGHT;

/// Normalize a raw performance_score to 0.0-1.0 range.
///
/// Returns `COLD_START_WEIGHT` (0.5) if max_score is zero (cold-start).
pub fn compute_engagement_score(performance_score: f64, max_score: f64) -> f64 {
    if max_score <= 0.0 {
        return COLD_START_WEIGHT;
    }
    (performance_score / max_score).clamp(0.0, 1.0)
}

/// Compute retrieval weight with exponential recency decay.
///
/// Formula: `engagement_score * exp(-0.693 * days_since / half_life)`
///
/// At `days_since = half_life`, weight ≈ engagement_score / 2.
/// At `days_since = 4 * half_life` (56 days), weight ≈ engagement_score * 0.0625.
pub fn compute_retrieval_weight(engagement_score: f64, days_since: f64, half_life: f64) -> f64 {
    if half_life <= 0.0 {
        return engagement_score;
    }
    engagement_score * (-0.693 * days_since / half_life).exp()
}
