---
work_package_id: WP03
title: Rate Limiting + Safety Module
lane: "done"
dependencies:
- WP02
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP02
base_commit: cdeb29c7786397b924bd45ecbfdb19a0e303d912
created_at: '2026-02-22T00:40:07.128079+00:00'
subtasks:
- T013
- T014
- T015
- T016
- T017
phase: Phase 0 - Foundation
assignee: ''
agent: "claude-opus"
shell_pid: "50187"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP03 -- Rate Limiting + Safety Module

## IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** -- Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ` ```python `, ` ```bash `

---

## Objectives & Success Criteria

- **Rate limits enforce correctly**: The agent never exceeds configured limits for replies/day, tweets/day, or threads/week under any circumstances (FR-013).
- **Period reset works**: When a rate limit period expires, the counter resets to zero and a new period begins automatically.
- **No duplicate replies**: The agent never replies to the same tweet twice, even across restarts (FR-015, SC-007).
- **Phrasing dedup works**: The agent detects and rejects replies with very similar phrasing to recent replies (FR-015).
- **Data retention cleanup runs**: Old records are pruned according to retention rules without deleting rate limit counters or records needed for deduplication (FR-029, FR-030).
- **Safety guard provides clear denial reasons**: When an action is blocked, the caller receives a specific reason (rate limited, already replied, similar phrasing).
- **All operations use transactions**: Check-then-increment for rate limits is atomic to prevent race conditions between concurrent loops.

## Context & Constraints

- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-013 (safety limits), FR-014 (delays with jitter), FR-015 (no duplicate replies, no repeated phrasing), FR-029 (retention cleanup), FR-030 (never delete rate limits or dedup records).
- **Data model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- RateLimitState entity (action_type PK, request_count, period_start, max_requests, period_seconds), retention rules per table.
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 3 (cleanup strategy: unreplied 7 days, replied 90 days, action log 14 days, VACUUM after 1000+ deletes).
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- Module dependency: safety depends on config + storage.
- **Constitution**: `.kittify/memory/constitution.md` -- No `unwrap()`, `///` doc comments, `cargo clippy -D warnings`.
- **Dependency on WP01**: Uses `ConfigError`, `LimitsConfig`, `IntervalsConfig`, `StorageConfig` from config module.
- **Dependency on WP02**: Uses `DbPool`, all storage CRUD functions, `StorageError`.
- **Concurrency model**: Multiple automation loops run concurrently on Tokio. The rate limiter must handle concurrent callers safely. SQLite transactions with WAL mode provide the necessary isolation.

## Subtasks & Detailed Guidance

### Subtask T013 -- Rate Limit State CRUD

- **Purpose**: Implement the database operations for rate limit tracking. Rate limits are stored in SQLite so they persist across restarts and can be atomically checked and incremented.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/rate_limits.rs`.
  2. Define the `RateLimit` Rust struct:
     ```rust
     #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
     pub struct RateLimit {
         pub action_type: String,
         pub request_count: i64,
         pub period_start: String,
         pub max_requests: i64,
         pub period_seconds: i64,
     }
     ```
  3. Implement `pub async fn init_rate_limits(pool: &DbPool, config: &LimitsConfig, intervals: &IntervalsConfig) -> Result<(), StorageError>`:
     a. Define the default rate limit rows:
        - `"reply"`: max_requests = `config.max_replies_per_day`, period_seconds = 86400 (24 hours).
        - `"tweet"`: max_requests = `config.max_tweets_per_day`, period_seconds = 86400.
        - `"thread"`: max_requests = `config.max_threads_per_week`, period_seconds = 604800 (7 days).
        - `"search"`: max_requests = 300, period_seconds = 900 (15 minutes, X API Basic tier limit).
        - `"mention_check"`: max_requests = 180, period_seconds = 900 (15 minutes, X API Basic tier limit).
     b. For each, execute `INSERT OR REPLACE INTO rate_limits (action_type, request_count, period_start, max_requests, period_seconds) VALUES (?, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), ?, ?)`. Use `INSERT OR REPLACE` so re-initialization updates limits from config without losing period tracking. Alternatively, use `INSERT OR IGNORE` to preserve existing counters and only insert if the row does not exist.
     c. **Decision**: Use `INSERT OR IGNORE` for `init_rate_limits` so that existing counters are preserved across restarts. If the user changes config limits, they can reset by deleting the database or the rate_limits rows.
  4. Implement `pub async fn check_rate_limit(pool: &DbPool, action_type: &str) -> Result<bool, StorageError>`:
     a. Within a SINGLE transaction:
        - Fetch the rate limit row for the given `action_type`.
        - If the row does not exist, return `Ok(true)` (no limit configured, allow the action).
        - Parse `period_start` as a timestamp. Calculate if the period has expired: `now - period_start >= period_seconds`.
        - If the period has expired, reset: `UPDATE rate_limits SET request_count = 0, period_start = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE action_type = ?`.
        - After potential reset, check: `request_count < max_requests`.
        - Return `Ok(true)` if under the limit, `Ok(false)` if at or over.
     b. Commit the transaction.
  5. Implement `pub async fn increment_rate_limit(pool: &DbPool, action_type: &str) -> Result<(), StorageError>`:
     - `UPDATE rate_limits SET request_count = request_count + 1 WHERE action_type = ?`.
  6. Implement `pub async fn get_all_rate_limits(pool: &DbPool) -> Result<Vec<RateLimit>, StorageError>`:
     - `SELECT * FROM rate_limits ORDER BY action_type`. Used for status reporting.
  7. Add `pub mod rate_limits;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/rate_limits.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: No -- should complete before T014 (RateLimiter wraps these functions).
- **Notes**: The `check_rate_limit` function does NOT increment the counter. Incrementing is separate so the caller can perform the action first and only increment on success. The check and increment are separate DB operations, which creates a TOCTOU (time-of-check-time-of-use) window. For this single-user CLI, this is acceptable. If stricter atomicity is needed, combine check+increment in a single transaction in `acquire_posting_permit` (T014). Use `chrono::Utc::now()` to get the current time in Rust when comparing against `period_start`. Parse `period_start` with `chrono::DateTime::parse_from_rfc3339` or `NaiveDateTime::parse_from_str`.

### Subtask T014 -- Rate Limiter

- **Purpose**: Create a higher-level `RateLimiter` struct that provides a clean API for the automation loops. It wraps the raw database operations from T013 and exposes purpose-specific methods.
- **Steps**:
  1. Create `crates/replyguy-core/src/safety/mod.rs`.
  2. Define the `RateLimiter` struct:
     ```rust
     pub struct RateLimiter {
         pool: DbPool,
     }

     impl RateLimiter {
         pub fn new(pool: DbPool) -> Self {
             Self { pool }
         }
     }
     ```
  3. Implement convenience check methods:
     - `pub async fn can_reply(&self) -> Result<bool, StorageError>`: Calls `check_rate_limit(&self.pool, "reply")`.
     - `pub async fn can_tweet(&self) -> Result<bool, StorageError>`: Calls `check_rate_limit(&self.pool, "tweet")`.
     - `pub async fn can_thread(&self) -> Result<bool, StorageError>`: Calls `check_rate_limit(&self.pool, "thread")`.
     - `pub async fn can_search(&self) -> Result<bool, StorageError>`: Calls `check_rate_limit(&self.pool, "search")`.
  4. Implement recording methods (called AFTER a successful action):
     - `pub async fn record_reply(&self) -> Result<(), StorageError>`: Calls `increment_rate_limit(&self.pool, "reply")`.
     - `pub async fn record_tweet(&self) -> Result<(), StorageError>`: Calls `increment_rate_limit(&self.pool, "tweet")`.
     - `pub async fn record_thread(&self) -> Result<(), StorageError>`: Calls `increment_rate_limit(&self.pool, "thread")`.
     - `pub async fn record_search(&self) -> Result<(), StorageError>`: Calls `increment_rate_limit(&self.pool, "search")`.
  5. Implement `pub async fn acquire_posting_permit(&self, action_type: &str) -> Result<bool, StorageError>`:
     a. This is a combined check-then-increment within a single transaction for critical posting actions.
     b. Begin a transaction.
     c. Fetch the rate limit row with `SELECT ... FOR UPDATE` semantics (SQLite achieves this via EXCLUSIVE transaction or by locking the row).
     d. Check if period expired, reset if so.
     e. If `request_count < max_requests`, increment and commit. Return `Ok(true)`.
     f. If at limit, rollback. Return `Ok(false)`.
     g. Note: For SQLite, use `BEGIN IMMEDIATE` to acquire a write lock at transaction start, preventing concurrent writers.
  6. Add `pub mod safety;` to `lib.rs`.
- **Files**:
  - `crates/replyguy-core/src/safety/mod.rs`
  - `crates/replyguy-core/src/lib.rs` (add module declaration)
- **Parallel?**: No -- depends on T013 for the underlying CRUD functions.
- **Notes**: The `RateLimiter` does not enforce delays between actions (that is the scheduler's job in WP07). It only enforces count-based limits. The `acquire_posting_permit` method is the preferred way to claim a rate limit slot for posting actions because it is atomic. The simpler `can_reply()` + `record_reply()` pattern is acceptable for the single-user case but `acquire_posting_permit` is safer. Export both so callers can choose. Consider cloning the pool (it is `Arc` internally) rather than taking ownership.

### Subtask T015 -- Duplicate Reply Prevention

- **Purpose**: Prevent the agent from replying to the same tweet twice (exact dedup) and from sending replies with very similar phrasing (phrasing dedup). These are critical safety features per FR-015 and SC-007.
- **Steps**:
  1. Create `crates/replyguy-core/src/safety/dedup.rs`.
  2. Define the `DedupChecker` struct:
     ```rust
     pub struct DedupChecker {
         pool: DbPool,
     }

     impl DedupChecker {
         pub fn new(pool: DbPool) -> Self {
             Self { pool }
         }
     }
     ```
  3. Implement `pub async fn has_replied_to(&self, tweet_id: &str) -> Result<bool, StorageError>`:
     - Delegates to `storage::replies::has_replied_to(&self.pool, tweet_id)`.
     - Returns `true` if a reply to this tweet already exists in `replies_sent`.
  4. Implement `pub async fn is_phrasing_similar(&self, new_reply: &str, limit: i64) -> Result<bool, StorageError>`:
     a. Fetch recent reply contents: `storage::replies::get_recent_reply_contents(&self.pool, limit)`.
     b. For each recent reply, check similarity against `new_reply`:
        - **Exact match**: If `new_reply == recent_reply`, return `true`.
        - **High word overlap**: Tokenize both strings into lowercase words (split on whitespace, strip punctuation). Calculate Jaccard similarity: `|intersection| / |union|`. If similarity >= 0.8 (80% word overlap), return `true`.
     c. If no recent reply is too similar, return `false`.
     d. The default `limit` should be 20 (check last 20 replies for phrasing similarity).
  5. Implement `pub async fn get_recent_reply_phrases(&self, limit: i64) -> Result<Vec<String>, StorageError>`:
     - Delegates to `storage::replies::get_recent_reply_contents(&self.pool, limit)`.
     - Exposed for testing and debugging.
  6. Add a helper function (private) for word tokenization:
     ```rust
     fn tokenize(text: &str) -> HashSet<String> {
         text.to_lowercase()
             .split_whitespace()
             .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
             .filter(|w| !w.is_empty())
             .collect()
     }
     ```
  7. Add a helper function (private) for Jaccard similarity:
     ```rust
     fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
         if a.is_empty() && b.is_empty() {
             return 1.0;
         }
         let intersection = a.intersection(b).count() as f64;
         let union = a.union(b).count() as f64;
         intersection / union
     }
     ```
  8. Add `pub mod dedup;` to `safety/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/safety/dedup.rs`
  - `crates/replyguy-core/src/safety/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T016.
- **Notes**: The Jaccard similarity threshold of 0.8 is a starting point. It catches near-identical rephrasing but allows genuinely different replies on similar topics. This can be tuned later without changing the architecture. The phrasing check is intentionally simple -- no need for embeddings or ML. The `limit` of 20 recent replies balances coverage with performance. Consider making the similarity threshold configurable in `ScoringConfig` or `LimitsConfig` if needed later. Edge case: if `new_reply` is empty, treat it as non-similar (it will fail other validations before posting).

### Subtask T016 -- Data Retention Cleanup

- **Purpose**: Implement the periodic cleanup job that prunes old records to prevent unbounded database growth, per FR-029 and FR-030.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/cleanup.rs`.
  2. Define a return type for cleanup stats:
     ```rust
     #[derive(Debug, Clone, serde::Serialize)]
     pub struct CleanupStats {
         pub discovered_tweets_deleted: u64,
         pub replies_deleted: u64,
         pub original_tweets_deleted: u64,
         pub threads_deleted: u64,
         pub action_log_deleted: u64,
         pub total_deleted: u64,
         pub vacuum_run: bool,
     }
     ```
  3. Implement `pub async fn run_cleanup(pool: &DbPool, retention_days: u32) -> Result<CleanupStats, StorageError>`:
     a. Calculate cutoff timestamps:
        - `unreplied_cutoff`: 7 days ago (fixed, unreplied tweets are low-value).
        - `replied_cutoff`: `retention_days` days ago (default 90).
        - `action_log_cutoff`: 14 days ago (fixed per data-model.md).
     b. **Important ordering**: Delete child records before parent records to respect foreign key constraints.
     c. Delete old replies first (before their parent discovered_tweets):
        ```sql
        DELETE FROM replies_sent
        WHERE created_at < ?  -- replied_cutoff
        ```
        Capture the number of rows affected.
     d. Delete unreplied discovered tweets older than 7 days:
        ```sql
        DELETE FROM discovered_tweets
        WHERE replied_to = 0 AND discovered_at < ?  -- unreplied_cutoff
        ```
     e. Delete replied discovered tweets older than retention_days:
        ```sql
        DELETE FROM discovered_tweets
        WHERE replied_to = 1 AND discovered_at < ?  -- replied_cutoff
        ```
     f. Delete old original tweets:
        ```sql
        DELETE FROM original_tweets
        WHERE created_at < ?  -- replied_cutoff
        ```
     g. Delete old threads (CASCADE will delete thread_tweets):
        ```sql
        DELETE FROM threads
        WHERE created_at < ?  -- replied_cutoff
        ```
     h. Delete old action log entries:
        ```sql
        DELETE FROM action_log
        WHERE created_at < ?  -- action_log_cutoff
        ```
     i. **Never delete rate_limits rows** (FR-030). Rate limit counters reset on period expiry but the rows persist.
     j. Calculate total deleted. If total > 1000, run `VACUUM` to reclaim disk space:
        ```sql
        VACUUM;
        ```
     k. Log cleanup stats using `tracing::info!`.
     l. Return `CleanupStats`.
  4. Add `pub mod cleanup;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/cleanup.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T015.
- **Notes**: Deletion order matters due to foreign key constraints. `replies_sent.target_tweet_id` references `discovered_tweets.id`. Delete replies BEFORE their parent discovered tweets. Alternatively, the foreign key could use `ON DELETE CASCADE`, but that was not specified in the schema (T008) and would change the data model contract. The `VACUUM` command can be slow on large databases and acquires an exclusive lock. Only run it when significant space can be reclaimed (>1000 rows deleted). Use `strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-7 days')` in SQL or calculate the cutoff in Rust with `chrono` and pass as a bind parameter. The Rust approach is preferred for testability (you can mock the current time).

### Subtask T017 -- Safety Orchestration

- **Purpose**: Create the `SafetyGuard` struct that combines the `RateLimiter` and `DedupChecker` into a single pre-flight check interface. All automation loops call `SafetyGuard` before taking any action.
- **Steps**:
  1. In `crates/replyguy-core/src/safety/mod.rs`, define the `SafetyGuard` struct:
     ```rust
     pub struct SafetyGuard {
         rate_limiter: RateLimiter,
         dedup_checker: DedupChecker,
     }

     impl SafetyGuard {
         pub fn new(pool: DbPool) -> Self {
             Self {
                 rate_limiter: RateLimiter::new(pool.clone()),
                 dedup_checker: DedupChecker::new(pool),
             }
         }
     }
     ```
  2. Define a denial reason enum:
     ```rust
     #[derive(Debug, Clone, PartialEq)]
     pub enum DenialReason {
         RateLimited { action_type: String, current: i64, max: i64 },
         AlreadyReplied { tweet_id: String },
         SimilarPhrasing,
     }

     impl std::fmt::Display for DenialReason {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             match self {
                 Self::RateLimited { action_type, current, max } =>
                     write!(f, "Rate limited: {} ({}/{})", action_type, current, max),
                 Self::AlreadyReplied { tweet_id } =>
                     write!(f, "Already replied to tweet {}", tweet_id),
                 Self::SimilarPhrasing =>
                     write!(f, "Reply phrasing too similar to recent replies"),
             }
         }
     }
     ```
  3. Implement `pub async fn can_reply_to(&self, tweet_id: &str, proposed_reply: Option<&str>) -> Result<Result<(), DenialReason>, StorageError>`:
     a. Check rate limit: `self.rate_limiter.can_reply()`. If `false`, fetch current rate limit state for the denial reason and return `Ok(Err(DenialReason::RateLimited { ... }))`.
     b. Check dedup: `self.dedup_checker.has_replied_to(tweet_id)`. If `true`, return `Ok(Err(DenialReason::AlreadyReplied { tweet_id }))`.
     c. If `proposed_reply` is `Some`, check phrasing: `self.dedup_checker.is_phrasing_similar(proposed_reply, 20)`. If `true`, return `Ok(Err(DenialReason::SimilarPhrasing))`.
     d. All checks passed: return `Ok(Ok(()))`.
  4. Implement `pub async fn can_post_tweet(&self) -> Result<Result<(), DenialReason>, StorageError>`:
     a. Check rate limit: `self.rate_limiter.can_tweet()`.
     b. If `false`, return denial with rate limit info.
     c. If `true`, return `Ok(Ok(()))`.
  5. Implement `pub async fn can_post_thread(&self) -> Result<Result<(), DenialReason>, StorageError>`:
     a. Check rate limit: `self.rate_limiter.can_thread()`.
     b. If `false`, return denial with rate limit info.
     c. If `true`, return `Ok(Ok(()))`.
  6. Implement recording methods that delegate to the rate limiter:
     - `pub async fn record_reply(&self) -> Result<(), StorageError>`
     - `pub async fn record_tweet(&self) -> Result<(), StorageError>`
     - `pub async fn record_thread(&self) -> Result<(), StorageError>`
  7. Expose the rate limiter and dedup checker for direct access if needed:
     - `pub fn rate_limiter(&self) -> &RateLimiter`
     - `pub fn dedup_checker(&self) -> &DedupChecker`
- **Files**:
  - `crates/replyguy-core/src/safety/mod.rs`
- **Parallel?**: No -- depends on T014 (RateLimiter) and T015 (DedupChecker).
- **Notes**: The return type `Result<Result<(), DenialReason>, StorageError>` separates infrastructure errors (outer `Result` -- database failure) from business logic denials (inner `Result` -- rate limited or duplicate). Callers handle them differently: infrastructure errors trigger retries or error logging; denial reasons trigger graceful skipping with an info log. The `SafetyGuard` is the primary interface for automation loops. They should NOT call `RateLimiter` or `DedupChecker` directly in most cases. The `DenialReason` enum can be extended later (e.g., `AccountRestricted`, `MaintenanceMode`) without changing the API shape. Consider logging denial reasons at `tracing::debug!` level inside the SafetyGuard methods.

## Test Strategy

- **Rate limit enforcement tests**:
  - Initialize rate limits with `max_requests = 3`. Perform 3 checks (all return true). Perform 4th check (returns false).
  - After period expiry (mock time or use a very short period), verify counter resets and checks pass again.
- **Period reset tests**:
  - Insert a rate limit row with `period_start` set to 25 hours ago and `period_seconds = 86400`. Call `check_rate_limit`. Verify the counter is reset to 0 and `period_start` is updated.
- **Dedup exact match tests**:
  - Insert a reply to tweet "123". Call `has_replied_to("123")`. Verify returns `true`. Call `has_replied_to("456")`. Verify returns `false`.
- **Phrasing similarity tests**:
  - Insert a reply with content "This is a great tool for developers". Call `is_phrasing_similar("This is a great tool for developers", 20)`. Verify returns `true` (exact match).
  - Call with "This is a fantastic tool for devs". Verify returns `true` (high overlap).
  - Call with "I love cooking pasta with fresh basil". Verify returns `false` (no overlap).
  - Test empty strings and single-word replies.
- **Cleanup tests**:
  - Insert records with various timestamps. Run cleanup with `retention_days = 90`. Verify:
    - Unreplied discovered tweets older than 7 days are deleted.
    - Replied discovered tweets older than 90 days are deleted.
    - Replied discovered tweets younger than 90 days are preserved.
    - Action log entries older than 14 days are deleted.
    - Rate limit rows are NEVER deleted.
  - Test VACUUM trigger: insert and delete >1000 rows, verify VACUUM runs. Insert and delete <1000 rows, verify VACUUM does not run.
- **Safety guard integration tests**:
  - Test `can_reply_to` returns `Ok(Ok(()))` when rate limit is available and tweet is new.
  - Test `can_reply_to` returns `Ok(Err(DenialReason::AlreadyReplied))` when tweet already replied to.
  - Test `can_reply_to` returns `Ok(Err(DenialReason::RateLimited))` when daily limit is reached.
  - Test `can_reply_to` returns `Ok(Err(DenialReason::SimilarPhrasing))` when proposed reply is too similar.
- Run with: `cargo test -p replyguy-core -- safety` and `cargo test -p replyguy-core -- cleanup`.

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Race condition between rate limit check and increment | Use `acquire_posting_permit` with a transaction for critical posting actions. For the single-user CLI case, the simpler check-then-increment pattern is also acceptable since Tokio tasks are cooperatively scheduled. |
| Cleanup deleting records needed for dedup | Reply records are retained for the full retention period (90 days). Unreplied discovered tweets are cleaned after 7 days, but their absence does not affect dedup (dedup checks `replies_sent`, not `discovered_tweets`). Rate limit rows are never deleted. |
| Foreign key constraint violation during cleanup | Delete child records (replies_sent) before parent records (discovered_tweets). This ordering is critical and must be enforced in the cleanup function. |
| VACUUM locking the database during active loops | Only VACUUM after >1000 deletes, and cleanup runs infrequently (every 6 hours per research.md). The VACUUM acquires an exclusive lock but completes quickly on small databases. |
| Phrasing similarity false positives blocking valid replies | The 0.8 Jaccard threshold is high enough to only catch near-identical phrasings. Short, common replies (e.g., "Thanks!") might trigger false positives -- consider a minimum token count threshold (e.g., skip similarity check for replies under 5 words). |
| Time-based calculations incorrect across timezones | All timestamps are UTC. Use `chrono::Utc::now()` exclusively. Never use local time. SQLite `strftime` with `'now'` also uses UTC. |

## Review Guidance

- **Rate limit correctness**: Verify that `check_rate_limit` correctly resets the period when expired. Check the timestamp comparison logic carefully (ISO-8601 string comparison vs parsed datetime comparison).
- **Transaction safety**: Verify `acquire_posting_permit` uses a transaction. Verify `check_rate_limit` resets the period and checks the count within the same transaction.
- **Dedup completeness**: Confirm both exact-match and phrasing-similarity checks are implemented. Verify the Jaccard similarity implementation handles edge cases (empty strings, identical strings, single words).
- **Cleanup ordering**: Verify replies are deleted BEFORE discovered tweets. Verify rate_limits rows are never included in any DELETE statement.
- **Cleanup retention boundaries**: Verify unreplied tweets use the 7-day cutoff, replied tweets use the configurable retention_days, and action_log uses the fixed 14-day cutoff.
- **DenialReason clarity**: Verify the Display implementation produces human-readable messages. Verify the SafetyGuard returns the correct DenialReason for each failure mode.
- **Error propagation**: Verify all `StorageError` instances are properly propagated (no `unwrap()`, no swallowed errors).
- **Doc comments**: All public structs, enums, and functions have `///` doc comments.
- **Quality gates**: `cargo clippy --workspace -- -D warnings` and `cargo fmt --all --check` pass.

## Activity Log

> **CRITICAL**: Activity log entries MUST be in chronological order (oldest first, newest last).

### How to Add Activity Log Entries

**When adding an entry**:
1. Scroll to the bottom of this file (Activity Log section below "Valid lanes")
2. **APPEND the new entry at the END** (do NOT prepend or insert in middle)
3. Use exact format: `- YYYY-MM-DDTHH:MM:SSZ -- agent_id -- lane=<lane> -- <action>`
4. Timestamp MUST be current time in UTC (check with `date -u "+%Y-%m-%dT%H:%M:%SZ"`)
5. Lane MUST match the frontmatter `lane:` field exactly
6. Agent ID should identify who made the change (claude-sonnet-4-5, codex, etc.)

**Format**:
```
- YYYY-MM-DDTHH:MM:SSZ -- <agent_id> -- lane=<lane> -- <brief action description>
```

**Example (correct chronological order)**:
```
- 2026-01-12T10:00:00Z -- system -- lane=planned -- Prompt created
- 2026-01-12T10:30:00Z -- claude -- lane=doing -- Started implementation
- 2026-01-12T11:00:00Z -- codex -- lane=for_review -- Implementation complete, ready for review
- 2026-01-12T11:30:00Z -- claude -- lane=done -- Review passed, all tests passing
```

**Common mistakes (DO NOT DO THIS)**:
- Adding new entry at the top (breaks chronological order)
- Using future timestamps (causes acceptance validation to fail)
- Lane mismatch: frontmatter says `lane: "done"` but log entry says `lane=doing`
- Inserting in middle instead of appending to end

**Why this matters**: The acceptance system reads the LAST activity log entry as the current state. If entries are out of order, acceptance will fail even when the work is complete.

**Initial entry**:
- 2026-02-21T22:00:00Z -- system -- lane=planned -- Prompt created.

---

### Updating Lane Status

To change a work package's lane, either:

1. **Edit directly**: Change the `lane:` field in frontmatter AND append activity log entry (at the end)
2. **Use CLI**: `spec-kitty agent tasks move-task <WPID> --to <lane> --note "message"` (recommended)

The CLI command updates both frontmatter and activity log automatically.

**Valid lanes**: `planned`, `doing`, `for_review`, `done`

### Optional Phase Subdirectories

For large features, organize prompts under `tasks/` to keep bundles grouped while maintaining lexical ordering.
- 2026-02-22T00:40:07Z – claude-opus – shell_pid=50187 – lane=doing – Assigned agent via workflow command
- 2026-02-22T00:47:25Z – claude-opus – shell_pid=50187 – lane=for_review – Ready for review: Rate limiter with period reset, dedup checker with Jaccard similarity, data retention cleanup, SafetyGuard orchestration. 95 tests pass, clippy clean, fmt clean.
- 2026-02-22T00:48:29Z – claude-opus – shell_pid=50187 – lane=done – Merged to main, 95 tests pass
