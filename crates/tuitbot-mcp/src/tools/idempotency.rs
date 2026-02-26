//! In-memory idempotency guard for mutation tools.
//!
//! [`IdempotencyStore`] hashes `(tool_name, params)` and rejects duplicate
//! mutation calls within a configurable time window (default 30 seconds).
//! Protects against agent retry storms sending the same mutation twice.
//!
//! [`MutationGuard`] extends this with DB-backed audit logging and
//! longer-term idempotency (5 minutes).

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tuitbot_core::storage::mutation_audit;
use tuitbot_core::storage::DbPool;

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

/// DB-backed idempotency window (seconds). Mutations with identical
/// fingerprints within this window return the cached result.
const DB_IDEMPOTENCY_WINDOW_SECS: u32 = 300;

/// Begin a mutation with DB idempotency check and audit record creation.
///
/// Returns:
/// - `Ok(MutationGuard)` — proceed with the mutation, then call `complete()` or `fail()`.
/// - `Err(String)` — duplicate detected; return this JSON directly.
pub async fn begin_mutation(
    pool: &DbPool,
    tool_name: &str,
    params_json: &str,
) -> Result<MutationGuard, String> {
    let params_hash = mutation_audit::compute_params_hash(tool_name, params_json);
    let params_summary = mutation_audit::truncate_summary(params_json, 500);

    // Check DB for recent successful duplicate.
    if let Ok(Some(existing)) = mutation_audit::find_recent_duplicate(
        pool,
        tool_name,
        &params_hash,
        DB_IDEMPOTENCY_WINDOW_SECS,
    )
    .await
    {
        // Log the duplicate attempt.
        let correlation_id = uuid_v4();
        let _ = mutation_audit::insert_pending(
            pool,
            &correlation_id,
            None,
            tool_name,
            &params_hash,
            &params_summary,
        )
        .await;
        if let Ok(Some(entry)) = mutation_audit::get_by_correlation_id(pool, &correlation_id).await
        {
            let _ = mutation_audit::mark_duplicate(pool, entry.id, &existing.correlation_id).await;
        }

        let cached_result = existing.result_summary.unwrap_or_default();
        return Err(crate::contract::ToolResponse::success(serde_json::json!({
            "duplicate": true,
            "original_correlation_id": existing.correlation_id,
            "cached_result": serde_json::from_str::<serde_json::Value>(&cached_result)
                .unwrap_or(serde_json::Value::String(cached_result)),
            "message": format!(
                "Identical {} was already executed successfully. Returning cached result.",
                tool_name
            ),
        }))
        .to_json());
    }

    // Insert pending audit record.
    let correlation_id = uuid_v4();
    let audit_id = mutation_audit::insert_pending(
        pool,
        &correlation_id,
        None,
        tool_name,
        &params_hash,
        &params_summary,
    )
    .await
    .map_err(|e| {
        crate::contract::ToolResponse::db_error(format!("Failed to create audit record: {e}"))
            .to_json()
    })?;

    Ok(MutationGuard {
        audit_id,
        correlation_id,
        tool_name: tool_name.to_string(),
    })
}

/// Handle for completing a mutation audit record.
///
/// Created by [`begin_mutation`]. Call [`complete`] on success or [`fail`] on error.
#[derive(Debug)]
pub struct MutationGuard {
    /// DB row ID of the pending audit record.
    pub audit_id: i64,
    /// Unique correlation ID for this mutation attempt.
    pub correlation_id: String,
    /// Tool name (e.g. `"x_post_tweet"`).
    pub tool_name: String,
}

impl MutationGuard {
    /// Mark the mutation as successfully completed.
    pub async fn complete(
        &self,
        pool: &DbPool,
        result_summary: &str,
        rollback_json: Option<&str>,
        elapsed_ms: u64,
    ) {
        let summary = mutation_audit::truncate_summary(result_summary, 500);
        let _ = mutation_audit::complete_success(
            pool,
            self.audit_id,
            &summary,
            rollback_json,
            elapsed_ms,
        )
        .await;
    }

    /// Mark the mutation as failed.
    pub async fn fail(&self, pool: &DbPool, error_msg: &str, elapsed_ms: u64) {
        let _ = mutation_audit::complete_failure(pool, self.audit_id, error_msg, elapsed_ms).await;
    }
}

/// Generate a UUID v4-like string without pulling in the `uuid` crate.
fn uuid_v4() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::SystemTime;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut hasher = DefaultHasher::new();
    nanos.hash(&mut hasher);
    count.hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let h1 = hasher.finish();
    count.wrapping_add(1).hash(&mut hasher);
    let h2 = hasher.finish();

    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (h1 >> 32) as u32,
        (h1 >> 16) as u16,
        h1 as u16 & 0x0fff,
        (h2 >> 48) as u16 & 0x3fff | 0x8000,
        h2 & 0xffffffffffff,
    )
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

    #[test]
    fn uuid_v4_format() {
        let id = uuid_v4();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[14..15], "4");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn uuid_v4_unique() {
        let ids: Vec<String> = (0..100).map(|_| uuid_v4()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(ids.len(), unique.len(), "UUIDs should be unique");
    }

    #[tokio::test]
    async fn begin_mutation_creates_audit_record() {
        let pool = tuitbot_core::storage::init_test_db()
            .await
            .expect("init db");

        let guard = begin_mutation(&pool, "x_post_tweet", r#"{"text":"hello"}"#)
            .await
            .expect("begin");

        assert!(!guard.correlation_id.is_empty());
        assert_eq!(guard.tool_name, "x_post_tweet");

        let entry = mutation_audit::get_by_correlation_id(&pool, &guard.correlation_id)
            .await
            .expect("get")
            .expect("found");
        assert_eq!(entry.status, "pending");
        assert_eq!(entry.tool_name, "x_post_tweet");
    }

    #[tokio::test]
    async fn begin_mutation_detects_db_duplicate() {
        let pool = tuitbot_core::storage::init_test_db()
            .await
            .expect("init db");

        let guard = begin_mutation(&pool, "x_post_tweet", r#"{"text":"hello"}"#)
            .await
            .expect("begin");
        guard
            .complete(&pool, r#"{"tweet_id":"999"}"#, None, 100)
            .await;

        let result = begin_mutation(&pool, "x_post_tweet", r#"{"text":"hello"}"#).await;
        assert!(result.is_err(), "should detect duplicate");
        let err_json = result.unwrap_err();
        assert!(err_json.contains("duplicate"));
        assert!(err_json.contains("999"));
    }

    #[tokio::test]
    async fn begin_mutation_allows_after_failure() {
        let pool = tuitbot_core::storage::init_test_db()
            .await
            .expect("init db");

        let guard = begin_mutation(&pool, "x_post_tweet", r#"{"text":"retry me"}"#)
            .await
            .expect("begin");
        guard.fail(&pool, "Rate limited", 50).await;

        let result = begin_mutation(&pool, "x_post_tweet", r#"{"text":"retry me"}"#).await;
        assert!(result.is_ok(), "retry after failure should be allowed");
    }
}
