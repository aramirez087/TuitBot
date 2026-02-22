---
work_package_id: WP09
title: Content + Thread Loops
lane: "done"
dependencies:
- WP05
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP01
base_commit: 54b47b462601a1e58d0222f08ae0a65ca3068a1d
created_at: '2026-02-22T01:32:09.855785+00:00'
subtasks: [T044, T045, T046, T047, T048, T049]
phase: Phase 2 - Extended Features
assignee: ''
agent: "claude-opus"
shell_pid: "88017"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP09 -- Content + Thread Loops

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

- Implement the content loop that generates and posts original educational tweets related to the user's niche on a configurable schedule, keeping the account active with thought-leadership content.
- Implement the thread loop that generates and posts multi-tweet educational threads (5-8 tweets) at a configurable interval, driving deeper engagement and follower growth.
- Integrate both loops with the safety module for rate limit enforcement and action logging.
- Provide dry-run support for both loops so users can preview generated content without posting.
- Expose single-shot CLI commands (`replyguy post`, `replyguy thread`) for on-demand content creation.
- **Success**: Content loop generates and posts educational tweets at the configured window interval, avoids topic repetition, and respects daily tweet limits. Thread loop generates 5-8 tweet threads, posts them as proper reply chains, handles partial failures, and respects weekly thread limits. Both loops support dry-run mode. CLI commands generate and post (or preview) content on demand.

## Context & Constraints

- **Spec references**: spec.md User Stories 4 (original content) and 5 (threads); FR-010 (post educational tweets when idle), FR-011 (generate 5-8 tweet threads at configurable interval), FR-013 (safety limits: max 4 tweets/day, max 1 thread/week), FR-014 (minimum delays with jitter).
- **Constitution**: Tokio async runtime. No `unwrap()` in production. Cross-platform.
- **Plan**: See `plan.md` for architecture. Content and thread loops feed into the posting queue (WP07). They depend on the LLM/ContentGenerator (WP05) for content generation.
- **Data model**: `OriginalTweet` (content tracking), `Thread` + `ThreadTweet` (thread tracking with partial failure support), `ActionLog` (audit trail), `RateLimitState` (limits). See `data-model.md`.
- **CLI contract**: See `contracts/cli-interface.md` for `replyguy post --dry-run --topic <TOPIC>` and `replyguy thread --dry-run --topic <TOPIC> --count <N>`.
- **Dependencies**: WP05 (LLM + ContentGenerator -- `generate_tweet`, `generate_thread`), WP07 (Runtime, LoopScheduler, CancellationToken, posting queue).
- **Priority**: These are P2/P3 features. They are not in the MVP scope but are important for complete autonomous operation.
- **Key difference from WP08**: Content/thread loops generate original content rather than replying to existing tweets. Thread posting is sequential (not through the posting queue) because reply chain order matters.

## Subtasks & Detailed Guidance

### Subtask T044 -- Content loop

- **Purpose**: Keep the user's X account active with original educational tweets when no recent content has been posted. Consistent posting establishes the founder as a thought leader and maintains audience engagement between discovery/mention reply cycles.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/content_loop.rs`.
  2. Define `ContentLoop` struct with fields: `content_generator: Arc<ContentGenerator>`, `safety_guard: Arc<SafetyGuard>`, `pool: Pool<Sqlite>`, `posting_queue_tx: mpsc::Sender<PostAction>`, `config: Arc<Config>`.
  3. Implement async method `run(self, cancel: CancellationToken, scheduler: LoopScheduler, dry_run: bool) -> Result<()>`:
     a. Maintain a `recent_topics: Vec<String>` in memory to track the last N topics used (where N = number of `industry_topics` / 2, minimum 3). This prevents repetition.
     b. Enter loop:
        - Check `cancel.is_cancelled()` -- break if true.
        - Query the database for the timestamp of the most recent `original_tweets` record with `status = 'sent'`.
        - If elapsed time since last tweet < `config.intervals.content_post_window_seconds`, skip this iteration (not time yet).
        - Check `safety_guard.can_post_tweet()` -- if daily tweet limit reached, log and skip.
        - Pick a random topic from `config.business.industry_topics` that is NOT in `recent_topics`. If all topics are recent, clear the list and pick any.
        - Generate tweet via `content_generator.generate_tweet(topic)`.
        - Validate tweet length <= 280 characters. If too long, retry once with an explicit instruction to be shorter. If still too long, truncate at the last word boundary before 280 chars and log a warning.
        - If `dry_run`: print `"DRY RUN: Would post tweet on topic '{topic}': \"{content}\""` and skip posting/recording.
        - If not dry_run: send `PostAction::Tweet { content }` to posting queue. Record in `original_tweets` table with topic, content, llm_provider, status = 'sent'. Log action to `action_log`.
        - Add topic to `recent_topics` (push, and pop oldest if at capacity).
        - Handle errors (log and continue, never crash).
        - Await `scheduler.tick()`.
- **Files**: `crates/replyguy-core/src/automation/content_loop.rs`
- **Parallel?**: Can be developed in parallel with T045 after understanding the shared patterns.
- **Notes**: The `content_post_window_seconds` default is 14400 (4 hours). The content loop's scheduler interval should be shorter than the window (e.g., check every 30 minutes) so the loop can react promptly when the window elapses. The 280-character limit is strict for X -- validate before posting. The topic randomization with anti-repetition keeps content diverse.

### Subtask T045 -- Thread loop

- **Purpose**: Generate and post multi-tweet educational threads that drive significantly more engagement and follower growth than individual tweets. Threads showcase expertise and provide in-depth value on industry topics.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/thread_loop.rs`.
  2. Define `ThreadLoop` struct with fields: `content_generator: Arc<ContentGenerator>`, `safety_guard: Arc<SafetyGuard>`, `x_api_client: Arc<dyn XApiClient>`, `pool: Pool<Sqlite>`, `config: Arc<Config>`.
  3. Note: The thread loop does NOT use the posting queue. Thread tweets must be posted sequentially to maintain reply chain order, so the thread loop posts directly via `XApiClient`.
  4. Implement async method `run(self, cancel: CancellationToken, scheduler: LoopScheduler, dry_run: bool) -> Result<()>`:
     a. Enter loop:
        - Check `cancel.is_cancelled()` -- break if true.
        - Query the database for the timestamp of the most recent `threads` record with `status = 'sent'`.
        - If elapsed time since last thread < `config.intervals.thread_interval_seconds`, skip this iteration.
        - Check `safety_guard.can_post_thread()` -- if weekly thread limit reached, log and skip.
        - Pick a topic from `config.business.industry_topics` (avoid repeating recent topics, same pattern as content loop).
        - Generate thread via `content_generator.generate_thread(topic)` -- returns `Vec<String>` with 5-8 tweets.
        - Validate each tweet in the thread is <= 280 characters. If any exceed, regenerate with explicit instruction. Max 3 regeneration attempts before logging error and skipping.
        - If `dry_run`: print the full thread with numbered tweets and exit iteration:
          ```
          DRY RUN: Would post thread on topic '{topic}' ({N} tweets):
            1/N: "{tweet_1}"
            2/N: "{tweet_2}"
            ...
          ```
        - If not dry_run, post as reply chain:
          a. Create a `Thread` record in DB with `topic`, `tweet_count`, `status = 'pending'`.
          b. Post the first tweet via `x_api_client.post_tweet(tweets[0])`. Capture the returned tweet ID as `root_tweet_id`. Update the Thread record with `root_tweet_id`.
          c. Record `ThreadTweet` with `position = 0`, `tweet_id`, `content`.
          d. For each subsequent tweet (index 1..N):
             - Post via `x_api_client.reply_to_tweet(previous_tweet_id, tweets[i])`. Capture the returned tweet ID.
             - Record `ThreadTweet` with `position = i`, `tweet_id`, `content`.
             - Add a small delay between posts (1-3 seconds) to avoid rapid-fire posting.
          e. If all tweets posted successfully: update Thread `status = 'sent'`, `tweet_count = N`.
          f. If posting fails mid-thread: update Thread `status = 'partial'`, `tweet_count = number_posted`. Log error with details of which tweet failed.
        - Log action to `action_log`.
        - Handle errors (log and continue, never crash).
        - Await `scheduler.tick()`.
- **Files**: `crates/replyguy-core/src/automation/thread_loop.rs`
- **Parallel?**: Can be developed in parallel with T044 after understanding shared patterns.
- **Notes**: Thread posting is inherently sequential -- each tweet must reply to the previous one. This is why threads bypass the posting queue. Partial failure is a real concern: if tweet 4/7 fails, tweets 1-3 are already posted and cannot be retracted. Record the partial state accurately. Do NOT retry a partially-posted thread automatically (it would create duplicate tweets). The `thread_interval_seconds` default is 604800 (7 days).

### Subtask T046 -- Loop safety integration

- **Purpose**: Ensure both the content and thread loops consistently check safety guards and log all actions, following the same patterns established in WP08 T040.
- **Steps**:
  1. Content loop safety:
     - Call `safety_guard.can_post_tweet()` before generating content. This checks the daily tweet rate limit (`max_tweets_per_day`, default 4).
     - Each posted tweet counts against the daily tweet limit.
     - Log all actions to `action_log`: `action_type = "tweet"` for content loop, with `status = "success"`, `"skipped"`, or `"failure"`.
  2. Thread loop safety:
     - Call `safety_guard.can_post_thread()` before generating a thread. This checks the weekly thread limit (`max_threads_per_week`, default 1).
     - Each posted thread counts as one unit against the weekly thread limit (regardless of tweet count within the thread).
     - Log all actions to `action_log`: `action_type = "thread"`, with `status = "success"`, `"partial"`, `"skipped"`, or `"failure"`.
  3. Error handling follows the same patterns as WP08 T040:
     - `LlmError`: skip current iteration, log error, continue.
     - `XApiError::RateLimited`: exponential backoff starting at 60 seconds.
     - `XApiError::AuthExpired`: attempt refresh once, pause loop if refresh fails.
     - Max consecutive error counter (default 10) with extended backoff.
  4. Share error handling utilities with WP08 via a common helper module (e.g., `automation/loop_helpers.rs` if created in WP08 T040).
- **Files**: `crates/replyguy-core/src/automation/content_loop.rs`, `crates/replyguy-core/src/automation/thread_loop.rs`
- **Parallel?**: No -- should be implemented after T044 and T045 establish the loop structure.
- **Notes**: The content loop's tweet limit is shared with other posting actions. If the posting queue has already consumed the daily tweet quota with replies, the content loop should respect that. The safety guard should track combined posting activity.

### Subtask T047 -- Dry-run support

- **Purpose**: Allow users to preview generated content (tweets and threads) without posting to X. Essential for reviewing content quality and verifying the generation pipeline before going live.
- **Steps**:
  1. Both `ContentLoop::run()` and `ThreadLoop::run()` accept a `dry_run: bool` parameter (already specified in T044 and T045).
  2. When `dry_run == true`:
     - Content loop: generate the tweet via LLM, print it, but do not post or record in `original_tweets`. Do record in `action_log` with `status = "dry_run"`.
     - Thread loop: generate the full thread via LLM, print all tweets numbered with thread structure, but do not post or record in `threads`/`thread_tweets`. Do record in `action_log` with `status = "dry_run"`.
  3. Dry-run thread display format:
     ```
     DRY RUN: Would post thread on topic 'SwiftUI' (6 tweets):
       1/6: "SwiftUI has transformed how we build macOS apps..."
       2/6: "The declarative syntax means less code, fewer bugs..."
       3/6: "One pattern I love is using @EnvironmentObject..."
       4/6: "For complex layouts, GeometryReader is your friend..."
       5/6: "Testing SwiftUI views is surprisingly straightforward..."
       6/6: "If you're building for macOS, give SwiftUI a serious look..."
     ```
  4. Use `info!` level for dry-run output so it appears even without `--verbose`.
- **Files**: `crates/replyguy-core/src/automation/content_loop.rs`, `crates/replyguy-core/src/automation/thread_loop.rs`
- **Parallel?**: Yes -- can be developed alongside T048 and T049.
- **Notes**: LLM calls still execute in dry-run mode (and consume API credits). Document this in the CLI help text. Dry-run mode is especially useful for threads since they are expensive to post (5-8 API calls) and cannot be easily retracted.

### Subtask T048 -- CLI `replyguy post` command

- **Purpose**: Provide a single-shot CLI command for generating and posting one educational tweet on demand, useful for manual content creation and testing the content generation pipeline.
- **Steps**:
  1. Create `crates/replyguy-cli/src/commands/post.rs`.
  2. Define the Clap subcommand struct:
     ```rust
     #[derive(clap::Args)]
     pub struct PostArgs {
         /// Generate tweet without posting
         #[arg(long)]
         dry_run: bool,
         /// Override topic (default: random from industry_topics)
         #[arg(long)]
         topic: Option<String>,
     }
     ```
  3. Implement the `execute` function:
     a. Load config (layered).
     b. Initialize database pool.
     c. Load/refresh OAuth tokens (skip if dry_run and tokens not available -- degrade gracefully).
     d. Initialize dependencies: `LlmProvider`, `ContentGenerator`, `SafetyGuard`.
     e. Determine topic: use `--topic` if provided, otherwise pick random from `config.business.industry_topics`.
     f. Generate tweet via `content_generator.generate_tweet(topic)`.
     g. Validate length <= 280 characters (retry once if too long).
     h. Display the generated tweet with topic and character count.
     i. If not dry_run: post via `XApiClient`, record in DB, display the posted tweet URL.
     j. If dry_run: display "DRY RUN: Tweet not posted."
     k. Exit after completion.
  4. Register the subcommand in `crates/replyguy-cli/src/commands/mod.rs` and `main.rs`.
- **Files**: `crates/replyguy-cli/src/commands/post.rs`, `crates/replyguy-cli/src/commands/mod.rs`, `crates/replyguy-cli/src/main.rs`
- **Parallel?**: Yes -- can be developed alongside T047 and T049.
- **Notes**: The `--topic` flag allows users to test specific topics. If the topic is not in `industry_topics`, that is fine -- the LLM can generate content on any topic. Display the character count so users can see how close to the 280 limit the generated content is.

### Subtask T049 -- CLI `replyguy thread` command

- **Purpose**: Provide a single-shot CLI command for generating and posting one educational thread on demand, useful for manual thread creation and testing the thread generation pipeline.
- **Steps**:
  1. Create `crates/replyguy-cli/src/commands/thread.rs`.
  2. Define the Clap subcommand struct:
     ```rust
     #[derive(clap::Args)]
     pub struct ThreadArgs {
         /// Generate thread without posting
         #[arg(long)]
         dry_run: bool,
         /// Override topic (default: random from industry_topics)
         #[arg(long)]
         topic: Option<String>,
         /// Number of tweets in thread (default: auto 5-8)
         #[arg(long)]
         count: Option<usize>,
     }
     ```
  3. Implement the `execute` function:
     a. Load config (layered).
     b. Initialize database pool.
     c. Load/refresh OAuth tokens (skip if dry_run and tokens not available).
     d. Initialize dependencies: `XApiClient`, `LlmProvider`, `ContentGenerator`, `SafetyGuard`.
     e. Determine topic: use `--topic` if provided, otherwise pick random.
     f. Determine count: use `--count` if provided (clamp to 2-15 range), otherwise let the LLM decide (prompt says 5-8).
     g. Generate thread via `content_generator.generate_thread(topic, count)`.
     h. Validate each tweet <= 280 characters.
     i. Display all tweets numbered with character counts.
     j. If not dry_run: post as reply chain (same logic as thread loop in T045), display thread URL (URL of first tweet).
     k. If dry_run: display "DRY RUN: Thread not posted."
     l. Exit after completion.
  4. Register the subcommand in `crates/replyguy-cli/src/commands/mod.rs` and `main.rs`.
- **Files**: `crates/replyguy-cli/src/commands/thread.rs`, `crates/replyguy-cli/src/commands/mod.rs`, `crates/replyguy-cli/src/main.rs`
- **Parallel?**: Yes -- can be developed alongside T047 and T048.
- **Notes**: The `--count` flag overrides the default 5-8 range. Clamp to a reasonable range (2-15) to prevent abuse. Thread posting logic should be shared with the thread loop (T045) -- extract into a reusable function. Display character counts per tweet so users can see margin.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Thread posting partial failure | Orphaned tweets on X that cannot be retracted | Record partial state with `status = 'partial'` and exact tweet count. Do NOT retry automatically. Log which tweets succeeded and which failed. User can manually clean up. |
| Generated content exceeds 280 characters | Post fails with API error | Validate length before posting. Retry generation once with explicit "keep under 280 characters" instruction. If still too long, truncate at word boundary. Max 3 retries for threads. |
| Topic repetition | Content feels stale and robotic | Track last N topics in memory. Exclude recent topics from random selection. Clear the list when all topics have been used. |
| Thread reply chain order broken | Thread appears as disconnected tweets | Post tweets sequentially, capturing each tweet ID for the next reply. Do not parallelize thread posting. Add small delays (1-3s) between posts. |
| Daily tweet limit shared with replies | Content loop starved by heavy reply activity | Check `can_post_tweet()` which accounts for all posting activity. Log when content is skipped due to limit. Consider reserving a portion of the daily limit for original content. |
| LLM generates off-topic content | Content does not match user's niche | Include business profile context in the generation prompt. Validate that generated content mentions relevant terms. Users can review via `--dry-run` before enabling. |

## Review Guidance

- Verify that thread posting is sequential and does NOT use the posting queue.
- Confirm each tweet in a thread replies to the previous tweet's ID, not the root tweet.
- Check that partial thread failure is handled correctly: Thread status = 'partial', ThreadTweets recorded only for successfully posted tweets.
- Ensure the content loop checks elapsed time since last original tweet before generating new content.
- Verify topic anti-repetition logic works correctly (especially edge cases: only 1 topic configured, all topics recently used).
- Confirm both loops respect their respective safety limits (`max_tweets_per_day`, `max_threads_per_week`).
- Check that dry-run mode generates content (LLM calls execute) but does not post or record to DB (except action_log with `status = "dry_run"`).
- Ensure CLI commands (`post`, `thread`) work correctly in both regular and dry-run modes.
- Verify the `--count` flag on `replyguy thread` is clamped to a reasonable range.
- Check that generated tweet content is validated for length before any post attempt.

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
- 2026-02-22T01:32:09Z – claude-opus – shell_pid=88017 – lane=doing – Assigned agent via workflow command
- 2026-02-22T01:38:32Z – claude-opus – shell_pid=88017 – lane=for_review – Ready for review: Content loop (tweet generation, topic anti-repetition, 280-char validation) + Thread loop (reply chain posting, partial failure handling, dry-run preview). 54 tests pass, clippy clean, fmt clean.
- 2026-02-22T01:40:59Z – claude-opus – shell_pid=88017 – lane=done – Merged to main. 307 tests pass, clippy clean, fmt clean.
