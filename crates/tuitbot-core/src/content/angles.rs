//! Domain types for the Hook Miner angle mining system.
//!
//! Defines the taxonomy of content angles and evidence types,
//! plus output containers for the mining pipeline.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::llm::TokenUsage;

/// Minimum number of evidence items required to generate angles.
/// Below this threshold, the pipeline returns a fallback state.
pub const MIN_EVIDENCE_COUNT: usize = 2;

/// Minimum average confidence across evidence items.
/// Below this threshold, evidence is considered noise.
pub const MIN_EVIDENCE_QUALITY: f64 = 0.3;

/// Content angle archetype for social media posts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngleType {
    /// Narrative-driven angle grounded in a personal or observed story.
    Story,
    /// Structured list angle (e.g., "3 things I learned…").
    Listicle,
    /// Bold opinion angle backed by evidence.
    HotTake,
}

impl fmt::Display for AngleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Story => write!(f, "story"),
            Self::Listicle => write!(f, "listicle"),
            Self::HotTake => write!(f, "hot_take"),
        }
    }
}

/// Type of evidence extracted from vault notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Opposing claims found across different notes.
    Contradiction,
    /// A specific statistic, metric, or quantified claim.
    DataPoint,
    /// A non-obvious insight or surprising connection.
    AhaMoment,
}

impl fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contradiction => write!(f, "contradiction"),
            Self::DataPoint => write!(f, "data_point"),
            Self::AhaMoment => write!(f, "aha_moment"),
        }
    }
}

/// A single piece of evidence extracted from a vault note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// The category of evidence.
    pub evidence_type: EvidenceType,
    /// Short excerpt or paraphrase (max 120 chars).
    pub citation_text: String,
    /// ID of the source content node.
    pub source_node_id: i64,
    /// Title of the source note.
    pub source_note_title: String,
    /// Optional heading hierarchy within the note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_heading_path: Option<String>,
    /// Extraction confidence (0.0–1.0).
    pub confidence: f64,
}

/// A content angle synthesized from extracted evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinedAngle {
    /// The angle archetype.
    pub angle_type: AngleType,
    /// Opening tweet text (max 280 chars).
    pub seed_text: String,
    /// Character count of `seed_text`.
    pub char_count: usize,
    /// Evidence items supporting this angle.
    pub evidence: Vec<EvidenceItem>,
    /// Confidence heuristic: "high" or "medium".
    pub confidence: String,
    /// One-sentence explanation of why this angle works.
    pub rationale: String,
}

/// Output from the angle mining pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngleMiningOutput {
    /// Generated angles (0–3).
    pub angles: Vec<MinedAngle>,
    /// If set, explains why full angle generation was skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    /// Average confidence across all extracted evidence.
    pub evidence_quality_score: f64,
    /// Token usage for the pipeline (extraction + generation).
    pub usage: TokenUsage,
    /// Model that produced the output.
    pub model: String,
    /// Provider name (e.g., "openai", "anthropic").
    pub provider: String,
}

/// Assign confidence label based on evidence strength.
///
/// Returns "high" if there are 2+ evidence items with average
/// confidence >= 0.6, otherwise "medium".
pub fn assign_angle_confidence(evidence: &[EvidenceItem]) -> String {
    if evidence.len() >= 2 {
        let avg = evidence.iter().map(|e| e.confidence).sum::<f64>() / evidence.len() as f64;
        if avg >= 0.6 {
            return "high".to_string();
        }
    }
    "medium".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_evidence(confidence: f64) -> EvidenceItem {
        EvidenceItem {
            evidence_type: EvidenceType::DataPoint,
            citation_text: "45% growth in Q3".to_string(),
            source_node_id: 1,
            source_note_title: "Metrics".to_string(),
            source_heading_path: None,
            confidence,
        }
    }

    #[test]
    fn angle_type_serde_roundtrip() {
        for variant in [AngleType::Story, AngleType::Listicle, AngleType::HotTake] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AngleType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn evidence_type_serde_roundtrip() {
        for variant in [
            EvidenceType::Contradiction,
            EvidenceType::DataPoint,
            EvidenceType::AhaMoment,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EvidenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }

    #[test]
    fn angle_type_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&AngleType::HotTake).unwrap(),
            "\"hot_take\""
        );
        assert_eq!(
            serde_json::to_string(&AngleType::Story).unwrap(),
            "\"story\""
        );
        assert_eq!(
            serde_json::to_string(&AngleType::Listicle).unwrap(),
            "\"listicle\""
        );
    }

    #[test]
    fn evidence_type_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&EvidenceType::AhaMoment).unwrap(),
            "\"aha_moment\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::DataPoint).unwrap(),
            "\"data_point\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::Contradiction).unwrap(),
            "\"contradiction\""
        );
    }

    #[test]
    fn assign_confidence_high() {
        let evidence = vec![sample_evidence(0.8), sample_evidence(0.7)];
        assert_eq!(assign_angle_confidence(&evidence), "high");
    }

    #[test]
    fn assign_confidence_medium_single_item() {
        let evidence = vec![sample_evidence(0.9)];
        assert_eq!(assign_angle_confidence(&evidence), "medium");
    }

    #[test]
    fn assign_confidence_medium_low_avg() {
        let evidence = vec![
            sample_evidence(0.3),
            sample_evidence(0.4),
            sample_evidence(0.5),
        ];
        assert_eq!(assign_angle_confidence(&evidence), "medium");
    }

    #[test]
    fn mining_output_serialization() {
        let output = AngleMiningOutput {
            angles: vec![MinedAngle {
                angle_type: AngleType::Story,
                seed_text: "A test seed".to_string(),
                char_count: 11,
                evidence: vec![sample_evidence(0.8)],
                confidence: "high".to_string(),
                rationale: "Strong narrative".to_string(),
            }],
            fallback_reason: None,
            evidence_quality_score: 0.8,
            usage: TokenUsage::default(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"angle_type\":\"story\""));
        assert!(!json.contains("fallback_reason"));
    }

    #[test]
    fn mining_output_fallback_serialization() {
        let output = AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("insufficient_evidence".to_string()),
            evidence_quality_score: 0.1,
            usage: TokenUsage::default(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"fallback_reason\":\"insufficient_evidence\""));
        assert!(json.contains("\"angles\":[]"));
    }
}
