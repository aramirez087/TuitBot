//! QA type definitions and structures.

use serde::{Deserialize, Serialize};

/// Severity level for QA flags.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum QaSeverity {
    /// Warning-level issue that should be reviewed.
    Soft,
    /// Blocks approval/publish unless explicitly overridden.
    Hard,
}

/// Category of QA issue.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QaCategory {
    /// Language policy or detection issue.
    Language,
    /// Brand voice or tone issue.
    Brand,
    /// Legal/regulatory compliance issue.
    Compliance,
}

/// A single QA flag/issue found during evaluation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct QaFlag {
    /// Machine-readable code for this flag (e.g., "language_mismatch").
    pub code: String,
    /// Severity level (soft warning vs hard blocker).
    pub severity: QaSeverity,
    /// Category of the issue.
    pub category: QaCategory,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Optional evidence/example text that triggered the flag.
    pub evidence: Option<String>,
    /// Optional remediation suggestion.
    pub suggestion: Option<String>,
}

/// Language detection result.
// TODO: used by language detection helpers once integrated into full QA pipeline
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageDetection {
    /// ISO 639-1 language code (e.g., "en", "es", "fr").
    pub code: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Per-category QA scores.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct QaScoreSummary {
    /// Overall QA score (0–100).
    pub overall: f32,
    /// Language policy compliance score (0–100).
    pub language: f32,
    /// Brand voice alignment score (0–100).
    pub brand: f32,
    /// Regulatory compliance score (0–100).
    pub compliance: f32,
}

/// Language detection context for a report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QaLanguages {
    /// Detected language of source/input text.
    pub source: Option<String>,
    /// Detected language of generated output.
    pub output: Option<String>,
    /// Target language per policy.
    pub policy_target: String,
}

/// Complete QA evaluation report.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QaReport {
    /// Whether hard flags require explicit override before publishing.
    pub requires_override: bool,
    /// Language context and detection results.
    pub languages: QaLanguages,
    /// Hard (blocking) flags found.
    pub hard_flags: Vec<QaFlag>,
    /// Soft (warning) flags found.
    pub soft_flags: Vec<QaFlag>,
    /// List of remediation recommendations.
    pub recommendations: Vec<String>,
    /// QA scores by category.
    pub score: QaScoreSummary,
}
