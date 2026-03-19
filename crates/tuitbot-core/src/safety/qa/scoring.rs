//! QA scoring and recommendation collection.

use super::types::{QaFlag, QaScoreSummary};

/// Compute aggregate QA scores from hard and soft flags.
pub(super) fn score_summary(hard_flags: &[QaFlag], soft_flags: &[QaFlag]) -> QaScoreSummary {
    let hard_count = hard_flags.len() as f32;
    let soft_count = soft_flags.len() as f32;

    // Penalty-based scoring: each hard flag deducts 20 points, each soft flag deducts 5 points.
    let overall = if hard_count == 0.0 && soft_count == 0.0 {
        100.0
    } else {
        (100.0 - (hard_count * 20.0) - (soft_count * 5.0)).max(0.0)
    };

    QaScoreSummary {
        overall,
        language: 100.0,
        brand: 100.0,
        compliance: 100.0,
    }
}

/// Collect unique remediation recommendations from flags.
pub(super) fn collect_recommendations(hard_flags: &[QaFlag], soft_flags: &[QaFlag]) -> Vec<String> {
    let mut recommendations = std::collections::HashSet::new();

    for flag in hard_flags.iter().chain(soft_flags.iter()) {
        if let Some(suggestion) = &flag.suggestion {
            recommendations.insert(suggestion.clone());
        }
    }

    let mut result: Vec<String> = recommendations.into_iter().collect();
    result.sort();
    result
}
