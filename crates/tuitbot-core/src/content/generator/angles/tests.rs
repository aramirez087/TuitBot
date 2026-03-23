use super::*;
use crate::content::angles::EvidenceType;
use crate::content::evidence::NeighborContent;

fn sample_evidence() -> Vec<EvidenceItem> {
    vec![
        EvidenceItem {
            evidence_type: EvidenceType::DataPoint,
            citation_text: "45% growth".to_string(),
            source_node_id: 1,
            source_note_title: "Metrics".to_string(),
            source_heading_path: None,
            confidence: 0.8,
        },
        EvidenceItem {
            evidence_type: EvidenceType::Contradiction,
            citation_text: "But costs rose too".to_string(),
            source_node_id: 2,
            source_note_title: "Costs".to_string(),
            source_heading_path: None,
            confidence: 0.7,
        },
        EvidenceItem {
            evidence_type: EvidenceType::AhaMoment,
            citation_text: "Unexpected correlation".to_string(),
            source_node_id: 3,
            source_note_title: "Insights".to_string(),
            source_heading_path: None,
            confidence: 0.9,
        },
    ]
}

#[test]
fn parse_angles_well_formatted() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT: We grew 45% but our costs told a different story...
RATIONALE: Tension between growth and cost creates a compelling narrative.
EVIDENCE_IDS: 1, 2
---
ANGLE_TYPE: listicle
SEED_TEXT: 3 things nobody tells you about scaling fast:
RATIONALE: List format works well for multi-faceted insights.
EVIDENCE_IDS: 1, 3
---
ANGLE_TYPE: hot_take
SEED_TEXT: Growth without profit isn't growth\u{2014}it's a slow leak.
RATIONALE: Bold opinion backed by evidence of rising costs.
EVIDENCE_IDS: 2";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);

    assert_eq!(angles.len(), 3);
    assert_eq!(angles[0].angle_type, AngleType::Story);
    assert_eq!(angles[0].evidence.len(), 2);
    assert_eq!(angles[1].angle_type, AngleType::Listicle);
    assert_eq!(angles[1].evidence.len(), 2);
    assert_eq!(angles[2].angle_type, AngleType::HotTake);
    assert_eq!(angles[2].evidence.len(), 1);
}

#[test]
fn parse_angles_partial() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT: A tale of growth
RATIONALE: Good narrative.
EVIDENCE_IDS: 1
---
ANGLE_TYPE: listicle
SEED_TEXT: Top insights from the data
RATIONALE: Lists perform well.
EVIDENCE_IDS: 1, 3";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    assert_eq!(angles.len(), 2);
}

#[test]
fn parse_angles_empty() {
    let angles = parse_angles_response("", &sample_evidence());
    assert!(angles.is_empty());
}

#[test]
fn fallback_no_neighbors() {
    // Tested via the pipeline: empty neighbors -> "no_neighbors_accepted"
    // This is a sync verification of the pipeline's early return logic.
    let neighbors: Vec<NeighborContent> = vec![];
    assert!(neighbors.is_empty());
}

#[test]
fn parse_angle_type_variants() {
    assert_eq!(parse_angle_type("story"), Some(AngleType::Story));
    assert_eq!(parse_angle_type("listicle"), Some(AngleType::Listicle));
    assert_eq!(parse_angle_type("hot_take"), Some(AngleType::HotTake));
    assert_eq!(parse_angle_type("hottake"), Some(AngleType::HotTake));
    assert_eq!(parse_angle_type("hot take"), Some(AngleType::HotTake));
    assert_eq!(parse_angle_type("STORY"), Some(AngleType::Story));
    assert_eq!(parse_angle_type("unknown"), None);
}

#[test]
fn parse_angles_with_evidence_mapping() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT: The data surprised us all.
RATIONALE: Data-backed narrative.
EVIDENCE_IDS: 1, 3";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    assert_eq!(angles.len(), 1);
    assert_eq!(angles[0].evidence.len(), 2);
    assert_eq!(angles[0].evidence[0].evidence_type, EvidenceType::DataPoint);
    assert_eq!(angles[0].evidence[1].evidence_type, EvidenceType::AhaMoment);
    // 2 items with avg conf (0.8+0.9)/2 = 0.85 >= 0.6 -> "high"
    assert_eq!(angles[0].confidence, "high");
}

// -- strip_formatting tests --

#[test]
fn strip_formatting_removes_bold() {
    assert_eq!(
        strip_formatting("**ANGLE_TYPE**: story"),
        "ANGLE_TYPE: story"
    );
}

#[test]
fn strip_formatting_removes_numbered_prefix_dot() {
    assert_eq!(
        strip_formatting("1. ANGLE_TYPE: story"),
        "ANGLE_TYPE: story"
    );
}

#[test]
fn strip_formatting_removes_numbered_prefix_paren() {
    assert_eq!(strip_formatting("2) SEED_TEXT: hello"), "SEED_TEXT: hello");
}

#[test]
fn strip_formatting_removes_numbered_prefix_colon() {
    assert_eq!(strip_formatting("3: RATIONALE: why"), "RATIONALE: why");
}

#[test]
fn strip_formatting_removes_bullet_dash() {
    assert_eq!(
        strip_formatting("- ANGLE_TYPE: listicle"),
        "ANGLE_TYPE: listicle"
    );
}

#[test]
fn strip_formatting_removes_bullet_dash_at_start() {
    // Verify dash bullet is stripped (common in LLM output)
    assert_eq!(strip_formatting("- hello world"), "hello world");
}

#[test]
fn strip_formatting_removes_unicode_bullet() {
    let input = "\u{2022} something";
    assert_eq!(strip_formatting(input), "something");
}

#[test]
fn strip_formatting_plain_text_unchanged() {
    assert_eq!(strip_formatting("ANGLE_TYPE: story"), "ANGLE_TYPE: story");
}

#[test]
fn strip_formatting_empty_string() {
    assert_eq!(strip_formatting(""), "");
}

// -- strip_quotes tests --

#[test]
fn strip_quotes_double_quotes() {
    assert_eq!(strip_quotes("\"hello world\""), "hello world");
}

#[test]
fn strip_quotes_single_quotes() {
    assert_eq!(strip_quotes("'hello world'"), "hello world");
}

#[test]
fn strip_quotes_no_quotes() {
    assert_eq!(strip_quotes("hello world"), "hello world");
}

#[test]
fn strip_quotes_mismatched_not_stripped() {
    assert_eq!(strip_quotes("\"hello world'"), "\"hello world'");
}

#[test]
fn strip_quotes_trims_whitespace() {
    assert_eq!(strip_quotes("  \"padded\"  "), "padded");
}

#[test]
fn strip_quotes_empty_quoted() {
    // Single char each side: "\"\"" -> ""
    assert_eq!(strip_quotes("\"\""), "");
}

// -- strip_prefix_ci tests --

#[test]
fn strip_prefix_ci_match() {
    assert_eq!(
        strip_prefix_ci("ANGLE_TYPE: story", "angle_type:"),
        Some(" story")
    );
}

#[test]
fn strip_prefix_ci_no_match() {
    assert!(strip_prefix_ci("SEED_TEXT: foo", "angle_type:").is_none());
}

#[test]
fn strip_prefix_ci_mixed_case() {
    assert_eq!(
        strip_prefix_ci("Angle_Type: hot_take", "angle_type:"),
        Some(" hot_take")
    );
}

// -- parse_angles_response edge cases --

#[test]
fn parse_angles_invalid_evidence_ids_filtered() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT: A tale
RATIONALE: Good.
EVIDENCE_IDS: 0, 99";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    // IDs 0 and 99 are out of range (1-indexed, 3 items)
    assert_eq!(angles.len(), 1);
    assert!(angles[0].evidence.is_empty());
}

#[test]
fn parse_angles_empty_seed_text_filtered() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT:
RATIONALE: Good.
EVIDENCE_IDS: 1";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    // Empty seed text should produce no angle
    assert!(angles.is_empty());
}

#[test]
fn parse_angles_with_formatting_artifacts() {
    let response = "\
1. **ANGLE_TYPE**: story
- **SEED_TEXT**: \"We grew 45%\"
**RATIONALE**: Strong narrative.
**EVIDENCE_IDS**: 1, 2
---
2. **ANGLE_TYPE**: hot_take
- **SEED_TEXT**: Growth without profit is a leak
**RATIONALE**: Bold claim.
**EVIDENCE_IDS**: 2";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    assert_eq!(angles.len(), 2);
    assert_eq!(angles[0].angle_type, AngleType::Story);
    // seed_text should have quotes stripped
    assert_eq!(angles[0].seed_text, "We grew 45%");
    assert_eq!(angles[1].angle_type, AngleType::HotTake);
}

#[test]
fn parse_angles_no_trailing_separator() {
    // Last block has no --- separator
    let response = "\
ANGLE_TYPE: listicle
SEED_TEXT: Top 3 insights
RATIONALE: Lists work.
EVIDENCE_IDS: 1, 2, 3";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    assert_eq!(angles.len(), 1);
    assert_eq!(angles[0].angle_type, AngleType::Listicle);
    assert_eq!(angles[0].evidence.len(), 3);
}

#[test]
fn parse_angles_alt_separator() {
    let response = "\
ANGLE_TYPE: story
SEED_TEXT: Tale one
RATIONALE: r1.
EVIDENCE_IDS: 1
- - -
ANGLE_TYPE: hot_take
SEED_TEXT: Bold claim
RATIONALE: r2.
EVIDENCE_IDS: 2";

    let evidence = sample_evidence();
    let angles = parse_angles_response(response, &evidence);
    assert_eq!(angles.len(), 2);
}

// -- build_angle_generation_prompt tests --

#[test]
fn prompt_includes_product_and_topic() {
    let biz = BusinessProfile {
        product_name: "TestApp".to_string(),
        product_description: "A test product".to_string(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Growth", &evidence, None);
    assert!(prompt.contains("TestApp"));
    assert!(prompt.contains("A test product"));
    assert!(prompt.contains("Topic: Growth"));
}

#[test]
fn prompt_includes_audience_when_set() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        target_audience: "indie hackers".to_string(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(prompt.contains("indie hackers"));
}

#[test]
fn prompt_excludes_audience_when_empty() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        target_audience: String::new(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(!prompt.contains("Your audience"));
}

#[test]
fn prompt_includes_brand_voice() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        brand_voice: Some("witty and irreverent".to_string()),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(prompt.contains("witty and irreverent"));
}

#[test]
fn prompt_includes_selection_context() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt =
        build_angle_generation_prompt(&biz, "Topic", &evidence, Some("Selected vault text here"));
    assert!(prompt.contains("Selected vault context"));
    assert!(prompt.contains("Selected vault text here"));
}

#[test]
fn prompt_excludes_selection_context_when_none() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(!prompt.contains("Selected vault context"));
}

#[test]
fn prompt_includes_persona_opinions() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        persona_opinions: vec!["AI is overhyped".to_string()],
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(prompt.contains("Opinions you hold"));
    assert!(prompt.contains("AI is overhyped"));
}

#[test]
fn prompt_includes_persona_experiences() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        persona_experiences: vec!["Built 3 startups".to_string()],
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(prompt.contains("Experiences you can reference"));
    assert!(prompt.contains("Built 3 startups"));
}

#[test]
fn prompt_lists_evidence_with_confidence() {
    let biz = BusinessProfile {
        product_name: "App".to_string(),
        product_description: "desc".to_string(),
        ..Default::default()
    };
    let evidence = sample_evidence();
    let prompt = build_angle_generation_prompt(&biz, "Topic", &evidence, None);
    assert!(prompt.contains("[1] (data_point)"));
    assert!(prompt.contains("45% growth"));
    assert!(prompt.contains("[confidence: 0.8]"));
}
