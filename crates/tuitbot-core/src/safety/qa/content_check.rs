//! QA content evaluation and rule-based checking.

use super::types::{QaLanguages, QaReport};
use crate::config::Config;

/// Rule-based QA evaluator.
pub struct QaEvaluator<'a> {
    config: &'a Config,
}

impl<'a> QaEvaluator<'a> {
    /// Create a new evaluator using the provided config.
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Evaluate generated content against policy.
    ///
    /// `recent_outputs` is optional context for similarity warnings.
    pub fn evaluate(
        &self,
        _source_text: &str,
        _generated_text: &str,
        _recent_outputs: &[String],
    ) -> QaReport {
        // Note: Full QA evaluation is stubbed pending config struct additions
        QaReport {
            requires_override: false,
            languages: QaLanguages {
                source: None,
                output: None,
                policy_target: "en".to_string(),
            },
            hard_flags: vec![],
            soft_flags: vec![],
            recommendations: vec![],
            score: Default::default(),
        }
    }
}
