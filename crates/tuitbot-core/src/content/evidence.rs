//! Evidence extraction engine for the Hook Miner.
//!
//! Three-stage pipeline:
//! 1. Regex pre-filter — scans neighbor snippets for numeric data points.
//! 2. LLM extraction — identifies contradictions, data points, and aha moments.
//! 3. Validation — rejects invalid node IDs, truncates citations, deduplicates.

use std::collections::{HashMap, HashSet};

use regex::Regex;
use serde::Deserialize;

use crate::content::angles::{EvidenceItem, EvidenceType};
use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider};

/// Maximum characters for citation text in evidence items.
const MAX_CITATION_CHARS: usize = 120;

/// Minimum confidence threshold — below this is noise.
const MIN_CONFIDENCE_FLOOR: f64 = 0.1;

/// A neighbor note's content projected for the extraction pipeline.
#[derive(Debug, Clone)]
pub struct NeighborContent {
    /// Content node ID.
    pub node_id: i64,
    /// Note title.
    pub note_title: String,
    /// Optional heading hierarchy.
    pub heading_path: Option<String>,
    /// Text snippet (up to 500 chars).
    pub snippet: String,
}

/// A candidate data point found by the regex pre-filter.
#[derive(Debug, Clone)]
pub struct CandidateDataPoint {
    /// The matched text fragment.
    pub text: String,
    /// Source node ID.
    pub source_node_id: i64,
    /// Source note title.
    pub source_note_title: String,
}

// ============================================================================
// Stage 1: Regex pre-filter
// ============================================================================

/// Scan neighbor snippets for numeric patterns that indicate data points.
///
/// Returns candidate evidence items without LLM involvement — pure function.
pub fn pre_filter_data_points(neighbors: &[NeighborContent]) -> Vec<CandidateDataPoint> {
    let patterns = [
        // Percentages: 45%, 3.5%
        r"\d+(?:\.\d+)?%",
        // Dollar amounts: $1,200, $50.00
        r"\$[\d,]+(?:\.\d+)?",
        // Multipliers: 3.5x, 10x
        r"\d+(?:\.\d+)?x\b",
        // ISO dates: 2025-06-15
        r"\d{4}-\d{2}-\d{2}",
        // Counts with units: 150 users, 1000 downloads
        r"\d+\s+(?:users|customers|downloads|revenue|sales|companies|teams|projects)",
    ];

    let combined = patterns.join("|");
    let re = Regex::new(&combined).expect("pre-filter regex is valid");

    let mut candidates = Vec::new();
    for neighbor in neighbors {
        for mat in re.find_iter(&neighbor.snippet) {
            // Extract a short context window around the match
            let start = mat.start().saturating_sub(20);
            let end = (mat.end() + 30).min(neighbor.snippet.len());
            let context = neighbor.snippet[start..end].trim().to_string();

            candidates.push(CandidateDataPoint {
                text: context,
                source_node_id: neighbor.node_id,
                source_note_title: neighbor.note_title.clone(),
            });
        }
    }
    candidates
}

// ============================================================================
// Stage 2: LLM extraction
// ============================================================================

/// Raw evidence item from LLM JSON output (before validation).
#[derive(Debug, Deserialize)]
struct RawEvidenceItem {
    evidence_type: String,
    citation_text: String,
    source_node_id: i64,
    #[serde(default = "default_confidence")]
    confidence: f64,
}

fn default_confidence() -> f64 {
    0.5
}

/// Extract evidence from neighbor content using an LLM.
pub async fn extract_evidence(
    provider: &dyn LlmProvider,
    topic: &str,
    neighbors: &[NeighborContent],
    candidates: &[CandidateDataPoint],
) -> Result<Vec<EvidenceItem>, LlmError> {
    let system = build_extraction_prompt(topic, neighbors, candidates);
    let user_message = "Extract evidence items as a JSON array.".to_string();

    let params = GenerationParams {
        max_tokens: 500,
        temperature: 0.3,
        ..Default::default()
    };

    let resp = provider.complete(&system, &user_message, &params).await?;

    tracing::debug!(
        raw_response = %resp.text,
        "Raw LLM response for evidence extraction"
    );

    parse_evidence_response(&resp.text, neighbors)
}

fn build_extraction_prompt(
    topic: &str,
    neighbors: &[NeighborContent],
    candidates: &[CandidateDataPoint],
) -> String {
    let mut prompt = format!(
        "You are an evidence mining engine. Given a topic and related note snippets, \
         extract evidence items that could support social media content angles.\n\n\
         Topic: {topic}\n\n\
         Related notes:\n"
    );

    for (i, n) in neighbors.iter().enumerate() {
        prompt.push_str(&format!(
            "[{}] (node_id: {}, title: \"{}\") \"{}\"\n",
            i + 1,
            n.node_id,
            n.note_title,
            n.snippet
        ));
    }

    if !candidates.is_empty() {
        prompt.push_str("\nCandidate data points found by scanning:\n");
        for c in candidates {
            prompt.push_str(&format!(
                "- \"{}\" from \"{}\"\n",
                c.text, c.source_note_title
            ));
        }
    }

    prompt.push_str(
        "\nExtract evidence as a JSON array. Each item:\n\
         {\n\
         \x20 \"evidence_type\": \"contradiction\" | \"data_point\" | \"aha_moment\",\n\
         \x20 \"citation_text\": \"exact quote or close paraphrase, max 120 chars\",\n\
         \x20 \"source_node_id\": <integer from the list above>,\n\
         \x20 \"confidence\": <0.0-1.0>\n\
         }\n\n\
         Rules:\n\
         - Only reference node_ids from the list above.\n\
         - citation_text must be grounded in the source snippet.\n\
         - For contradictions, identify opposing claims across different notes.\n\
         - For aha_moments, identify non-obvious connections.\n\
         - For data_points, confirm the candidate data points are relevant to the topic.\n\
         - Return [] if no meaningful evidence found.",
    );

    prompt
}

/// Parse evidence JSON from LLM response, with fallback for fenced code blocks.
pub fn parse_evidence_response(
    text: &str,
    neighbors: &[NeighborContent],
) -> Result<Vec<EvidenceItem>, LlmError> {
    let trimmed = text.trim();

    // Try direct JSON parse first
    if let Ok(raw_items) = serde_json::from_str::<Vec<RawEvidenceItem>>(trimmed) {
        return Ok(convert_raw_evidence(raw_items, neighbors));
    }

    // Fallback: extract JSON array from fenced code block
    if let Some(json_str) = extract_json_from_code_block(trimmed) {
        if let Ok(raw_items) = serde_json::from_str::<Vec<RawEvidenceItem>>(json_str) {
            return Ok(convert_raw_evidence(raw_items, neighbors));
        }
    }

    // Fallback: try to find a JSON array anywhere in the text
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            let slice = &trimmed[start..=end];
            if let Ok(raw_items) = serde_json::from_str::<Vec<RawEvidenceItem>>(slice) {
                return Ok(convert_raw_evidence(raw_items, neighbors));
            }
        }
    }

    tracing::warn!(
        raw_response = %text,
        "Could not parse evidence extraction response as JSON"
    );

    // Return empty rather than error — the pipeline handles empty evidence gracefully.
    Ok(vec![])
}

fn extract_json_from_code_block(text: &str) -> Option<&str> {
    let start_marker = "```json";
    let end_marker = "```";

    let start = text.find(start_marker)?;
    let json_start = start + start_marker.len();
    let rest = &text[json_start..];
    let end = rest.find(end_marker)?;
    Some(rest[..end].trim())
}

fn convert_raw_evidence(
    raw: Vec<RawEvidenceItem>,
    neighbors: &[NeighborContent],
) -> Vec<EvidenceItem> {
    let title_map: HashMap<i64, &NeighborContent> =
        neighbors.iter().map(|n| (n.node_id, n)).collect();

    raw.into_iter()
        .filter_map(|r| {
            let evidence_type = match r.evidence_type.as_str() {
                "contradiction" => EvidenceType::Contradiction,
                "data_point" => EvidenceType::DataPoint,
                "aha_moment" => EvidenceType::AhaMoment,
                _ => return None,
            };

            let neighbor = title_map.get(&r.source_node_id);
            let source_note_title = neighbor.map(|n| n.note_title.clone()).unwrap_or_default();
            let source_heading_path = neighbor.and_then(|n| n.heading_path.clone());

            Some(EvidenceItem {
                evidence_type,
                citation_text: r.citation_text,
                source_node_id: r.source_node_id,
                source_note_title,
                source_heading_path,
                confidence: r.confidence,
            })
        })
        .collect()
}

// ============================================================================
// Stage 3: Validation
// ============================================================================

/// Validate and clean extracted evidence items.
///
/// - Rejects items with node IDs not in the accepted set.
/// - Truncates citation text to 120 chars (preserving partial value).
/// - Rejects items with confidence below 0.1 (noise floor).
/// - Deduplicates by (evidence_type, source_node_id), keeping highest confidence.
pub fn validate_evidence(
    evidence: Vec<EvidenceItem>,
    accepted_node_ids: &HashSet<i64>,
) -> Vec<EvidenceItem> {
    let mut items: Vec<EvidenceItem> = evidence
        .into_iter()
        // Reject invalid node IDs
        .filter(|e| accepted_node_ids.contains(&e.source_node_id))
        // Reject noise-floor confidence
        .filter(|e| e.confidence >= MIN_CONFIDENCE_FLOOR)
        // Truncate long citations
        .map(|mut e| {
            if e.citation_text.len() > MAX_CITATION_CHARS {
                let truncated = truncate_at_char_boundary(&e.citation_text, MAX_CITATION_CHARS - 3);
                e.citation_text = format!("{truncated}...");
            }
            e
        })
        .collect();

    // Deduplicate by (evidence_type, source_node_id), keeping highest confidence
    let mut seen: HashMap<(EvidenceType, i64), usize> = HashMap::new();
    let mut deduped: Vec<EvidenceItem> = Vec::new();

    for item in items.drain(..) {
        let key = (item.evidence_type, item.source_node_id);
        if let Some(&idx) = seen.get(&key) {
            if item.confidence > deduped[idx].confidence {
                deduped[idx] = item;
            }
        } else {
            seen.insert(key, deduped.len());
            deduped.push(item);
        }
    }

    deduped
}

/// Truncate a string at the last valid char boundary at or before `max_len`.
fn truncate_at_char_boundary(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neighbor(id: i64, title: &str, snippet: &str) -> NeighborContent {
        NeighborContent {
            node_id: id,
            note_title: title.to_string(),
            heading_path: None,
            snippet: snippet.to_string(),
        }
    }

    // -- Pre-filter tests --

    #[test]
    fn pre_filter_catches_percentages() {
        let neighbors = vec![neighbor(1, "Growth", "Revenue grew 45% last quarter")];
        let candidates = pre_filter_data_points(&neighbors);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].text.contains("45%"));
    }

    #[test]
    fn pre_filter_catches_dollars() {
        let neighbors = vec![neighbor(1, "Revenue", "Earned $1,200 in MRR")];
        let candidates = pre_filter_data_points(&neighbors);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].text.contains("$1,200"));
    }

    #[test]
    fn pre_filter_catches_multipliers() {
        let neighbors = vec![neighbor(1, "Perf", "Achieved 3.5x improvement")];
        let candidates = pre_filter_data_points(&neighbors);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].text.contains("3.5x"));
    }

    #[test]
    fn pre_filter_catches_iso_dates() {
        let neighbors = vec![neighbor(1, "Launch", "Launched on 2025-06-15")];
        let candidates = pre_filter_data_points(&neighbors);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].text.contains("2025-06-15"));
    }

    #[test]
    fn pre_filter_catches_counts() {
        let neighbors = vec![neighbor(1, "Users", "150 users migrated to v2")];
        let candidates = pre_filter_data_points(&neighbors);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].text.contains("150 users"));
    }

    #[test]
    fn pre_filter_empty_on_plain_text() {
        let neighbors = vec![neighbor(1, "Note", "No numbers here at all")];
        let candidates = pre_filter_data_points(&neighbors);
        assert!(candidates.is_empty());
    }

    // -- Validation tests --

    fn evidence_item(
        etype: EvidenceType,
        node_id: i64,
        confidence: f64,
        citation: &str,
    ) -> EvidenceItem {
        EvidenceItem {
            evidence_type: etype,
            citation_text: citation.to_string(),
            source_node_id: node_id,
            source_note_title: "Note".to_string(),
            source_heading_path: None,
            confidence,
        }
    }

    #[test]
    fn validate_rejects_invalid_node_ids() {
        let accepted: HashSet<i64> = [1, 2].into_iter().collect();
        let evidence = vec![
            evidence_item(EvidenceType::DataPoint, 1, 0.8, "valid"),
            evidence_item(EvidenceType::DataPoint, 99, 0.9, "invalid node"),
        ];
        let result = validate_evidence(evidence, &accepted);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source_node_id, 1);
    }

    #[test]
    fn validate_truncates_long_citations() {
        let accepted: HashSet<i64> = [1].into_iter().collect();
        let long_citation = "a".repeat(200);
        let evidence = vec![evidence_item(
            EvidenceType::DataPoint,
            1,
            0.8,
            &long_citation,
        )];
        let result = validate_evidence(evidence, &accepted);
        assert_eq!(result.len(), 1);
        assert!(result[0].citation_text.len() <= MAX_CITATION_CHARS);
        assert!(result[0].citation_text.ends_with("..."));
    }

    #[test]
    fn validate_rejects_low_confidence() {
        let accepted: HashSet<i64> = [1].into_iter().collect();
        let evidence = vec![evidence_item(EvidenceType::DataPoint, 1, 0.05, "low conf")];
        let result = validate_evidence(evidence, &accepted);
        assert!(result.is_empty());
    }

    #[test]
    fn validate_deduplicates() {
        let accepted: HashSet<i64> = [1].into_iter().collect();
        let evidence = vec![
            evidence_item(EvidenceType::DataPoint, 1, 0.6, "first"),
            evidence_item(EvidenceType::DataPoint, 1, 0.9, "second (higher)"),
        ];
        let result = validate_evidence(evidence, &accepted);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].citation_text, "second (higher)");
    }

    // -- Parse tests --

    #[test]
    fn parse_evidence_json() {
        let json = r#"[
            {
                "evidence_type": "data_point",
                "citation_text": "45% growth",
                "source_node_id": 1,
                "confidence": 0.8
            }
        ]"#;
        let neighbors = vec![neighbor(1, "Growth", "Revenue grew 45% last quarter")];
        let result = parse_evidence_response(json, &neighbors).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].evidence_type, EvidenceType::DataPoint);
        assert_eq!(result[0].source_note_title, "Growth");
    }

    #[test]
    fn parse_evidence_json_in_code_block() {
        let text = "Here is the evidence:\n```json\n[\n{\"evidence_type\": \"aha_moment\", \
                    \"citation_text\": \"surprise\", \"source_node_id\": 2, \"confidence\": 0.7}\n]\n```";
        let neighbors = vec![neighbor(2, "Insights", "A surprising finding")];
        let result = parse_evidence_response(text, &neighbors).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].evidence_type, EvidenceType::AhaMoment);
    }

    #[test]
    fn parse_evidence_empty_array() {
        let result = parse_evidence_response("[]", &[]).unwrap();
        assert!(result.is_empty());
    }
}
