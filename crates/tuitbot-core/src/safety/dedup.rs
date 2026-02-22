//! Duplicate reply prevention.
//!
//! Provides exact-match deduplication (never reply to the same tweet twice)
//! and phrasing similarity detection (reject replies too similar to recent ones).

use crate::error::StorageError;
use crate::storage::DbPool;
use std::collections::HashSet;

/// Checks for duplicate and similar replies.
pub struct DedupChecker {
    pool: DbPool,
}

impl DedupChecker {
    /// Create a new dedup checker backed by the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Check if a reply has already been sent to the given tweet.
    ///
    /// Returns `true` if a reply exists in `replies_sent` for this tweet ID.
    pub async fn has_replied_to(&self, tweet_id: &str) -> Result<bool, StorageError> {
        crate::storage::replies::has_replied_to(&self.pool, tweet_id).await
    }

    /// Check if a proposed reply is too similar to recent replies.
    ///
    /// Compares against the last `limit` replies using Jaccard word similarity.
    /// Returns `true` if any recent reply has >= 0.8 similarity or is an exact match.
    /// Replies shorter than 5 words skip the similarity check (too short for meaningful comparison).
    pub async fn is_phrasing_similar(
        &self,
        new_reply: &str,
        limit: i64,
    ) -> Result<bool, StorageError> {
        if new_reply.is_empty() {
            return Ok(false);
        }

        let recent = crate::storage::replies::get_recent_reply_contents(&self.pool, limit).await?;
        let new_tokens = tokenize(new_reply);

        for recent_reply in &recent {
            // Exact match check
            if new_reply == recent_reply {
                return Ok(true);
            }

            // Skip similarity check for very short replies
            if new_tokens.len() < 5 {
                continue;
            }

            let recent_tokens = tokenize(recent_reply);
            if jaccard_similarity(&new_tokens, &recent_tokens) >= 0.8 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get recent reply contents for testing and debugging.
    pub async fn get_recent_reply_phrases(&self, limit: i64) -> Result<Vec<String>, StorageError> {
        crate::storage::replies::get_recent_reply_contents(&self.pool, limit).await
    }
}

/// Tokenize text into a set of lowercase alphanumeric words.
fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Calculate Jaccard similarity between two word sets.
///
/// Returns a value between 0.0 (no overlap) and 1.0 (identical sets).
/// Two empty sets are considered identical (returns 1.0).
fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    intersection / union
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;
    use crate::storage::replies::{insert_reply, ReplySent};

    fn sample_reply(target_id: &str, content: &str) -> ReplySent {
        ReplySent {
            id: 0,
            target_tweet_id: target_id.to_string(),
            reply_tweet_id: Some("r_123".to_string()),
            reply_content: content.to_string(),
            llm_provider: None,
            llm_model: None,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            status: "sent".to_string(),
            error_message: None,
        }
    }

    #[test]
    fn tokenize_basic() {
        let tokens = tokenize("Hello, World! This is a test.");
        assert!(tokens.contains("hello"));
        assert!(tokens.contains("world"));
        assert!(tokens.contains("test"));
        assert!(!tokens.contains(""));
    }

    #[test]
    fn tokenize_strips_punctuation() {
        let tokens = tokenize("(great) [tool] {for} developers!");
        assert!(tokens.contains("great"));
        assert!(tokens.contains("tool"));
        assert!(tokens.contains("for"));
        assert!(tokens.contains("developers"));
    }

    #[test]
    fn tokenize_empty_string() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn jaccard_identical_sets() {
        let a: HashSet<String> = ["hello", "world"].iter().map(|s| s.to_string()).collect();
        let b = a.clone();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_disjoint_sets() {
        let a: HashSet<String> = ["hello", "world"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["foo", "bar"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let a: HashSet<String> = ["hello", "world", "foo"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let b: HashSet<String> = ["hello", "world", "bar"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        // intersection=2, union=4 => 0.5
        let sim = jaccard_similarity(&a, &b);
        assert!((sim - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_empty_sets() {
        let a: HashSet<String> = HashSet::new();
        let b: HashSet<String> = HashSet::new();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn has_replied_to_works() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        assert!(!checker.has_replied_to("tweet_123").await.expect("check"));

        let reply = sample_reply("tweet_123", "Some reply");
        insert_reply(&pool, &reply).await.expect("insert");

        assert!(checker.has_replied_to("tweet_123").await.expect("check"));
        assert!(!checker.has_replied_to("tweet_456").await.expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_exact_match() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let reply = sample_reply("t1", "This is a great tool for developers");
        insert_reply(&pool, &reply).await.expect("insert");

        assert!(checker
            .is_phrasing_similar("This is a great tool for developers", 20)
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_high_overlap() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let reply = sample_reply("t1", "This is a great tool for developers and engineers");
        insert_reply(&pool, &reply).await.expect("insert");

        // Very similar phrasing (most words overlap)
        assert!(checker
            .is_phrasing_similar("This is a great tool for developers and designers", 20)
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_no_overlap() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let reply = sample_reply("t1", "This is a great tool for developers and engineers");
        insert_reply(&pool, &reply).await.expect("insert");

        assert!(!checker
            .is_phrasing_similar("I love cooking pasta with fresh basil and tomatoes", 20)
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_empty_string() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let reply = sample_reply("t1", "Some reply");
        insert_reply(&pool, &reply).await.expect("insert");

        assert!(!checker.is_phrasing_similar("", 20).await.expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_short_reply_skips_similarity() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let reply = sample_reply("t1", "Great point!");
        insert_reply(&pool, &reply).await.expect("insert");

        // Short reply (< 5 words) - exact match still works
        assert!(checker
            .is_phrasing_similar("Great point!", 20)
            .await
            .expect("check"));

        // But similar short phrases don't trigger (avoids false positives)
        assert!(!checker
            .is_phrasing_similar("Good point!", 20)
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn is_phrasing_similar_no_recent_replies() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        assert!(!checker
            .is_phrasing_similar("Any reply text here that is long enough to test", 20)
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn get_recent_reply_phrases_works() {
        let pool = init_test_db().await.expect("init db");
        let checker = DedupChecker::new(pool.clone());

        let r1 = sample_reply("t1", "Reply one");
        let r2 = sample_reply("t2", "Reply two");
        insert_reply(&pool, &r1).await.expect("ins1");
        insert_reply(&pool, &r2).await.expect("ins2");

        let phrases = checker.get_recent_reply_phrases(5).await.expect("get");
        assert_eq!(phrases.len(), 2);
    }
}
