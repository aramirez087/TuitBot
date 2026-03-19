//! Deterministic QA evaluator for generated content.
//!
//! This module builds a structured QA report that can be persisted alongside
//! drafts and approval items. It is intentionally rule-based and deterministic
//! so behavior is predictable and testable.

pub use content_check::QaEvaluator;
pub use types::{QaCategory, QaFlag, QaLanguages, QaReport, QaScoreSummary, QaSeverity};

mod account_check;
mod content_check;
mod scoring;
pub mod types;

#[cfg(test)]
mod tests;
