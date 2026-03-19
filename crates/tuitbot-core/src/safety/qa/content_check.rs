//! Content-level QA evaluation.

use crate::config::Config;

use super::types::{QaLanguages, QaReport};

/// Rule-based QA evaluator for generated content.
pub struct QaEvaluator<'a> {
    #[allow(dead_code)]
    config: &'a Config,
}

impl<'a> QaEvaluator<'a> {
    /// Create a new evaluator using the provided config.
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Evaluate generated content against policy.
    ///
    /// Returns a `QaReport` with flags, scores, and recommendations.
    /// `recent_outputs` provides context for similarity checks.
    pub fn evaluate(
        &self,
        _source_text: &str,
        _generated_text: &str,
        _recent_outputs: &[String],
    ) -> QaReport {
        // Stub: Full evaluation logic pending config struct updates for:
        // - language_policy (LanguagePolicyMode, supported_languages, default_reply_language)
        // - glossary_terms (term, approved_aliases, preserve_exact)
        // - brand_voice_profile (tone, themes, prohibited_phrases)
        // - forbidden_terms (compliance words)
        // - link_policy (blocked_domains, require_utm_params)
        //
        // For now, return a passing report. Once config is extended, this
        // stub will be replaced with full rule-based evaluation logic.
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
            score: super::types::QaScoreSummary {
                overall: 100.0,
                language: 100.0,
                brand: 100.0,
                compliance: 100.0,
            },
        }
    }
}
