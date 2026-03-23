//! In-memory semantic search index with brute-force cosine similarity.
//!
//! Stores embedding vectors in memory for nearest-neighbor search. Uses a
//! linear scan with cosine distance — acceptable for <50K vectors. The index
//! can be swapped for an HNSW implementation later via the same API.
//!
//! SQLite is the source of truth; this index is rebuilt from DB at startup.

use std::collections::HashMap;

/// Error type for semantic index operations.
#[derive(Debug, thiserror::Error)]
pub enum SemanticSearchError {
    /// Vector dimension does not match index configuration.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual dimension provided.
        actual: usize,
    },

    /// Index has reached its configured capacity.
    #[error("index full: capacity is {0}")]
    IndexFull(usize),

    /// Internal error.
    #[error("semantic index error: {0}")]
    Internal(String),
}

/// In-memory vector index for semantic search.
pub struct SemanticIndex {
    vectors: HashMap<i64, Vec<f32>>,
    dimension: usize,
    model_id: String,
    capacity: usize,
}

impl SemanticIndex {
    /// Create a new empty index.
    pub fn new(dimension: usize, model_id: String, capacity: usize) -> Self {
        Self {
            vectors: HashMap::with_capacity(capacity.min(1024)),
            dimension,
            model_id,
            capacity,
        }
    }

    /// Insert a vector for a chunk. Overwrites if chunk_id already exists.
    pub fn insert(
        &mut self,
        chunk_id: i64,
        embedding: Vec<f32>,
    ) -> Result<(), SemanticSearchError> {
        if embedding.len() != self.dimension {
            return Err(SemanticSearchError::DimensionMismatch {
                expected: self.dimension,
                actual: embedding.len(),
            });
        }

        if !self.vectors.contains_key(&chunk_id) && self.vectors.len() >= self.capacity {
            return Err(SemanticSearchError::IndexFull(self.capacity));
        }

        self.vectors.insert(chunk_id, embedding);
        Ok(())
    }

    /// Remove a vector by chunk_id. Returns false if not found.
    pub fn remove(&mut self, chunk_id: i64) -> bool {
        self.vectors.remove(&chunk_id).is_some()
    }

    /// Search for the top-k nearest vectors by cosine similarity.
    ///
    /// Returns `(chunk_id, distance)` pairs sorted ascending by distance
    /// (smaller = more similar). Distance = 1.0 - cosine_similarity.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(i64, f32)> {
        if self.vectors.is_empty() || k == 0 {
            return vec![];
        }

        let mut scored: Vec<(i64, f32)> = self
            .vectors
            .iter()
            .map(|(&chunk_id, vec)| {
                let dist = cosine_distance(query, vec);
                (chunk_id, dist)
            })
            .collect();

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Number of vectors in the index.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Model identifier for this index.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Vector dimension for this index.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Clear and rebuild the index from a batch of (chunk_id, embedding) pairs.
    pub fn rebuild_from(&mut self, embeddings: Vec<(i64, Vec<f32>)>) {
        self.vectors.clear();
        for (chunk_id, vec) in embeddings {
            if vec.len() == self.dimension {
                self.vectors.insert(chunk_id, vec);
            }
        }
    }
}

/// Compute cosine distance between two vectors: 1.0 - cosine_similarity.
///
/// Returns 1.0 (max distance) if either vector has zero magnitude.
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < f32::EPSILON {
        return 1.0;
    }

    1.0 - (dot / denom)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vec(val: f32, dim: usize) -> Vec<f32> {
        vec![val; dim]
    }

    #[test]
    fn insert_and_search_finds_nearest() {
        let mut idx = SemanticIndex::new(3, "test".to_string(), 100);
        idx.insert(1, vec![1.0, 0.0, 0.0]).unwrap();
        idx.insert(2, vec![0.0, 1.0, 0.0]).unwrap();
        idx.insert(3, vec![0.9, 0.1, 0.0]).unwrap();

        let results = idx.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        // chunk 1 should be the closest (exact match)
        assert_eq!(results[0].0, 1);
        assert!(results[0].1 < 0.01);
    }

    #[test]
    fn search_returns_correct_top_k() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 100);
        for i in 0..10 {
            idx.insert(i, vec![i as f32, 1.0]).unwrap();
        }

        let results = idx.search(&[9.0, 1.0], 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 9); // closest
    }

    #[test]
    fn remove_makes_vector_unfindable() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 100);
        idx.insert(1, vec![1.0, 0.0]).unwrap();
        idx.insert(2, vec![0.0, 1.0]).unwrap();

        assert!(idx.remove(1));
        assert_eq!(idx.len(), 1);

        let results = idx.search(&[1.0, 0.0], 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 2);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 100);
        assert!(!idx.remove(999));
    }

    #[test]
    fn dimension_mismatch_on_insert() {
        let mut idx = SemanticIndex::new(3, "test".to_string(), 100);
        let err = idx.insert(1, vec![1.0, 2.0]).unwrap_err();
        matches!(
            err,
            SemanticSearchError::DimensionMismatch {
                expected: 3,
                actual: 2,
            }
        );
    }

    #[test]
    fn empty_search_returns_empty() {
        let idx = SemanticIndex::new(3, "test".to_string(), 100);
        let results = idx.search(&[1.0, 0.0, 0.0], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn search_with_k_zero_returns_empty() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 100);
        idx.insert(1, vec![1.0, 0.0]).unwrap();
        let results = idx.search(&[1.0, 0.0], 0);
        assert!(results.is_empty());
    }

    #[test]
    fn rebuild_replaces_all_contents() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 100);
        idx.insert(1, vec![1.0, 0.0]).unwrap();
        idx.insert(2, vec![0.0, 1.0]).unwrap();
        assert_eq!(idx.len(), 2);

        idx.rebuild_from(vec![(10, vec![0.5, 0.5]), (11, vec![0.3, 0.7])]);
        assert_eq!(idx.len(), 2);
        assert!(!idx.vectors.contains_key(&1));
        assert!(idx.vectors.contains_key(&10));
    }

    #[test]
    fn capacity_limit_respected() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 2);
        idx.insert(1, vec![1.0, 0.0]).unwrap();
        idx.insert(2, vec![0.0, 1.0]).unwrap();

        let err = idx.insert(3, vec![0.5, 0.5]).unwrap_err();
        matches!(err, SemanticSearchError::IndexFull(2));
    }

    #[test]
    fn overwrite_existing_does_not_count_as_new() {
        let mut idx = SemanticIndex::new(2, "test".to_string(), 2);
        idx.insert(1, vec![1.0, 0.0]).unwrap();
        idx.insert(2, vec![0.0, 1.0]).unwrap();
        // Overwrite chunk 1 — should not trigger IndexFull
        idx.insert(1, vec![0.5, 0.5]).unwrap();
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn accessors() {
        let idx = SemanticIndex::new(768, "nomic-embed-text".to_string(), 50_000);
        assert_eq!(idx.dimension(), 768);
        assert_eq!(idx.model_id(), "nomic-embed-text");
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn cosine_distance_identical_vectors() {
        let dist = cosine_distance(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
        assert!(dist.abs() < 1e-5);
    }

    #[test]
    fn cosine_distance_orthogonal_vectors() {
        let dist = cosine_distance(&[1.0, 0.0], &[0.0, 1.0]);
        assert!((dist - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_distance_opposite_vectors() {
        let dist = cosine_distance(&[1.0, 0.0], &[-1.0, 0.0]);
        assert!((dist - 2.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_distance_zero_vector() {
        let dist = cosine_distance(&[0.0, 0.0], &[1.0, 1.0]);
        assert!((dist - 1.0).abs() < 1e-5);
    }

    #[test]
    fn rebuild_skips_wrong_dimension() {
        let mut idx = SemanticIndex::new(3, "test".to_string(), 100);
        idx.rebuild_from(vec![
            (1, vec![1.0, 2.0, 3.0]), // correct dim
            (2, vec![1.0, 2.0]),      // wrong dim — skipped
            (3, make_vec(0.5, 3)),    // correct dim
        ]);
        assert_eq!(idx.len(), 2);
        assert!(idx.vectors.contains_key(&1));
        assert!(!idx.vectors.contains_key(&2));
        assert!(idx.vectors.contains_key(&3));
    }
}
