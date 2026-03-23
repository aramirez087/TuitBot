use std::collections::HashSet;

use super::*;
use crate::content::angles::EvidenceType;

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

// -- truncate_at_char_boundary tests --

#[test]
fn truncate_short_string_is_noop() {
    assert_eq!(truncate_at_char_boundary("hello", 10), "hello");
}

#[test]
fn truncate_at_exact_length() {
    assert_eq!(truncate_at_char_boundary("hello", 5), "hello");
}

#[test]
fn truncate_ascii_string() {
    assert_eq!(truncate_at_char_boundary("hello world", 5), "hello");
}

#[test]
fn truncate_multibyte_respects_boundary() {
    // Each emoji is 4 bytes; truncating mid-emoji should back up
    let s = "ab\u{1F600}cd"; // "ab😀cd" — 8 bytes total
    let result = truncate_at_char_boundary(s, 3);
    // byte 3 is mid-emoji, should back up to byte 2
    assert_eq!(result, "ab");
}

#[test]
fn truncate_two_byte_chars() {
    let s = "\u{00E9}\u{00E9}\u{00E9}"; // "ééé" — 6 bytes
    let result = truncate_at_char_boundary(s, 3);
    // byte 3 is mid-char, should back up to byte 2 (1 char)
    assert_eq!(result, "\u{00E9}");
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate_at_char_boundary("", 5), "");
}

#[test]
fn truncate_zero_max_len() {
    assert_eq!(truncate_at_char_boundary("hello", 0), "");
}

// -- extract_json_from_code_block tests --

#[test]
fn extract_json_code_block_basic() {
    let text = "Here:\n```json\n[1, 2, 3]\n```\nDone.";
    assert_eq!(extract_json_from_code_block(text), Some("[1, 2, 3]"));
}

#[test]
fn extract_json_code_block_no_marker() {
    assert!(extract_json_from_code_block("no code block here").is_none());
}

#[test]
fn extract_json_code_block_no_end_marker() {
    let text = "```json\n[1, 2, 3]\nno closing";
    // The end marker search looks for ``` after the json content
    // "no closing" doesn't contain ```, so returns None
    assert!(extract_json_from_code_block(text).is_none());
}

// -- parse_evidence_response fallback paths --

#[test]
fn parse_evidence_fallback_embedded_array() {
    let text = "Some preamble text [
        {\"evidence_type\": \"data_point\", \"citation_text\": \"test\", \"source_node_id\": 1, \"confidence\": 0.5}
    ] and trailing text";
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let result = parse_evidence_response(text, &neighbors).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].evidence_type, EvidenceType::DataPoint);
}

#[test]
fn parse_evidence_unparseable_returns_empty() {
    let result = parse_evidence_response("totally not json", &[]).unwrap();
    assert!(result.is_empty());
}

// -- convert_raw_evidence edge cases --

#[test]
fn parse_evidence_unknown_type_filtered_out() {
    let json = r#"[
        {"evidence_type": "unknown_type", "citation_text": "test", "source_node_id": 1, "confidence": 0.5}
    ]"#;
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let result = parse_evidence_response(json, &neighbors).unwrap();
    assert!(result.is_empty());
}

#[test]
fn parse_evidence_unknown_node_id_still_parsed() {
    let json = r#"[
        {"evidence_type": "contradiction", "citation_text": "x", "source_node_id": 999, "confidence": 0.7}
    ]"#;
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let result = parse_evidence_response(json, &neighbors).unwrap();
    assert_eq!(result.len(), 1);
    // source_note_title defaults to empty when node_id not in neighbors
    assert_eq!(result[0].source_note_title, "");
}

#[test]
fn parse_evidence_default_confidence() {
    let json = r#"[{"evidence_type": "aha_moment", "citation_text": "test", "source_node_id": 1}]"#;
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let result = parse_evidence_response(json, &neighbors).unwrap();
    assert_eq!(result.len(), 1);
    assert!((result[0].confidence - 0.5).abs() < f64::EPSILON);
}

// -- build_extraction_prompt tests --

#[test]
fn build_extraction_prompt_includes_topic() {
    let neighbors = vec![neighbor(1, "Note", "snippet text")];
    let prompt = build_extraction_prompt("AI safety", &neighbors, &[]);
    assert!(prompt.contains("Topic: AI safety"));
    assert!(prompt.contains("snippet text"));
    assert!(prompt.contains("node_id: 1"));
}

#[test]
fn build_extraction_prompt_includes_candidates() {
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let candidates = vec![CandidateDataPoint {
        text: "grew 45%".to_string(),
        source_node_id: 1,
        source_note_title: "Note".to_string(),
    }];
    let prompt = build_extraction_prompt("Growth", &neighbors, &candidates);
    assert!(prompt.contains("Candidate data points"));
    assert!(prompt.contains("grew 45%"));
}

#[test]
fn build_extraction_prompt_no_candidates_section() {
    let neighbors = vec![neighbor(1, "Note", "snippet")];
    let prompt = build_extraction_prompt("Topic", &neighbors, &[]);
    assert!(!prompt.contains("Candidate data points"));
}

// -- validate_evidence with dedup keeping higher confidence --

#[test]
fn validate_dedup_keeps_first_when_equal_confidence() {
    let accepted: HashSet<i64> = [1].into_iter().collect();
    let evidence = vec![
        evidence_item(EvidenceType::DataPoint, 1, 0.7, "first"),
        evidence_item(EvidenceType::DataPoint, 1, 0.7, "second"),
    ];
    let result = validate_evidence(evidence, &accepted);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].citation_text, "first");
}

#[test]
fn validate_different_types_not_deduped() {
    let accepted: HashSet<i64> = [1].into_iter().collect();
    let evidence = vec![
        evidence_item(EvidenceType::DataPoint, 1, 0.8, "data"),
        evidence_item(EvidenceType::Contradiction, 1, 0.8, "contra"),
    ];
    let result = validate_evidence(evidence, &accepted);
    assert_eq!(result.len(), 2);
}

#[test]
fn validate_confidence_exactly_at_floor() {
    let accepted: HashSet<i64> = [1].into_iter().collect();
    let evidence = vec![evidence_item(EvidenceType::DataPoint, 1, 0.1, "at floor")];
    let result = validate_evidence(evidence, &accepted);
    assert_eq!(result.len(), 1);
}

// -- Neighbor with heading_path --

#[test]
fn parse_evidence_preserves_heading_path() {
    let json = r#"[{"evidence_type": "data_point", "citation_text": "x", "source_node_id": 1, "confidence": 0.8}]"#;
    let neighbors = vec![NeighborContent {
        node_id: 1,
        note_title: "Note".to_string(),
        heading_path: Some("# Heading > ## Sub".to_string()),
        snippet: "snippet".to_string(),
    }];
    let result = parse_evidence_response(json, &neighbors).unwrap();
    assert_eq!(
        result[0].source_heading_path.as_deref(),
        Some("# Heading > ## Sub")
    );
}

// -- Pre-filter with multiple matches --

#[test]
fn pre_filter_multiple_matches_in_one_snippet() {
    let neighbors = vec![neighbor(
        1,
        "Stats",
        "Revenue grew 45% and we gained 150 users in Q2",
    )];
    let candidates = pre_filter_data_points(&neighbors);
    assert_eq!(candidates.len(), 2);
}

#[test]
fn pre_filter_multiple_neighbors() {
    let neighbors = vec![
        neighbor(1, "A", "Growth was 10x"),
        neighbor(2, "B", "Nothing to see"),
        neighbor(3, "C", "Date was 2025-01-01"),
    ];
    let candidates = pre_filter_data_points(&neighbors);
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].source_node_id, 1);
    assert_eq!(candidates[1].source_node_id, 3);
}
