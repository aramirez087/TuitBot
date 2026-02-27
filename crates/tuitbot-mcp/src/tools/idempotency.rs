//! In-memory idempotency guard for mutation tools.
//!
//! [`IdempotencyStore`] hashes `(tool_name, params)` and rejects duplicate
//! mutation calls within a configurable time window (default 30 seconds).
//! Protects against agent retry storms sending the same mutation twice.
//!
//! DB-backed idempotency (5-minute window) and audit recording are handled
//! by the unified `MutationGateway` in `tuitbot-core::mutation_gateway`.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Thread-safe idempotency store using fingerprint hashing.
pub struct IdempotencyStore {
    entries: Mutex<HashMap<u64, Instant>>,
    window: Duration,
}

impl IdempotencyStore {
    /// Create a new store with the default 30-second dedup window.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            window: Duration::from_secs(30),
        }
    }

    /// Create a store with a custom dedup window (for testing).
    #[cfg(test)]
    pub fn with_window(window: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            window,
        }
    }

    /// Check if this `(tool_name, params)` combination is a duplicate.
    ///
    /// Returns `Some(error_json)` if the call is a duplicate within the
    /// dedup window, `None` if the call should proceed.
    /// Records the fingerprint on first occurrence.
    pub fn check_and_record(&self, tool_name: &str, params_json: &str) -> Option<String> {
        let fingerprint = Self::compute_fingerprint(tool_name, params_json);
        let now = Instant::now();

        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());

        // Evict expired entries.
        entries.retain(|_, ts| now.duration_since(*ts) < self.window);

        if let Some(first_seen) = entries.get(&fingerprint) {
            let age_ms = now.duration_since(*first_seen).as_millis();
            return Some(
                crate::contract::ToolResponse::error(
                    crate::contract::ErrorCode::ValidationError,
                    format!(
                        "Duplicate {tool_name} call detected ({age_ms}ms ago). \
                         Wait {}s before retrying identical mutations.",
                        self.window.as_secs()
                    ),
                )
                .to_json(),
            );
        }

        entries.insert(fingerprint, now);
        None
    }

    fn compute_fingerprint(tool_name: &str, params_json: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        tool_name.hash(&mut hasher);
        params_json.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_duplicate_within_window() {
        let store = IdempotencyStore::new();
        let first = store.check_and_record("post_tweet", r#"{"text":"hi"}"#);
        assert!(first.is_none(), "first call should succeed");

        let second = store.check_and_record("post_tweet", r#"{"text":"hi"}"#);
        assert!(second.is_some(), "duplicate should be blocked");
        let err_json = second.unwrap();
        assert!(err_json.contains("Duplicate"));
        assert!(err_json.contains("validation_error"));
    }

    #[test]
    fn allows_different_params() {
        let store = IdempotencyStore::new();
        let first = store.check_and_record("post_tweet", r#"{"text":"hello"}"#);
        assert!(first.is_none());

        let second = store.check_and_record("post_tweet", r#"{"text":"world"}"#);
        assert!(second.is_none(), "different params should be allowed");
    }

    #[test]
    fn allows_after_eviction() {
        let store = IdempotencyStore::with_window(Duration::from_millis(1));
        let first = store.check_and_record("post_tweet", r#"{"text":"hi"}"#);
        assert!(first.is_none());

        // Wait for the window to expire.
        std::thread::sleep(Duration::from_millis(5));

        let second = store.check_and_record("post_tweet", r#"{"text":"hi"}"#);
        assert!(second.is_none(), "should be allowed after window expires");
    }

    #[test]
    fn different_tools_same_params_allowed() {
        let store = IdempotencyStore::new();
        let first = store.check_and_record("like_tweet", r#"{"tweet_id":"123"}"#);
        assert!(first.is_none());

        let second = store.check_and_record("retweet", r#"{"tweet_id":"123"}"#);
        assert!(second.is_none(), "different tools should be independent");
    }
}
