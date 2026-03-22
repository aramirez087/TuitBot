//! Graph-aware neighbor expansion, ranking, and classification.
//!
//! Expands 1-hop neighbors around a selected note using direct links,
//! backlinks, and shared tags. Ranks them deterministically and attaches
//! human-readable reason labels for the frontend.

use std::collections::HashMap;

use crate::error::StorageError;
use crate::storage::watchtower;
use crate::storage::DbPool;

/// Maximum neighbors to return by default.
pub const DEFAULT_MAX_NEIGHBORS: u32 = 8;

/// Maximum fragments from any single graph neighbor in the final prompt.
pub const MAX_GRAPH_FRAGMENTS_PER_NOTE: u32 = 3;

/// Snippet length for neighbor items.
const SNIPPET_LEN: usize = 120;

// ============================================================================
// Scoring weights
// ============================================================================

const WEIGHT_DIRECT_LINK: f64 = 3.0;
const WEIGHT_BACKLINK: f64 = 2.0;
const WEIGHT_SHARED_TAG: f64 = 1.0;
const WEIGHT_CHUNK_BOOST: f64 = 0.5;

// ============================================================================
// Types
// ============================================================================

/// Reason label explaining why a related note was suggested.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionReason {
    LinkedNote,
    Backlink,
    SharedTag,
    MutualLink,
}

/// Intent hint for the frontend to frame the suggestion.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionIntent {
    ProTip,
    Counterpoint,
    Evidence,
    Related,
}

/// A related note discovered via graph expansion.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNeighbor {
    pub node_id: i64,
    pub node_title: Option<String>,
    pub relative_path: String,
    pub reason: SuggestionReason,
    pub reason_label: String,
    pub intent: SuggestionIntent,
    pub matched_tags: Vec<String>,
    pub edge_count: u32,
    pub shared_tag_count: u32,
    pub score: f64,
    pub snippet: Option<String>,
    pub best_chunk_id: Option<i64>,
    pub heading_path: Option<String>,
}

/// Graph state for API responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphState {
    Available,
    NoRelatedNotes,
    UnresolvedLinks,
    FallbackActive,
    NodeNotIndexed,
}

// ============================================================================
// Scoring (pure functions)
// ============================================================================

/// Compute composite neighbor score from edge counts and chunk boost.
pub fn compute_neighbor_score(
    direct_links: u32,
    backlinks: u32,
    shared_tags: u32,
    best_chunk_boost: f64,
) -> f64 {
    WEIGHT_DIRECT_LINK * f64::from(direct_links)
        + WEIGHT_BACKLINK * f64::from(backlinks)
        + WEIGHT_SHARED_TAG * f64::from(shared_tags)
        + WEIGHT_CHUNK_BOOST * best_chunk_boost
}

// ============================================================================
// Classification (pure functions)
// ============================================================================

/// Classify the primary reason a neighbor was suggested.
pub fn classify_suggestion_reason(
    direct_count: u32,
    backlink_count: u32,
    shared_tag_count: u32,
) -> SuggestionReason {
    let has_direct = direct_count > 0;
    let has_backlink = backlink_count > 0;
    if has_direct && has_backlink {
        SuggestionReason::MutualLink
    } else if has_direct {
        SuggestionReason::LinkedNote
    } else if has_backlink {
        SuggestionReason::Backlink
    } else if shared_tag_count > 0 {
        SuggestionReason::SharedTag
    } else {
        // Shouldn't happen, but safe default.
        SuggestionReason::LinkedNote
    }
}

/// Classify the intent from an edge label using keyword heuristics.
pub fn classify_suggestion_intent(edge_label: Option<&str>) -> SuggestionIntent {
    let label = match edge_label {
        Some(l) => l.to_lowercase(),
        None => return SuggestionIntent::Related,
    };

    if label.contains("counterpoint")
        || label.contains(" vs ")
        || label.contains("alternative")
        || label.contains("contrast")
    {
        SuggestionIntent::Counterpoint
    } else if label.contains("tip")
        || label.contains("how-to")
        || label.contains("how to")
        || label.contains("guide")
    {
        SuggestionIntent::ProTip
    } else if label.contains("data")
        || label.contains("evidence")
        || label.contains("study")
        || label.contains("stat")
    {
        SuggestionIntent::Evidence
    } else {
        SuggestionIntent::Related
    }
}

/// Build a human-readable reason label string.
pub fn build_reason_label(reason: &SuggestionReason, matched_tags: &[String]) -> String {
    match reason {
        SuggestionReason::LinkedNote => "linked note".to_string(),
        SuggestionReason::Backlink => "backlink".to_string(),
        SuggestionReason::MutualLink => "mutual link".to_string(),
        SuggestionReason::SharedTag => {
            if matched_tags.is_empty() {
                "shared tag".to_string()
            } else {
                let tags: Vec<String> = matched_tags.iter().map(|t| format!("#{t}")).collect();
                format!("shared tag: {}", tags.join(", "))
            }
        }
    }
}

// ============================================================================
// Graph expansion (DB-backed)
// ============================================================================

/// Intermediate accumulator for grouping edges by target node.
struct NeighborAccum {
    direct_links: u32,
    backlinks: u32,
    shared_tags: Vec<String>,
    best_edge_label: Option<String>,
}

/// Expand 1-hop graph neighbors around a selected note.
///
/// Queries outgoing edges (forward links) and incoming edges (backlinks),
/// plus shared-tag neighbors. Groups by target node, scores, enriches with
/// node metadata and best chunk, and returns top `max_neighbors` results.
pub async fn expand_graph_neighbors(
    pool: &DbPool,
    account_id: &str,
    node_id: i64,
    max_neighbors: u32,
) -> Result<Vec<GraphNeighbor>, StorageError> {
    let max = if max_neighbors == 0 {
        DEFAULT_MAX_NEIGHBORS
    } else {
        max_neighbors
    };

    // 1. Fetch outgoing edges (forward links from this node).
    let outgoing = watchtower::get_edges_for_source(pool, account_id, node_id).await?;

    // 2. Fetch incoming edges (backlinks pointing to this node).
    let incoming = watchtower::get_edges_for_target(pool, account_id, node_id).await?;

    // 3. Fetch shared-tag neighbors.
    let tag_neighbors =
        watchtower::find_shared_tag_neighbors(pool, account_id, node_id, max * 2).await?;

    // 4. Group all neighbors by target node ID.
    let mut accum: HashMap<i64, NeighborAccum> = HashMap::new();

    for edge in &outgoing {
        let entry = accum.entry(edge.target_node_id).or_insert(NeighborAccum {
            direct_links: 0,
            backlinks: 0,
            shared_tags: Vec::new(),
            best_edge_label: None,
        });
        match edge.edge_type.as_str() {
            "backlink" => entry.backlinks += 1,
            "shared_tag" => {
                if let Some(label) = &edge.edge_label {
                    if !entry.shared_tags.contains(label) {
                        entry.shared_tags.push(label.clone());
                    }
                }
            }
            _ => entry.direct_links += 1, // wikilink, markdown_link
        }
        if entry.best_edge_label.is_none() && edge.edge_type != "shared_tag" {
            entry.best_edge_label = edge.edge_label.clone();
        }
    }

    for edge in &incoming {
        // Skip self-referential edges.
        if edge.source_node_id == node_id {
            continue;
        }
        let entry = accum.entry(edge.source_node_id).or_insert(NeighborAccum {
            direct_links: 0,
            backlinks: 0,
            shared_tags: Vec::new(),
            best_edge_label: None,
        });
        match edge.edge_type.as_str() {
            "wikilink" | "markdown_link" => entry.backlinks += 1,
            "shared_tag" => {
                if let Some(label) = &edge.edge_label {
                    if !entry.shared_tags.contains(label) {
                        entry.shared_tags.push(label.clone());
                    }
                }
            }
            _ => entry.backlinks += 1,
        }
        if entry.best_edge_label.is_none() && edge.edge_type != "shared_tag" {
            entry.best_edge_label = edge.edge_label.clone();
        }
    }

    for (neighbor_node_id, tag_text) in &tag_neighbors {
        let entry = accum.entry(*neighbor_node_id).or_insert(NeighborAccum {
            direct_links: 0,
            backlinks: 0,
            shared_tags: Vec::new(),
            best_edge_label: None,
        });
        if !entry.shared_tags.contains(tag_text) {
            entry.shared_tags.push(tag_text.clone());
        }
    }

    if accum.is_empty() {
        return Ok(Vec::new());
    }

    // 5. Batch-fetch node metadata.
    let neighbor_ids: Vec<i64> = accum.keys().copied().collect();
    let nodes = watchtower::get_nodes_by_ids(pool, account_id, &neighbor_ids).await?;
    let node_map: HashMap<i64, &watchtower::ContentNode> =
        nodes.iter().map(|n| (n.id, n)).collect();

    // 6. Batch-fetch best chunk per neighbor.
    let best_chunks =
        watchtower::get_best_chunks_for_nodes(pool, account_id, &neighbor_ids).await?;
    let chunk_map: HashMap<i64, &watchtower::ContentChunk> =
        best_chunks.iter().map(|c| (c.node_id, c)).collect();

    // 7. Build scored neighbor list.
    let mut neighbors: Vec<GraphNeighbor> = accum
        .into_iter()
        .filter_map(|(nid, acc)| {
            let node = node_map.get(&nid)?;
            let shared_tag_count = acc.shared_tags.len() as u32;
            let edge_count = acc.direct_links + acc.backlinks + shared_tag_count;

            let chunk_boost = chunk_map
                .get(&nid)
                .map(|c| c.retrieval_boost)
                .unwrap_or(0.0);

            let score = compute_neighbor_score(
                acc.direct_links,
                acc.backlinks,
                shared_tag_count,
                chunk_boost,
            );

            let reason =
                classify_suggestion_reason(acc.direct_links, acc.backlinks, shared_tag_count);
            let intent = classify_suggestion_intent(acc.best_edge_label.as_deref());
            let reason_label = build_reason_label(&reason, &acc.shared_tags);

            let (snippet, best_chunk_id, heading_path) = match chunk_map.get(&nid) {
                Some(c) => (
                    Some(truncate(c.chunk_text.as_str(), SNIPPET_LEN)),
                    Some(c.id),
                    if c.heading_path.is_empty() {
                        None
                    } else {
                        Some(c.heading_path.clone())
                    },
                ),
                None => (None, None, None),
            };

            Some(GraphNeighbor {
                node_id: nid,
                node_title: node.title.clone(),
                relative_path: node.relative_path.clone(),
                reason,
                reason_label,
                intent,
                matched_tags: acc.shared_tags,
                edge_count,
                shared_tag_count,
                score,
                snippet,
                best_chunk_id,
                heading_path,
            })
        })
        .collect();

    // 8. Sort: score DESC, edge_count DESC, node_id ASC.
    neighbors.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.edge_count.cmp(&a.edge_count))
            .then(a.node_id.cmp(&b.node_id))
    });

    // 9. Cap at max neighbors.
    neighbors.truncate(max as usize);

    Ok(neighbors)
}

// ============================================================================
// Helpers
// ============================================================================

fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let mut end = max_len.saturating_sub(3);
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- compute_neighbor_score --

    #[test]
    fn score_weights_verified() {
        let score = compute_neighbor_score(1, 1, 1, 1.0);
        // 3.0 + 2.0 + 1.0 + 0.5 = 6.5
        assert!((score - 6.5).abs() < f64::EPSILON);
    }

    #[test]
    fn score_zero_inputs() {
        let score = compute_neighbor_score(0, 0, 0, 0.0);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_direct_only() {
        let score = compute_neighbor_score(2, 0, 0, 0.0);
        assert!((score - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_backlink_only() {
        let score = compute_neighbor_score(0, 3, 0, 0.0);
        assert!((score - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_shared_tag_only() {
        let score = compute_neighbor_score(0, 0, 4, 0.0);
        assert!((score - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_chunk_boost_contribution() {
        let score = compute_neighbor_score(0, 0, 0, 2.5);
        assert!((score - 1.25).abs() < f64::EPSILON);
    }

    // -- classify_suggestion_reason --

    #[test]
    fn reason_mutual_link() {
        assert_eq!(
            classify_suggestion_reason(1, 1, 0),
            SuggestionReason::MutualLink
        );
    }

    #[test]
    fn reason_linked_note() {
        assert_eq!(
            classify_suggestion_reason(1, 0, 0),
            SuggestionReason::LinkedNote
        );
    }

    #[test]
    fn reason_backlink() {
        assert_eq!(
            classify_suggestion_reason(0, 1, 0),
            SuggestionReason::Backlink
        );
    }

    #[test]
    fn reason_shared_tag() {
        assert_eq!(
            classify_suggestion_reason(0, 0, 2),
            SuggestionReason::SharedTag
        );
    }

    #[test]
    fn reason_mutual_takes_precedence_over_tags() {
        assert_eq!(
            classify_suggestion_reason(1, 1, 3),
            SuggestionReason::MutualLink
        );
    }

    // -- classify_suggestion_intent --

    #[test]
    fn intent_none_label() {
        assert_eq!(classify_suggestion_intent(None), SuggestionIntent::Related);
    }

    #[test]
    fn intent_counterpoint() {
        assert_eq!(
            classify_suggestion_intent(Some("see counterpoint")),
            SuggestionIntent::Counterpoint
        );
    }

    #[test]
    fn intent_vs() {
        assert_eq!(
            classify_suggestion_intent(Some("React vs Vue")),
            SuggestionIntent::Counterpoint
        );
    }

    #[test]
    fn intent_pro_tip() {
        assert_eq!(
            classify_suggestion_intent(Some("quick tip")),
            SuggestionIntent::ProTip
        );
    }

    #[test]
    fn intent_guide() {
        assert_eq!(
            classify_suggestion_intent(Some("setup guide")),
            SuggestionIntent::ProTip
        );
    }

    #[test]
    fn intent_evidence() {
        assert_eq!(
            classify_suggestion_intent(Some("research data")),
            SuggestionIntent::Evidence
        );
    }

    #[test]
    fn intent_study() {
        assert_eq!(
            classify_suggestion_intent(Some("case study")),
            SuggestionIntent::Evidence
        );
    }

    #[test]
    fn intent_default_related() {
        assert_eq!(
            classify_suggestion_intent(Some("just a note")),
            SuggestionIntent::Related
        );
    }

    // -- build_reason_label --

    #[test]
    fn label_linked_note() {
        assert_eq!(
            build_reason_label(&SuggestionReason::LinkedNote, &[]),
            "linked note"
        );
    }

    #[test]
    fn label_backlink() {
        assert_eq!(
            build_reason_label(&SuggestionReason::Backlink, &[]),
            "backlink"
        );
    }

    #[test]
    fn label_mutual_link() {
        assert_eq!(
            build_reason_label(&SuggestionReason::MutualLink, &[]),
            "mutual link"
        );
    }

    #[test]
    fn label_shared_tag_no_tags() {
        assert_eq!(
            build_reason_label(&SuggestionReason::SharedTag, &[]),
            "shared tag"
        );
    }

    #[test]
    fn label_shared_tag_single() {
        assert_eq!(
            build_reason_label(&SuggestionReason::SharedTag, &["rust".to_string()]),
            "shared tag: #rust"
        );
    }

    #[test]
    fn label_shared_tag_multiple() {
        let tags = vec!["rust".to_string(), "async".to_string()];
        assert_eq!(
            build_reason_label(&SuggestionReason::SharedTag, &tags),
            "shared tag: #rust, #async"
        );
    }

    // -- truncate --

    #[test]
    fn truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long() {
        let result = truncate("hello world this is long text", 10);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 13);
    }

    // -- SuggestionReason serde --

    #[test]
    fn reason_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&SuggestionReason::LinkedNote).unwrap(),
            "\"linked_note\""
        );
        assert_eq!(
            serde_json::to_string(&SuggestionReason::MutualLink).unwrap(),
            "\"mutual_link\""
        );
        assert_eq!(
            serde_json::to_string(&SuggestionReason::SharedTag).unwrap(),
            "\"shared_tag\""
        );
    }

    // -- GraphState serde --

    #[test]
    fn graph_state_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&GraphState::NoRelatedNotes).unwrap(),
            "\"no_related_notes\""
        );
        assert_eq!(
            serde_json::to_string(&GraphState::FallbackActive).unwrap(),
            "\"fallback_active\""
        );
    }

    #[test]
    fn graph_state_all_variants_serialize() {
        assert_eq!(
            serde_json::to_string(&GraphState::Available).unwrap(),
            "\"available\""
        );
        assert_eq!(
            serde_json::to_string(&GraphState::UnresolvedLinks).unwrap(),
            "\"unresolved_links\""
        );
        assert_eq!(
            serde_json::to_string(&GraphState::NodeNotIndexed).unwrap(),
            "\"node_not_indexed\""
        );
    }

    #[test]
    fn score_tag_only_neighbor() {
        // 0 direct, 0 backlinks, 2 shared_tags, no chunk boost = 2.0
        let score = compute_neighbor_score(0, 0, 2, 0.0);
        assert!((score - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn classify_reason_zero_direct_zero_backlink_with_tags() {
        assert_eq!(
            classify_suggestion_reason(0, 0, 5),
            SuggestionReason::SharedTag
        );
    }

    #[test]
    fn classify_reason_zero_everything_defaults_linked() {
        // Edge case: no links, no backlinks, no tags → defaults to LinkedNote
        assert_eq!(
            classify_suggestion_reason(0, 0, 0),
            SuggestionReason::LinkedNote
        );
    }
}
