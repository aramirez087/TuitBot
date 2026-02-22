---
work_package_id: WP02
title: SQLite Storage Layer
lane: "done"
dependencies:
- WP01
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP01
base_commit: 54b47b462601a1e58d0222f08ae0a65ca3068a1d
created_at: '2026-02-22T00:34:49.829739+00:00'
subtasks:
- T007
- T008
- T009
- T010
- T011
- T012
phase: Phase 0 - Foundation
assignee: ''
agent: "claude-opus"
shell_pid: "45659"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP02 -- SQLite Storage Layer

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

- **Database initializes**: `init_db()` creates the SQLite file (if missing), runs all migrations, and returns a healthy connection pool.
- **Schema is correct**: All tables match `data-model.md` exactly -- column names, types, constraints, foreign keys, indexes, and defaults.
- **CRUD operations work**: Every entity (discovered tweets, replies, original tweets, threads, thread tweets, action log) has working insert, query, and update functions.
- **Compile-time verification**: `cargo build --workspace` succeeds with the migration embedded via `sqlx::migrate!()`.
- **WAL mode active**: The database uses Write-Ahead Logging with `synchronous = NORMAL` for concurrent read/write performance.
- **All operations return typed errors**: Every storage function returns `Result<T, StorageError>` using the error types from WP01.

## Context & Constraints

- **Data model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- definitive schema for all tables, columns, types, constraints, indexes, and retention rules.
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 3 (SQLite Storage with SQLx): connection configuration, data type decisions, cleanup strategy.
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- Project Structure (storage module layout), Architecture (module dependencies).
- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-012 (persist all state), FR-029 (retention cleanup), FR-030 (never delete rate limits or dedup records).
- **Constitution**: `.kittify/memory/constitution.md` -- No `unwrap()`, `///` doc comments on public APIs, `cargo clippy -D warnings`.
- **Dependency on WP01**: Requires `StorageError` from `error.rs` and `StorageConfig` from `config/mod.rs`.
- **Data type conventions**: Tweet IDs are `TEXT` (exceed i64 range). All timestamps are `TEXT` in ISO-8601 UTC format. Booleans use `INTEGER` (0/1). Metadata fields use `TEXT` with serialized JSON.
- **SQLx version**: 0.8 with features `sqlite`, `runtime-tokio`, `migrate`.

## Subtasks & Detailed Guidance

### Subtask T007 -- Database Initialization

- **Purpose**: Create the database initialization function that configures the connection pool with optimal settings for a background daemon (WAL mode, connection pooling, foreign keys) and runs embedded migrations.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/mod.rs`.
  2. Import and re-export the pool type: `pub type DbPool = sqlx::SqlitePool;`.
  3. Implement `pub async fn init_db(db_path: &str) -> Result<DbPool, StorageError>`:
     a. Expand `~` in `db_path` to the user's home directory.
     b. Create parent directories if they do not exist (`std::fs::create_dir_all`).
     c. Build `SqliteConnectOptions`:
        ```rust
        SqliteConnectOptions::new()
            .filename(expanded_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5))
            .optimize_on_close(true, None)
            .foreign_keys(true)
        ```
     d. Build `SqlitePoolOptions`:
        ```rust
        SqlitePoolOptions::new()
            .max_connections(4)
            .min_connections(1)
            .idle_timeout(Duration::from_secs(300))
            .connect_with(connect_options)
            .await
            .map_err(|e| StorageError::Connection { source: e })?
        ```
     e. Run migrations:
        ```rust
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .map_err(|e| StorageError::Migration { source: e })?;
        ```
     f. Return the pool.
  4. Add `pub mod storage;` to `lib.rs`.
  5. Create `build.rs` at the `replyguy-core` crate root:
     ```rust
     fn main() {
         println!("cargo:rerun-if-changed=../../migrations");
     }
     ```
     This ensures the crate recompiles when migration files change.
- **Files**:
  - `crates/replyguy-core/src/storage/mod.rs`
  - `crates/replyguy-core/src/lib.rs` (add module declaration)
  - `crates/replyguy-core/build.rs`
- **Parallel?**: No -- must complete before T009-T012 (all CRUD modules depend on the pool and migration).
- **Notes**: The `sqlx::migrate!()` macro path is relative to the crate root, not the workspace root. Since the crate is at `crates/replyguy-core/` and migrations are at the workspace root `migrations/`, the path is `"../../migrations"`. Verify this path resolves correctly during `cargo build`. The `optimize_on_close` call runs `PRAGMA optimize` when the last connection closes, improving query planner statistics. Foreign keys must be explicitly enabled in SQLite (they are off by default).

### Subtask T008 -- Initial Migration Schema

- **Purpose**: Define the database schema as a SQLx migration file. This file will be compiled into the binary via `sqlx::migrate!()` and executed automatically on first run.
- **Steps**:
  1. Create `migrations/20260221000000_initial_schema.sql` at the workspace root.
  2. Define the `discovered_tweets` table:
     ```sql
     CREATE TABLE IF NOT EXISTS discovered_tweets (
         id TEXT PRIMARY KEY,
         author_id TEXT NOT NULL,
         author_username TEXT NOT NULL,
         content TEXT NOT NULL,
         like_count INTEGER NOT NULL DEFAULT 0,
         retweet_count INTEGER NOT NULL DEFAULT 0,
         reply_count INTEGER NOT NULL DEFAULT 0,
         impression_count INTEGER DEFAULT 0,
         relevance_score REAL,
         matched_keyword TEXT,
         discovered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         replied_to INTEGER NOT NULL DEFAULT 0
     );
     CREATE INDEX IF NOT EXISTS idx_discovered_tweets_discovered_at ON discovered_tweets(discovered_at);
     CREATE INDEX IF NOT EXISTS idx_discovered_tweets_matched_keyword ON discovered_tweets(matched_keyword);
     CREATE INDEX IF NOT EXISTS idx_discovered_tweets_replied_score ON discovered_tweets(replied_to, relevance_score DESC);
     ```
  3. Define the `replies_sent` table:
     ```sql
     CREATE TABLE IF NOT EXISTS replies_sent (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         target_tweet_id TEXT NOT NULL,
         reply_tweet_id TEXT,
         reply_content TEXT NOT NULL,
         llm_provider TEXT,
         llm_model TEXT,
         created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         status TEXT NOT NULL DEFAULT 'sent',
         error_message TEXT
     );
     CREATE INDEX IF NOT EXISTS idx_replies_sent_created_at ON replies_sent(created_at);
     CREATE INDEX IF NOT EXISTS idx_replies_sent_target_tweet_id ON replies_sent(target_tweet_id);
     ```
  4. Define the `original_tweets` table:
     ```sql
     CREATE TABLE IF NOT EXISTS original_tweets (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         tweet_id TEXT,
         content TEXT NOT NULL,
         topic TEXT,
         llm_provider TEXT,
         created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         status TEXT NOT NULL DEFAULT 'sent',
         error_message TEXT
     );
     CREATE INDEX IF NOT EXISTS idx_original_tweets_created_at ON original_tweets(created_at);
     ```
  5. Define the `threads` table:
     ```sql
     CREATE TABLE IF NOT EXISTS threads (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         topic TEXT NOT NULL,
         tweet_count INTEGER NOT NULL DEFAULT 0,
         root_tweet_id TEXT,
         created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         status TEXT NOT NULL DEFAULT 'sent'
     );
     ```
  6. Define the `thread_tweets` table:
     ```sql
     CREATE TABLE IF NOT EXISTS thread_tweets (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         thread_id INTEGER NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
         position INTEGER NOT NULL,
         tweet_id TEXT,
         content TEXT NOT NULL,
         created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         UNIQUE(thread_id, position)
     );
     ```
  7. Define the `rate_limits` table:
     ```sql
     CREATE TABLE IF NOT EXISTS rate_limits (
         action_type TEXT PRIMARY KEY,
         request_count INTEGER NOT NULL DEFAULT 0,
         period_start TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
         max_requests INTEGER NOT NULL,
         period_seconds INTEGER NOT NULL
     );
     ```
  8. Define the `action_log` table:
     ```sql
     CREATE TABLE IF NOT EXISTS action_log (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         action_type TEXT NOT NULL,
         status TEXT NOT NULL DEFAULT 'success',
         message TEXT,
         metadata TEXT,
         created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
     );
     CREATE INDEX IF NOT EXISTS idx_action_log_created_at ON action_log(created_at);
     CREATE INDEX IF NOT EXISTS idx_action_log_type_created ON action_log(action_type, created_at);
     ```
- **Files**:
  - `migrations/20260221000000_initial_schema.sql`
- **Parallel?**: No -- must complete before T009-T012.
- **Notes**: Use `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` for idempotency (though SQLx tracks applied migrations, defensive SQL is good practice). SQLite `DEFAULT (strftime(...))` provides server-side timestamp defaults. Do NOT use `CURRENT_TIMESTAMP` as it returns datetime format without the `T` separator. The `metadata` column in `action_log` stores arbitrary JSON as TEXT. Foreign key from `replies_sent.target_tweet_id` to `discovered_tweets.id` does NOT use ON DELETE CASCADE because we want to keep reply records even if the source tweet record is cleaned up (the FK constraint will need consideration during cleanup -- see WP03). Cross-reference every column against `data-model.md` to ensure nothing is missed.

### Subtask T009 -- Discovered Tweets CRUD

- **Purpose**: Implement all database operations for discovered tweets: inserting new discoveries, querying by ID, marking as replied, and fetching unreplied tweets above a score threshold.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/tweets.rs`.
  2. Define the `DiscoveredTweet` Rust struct with `#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]`:
     ```rust
     pub struct DiscoveredTweet {
         pub id: String,
         pub author_id: String,
         pub author_username: String,
         pub content: String,
         pub like_count: i64,
         pub retweet_count: i64,
         pub reply_count: i64,
         pub impression_count: Option<i64>,
         pub relevance_score: Option<f64>,
         pub matched_keyword: Option<String>,
         pub discovered_at: String,
         pub replied_to: i64,  // 0 or 1
     }
     ```
  3. Implement the following functions (all take `&DbPool` as the first argument):
     - `pub async fn insert_discovered_tweet(pool: &DbPool, tweet: &DiscoveredTweet) -> Result<(), StorageError>`: Insert using `sqlx::query!` or `sqlx::query`. Use `INSERT OR IGNORE` to handle duplicate tweet IDs gracefully.
     - `pub async fn get_tweet_by_id(pool: &DbPool, tweet_id: &str) -> Result<Option<DiscoveredTweet>, StorageError>`: Fetch a single tweet by its X ID. Return `None` if not found.
     - `pub async fn mark_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError>`: `UPDATE discovered_tweets SET replied_to = 1 WHERE id = ?`.
     - `pub async fn get_unreplied_tweets_above_score(pool: &DbPool, threshold: f64) -> Result<Vec<DiscoveredTweet>, StorageError>`: `SELECT * FROM discovered_tweets WHERE replied_to = 0 AND relevance_score >= ? ORDER BY relevance_score DESC`.
     - `pub async fn tweet_exists(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError>`: `SELECT EXISTS(SELECT 1 FROM discovered_tweets WHERE id = ?)`. Return true/false.
  4. Add `pub mod tweets;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/tweets.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T010, T011, T012 after T007+T008 complete.
- **Notes**: Use `sqlx::query(...)` with `.bind()` for runtime queries (simpler than compile-time checked `sqlx::query!` which requires `DATABASE_URL`). Map all `sqlx::Error` to `StorageError::Query { source: e }`. The `replied_to` field uses `i64` in Rust to match SQLite's `INTEGER` type. Use `sqlx::query_as::<_, DiscoveredTweet>(...)` for queries that return rows.

### Subtask T010 -- Replies Sent CRUD

- **Purpose**: Implement database operations for tracking replies the agent has sent, enabling deduplication checks and daily count enforcement.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/replies.rs`.
  2. Define the `ReplySent` Rust struct with `#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]`:
     ```rust
     pub struct ReplySent {
         pub id: i64,
         pub target_tweet_id: String,
         pub reply_tweet_id: Option<String>,
         pub reply_content: String,
         pub llm_provider: Option<String>,
         pub llm_model: Option<String>,
         pub created_at: String,
         pub status: String,
         pub error_message: Option<String>,
     }
     ```
  3. Implement the following functions:
     - `pub async fn insert_reply(pool: &DbPool, reply: &ReplySent) -> Result<i64, StorageError>`: Insert a new reply record. Return the auto-generated `id`. Do not include `id` in the INSERT (it is AUTOINCREMENT).
     - `pub async fn get_replies_since(pool: &DbPool, since: &str) -> Result<Vec<ReplySent>, StorageError>`: Fetch all replies with `created_at >= since`. Used for daily count checks and reporting.
     - `pub async fn has_replied_to(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError>`: `SELECT EXISTS(SELECT 1 FROM replies_sent WHERE target_tweet_id = ?)`. Core dedup check.
     - `pub async fn get_recent_reply_contents(pool: &DbPool, limit: i64) -> Result<Vec<String>, StorageError>`: `SELECT reply_content FROM replies_sent ORDER BY created_at DESC LIMIT ?`. Used by the phrasing dedup checker (WP03) to avoid repetitive replies.
     - `pub async fn count_replies_today(pool: &DbPool) -> Result<i64, StorageError>`: Count replies where `created_at` is today (UTC). Use `date(created_at) = date('now')` in SQL.
  4. Add `pub mod replies;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/replies.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T009, T011, T012.
- **Notes**: The `insert_reply` function should accept a struct without the `id` field, or accept the full struct but ignore the `id` in the INSERT statement. Consider defining a separate `NewReplySent` struct for inserts if cleaner. The `count_replies_today` function is used by the rate limiter (WP03) to enforce `max_replies_per_day`. Use `date(created_at) = date('now')` for UTC-based day boundaries.

### Subtask T011 -- Original Tweets and Threads CRUD

- **Purpose**: Implement database operations for original tweets and educational threads, supporting the content and thread automation loops.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/threads.rs`.
  2. Define Rust structs:
     ```rust
     #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
     pub struct OriginalTweet {
         pub id: i64,
         pub tweet_id: Option<String>,
         pub content: String,
         pub topic: Option<String>,
         pub llm_provider: Option<String>,
         pub created_at: String,
         pub status: String,
         pub error_message: Option<String>,
     }

     #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
     pub struct Thread {
         pub id: i64,
         pub topic: String,
         pub tweet_count: i64,
         pub root_tweet_id: Option<String>,
         pub created_at: String,
         pub status: String,
     }

     #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
     pub struct ThreadTweet {
         pub id: i64,
         pub thread_id: i64,
         pub position: i64,
         pub tweet_id: Option<String>,
         pub content: String,
         pub created_at: String,
     }
     ```
  3. Implement original tweet functions:
     - `pub async fn insert_original_tweet(pool: &DbPool, tweet: &OriginalTweet) -> Result<i64, StorageError>`: Insert and return the auto-generated `id`.
     - `pub async fn get_last_original_tweet_time(pool: &DbPool) -> Result<Option<String>, StorageError>`: `SELECT created_at FROM original_tweets WHERE status = 'sent' ORDER BY created_at DESC LIMIT 1`. Used by the content loop to check if the posting window has elapsed.
     - `pub async fn count_tweets_today(pool: &DbPool) -> Result<i64, StorageError>`: Count original tweets posted today (UTC).
  4. Implement thread functions:
     - `pub async fn insert_thread(pool: &DbPool, thread: &Thread) -> Result<i64, StorageError>`: Insert thread record, return auto-generated `id`.
     - `pub async fn insert_thread_tweets(pool: &DbPool, thread_id: i64, tweets: &[ThreadTweet]) -> Result<(), StorageError>`: Insert all tweets for a thread. Use a transaction to ensure atomicity -- either all tweets insert or none do.
     - `pub async fn get_last_thread_time(pool: &DbPool) -> Result<Option<String>, StorageError>`: `SELECT created_at FROM threads WHERE status = 'sent' ORDER BY created_at DESC LIMIT 1`. Used by the thread loop to check if the interval has elapsed.
     - `pub async fn count_threads_this_week(pool: &DbPool) -> Result<i64, StorageError>`: Count threads posted in the current week (since Monday 00:00 UTC). Use `created_at >= date('now', 'weekday 0', '-6 days')` or equivalent SQLite date math.
  5. Add `pub mod threads;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/threads.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T009, T010, T012.
- **Notes**: Thread tweet insertion must be transactional. Use `pool.begin()` to start a transaction, insert all tweets, then `tx.commit()`. If any insert fails, the transaction rolls back automatically. The `count_threads_this_week` function needs careful date math -- SQLite's `weekday 0` is Sunday, so adjust for Monday-based weeks if needed. Consider using `strftime('%W', created_at) = strftime('%W', 'now')` for ISO week matching.

### Subtask T012 -- Action Log Operations

- **Purpose**: Implement the append-only action log that records every agent action for auditing, status reporting, and debugging.
- **Steps**:
  1. Create `crates/replyguy-core/src/storage/action_log.rs`.
  2. Define the `ActionLogEntry` Rust struct:
     ```rust
     #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
     pub struct ActionLogEntry {
         pub id: i64,
         pub action_type: String,
         pub status: String,
         pub message: Option<String>,
         pub metadata: Option<String>,  // JSON string
         pub created_at: String,
     }
     ```
  3. Implement the following functions:
     - `pub async fn log_action(pool: &DbPool, action_type: &str, status: &str, message: Option<&str>, metadata: Option<&str>) -> Result<(), StorageError>`: Insert a new action log entry. The `metadata` parameter is a pre-serialized JSON string (the caller is responsible for serialization). The `created_at` field uses the SQL default.
     - `pub async fn get_actions_since(pool: &DbPool, since: &str, action_type: Option<&str>) -> Result<Vec<ActionLogEntry>, StorageError>`: Fetch actions since the given ISO-8601 timestamp. If `action_type` is `Some`, filter by it; otherwise return all action types. Order by `created_at ASC`.
     - `pub async fn get_action_counts_since(pool: &DbPool, since: &str) -> Result<std::collections::HashMap<String, i64>, StorageError>`: `SELECT action_type, COUNT(*) as count FROM action_log WHERE created_at >= ? GROUP BY action_type`. Return as a HashMap for easy lookup. Used by the periodic status summary (WP07).
  4. Add `pub mod action_log;` to `storage/mod.rs`.
- **Files**:
  - `crates/replyguy-core/src/storage/action_log.rs`
  - `crates/replyguy-core/src/storage/mod.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T009, T010, T011.
- **Notes**: The action log is append-only during normal operation. Only the cleanup job (WP03, T016) deletes old entries. Valid `action_type` values: `"search"`, `"reply"`, `"tweet"`, `"thread"`, `"mention_check"`, `"cleanup"`, `"auth_refresh"`. Valid `status` values: `"success"`, `"failure"`, `"skipped"`. The `metadata` field supports arbitrary JSON for future extensibility (e.g., storing tweet IDs, error details, timing metrics). The `get_action_counts_since` function iterates over query rows to build the HashMap manually since SQLx does not return HashMap directly.

## Test Strategy

- **Database init test**: Create a temporary database in `/tmp` or using `tempfile` crate, call `init_db`, verify pool is connected, verify all tables exist via `SELECT name FROM sqlite_master WHERE type='table'`.
- **Migration test**: Verify migration runs without error on a fresh database. Run it twice to confirm idempotency (SQLx tracks applied migrations).
- **CRUD tests for each entity**: For each of T009-T012, write tests that:
  - Insert a record and verify it is retrievable.
  - Insert multiple records and verify count/listing functions.
  - Test edge cases: duplicate insert (tweet_exists), empty result sets, boundary conditions for date-based queries.
- **Thread transaction test**: Insert a thread with tweets, verify all tweets are present. Force a failure mid-insert (e.g., duplicate position) and verify the transaction rolls back (no partial data).
- **Action log aggregation test**: Insert actions of various types, call `get_action_counts_since`, verify the HashMap contains correct counts.
- Run with: `cargo test -p replyguy-core -- storage` (assuming test module names contain "storage").
- **Tip**: Use a helper function that creates an in-memory SQLite database (`sqlite::memory:`) for fast, isolated tests. Ensure migrations are run against the in-memory DB.

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Migration schema changes in later WPs break existing data | Use incremental migrations (new files, never modify existing ones). Each migration file has a unique timestamp prefix. |
| SQLx compile-time checking requires `DATABASE_URL` env var | Use runtime queries (`sqlx::query(...)` with `.bind()`) instead of compile-time checked `sqlx::query!()` macros, OR set `DATABASE_URL` in `.env` for development. Runtime queries are simpler for this project. |
| `sqlx::migrate!()` macro path resolution | The path is relative to the crate's `Cargo.toml`, not the workspace root. Since the crate is at `crates/replyguy-core/`, the path to `migrations/` at the workspace root is `"../../migrations"`. Verify during build. |
| No FK from `replies_sent` to `discovered_tweets` | Cleanup can delete discovered tweets independently. The `target_tweet_id` index supports dedup queries without referential integrity constraints. |
| Thread insert partial failure leaves inconsistent state | Use SQLx transactions (`pool.begin()` / `tx.commit()`) for all multi-row inserts. Test rollback behavior explicitly. |
| Timestamp format inconsistencies between Rust `chrono` and SQLite `strftime` | Standardize on `%Y-%m-%dT%H:%M:%SZ` format everywhere. Use `chrono::Utc::now().format(...)` in Rust and `strftime('%Y-%m-%dT%H:%M:%SZ', 'now')` in SQL defaults. |

## Review Guidance

- **Schema compliance**: Compare every table and column in the migration file against `data-model.md`. Check column names, types, constraints, defaults, foreign keys, and indexes are exactly as specified.
- **Index coverage**: Verify all indexes from `data-model.md` are present. Check that the composite index on `discovered_tweets(replied_to, relevance_score DESC)` is created correctly.
- **CRUD completeness**: Confirm every function listed in the subtask steps is implemented. Check return types match expectations (e.g., `insert` returns the new ID, `exists` returns bool).
- **Error mapping**: Verify all `sqlx::Error` instances are mapped to `StorageError::Query` or `StorageError::Connection` appropriately. No raw `unwrap()` calls.
- **Transaction usage**: Confirm `insert_thread_tweets` uses a transaction. Verify the transaction commits on success and rolls back on failure.
- **Pool configuration**: Verify `max_connections(4)`, `min_connections(1)`, `idle_timeout(300s)`, WAL mode, `synchronous(Normal)`, `busy_timeout(5s)`, `foreign_keys(true)`, `optimize_on_close(true)`.
- **Build verification**: `cargo build --workspace` succeeds. The `build.rs` file triggers recompilation when migrations change.
- **Doc comments**: All public structs and functions have `///` doc comments.
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
- 2026-02-22T00:34:49Z – claude-opus – shell_pid=45659 – lane=doing – Assigned agent via workflow command
- 2026-02-22T00:38:59Z – claude-opus – shell_pid=45659 – lane=for_review – Ready for review: SQLite storage layer with all 7 tables, CRUD for all entities, transactional thread inserts, 51 tests pass, clippy and fmt clean.
- 2026-02-22T00:39:18Z – claude-opus – shell_pid=45659 – lane=done – Merged to main. 51 tests pass.
