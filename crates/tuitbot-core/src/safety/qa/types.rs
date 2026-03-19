//! Type definitions for QA evaluation.

use serde::{Deserialize, Serialize};

/// Severity used for QA flags.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QaSeverity {
    /// Blocks approval/publish unless explicitly overridden.
    Hard,
    /// Warning-level issue that should be reviewed.
    Soft,
}

/// QA category for scoring and grouping.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QaCategory {
    /// Language and bilingual policy checks.
    Language,
    /// Brand voice style and glossary checks.
    Brand,
    /// Compliance checks (claims, links, UTM requirements).
    Compliance,
}

/// Structured issue emitted by the evaluator.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct QaFlag {
    /// Stable identifier for the rule that fired.
    pub code: String,
    /// Hard vs soft severity.
    pub severity: QaSeverity,
    /// Category used for score rollups.
    pub category: QaCategory,
    /// Human-readable summary.
    pub message: String,
    /// Optional excerpt/value that triggered the flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Optional remediation guidance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Aggregate score rollup for UI display.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct QaScoreSummary {
    /// Overall score in [0, 100].
    pub overall: f32,
    /// Language-policy dimension score in [0, 100].
    pub language: f32,
    /// Brand dimension score in [0, 100].
    pub brand: f32,
    /// Compliance dimension score in [0, 100].
    pub compliance: f32,
}

impl Default for QaScoreSummary {
    fn default() -> Self {
        Self {
            overall: 100.0,
            language: 100.0,
            brand: 100.0,
            compliance: 100.0,
        }
    }
}

/// Captures language detection context.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct QaLanguages {
    /// Detected source language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Detected output language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Policy-selected target language.
    pub policy_target: String,
}

/// Complete QA report artifact.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct QaReport {
    /// Hard failures requiring override/edit.
    pub hard_flags: Vec<QaFlag>,
    /// Soft warnings for reviewer visibility.
    pub soft_flags: Vec<QaFlag>,
    /// Consolidated remediation recommendations.
    pub recommendations: Vec<String>,
    /// Quality score rollup.
    pub score: QaScoreSummary,
    /// Language metadata used by checks.
    pub languages: QaLanguages,
    /// `true` when hard flags exist.
    pub requires_override: bool,
}

impl Default for QaReport {
    fn default() -> Self {
        Self {
            hard_flags: Vec::new(),
            soft_flags: Vec::new(),
            recommendations: Vec::new(),
            score: QaScoreSummary {
                overall: 100.0,
                language: 100.0,
                brand: 100.0,
                compliance: 100.0,
            },
            languages: QaLanguages {
                source: None,
                output: None,
                policy_target: "en".to_string(),
            },
            requires_override: false,
        }
    }
}

/// Internal type capturing language detection results.
#[derive(Debug, Clone)]
pub(super) struct LanguageDetection {
    pub code: String,
    pub confidence: f32,
}
