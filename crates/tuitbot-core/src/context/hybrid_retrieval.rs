//! Hybrid retrieval blending semantic, keyword, and graph signals.
//!
//! Uses Reciprocal Rank Fusion (RRF) to merge heterogeneous result lists
//! into a single ranked list with per-result `MatchReason` explanations.

use std::collections::HashMap;

use crate::context::retrieval::MatchReason;
use crate::context::semantic_search::SemanticHit;
use crate::error::StorageError;
use crate::storage::watchtower::{self, ChunkWithNodeContext};
use crate::storage::DbPool;

/// RRF constant from the original paper. Prevents high-ranked items
/// from dominating the fused score.
const RRF_K: f64 = 60.0;

/// Maximum snippet length for evidence results.
const SNIPPET_MAX_LEN: usize = 120;

/// A single result from hybrid retrieval with match explanation.
#[derive(Debug, Clone)]
pub struct EvidenceResult {
    /// Content chunk ID.
    pub chunk_id: i64,
    /// Parent content node ID.
    pub node_id: i64,
    /// Heading hierarchy path.
    pub heading_path: String,
    /// Relative file path of the source note.
    pub source_path: String,
    /// Title of the source note.
    pub source_title: Option<String>,
    /// Short excerpt from the chunk text.
    pub snippet: String,
    /// RRF-fused score (relative, not absolute).
    pub score: f64,
    /// How this result was matched.
    pub match_reason: MatchReason,
    /// Title of the parent node (if available).
    pub node_title: Option<String>,
}

/// Blend semantic, keyword, and graph signals into a unified ranked result list.
///
/// When `semantic_hits` is `None`, falls back to keyword-only retrieval.
/// When `selected_node_ids` is provided, graph neighbors contribute to ranking.
pub async fn hybrid_search(
    pool: &DbPool,
    account_id: &str,
    query: &str,
    semantic_hits: Option<&[SemanticHit]>,
    selected_node_ids: Option<&[i64]>,
    limit: u32,
) -> Result<Vec<EvidenceResult>, StorageError> {
    if query.is_empty() && semantic_hits.map_or(true, |h| h.is_empty()) {
        return Ok(Vec::new());
    }

    // Collect all chunk metadata we'll need for the final results.
    let mut chunk_meta: HashMap<i64, ChunkMeta> = HashMap::new();

    // Track which lists each chunk_id appears in (for MatchReason classification).
    let mut semantic_ranks: HashMap<i64, usize> = HashMap::new();
    let mut keyword_ranks: HashMap<i64, usize> = HashMap::new();
    let mut graph_ranks: HashMap<i64, usize> = HashMap::new();

    // --- Semantic results ---
    if let Some(hits) = semantic_hits {
        for (rank, hit) in hits.iter().enumerate() {
            semantic_ranks.insert(hit.chunk_id, rank + 1);
        }

        // Enrich semantic-only hits with chunk metadata from DB.
        let sem_ids: Vec<i64> = hits.iter().map(|h| h.chunk_id).collect();
        if !sem_ids.is_empty() {
            let enriched =
                watchtower::get_chunks_with_context_by_ids(pool, account_id, &sem_ids).await?;
            for cwc in enriched {
                chunk_meta.insert(cwc.chunk.id, ChunkMeta::from_cwc(cwc));
            }
        }
    }

    // --- Keyword results ---
    if !query.is_empty() {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        if !keywords.is_empty() {
            let headroom = (limit * 2).max(20);
            let kw_results =
                watchtower::search_chunks_with_context(pool, account_id, &keywords, headroom)
                    .await?;
            for (rank, cwc) in kw_results.into_iter().enumerate() {
                keyword_ranks.insert(cwc.chunk.id, rank + 1);
                chunk_meta
                    .entry(cwc.chunk.id)
                    .or_insert_with(|| ChunkMeta::from_cwc(cwc));
            }
        }
    }

    // --- Graph results ---
    if let Some(node_ids) = selected_node_ids {
        if !node_ids.is_empty() {
            let headroom = (limit * 2).max(20);
            let graph_results =
                watchtower::get_chunks_for_nodes_with_context(pool, account_id, node_ids, headroom)
                    .await?;
            for (rank, cwc) in graph_results.into_iter().enumerate() {
                graph_ranks.insert(cwc.chunk.id, rank + 1);
                chunk_meta
                    .entry(cwc.chunk.id)
                    .or_insert_with(|| ChunkMeta::from_cwc(cwc));
            }
        }
    }

    // --- RRF fusion ---
    let all_chunk_ids: Vec<i64> = chunk_meta.keys().copied().collect();
    let mut scored: Vec<(i64, f64, MatchReason)> = all_chunk_ids
        .into_iter()
        .map(|cid| {
            let mut rrf_score = 0.0;
            let mut sources = 0u8;

            if let Some(&rank) = semantic_ranks.get(&cid) {
                rrf_score += 1.0 / (RRF_K + rank as f64);
                sources |= 0b001;
            }
            if let Some(&rank) = keyword_ranks.get(&cid) {
                rrf_score += 1.0 / (RRF_K + rank as f64);
                sources |= 0b010;
            }
            if let Some(&rank) = graph_ranks.get(&cid) {
                rrf_score += 1.0 / (RRF_K + rank as f64);
                sources |= 0b100;
            }

            let reason = classify_match_reason(sources);
            (cid, rrf_score, reason)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit as usize);

    // --- Build final results ---
    let results = scored
        .into_iter()
        .filter_map(|(cid, score, reason)| {
            chunk_meta.remove(&cid).map(|meta| EvidenceResult {
                chunk_id: cid,
                node_id: meta.node_id,
                heading_path: meta.heading_path,
                source_path: meta.source_path,
                source_title: meta.source_title.clone(),
                snippet: meta.snippet,
                score,
                match_reason: reason,
                node_title: meta.source_title,
            })
        })
        .collect();

    Ok(results)
}

/// Classify match reason from a bitmask of source lists.
fn classify_match_reason(sources: u8) -> MatchReason {
    match sources.count_ones() {
        0 => MatchReason::Keyword, // shouldn't happen
        1 => match sources {
            0b001 => MatchReason::Semantic,
            0b010 => MatchReason::Keyword,
            0b100 => MatchReason::Graph,
            _ => MatchReason::Keyword,
        },
        _ => MatchReason::Hybrid,
    }
}

/// Intermediate metadata for a chunk, used during RRF fusion.
struct ChunkMeta {
    node_id: i64,
    heading_path: String,
    source_path: String,
    source_title: Option<String>,
    snippet: String,
}

impl ChunkMeta {
    fn from_cwc(cwc: ChunkWithNodeContext) -> Self {
        Self {
            node_id: cwc.chunk.node_id,
            heading_path: cwc.chunk.heading_path,
            source_path: cwc.relative_path,
            source_title: cwc.source_title,
            snippet: truncate_text(&cwc.chunk.chunk_text, SNIPPET_MAX_LEN),
        }
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_semantic_only() {
        assert_eq!(classify_match_reason(0b001), MatchReason::Semantic);
    }

    #[test]
    fn classify_keyword_only() {
        assert_eq!(classify_match_reason(0b010), MatchReason::Keyword);
    }

    #[test]
    fn classify_graph_only() {
        assert_eq!(classify_match_reason(0b100), MatchReason::Graph);
    }

    #[test]
    fn classify_hybrid_semantic_keyword() {
        assert_eq!(classify_match_reason(0b011), MatchReason::Hybrid);
    }

    #[test]
    fn classify_hybrid_all_three() {
        assert_eq!(classify_match_reason(0b111), MatchReason::Hybrid);
    }

    #[test]
    fn classify_hybrid_semantic_graph() {
        assert_eq!(classify_match_reason(0b101), MatchReason::Hybrid);
    }

    #[test]
    fn rrf_score_rank_1_is_highest() {
        // Rank 1 should produce higher score than rank 2
        let score_1 = 1.0 / (RRF_K + 1.0);
        let score_2 = 1.0 / (RRF_K + 2.0);
        assert!(score_1 > score_2);
    }

    #[test]
    fn rrf_multi_list_scores_higher_than_single() {
        // A chunk in two lists should score higher than one in a single list
        let single = 1.0 / (RRF_K + 1.0);
        let double = 1.0 / (RRF_K + 1.0) + 1.0 / (RRF_K + 1.0);
        assert!(double > single);
    }

    #[test]
    fn truncate_text_short() {
        assert_eq!(truncate_text("hello", 10), "hello");
    }

    #[test]
    fn truncate_text_long() {
        let text = "a".repeat(200);
        let result = truncate_text(&text, SNIPPET_MAX_LEN);
        assert!(result.ends_with("..."));
        assert!(result.len() <= SNIPPET_MAX_LEN);
    }

    #[tokio::test]
    async fn empty_query_empty_semantic_returns_empty() {
        let db = crate::storage::init_test_db().await.unwrap();
        let results = hybrid_search(&db, "test-acct", "", None, None, 10)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn keyword_only_with_no_matches_returns_empty() {
        let db = crate::storage::init_test_db().await.unwrap();
        let results = hybrid_search(&db, "test-acct", "nonexistent", None, None, 10)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn limit_zero_returns_empty() {
        let db = crate::storage::init_test_db().await.unwrap();
        let results = hybrid_search(&db, "test-acct", "test", None, None, 0)
            .await
            .unwrap();
        assert!(results.is_empty());
    }
}
