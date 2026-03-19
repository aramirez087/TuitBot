//! QA scoring and recommendation generation.

use std::collections::{HashMap, HashSet};

use super::types::{QaCategory, QaFlag, QaScoreSummary};

/// Generate a score summary from hard and soft flags.
///
/// Hard flags incur a 35-point penalty per flag, soft flags incur a 12-point penalty.
/// Each category's score is independently computed, then the overall score is the average.
pub fn score_summary(hard_flags: &[QaFlag], soft_flags: &[QaFlag]) -> QaScoreSummary {
    let mut penalties: HashMap<QaCategory, f32> = HashMap::new();
    for flag in hard_flags {
        *penalties.entry(flag.category.clone()).or_insert(0.0) += 35.0;
    }
    for flag in soft_flags {
        *penalties.entry(flag.category.clone()).or_insert(0.0) += 12.0;
    }

    let language = (100.0 - penalties.get(&QaCategory::Language).copied().unwrap_or(0.0)).max(0.0);
    let brand = (100.0 - penalties.get(&QaCategory::Brand).copied().unwrap_or(0.0)).max(0.0);
    let compliance = (100.0
        - penalties
            .get(&QaCategory::Compliance)
            .copied()
            .unwrap_or(0.0))
    .max(0.0);
    let overall = ((language + brand + compliance) / 3.0).clamp(0.0, 100.0);

    QaScoreSummary {
        overall,
        language,
        brand,
        compliance,
    }
}

/// Collect deduplicated remediation recommendations from flag suggestions.
pub fn collect_recommendations(hard_flags: &[QaFlag], soft_flags: &[QaFlag]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut recommendations = Vec::new();

    for flag in hard_flags.iter().chain(soft_flags.iter()) {
        if let Some(suggestion) = &flag.suggestion {
            if seen.insert(suggestion.clone()) {
                recommendations.push(suggestion.clone());
            }
        }
    }
    recommendations
}
