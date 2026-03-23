//! Pure semantic search over the in-memory `SemanticIndex`.
//!
//! Converts raw `(chunk_id, distance)` pairs from the index into scored
//! `SemanticHit` results. The caller is responsible for embedding the query
//! text before calling `semantic_search()` — this keeps the function testable
//! without mocking HTTP.

use super::semantic_index::SemanticIndex;

/// A single hit from the semantic index with chunk ID and similarity score.
#[derive(Debug, Clone)]
pub struct SemanticHit {
    /// Content chunk ID that matched.
    pub chunk_id: i64,
    /// Cosine distance (0.0 = identical, 2.0 = opposite).
    pub distance: f32,
    /// Cosine similarity (1.0 - distance). Higher = more relevant.
    pub similarity: f32,
}

/// Search the in-memory semantic index for the nearest chunks to `query_embedding`.
///
/// Returns up to `limit` hits sorted by descending similarity. Returns an empty
/// vec when the index is empty (fail-open).
pub fn semantic_search(
    index: &SemanticIndex,
    query_embedding: &[f32],
    limit: usize,
) -> Vec<SemanticHit> {
    let raw = index.search(query_embedding, limit);

    raw.into_iter()
        .map(|(chunk_id, distance)| SemanticHit {
            chunk_id,
            distance,
            similarity: 1.0 - distance,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(dim: usize) -> SemanticIndex {
        SemanticIndex::new(dim, "test-model".to_string(), 1000)
    }

    #[test]
    fn empty_index_returns_empty_results() {
        let idx = make_index(3);
        let hits = semantic_search(&idx, &[1.0, 0.0, 0.0], 5);
        assert!(hits.is_empty());
    }

    #[test]
    fn search_returns_sorted_by_similarity() {
        let mut idx = make_index(3);
        idx.insert(1, vec![1.0, 0.0, 0.0]).unwrap();
        idx.insert(2, vec![0.9, 0.1, 0.0]).unwrap();
        idx.insert(3, vec![0.0, 1.0, 0.0]).unwrap();

        let hits = semantic_search(&idx, &[1.0, 0.0, 0.0], 3);
        assert_eq!(hits.len(), 3);
        // First hit should be the exact match (highest similarity)
        assert_eq!(hits[0].chunk_id, 1);
        assert!(hits[0].similarity > 0.99);
        // Similarity should be descending
        assert!(hits[0].similarity >= hits[1].similarity);
        assert!(hits[1].similarity >= hits[2].similarity);
    }

    #[test]
    fn limit_enforcement() {
        let mut idx = make_index(2);
        for i in 0..10 {
            idx.insert(i, vec![i as f32, 1.0]).unwrap();
        }

        let hits = semantic_search(&idx, &[9.0, 1.0], 3);
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn zero_limit_returns_empty() {
        let mut idx = make_index(2);
        idx.insert(1, vec![1.0, 0.0]).unwrap();

        let hits = semantic_search(&idx, &[1.0, 0.0], 0);
        assert!(hits.is_empty());
    }

    #[test]
    fn similarity_is_one_minus_distance() {
        let mut idx = make_index(2);
        idx.insert(1, vec![1.0, 0.0]).unwrap();

        let hits = semantic_search(&idx, &[1.0, 0.0], 1);
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert!((h.similarity - (1.0 - h.distance)).abs() < 1e-6);
    }

    #[test]
    fn orthogonal_vectors_have_zero_similarity() {
        let mut idx = make_index(2);
        idx.insert(1, vec![0.0, 1.0]).unwrap();

        let hits = semantic_search(&idx, &[1.0, 0.0], 1);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].similarity.abs() < 1e-5);
    }
}
