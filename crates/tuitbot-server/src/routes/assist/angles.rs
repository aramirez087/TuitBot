//! POST /api/assist/angles — mine evidence-backed content angles from vault notes.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use tuitbot_core::content::evidence::NeighborContent;
use tuitbot_core::context::retrieval::VaultCitation;
use tuitbot_core::storage::watchtower;

use crate::account::AccountContext;
use crate::error::ApiError;
use crate::state::AppState;

use super::get_generator;

// ---------------------------------------------------------------------------
// Request / Response DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AssistAnglesRequest {
    pub topic: String,
    #[serde(default)]
    pub accepted_neighbor_ids: Vec<i64>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}

#[derive(Serialize)]
pub struct AssistAnglesResponse {
    pub angles: Vec<MinedAngleDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub topic: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

#[derive(Serialize)]
pub struct MinedAngleDto {
    pub angle_type: String,
    pub seed_text: String,
    pub char_count: usize,
    pub evidence: Vec<EvidenceItemDto>,
    pub confidence: String,
    pub rationale: String,
}

#[derive(Serialize)]
pub struct EvidenceItemDto {
    pub evidence_type: String,
    pub citation_text: String,
    pub source_node_id: i64,
    pub source_note_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_heading_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn assist_angles(
    State(state): State<Arc<AppState>>,
    ctx: AccountContext,
    Json(body): Json<AssistAnglesRequest>,
) -> Result<Json<AssistAnglesResponse>, ApiError> {
    if body.topic.trim().is_empty() {
        return Err(ApiError::BadRequest("topic must not be empty".to_string()));
    }

    if body.accepted_neighbor_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "accepted_neighbor_ids must not be empty".to_string(),
        ));
    }

    let gen = get_generator(&state, &ctx.account_id).await?;

    // Fetch neighbor content from the vault (account-scoped)
    let chunks = watchtower::get_chunks_for_nodes_with_context(
        &state.db,
        &ctx.account_id,
        &body.accepted_neighbor_ids,
        10, // up to 10 chunks across all accepted neighbors
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Build NeighborContent from chunks, deduplicating by node_id (keep first/best chunk)
    let mut seen_nodes = std::collections::HashSet::new();
    let mut neighbors: Vec<NeighborContent> = Vec::new();

    for chunk in &chunks {
        if seen_nodes.insert(chunk.chunk.node_id) {
            let snippet = truncate_snippet(&chunk.chunk.chunk_text, 500);
            neighbors.push(NeighborContent {
                node_id: chunk.chunk.node_id,
                note_title: chunk
                    .source_title
                    .clone()
                    .unwrap_or_else(|| chunk.relative_path.clone()),
                heading_path: if chunk.chunk.heading_path.is_empty() {
                    None
                } else {
                    Some(chunk.chunk.heading_path.clone())
                },
                snippet,
            });
        }
    }

    // Resolve selection context if session_id provided
    let selection_context = if let Some(ref sid) = body.session_id {
        use crate::routes::rag_helpers::resolve_selection_rag_context;
        let sel = resolve_selection_rag_context(&state, &ctx.account_id, sid).await;
        sel.and_then(|s| s.selected_text)
    } else {
        None
    };

    let output = gen
        .generate_mined_angles(&body.topic, &neighbors, selection_context.as_deref())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Build vault citations from evidence sources
    let mut citations: Vec<VaultCitation> = Vec::new();
    let mut cited_nodes = std::collections::HashSet::new();
    for angle in &output.angles {
        for ev in &angle.evidence {
            if cited_nodes.insert(ev.source_node_id) {
                // Find matching chunk for citation metadata
                if let Some(chunk) = chunks.iter().find(|c| c.chunk.node_id == ev.source_node_id) {
                    citations.push(VaultCitation {
                        chunk_id: chunk.chunk.id,
                        node_id: chunk.chunk.node_id,
                        heading_path: chunk.chunk.heading_path.clone(),
                        source_path: chunk.relative_path.clone(),
                        source_title: chunk.source_title.clone(),
                        snippet: ev.citation_text.clone(),
                        retrieval_boost: chunk.chunk.retrieval_boost,
                        edge_type: None,
                        edge_label: None,
                    });
                }
            }
        }
    }

    // Map core types to DTOs
    let angle_dtos: Vec<MinedAngleDto> = output
        .angles
        .into_iter()
        .map(|a| MinedAngleDto {
            angle_type: a.angle_type.to_string(),
            seed_text: a.seed_text,
            char_count: a.char_count,
            evidence: a
                .evidence
                .into_iter()
                .map(|e| EvidenceItemDto {
                    evidence_type: e.evidence_type.to_string(),
                    citation_text: e.citation_text,
                    source_node_id: e.source_node_id,
                    source_note_title: e.source_note_title,
                    source_heading_path: e.source_heading_path,
                })
                .collect(),
            confidence: a.confidence,
            rationale: a.rationale,
        })
        .collect();

    Ok(Json(AssistAnglesResponse {
        angles: angle_dtos,
        fallback_reason: output.fallback_reason,
        topic: body.topic,
        vault_citations: citations,
    }))
}

fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_request_deserializes_minimal() {
        let json = r#"{"topic": "Rust testing", "accepted_neighbor_ids": [1]}"#;
        let req: AssistAnglesRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.topic, "Rust testing");
        assert_eq!(req.accepted_neighbor_ids, vec![1]);
        assert!(req.session_id.is_none());
        assert!(req.selected_node_ids.is_none());
    }

    #[test]
    fn angles_request_deserializes_full() {
        let json = r#"{
            "topic": "Growth metrics",
            "accepted_neighbor_ids": [1, 2, 3],
            "session_id": "sess-abc",
            "selected_node_ids": [10, 20]
        }"#;
        let req: AssistAnglesRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.topic, "Growth metrics");
        assert_eq!(req.accepted_neighbor_ids, vec![1, 2, 3]);
        assert_eq!(req.session_id.unwrap(), "sess-abc");
        assert_eq!(req.selected_node_ids.unwrap(), vec![10, 20]);
    }

    #[test]
    fn angles_response_omits_empty_citations() {
        let resp = AssistAnglesResponse {
            angles: vec![],
            fallback_reason: None,
            topic: "test".to_string(),
            vault_citations: vec![],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("vault_citations"));
        assert!(!json.contains("fallback_reason"));
    }

    #[test]
    fn angles_response_includes_fallback() {
        let resp = AssistAnglesResponse {
            angles: vec![],
            fallback_reason: Some("insufficient_evidence".to_string()),
            topic: "test".to_string(),
            vault_citations: vec![],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"fallback_reason\":\"insufficient_evidence\""));
    }

    #[test]
    fn mined_angle_dto_serialization() {
        let dto = MinedAngleDto {
            angle_type: "story".to_string(),
            seed_text: "A tale of growth".to_string(),
            char_count: 16,
            evidence: vec![EvidenceItemDto {
                evidence_type: "data_point".to_string(),
                citation_text: "45% growth".to_string(),
                source_node_id: 1,
                source_note_title: "Metrics".to_string(),
                source_heading_path: Some("# Results".to_string()),
            }],
            confidence: "high".to_string(),
            rationale: "Strong narrative arc.".to_string(),
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        assert!(json.contains("\"angle_type\":\"story\""));
        assert!(json.contains("\"evidence_type\":\"data_point\""));
        assert!(json.contains("\"source_heading_path\":\"# Results\""));
    }

    #[test]
    fn evidence_item_dto_omits_null_heading() {
        let dto = EvidenceItemDto {
            evidence_type: "aha_moment".to_string(),
            citation_text: "Unexpected".to_string(),
            source_node_id: 1,
            source_note_title: "Note".to_string(),
            source_heading_path: None,
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        assert!(!json.contains("source_heading_path"));
    }

    #[test]
    fn truncate_snippet_within_limit() {
        assert_eq!(truncate_snippet("short", 500), "short");
    }

    #[test]
    fn truncate_snippet_over_limit() {
        let long = "a".repeat(600);
        let result = truncate_snippet(&long, 500);
        assert!(result.len() <= 503); // 500 + "..."
        assert!(result.ends_with("..."));
    }
}
