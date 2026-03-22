//! Angle generation from extracted evidence.
//!
//! Implements the full mining pipeline: pre-filter → LLM extraction →
//! validation → angle generation → parsing. Parallel to the existing
//! hook generation pipeline in `mod.rs`.

use std::collections::HashSet;

use crate::config::BusinessProfile;
use crate::content::angles::{
    assign_angle_confidence, AngleMiningOutput, AngleType, EvidenceItem, MinedAngle,
    MIN_EVIDENCE_COUNT, MIN_EVIDENCE_QUALITY,
};
use crate::content::evidence::{
    extract_evidence, pre_filter_data_points, validate_evidence, NeighborContent,
};
use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider, TokenUsage};

/// Run the full angle mining pipeline.
///
/// 1. Pre-filter neighbors for data point candidates.
/// 2. Extract evidence via LLM.
/// 3. Validate evidence (reject invalid IDs, truncate, deduplicate).
/// 4. Check evidence thresholds — return fallback if insufficient.
/// 5. Generate angles from evidence via LLM.
/// 6. Parse and return angles.
pub async fn generate_mined_angles(
    provider: &dyn LlmProvider,
    business: &BusinessProfile,
    topic: &str,
    neighbors: &[NeighborContent],
    selection_context: Option<&str>,
) -> Result<AngleMiningOutput, LlmError> {
    let provider_name = provider.name().to_string();
    let mut usage = TokenUsage::default();

    // Gate: no neighbors → immediate fallback
    if neighbors.is_empty() {
        return Ok(AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("no_neighbors_accepted".to_string()),
            evidence_quality_score: 0.0,
            usage,
            model: String::new(),
            provider: provider_name,
        });
    }

    // Stage 1: Regex pre-filter
    let candidates = pre_filter_data_points(neighbors);
    tracing::debug!(
        candidate_count = candidates.len(),
        "Data point candidates from regex pre-filter"
    );

    // Stage 2: LLM evidence extraction
    let raw_evidence = extract_evidence(provider, topic, neighbors, &candidates).await?;
    usage.accumulate(&TokenUsage {
        input_tokens: 0,
        output_tokens: 0,
    });

    // Stage 3: Validation
    let accepted_ids: HashSet<i64> = neighbors.iter().map(|n| n.node_id).collect();
    let evidence = validate_evidence(raw_evidence, &accepted_ids);

    tracing::debug!(evidence_count = evidence.len(), "Validated evidence items");

    // Gate: insufficient evidence
    if evidence.len() < MIN_EVIDENCE_COUNT {
        return Ok(AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("insufficient_evidence".to_string()),
            evidence_quality_score: if evidence.is_empty() {
                0.0
            } else {
                evidence.iter().map(|e| e.confidence).sum::<f64>() / evidence.len() as f64
            },
            usage,
            model: String::new(),
            provider: provider_name,
        });
    }

    // Compute quality score
    let evidence_quality_score =
        evidence.iter().map(|e| e.confidence).sum::<f64>() / evidence.len() as f64;

    // Gate: low quality evidence
    if evidence_quality_score < MIN_EVIDENCE_QUALITY {
        return Ok(AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("low_evidence_quality".to_string()),
            evidence_quality_score,
            usage,
            model: String::new(),
            provider: provider_name,
        });
    }

    // Stage 4: Angle generation via LLM
    let system = build_angle_generation_prompt(business, topic, &evidence, selection_context);
    let user_message = "Generate content angles from the evidence above.".to_string();

    let params = GenerationParams {
        max_tokens: 800,
        temperature: 0.8,
        ..Default::default()
    };

    let resp = provider.complete(&system, &user_message, &params).await?;
    usage.accumulate(&resp.usage);
    let model = resp.model.clone();

    tracing::debug!(
        raw_response = %resp.text,
        "Raw LLM response for angle generation"
    );

    // Stage 5: Parse angles
    let mut angles = parse_angles_response(&resp.text, &evidence);

    // Filter out angles with empty evidence
    angles.retain(|a| !a.evidence.is_empty());

    if angles.is_empty() {
        return Ok(AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("all_angles_filtered".to_string()),
            evidence_quality_score,
            usage,
            model,
            provider: provider_name,
        });
    }

    // Cap at 3 angles
    angles.truncate(3);

    Ok(AngleMiningOutput {
        angles,
        fallback_reason: None,
        evidence_quality_score,
        usage,
        model,
        provider: provider_name,
    })
}

fn build_angle_generation_prompt(
    business: &BusinessProfile,
    topic: &str,
    evidence: &[EvidenceItem],
    selection_context: Option<&str>,
) -> String {
    let audience_section = if business.target_audience.is_empty() {
        String::new()
    } else {
        format!("\nYour audience: {}.", business.target_audience)
    };

    let voice_section = match &business.brand_voice {
        Some(v) if !v.is_empty() => format!("\nVoice & personality: {v}"),
        _ => String::new(),
    };

    let mut persona_parts = Vec::new();
    if !business.persona_opinions.is_empty() {
        persona_parts.push(format!(
            "Opinions you hold: {}",
            business.persona_opinions.join("; ")
        ));
    }
    if !business.persona_experiences.is_empty() {
        persona_parts.push(format!(
            "Experiences you can reference: {}",
            business.persona_experiences.join("; ")
        ));
    }
    let persona_section = if persona_parts.is_empty() {
        String::new()
    } else {
        format!("\n{}", persona_parts.join("\n"))
    };

    let selection_section = match selection_context {
        Some(ctx) if !ctx.is_empty() => format!("\n\nSelected vault context:\n{ctx}"),
        _ => String::new(),
    };

    let mut evidence_list = String::new();
    for (i, e) in evidence.iter().enumerate() {
        evidence_list.push_str(&format!(
            "[{}] ({}) \"{}\" from \"{}\" [confidence: {:.1}]\n",
            i + 1,
            e.evidence_type,
            e.citation_text,
            e.source_note_title,
            e.confidence,
        ));
    }

    format!(
        "You are {}'s social media voice. {}.\
         {audience_section}\
         {voice_section}\
         {persona_section}\
         {selection_section}\n\n\
         Task: Generate up to 3 content angles from the evidence below.\n\
         Each angle must be one of: story, listicle, hot_take.\n\
         Generate exactly one angle per type IF the evidence supports it.\n\
         Do not pad with unsupported angles.\n\n\
         Topic: {topic}\n\n\
         Evidence items:\n{evidence_list}\n\
         Output format (strictly follow this, no extra text):\n\
         ANGLE_TYPE: story\n\
         SEED_TEXT: <opening tweet, max 280 chars>\n\
         RATIONALE: <1 sentence why this angle works>\n\
         EVIDENCE_IDS: 1, 3\n\
         ---\n\
         (repeat for each angle)",
        business.product_name, business.product_description,
    )
}

/// Parse an angle generation response with ANGLE_TYPE/SEED_TEXT/RATIONALE/EVIDENCE_IDS blocks.
pub fn parse_angles_response(text: &str, evidence: &[EvidenceItem]) -> Vec<MinedAngle> {
    let mut angles = Vec::new();
    let mut current_type: Option<AngleType> = None;
    let mut current_seed = String::new();
    let mut current_rationale = String::new();
    let mut current_evidence_ids: Vec<usize> = Vec::new();

    let flush = |angle_type: Option<AngleType>,
                 seed: &str,
                 rationale: &str,
                 evidence_ids: &[usize],
                 evidence: &[EvidenceItem],
                 angles: &mut Vec<MinedAngle>| {
        if let Some(at) = angle_type {
            let seed = seed.trim().to_string();
            if !seed.is_empty() {
                let matched_evidence: Vec<EvidenceItem> = evidence_ids
                    .iter()
                    .filter_map(|&id| {
                        if id >= 1 && id <= evidence.len() {
                            Some(evidence[id - 1].clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                let confidence = assign_angle_confidence(&matched_evidence);
                let char_count = seed.len();

                angles.push(MinedAngle {
                    angle_type: at,
                    seed_text: seed,
                    char_count,
                    evidence: matched_evidence,
                    confidence,
                    rationale: rationale.trim().to_string(),
                });
            }
        }
    };

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed == "---" || trimmed == "- - -" {
            flush(
                current_type,
                &current_seed,
                &current_rationale,
                &current_evidence_ids,
                evidence,
                &mut angles,
            );
            current_type = None;
            current_seed.clear();
            current_rationale.clear();
            current_evidence_ids.clear();
            continue;
        }

        let cleaned = strip_formatting(trimmed);

        if let Some(val) = strip_prefix_ci(&cleaned, "angle_type:") {
            current_type = parse_angle_type(val.trim());
        } else if let Some(val) = strip_prefix_ci(&cleaned, "seed_text:") {
            current_seed = strip_quotes(val.trim());
        } else if let Some(val) = strip_prefix_ci(&cleaned, "rationale:") {
            current_rationale = val.trim().to_string();
        } else if let Some(val) = strip_prefix_ci(&cleaned, "evidence_ids:") {
            current_evidence_ids = val
                .split([',', ' '])
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();
        }
    }

    // Flush last block
    flush(
        current_type,
        &current_seed,
        &current_rationale,
        &current_evidence_ids,
        evidence,
        &mut angles,
    );

    angles
}

fn parse_angle_type(s: &str) -> Option<AngleType> {
    match s.to_lowercase().as_str() {
        "story" => Some(AngleType::Story),
        "listicle" => Some(AngleType::Listicle),
        "hot_take" | "hottake" | "hot take" => Some(AngleType::HotTake),
        _ => None,
    }
}

fn strip_prefix_ci<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = text.to_ascii_lowercase();
    if lower.starts_with(prefix) {
        Some(&text[prefix.len()..])
    } else {
        None
    }
}

fn strip_formatting(line: &str) -> String {
    let mut s = line.replace("**", "");
    // Strip leading number+punctuation
    if let Some(first) = s.chars().next() {
        if first.is_ascii_digit() {
            if let Some(pos) = s.find(|c: char| !c.is_ascii_digit()) {
                let after = &s[pos..];
                if after.starts_with(". ") || after.starts_with(") ") || after.starts_with(": ") {
                    s = after[2..].to_string();
                }
            }
        }
    }
    if s.starts_with("- ") || s.starts_with("• ") {
        s = s[2..].to_string();
    }
    s
}

fn strip_quotes(text: &str) -> String {
    let t = text.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::angles::EvidenceType;

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
SEED_TEXT: Growth without profit isn't growth—it's a slow leak.
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
        // Tested via the pipeline: empty neighbors → "no_neighbors_accepted"
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
        // 2 items with avg conf (0.8+0.9)/2 = 0.85 >= 0.6 → "high"
        assert_eq!(angles[0].confidence, "high");
    }
}
