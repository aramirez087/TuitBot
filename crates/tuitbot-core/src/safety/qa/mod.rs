//! Deterministic QA evaluator for generated content.
//!
//! This module builds a structured QA report that can be persisted alongside
//! drafts and approval items. It is intentionally rule-based and deterministic
//! so behavior is predictable and testable.

<<<<<<< HEAD
pub use content_check::QaEvaluator;
pub use types::{QaCategory, QaFlag, QaLanguages, QaReport, QaScoreSummary, QaSeverity};

mod account_check;
mod content_check;
mod scoring;
pub mod types;
=======
pub use crate::safety::qa::content_check::QaEvaluator;
pub use crate::safety::qa::scoring::{collect_recommendations, score_summary};
pub use crate::safety::qa::types::{
    QaCategory, QaFlag, QaLanguages, QaReport, QaScoreSummary, QaSeverity,
};

mod account_check;
mod content_check;
mod helpers;
mod scoring;
mod types;
>>>>>>> 388a639 (refactor: split schedule.rs (1478 lines) → schedule/{mod,planner,executor,recurrence})

#[cfg(test)]
mod tests;
