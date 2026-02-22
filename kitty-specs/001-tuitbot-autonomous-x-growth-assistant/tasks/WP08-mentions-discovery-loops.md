---
work_package_id: WP08
title: Mentions + Discovery Loops
lane: "done"
dependencies:
- WP04
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP01
base_commit: 54b47b462601a1e58d0222f08ae0a65ca3068a1d
created_at: '2026-02-22T01:21:16.699693+00:00'
subtasks: [T038, T039, T040, T041, T042, T043]
phase: Phase 1 - Core Features
assignee: ''
agent: "claude-opus"
shell_pid: "79992"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP08 -- Mentions + Discovery Loops

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

- Implement the mentions monitoring loop that fetches new @-mentions from X API, generates contextual replies via LLM, and posts them through the posting queue.
- Implement the tweet discovery loop that searches X using configured keywords, scores each tweet with the scoring engine, filters by threshold, generates replies for qualifying tweets, and posts them through the posting queue.
- Integrate both loops with the safety module (rate limits, dedup checks, action logging) and handle all error types gracefully (rate limits, auth expiry, LLM failures, network errors).
- Provide dry-run support for both loops so users can observe the full pipeline without any posts being made.
- Expose single-shot CLI commands (`replyguy mentions`, `replyguy discover`) for manual testing and one-off execution.
- **Success**: Mentions loop correctly identifies and replies to new mentions without duplicates. Discovery loop searches, scores, filters, and replies to qualifying tweets. Both loops respect rate limits, log all actions, handle errors without crashing, and support dry-run mode. CLI commands execute a single iteration and exit cleanly.

## Context & Constraints

- **Spec references**: spec.md User Stories 2 (discovery) and 3 (mentions); FR-004 (discover tweets by keyword), FR-005 (score 0-100), FR-006 (threshold filter), FR-007 (generate replies <= 3 sentences), FR-009 (monitor mentions), FR-014 (minimum delays with jitter), FR-015 (no duplicate replies), FR-017 (graceful error handling), FR-018 (CLI commands with `--dry-run`).
- **Constitution**: Tokio async runtime. No `unwrap()` in production. `thiserror` for library errors, `anyhow` in binary.
- **Plan**: See `plan.md` for data flow diagram. Both loops sit between the X API client and the posting queue, with the safety module as a gatekeeper.
- **Data model**: `DiscoveredTweet` (tweet storage + dedup), `ReplySent` (reply tracking), `ActionLog` (audit trail), `RateLimitState` (rate limit tracking). See `data-model.md`.
- **CLI contract**: See `contracts/cli-interface.md` for `replyguy discover --dry-run --limit <N>` and `replyguy mentions --dry-run --limit <N>`.
- **Dependencies**: WP04 (X API client -- `search_tweets`, `get_mentions`, `reply_to_tweet`), WP05 (LLM + ContentGenerator -- `generate_reply`), WP07 (Runtime, LoopScheduler, CancellationToken, posting queue).
- **This is the core engagement engine** -- it delivers the primary value proposition of ReplyGuy (autonomous tweet discovery and reply).

## Subtasks & Detailed Guidance

### Subtask T038 -- Mentions loop

- **Purpose**: Monitor the authenticated user's X mentions and automatically reply with helpful, contextual responses. This ensures the user never misses an engagement opportunity where someone is directly talking about or to them.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/mentions_loop.rs`.
  2. Define `MentionsLoop` struct with fields: `x_api_client: Arc<dyn XApiClient>`, `content_generator: Arc<ContentGenerator>`, `safety_guard: Arc<SafetyGuard>`, `pool: Pool<Sqlite>`, `posting_queue_tx: mpsc::Sender<PostAction>`, `config: Arc<Config>`.
  3. Implement async method `run(self, cancel: CancellationToken, scheduler: LoopScheduler) -> Result<()>`:
     a. Load `since_id` from persistent storage (DB table or a simple key-value row). If no stored value, start from the current time (fetch recent mentions only).
     b. Enter loop:
        - Check `cancel.is_cancelled()` -- break if true.
        - Fetch mentions since `since_id` via `x_api_client.get_mentions(since_id)`.
        - For each new mention:
          - Check `safety_guard.can_reply_to(mention.id)` -- skip if already replied or rate limited.
          - Generate reply via `content_generator.generate_reply(&mention, &mention.author_username)`.
          - Send `PostAction::Reply { tweet_id: mention.id, content: reply_text }` to posting queue.
          - Record the reply in the `replies_sent` table.
          - Log action to `action_log` with `action_type = "reply"`, `status = "success"`.
        - Update `since_id` to the highest mention ID seen.
        - Persist `since_id` to DB (survives restarts).
        - Handle errors: log and continue (do not crash the loop).
        - Await `scheduler.tick()`.
  4. Error handling per mention:
     - `XApiError::RateLimited` -- log warning, break inner loop, wait for rate limit reset.
     - `LlmError` -- log error, skip this mention, continue to next.
     - `XApiError::AuthExpired` -- attempt token refresh once; if refresh fails, pause loop (sleep for extended period or exit loop).
- **Files**: `crates/replyguy-core/src/automation/mentions_loop.rs`
- **Parallel?**: Can be developed in parallel with T039 after the shared interface patterns are established.
- **Notes**: The `since_id` must be persisted to survive process restarts. Consider storing it in the `rate_limit_state` table (as a special row with `action_type = "mention_since_id"`) or a dedicated key-value table. The mentions endpoint returns results in reverse chronological order; update `since_id` to the maximum ID across all results.

### Subtask T039 -- Discovery loop

- **Purpose**: Proactively search X for tweets related to the user's niche, score them for relevance and viral potential, and reply to high-scoring tweets with valuable content. This is the primary growth mechanism.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/discovery_loop.rs`.
  2. Define `DiscoveryLoop` struct with fields: `x_api_client: Arc<dyn XApiClient>`, `content_generator: Arc<ContentGenerator>`, `safety_guard: Arc<SafetyGuard>`, `scoring_engine: Arc<ScoringEngine>`, `pool: Pool<Sqlite>`, `posting_queue_tx: mpsc::Sender<PostAction>`, `config: Arc<Config>`.
  3. Implement async method `run(self, cancel: CancellationToken, scheduler: LoopScheduler) -> Result<()>`:
     a. Build combined keyword list from `config.business.product_keywords` and `config.business.competitor_keywords`.
     b. Track a keyword rotation index (which keyword to search next) to distribute API usage across iterations rather than searching all keywords every time.
     c. Enter loop:
        - Check `cancel.is_cancelled()` -- break if true.
        - Select the next keyword(s) to search (rotate through the list). Search 1-3 keywords per iteration depending on rate limit budget.
        - For each keyword, call `x_api_client.search_tweets(keyword, ...)`.
        - For each result tweet:
          - Check if already in `discovered_tweets` table (dedup by tweet ID). If exists, skip.
          - Insert into `discovered_tweets` table with `replied_to = 0`.
          - Score with `scoring_engine.score(&tweet)`.
          - Update `relevance_score` and `matched_keyword` in DB.
          - If score >= `config.scoring.threshold`:
            - Check `safety_guard.can_reply_to(tweet.id)` -- skip if rate limited or already replied.
            - Generate reply via `content_generator.generate_reply(&tweet, &tweet.author_username)`.
            - Send `PostAction::Reply { tweet_id: tweet.id, content: reply_text }` to posting queue.
            - Update `replied_to = 1` in `discovered_tweets`.
            - Record reply in `replies_sent` table.
            - Log action to `action_log`.
          - If score < threshold: log at `debug!` level ("Tweet {id} scored {score}, below threshold {threshold}, skipping").
        - Handle errors (see T040).
        - Await `scheduler.tick()`.
  4. Track `x-rate-limit-remaining` header from search responses. If remaining < 5, stop searching early and wait for reset.
- **Files**: `crates/replyguy-core/src/automation/discovery_loop.rs`
- **Parallel?**: Can be developed in parallel with T038 after shared patterns are established.
- **Notes**: Keyword rotation prevents hitting all keywords every cycle, which would exhaust rate limits quickly on Basic tier (which has limited search quota). A simple round-robin index stored in memory is sufficient (no need to persist -- restarting from keyword 0 is fine). Consider shuffling the keyword list at startup for variety. The discovery loop is disabled on Free tier (no search endpoint access).

### Subtask T040 -- Loop safety integration

- **Purpose**: Ensure both loops consistently check safety guards, log all actions, and handle error types uniformly. This subtask defines the shared error handling patterns rather than duplicating logic.
- **Steps**:
  1. Both loops must call `SafetyGuard` before every reply attempt:
     - `can_reply_to(tweet_id)` -- checks both rate limit and dedup.
     - If rate limited: log `warn!("Reply rate limit reached, skipping tweet {id}")` and skip.
     - If already replied: log `debug!("Already replied to tweet {id}, skipping")` and skip.
  2. Both loops must log every action to `action_log`:
     - Success: `action_type = "reply"`, `status = "success"`, `message = "Replied to tweet {id}"`.
     - Skip (safety): `action_type = "reply"`, `status = "skipped"`, `message = "Rate limited"` or `"Duplicate"`.
     - Failure: `action_type = "reply"`, `status = "failure"`, `message = "{error details}"`.
  3. Error handling patterns (shared across both loops):
     - `XApiError::RateLimited`: Check `retry-after` header if available. If not, use exponential backoff starting at 60 seconds, capped at 15 minutes. Log `warn!`.
     - `LlmError::*`: Skip the current tweet, log `error!`, continue to next tweet. Do not crash the loop.
     - `XApiError::AuthExpired`: Attempt one token refresh via `x_api_client.refresh_token()`. If refresh succeeds, retry the failed operation. If refresh fails, log `error!("Auth token expired and refresh failed, pausing loop")` and sleep for 5 minutes before retrying.
     - `StorageError`: Log `error!` and continue. Database errors should not crash the loop (best-effort logging).
  4. Implement a shared helper function or trait for common error handling logic to avoid code duplication between the two loops.
  5. Add a max consecutive error counter (default: 10). If a loop encounters 10 consecutive errors without a successful action, pause the loop for an extended backoff (e.g., 5 minutes) and log `warn!("Loop paused due to {N} consecutive errors")`. Reset counter on success.
- **Files**: `crates/replyguy-core/src/automation/mentions_loop.rs`, `crates/replyguy-core/src/automation/discovery_loop.rs`, possibly a shared `crates/replyguy-core/src/automation/loop_helpers.rs`
- **Parallel?**: No -- should be implemented after T038 and T039 establish the loop structure.
- **Notes**: The consecutive error counter is a safety net against infinite retry loops when a persistent issue (e.g., revoked API access) prevents any progress. Consider making the max consecutive error count configurable.

### Subtask T041 -- Dry-run support

- **Purpose**: Allow users to observe the full pipeline (fetch, score, generate content) without any posts being made. This is essential for testing, tuning scoring thresholds, and reviewing generated content before going live.
- **Steps**:
  1. Add a `dry_run: bool` parameter to both `MentionsLoop::run()` and `DiscoveryLoop::run()` (or to the struct itself).
  2. When `dry_run == true`:
     - Execute the full pipeline: fetch tweets/mentions, score (discovery only), generate reply content via LLM.
     - Instead of sending `PostAction` to the posting queue, print: `"DRY RUN: Would reply to tweet {id} by @{author}: \"{reply_text}\""`.
     - For discovery: also print the score: `"DRY RUN: Tweet {id} by @{author} scored {score}/100 -- Would reply: \"{reply_text}\""`.
     - Skip recording in `replies_sent` table (do not mark as replied).
     - Still record discovered tweets and scores in `discovered_tweets` table (this data is useful for tuning).
     - Still log actions to `action_log` with `status = "dry_run"` to distinguish from real actions.
  3. Use `info!` level for dry-run output so it appears even without `--verbose`.
- **Files**: `crates/replyguy-core/src/automation/mentions_loop.rs`, `crates/replyguy-core/src/automation/discovery_loop.rs`
- **Parallel?**: Yes -- can be developed alongside T042 and T043.
- **Notes**: Dry-run mode is critical for user trust. Users should be able to run in dry-run mode for hours to observe behavior before enabling real posting. The LLM calls still execute (and consume API credits), so note this in the CLI help text.

### Subtask T042 -- CLI `replyguy mentions` command

- **Purpose**: Provide a single-shot CLI command for manually checking and replying to mentions, useful for testing the mentions pipeline without running the full agent.
- **Steps**:
  1. Create `crates/replyguy-cli/src/commands/mentions.rs`.
  2. Define the Clap subcommand struct:
     ```rust
     #[derive(clap::Args)]
     pub struct MentionsArgs {
         /// Retrieve mentions and generate replies without posting
         #[arg(long)]
         dry_run: bool,
         /// Maximum mentions to process
         #[arg(long, default_value = "20")]
         limit: usize,
     }
     ```
  3. Implement the `execute` function:
     a. Load config (layered).
     b. Initialize database pool.
     c. Load/refresh OAuth tokens.
     d. Initialize dependencies: `XApiClient`, `LlmProvider`, `ContentGenerator`, `SafetyGuard`.
     e. Run one iteration of the mentions loop logic (not continuous -- no scheduler, no cancellation token).
     f. Respect the `--limit` flag to cap the number of mentions processed.
     g. Display results: for each mention processed, show the mention text, author, and the generated reply (or "Would reply..." in dry-run mode).
     h. Exit after completion with appropriate exit code.
  4. Register the subcommand in `crates/replyguy-cli/src/commands/mod.rs` and `main.rs`.
- **Files**: `crates/replyguy-cli/src/commands/mentions.rs`, `crates/replyguy-cli/src/commands/mod.rs`, `crates/replyguy-cli/src/main.rs`
- **Parallel?**: Yes -- can be developed alongside T041 and T043.
- **Notes**: This command shares logic with `MentionsLoop` but runs single-shot. Consider extracting the core iteration logic into a shared function that both the loop and the CLI command can call. The `--limit` flag is important for controlling costs (each mention triggers an LLM call).

### Subtask T043 -- CLI `replyguy discover` command

- **Purpose**: Provide a single-shot CLI command for manually running tweet discovery, scoring, and optional reply. Useful for testing the discovery pipeline and tuning scoring thresholds.
- **Steps**:
  1. Create `crates/replyguy-cli/src/commands/discover.rs`.
  2. Define the Clap subcommand struct:
     ```rust
     #[derive(clap::Args)]
     pub struct DiscoverArgs {
         /// Search and score tweets without posting replies
         #[arg(long)]
         dry_run: bool,
         /// Maximum tweets to process
         #[arg(long, default_value = "50")]
         limit: usize,
     }
     ```
  3. Implement the `execute` function:
     a. Load config (layered).
     b. Initialize database pool.
     c. Load/refresh OAuth tokens.
     d. Initialize dependencies: `XApiClient`, `LlmProvider`, `ContentGenerator`, `SafetyGuard`, `ScoringEngine`.
     e. Search all keywords (not rotating -- single-shot runs all keywords).
     f. Score all results via `ScoringEngine`.
     g. Display results sorted by score descending. For each tweet show: score, author, content snippet, keyword matched, and verdict (REPLY/SKIP based on threshold).
     h. For qualifying tweets (score >= threshold): generate reply and post (unless `--dry-run`).
     i. Respect `--limit` to cap total tweets processed across all keywords.
     j. Exit after completion.
  4. Register the subcommand in `crates/replyguy-cli/src/commands/mod.rs` and `main.rs`.
- **Files**: `crates/replyguy-cli/src/commands/discover.rs`, `crates/replyguy-cli/src/commands/mod.rs`, `crates/replyguy-cli/src/main.rs`
- **Parallel?**: Yes -- can be developed alongside T041 and T042.
- **Notes**: The discover command is the primary diagnostic tool for users to tune their keywords and scoring weights. The sorted score display helps users understand which tweets the agent would engage with and why. Consider adding a summary line at the end: "Searched {N} keywords, found {M} tweets, {K} qualifying (>= {threshold})."

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| X API rate exhaustion during discovery | Discovery loop stalls, no new tweets found | Track `x-rate-limit-remaining` header on every response. Stop searching early if remaining < 5. Rotate keywords to distribute usage. |
| LLM failures mid-loop | Missed reply opportunities | Skip the current tweet, log error, continue to next. Never crash the loop for an LLM error. |
| Infinite loop on persistent errors | CPU spin, log spam, no useful work | Max consecutive error counter (default 10). Extended backoff (5 min) after threshold reached. Reset on success. |
| `since_id` lost on crash | Re-process old mentions, potential duplicate replies | Persist `since_id` to DB after every successful fetch. Dedup check in `SafetyGuard` prevents duplicate replies even if `since_id` is stale. |
| Discovery loop on Free tier | 403 errors on search endpoint | Skip discovery loop entirely when API tier is Free. Log at startup: "Discovery loop disabled (Free tier -- search not available)." |
| Scoring threshold too low/high | Too many/few replies | Expose `replyguy discover --dry-run` for users to observe scoring and tune threshold. Log score distributions at `debug!` level. |

## Review Guidance

- Verify that `since_id` is persisted to the database and loaded on restart.
- Confirm both loops check `SafetyGuard` before every reply attempt.
- Check that every action (success, skip, failure) is logged to `action_log`.
- Ensure the discovery loop rotates keywords across iterations rather than searching all keywords every time.
- Verify dry-run mode executes the full pipeline but does not post or record replies.
- Confirm error handling follows the patterns in T040 (rate limit backoff, LLM skip, auth refresh).
- Check that CLI commands (`mentions`, `discover`) run single-shot and exit cleanly.
- Ensure the `--limit` flag is respected in both CLI commands.
- Verify the discovery loop is disabled on Free tier without errors.
- Check that the consecutive error counter prevents infinite retry loops.

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
- 2026-02-22T01:21:16Z – claude-opus – shell_pid=79992 – lane=doing – Assigned agent via workflow command
- 2026-02-22T01:30:39Z – claude-opus – shell_pid=79992 – lane=for_review – Ready for review: Mentions loop + Discovery loop with trait-based ports, keyword rotation, dedup, dry-run, consecutive error tracking. 54 tests pass, clippy clean, fmt clean.
- 2026-02-22T01:31:59Z – claude-opus – shell_pid=79992 – lane=done – Merged to main. 281 tests pass, clippy clean, fmt clean.
