---
work_package_id: WP07
title: Automation Runtime + Scheduler
lane: "done"
dependencies: [WP03]
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP01
base_commit: 54b47b462601a1e58d0222f08ae0a65ca3068a1d
created_at: '2026-02-22T01:14:12.994435+00:00'
subtasks: [T033, T034, T035, T036, T037]
phase: Phase 1 - Core Features
assignee: ''
agent: "claude-opus"
shell_pid: "75157"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP07 -- Automation Runtime + Scheduler

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

- Build the loop scheduler with configurable interval and randomized jitter so automation loops fire on natural, non-predictable cadences.
- Implement the runtime orchestrator that spawns all automation loops as concurrent Tokio tasks and manages their lifecycle via a shared `CancellationToken`.
- Register OS signal handlers (Ctrl+C, SIGTERM) to trigger graceful shutdown, allowing in-progress work to complete before exit.
- Create a serialized posting queue so all loops funnel post actions through a single channel, preventing race conditions and ensuring rate limits are respected globally.
- Implement a periodic status summary reporter that queries the action log and prints human-readable stats at a configurable interval.
- **Success**: Scheduler fires at configured intervals with observable jitter. Runtime spawns and cleanly stops all loops. SIGTERM triggers graceful shutdown within 30 seconds. Posting queue serializes concurrent post requests. Status summary prints accurate counts when enabled.

## Context & Constraints

- **Spec references**: spec.md FR-016 (continuous agent), FR-017 (graceful error handling), FR-022 (graceful shutdown), FR-024 (status summary).
- **Constitution**: Tokio async runtime required. No `unwrap()` in production code. Cross-platform (Linux, macOS, Windows).
- **Plan**: See `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` for module dependency graph. The `automation/` module depends on all other modules.
- **Data model**: See `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` for `ActionLog` entity (used by status reporter) and `RateLimitState` (consulted by posting queue).
- **CLI contract**: See `kitty-specs/001-replyguy-autonomous-x-growth-assistant/contracts/cli-interface.md` for `replyguy run --status-interval` flag.
- **Dependencies**: WP03 (rate limiting and safety module) must be complete. The posting queue checks rate limits before posting. The runtime conditionally enables loops based on API tier (from WP04, but tier detection is a runtime concern).
- **Crate location**: All code in `crates/replyguy-core/src/automation/`.
- **Key crates**: `tokio` (spawn, sleep, signal, mpsc), `tokio_util` (CancellationToken), `rand` (jitter), `tracing` (logging).

## Subtasks & Detailed Guidance

### Subtask T033 -- Scheduler module

- **Purpose**: Provide a reusable scheduler that each automation loop uses to pace its iterations. The jitter prevents predictable patterns, making the agent's behavior appear more natural and reducing the chance of coordinated API bursts.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/scheduler.rs`.
  2. Define `LoopScheduler` struct with fields: `interval: Duration`, `min_delay: Duration`, `max_delay: Duration` (jitter range, sourced from config `limits.min_action_delay_seconds` and `limits.max_action_delay_seconds`).
  3. Implement `LoopScheduler::new(interval: Duration, min_delay: Duration, max_delay: Duration) -> Self`.
  4. Implement async method `tick(&self)` that computes `sleep_duration = interval + random_jitter` where `random_jitter` is drawn from `rand::thread_rng().gen_range(self.min_delay..=self.max_delay)` and then awaits `tokio::time::sleep(sleep_duration)`.
  5. Implement method `reset(&mut self)` to restart the interval timer (useful after errors or manual triggers).
  6. Each automation loop (mentions, discovery, content, thread) instantiates its own `LoopScheduler` with its configured interval from the `[intervals]` config section.
- **Files**: `crates/replyguy-core/src/automation/scheduler.rs`
- **Parallel?**: Yes -- can be developed independently of T034-T037.
- **Notes**: The jitter range is global (from `[limits]`), but the base interval is per-loop (from `[intervals]`). If `min_delay == max_delay`, jitter is effectively fixed. Consider logging the computed sleep duration at `debug!` level for observability.

### Subtask T034 -- Runtime orchestrator

- **Purpose**: The runtime orchestrator is the central coordinator that owns all dependencies, spawns each loop as a concurrent task, and manages lifecycle. It is the main entry point called by the `replyguy run` CLI command.
- **Steps**:
  1. Create `crates/replyguy-core/src/automation/mod.rs`.
  2. Define `Runtime` struct holding all shared dependencies: `pool: Pool<Sqlite>`, `x_api_client: Arc<dyn XApiClient>`, `llm_provider: Arc<dyn LlmProvider>`, `config: Arc<Config>`, `safety_guard: Arc<SafetyGuard>`, `scoring_engine: Arc<ScoringEngine>`, `content_generator: Arc<ContentGenerator>`.
  3. Implement `Runtime::new(...)` constructor accepting all dependencies.
  4. Implement async method `start(&mut self) -> Result<()>` that:
     a. Creates a `CancellationToken` (from `tokio_util::sync`).
     b. Creates the posting queue channel (`tokio::sync::mpsc::channel` with bounded capacity).
     c. Spawns the posting queue consumer task (T036).
     d. Spawns the status reporter task if enabled (T037).
     e. Conditionally spawns each automation loop as `tokio::spawn`, passing clones of the `CancellationToken`, dependencies, and the posting queue sender. Skip the discovery loop if the detected API tier is Free.
     f. Stores all `JoinHandle`s in a `Vec<JoinHandle<()>>`.
  5. Implement async method `stop(&self) -> Result<()>` that:
     a. Cancels the `CancellationToken`.
     b. Logs "Shutting down gracefully...".
     c. Awaits all `JoinHandle`s with a timeout of 30 seconds using `tokio::time::timeout`.
     d. If timeout exceeded, logs a warning and force-exits.
  6. Re-export submodules: `scheduler`, `mentions_loop`, `discovery_loop`, `content_loop`, `thread_loop`.
- **Files**: `crates/replyguy-core/src/automation/mod.rs`
- **Parallel?**: No -- depends on T033 (scheduler) and coordinates with T035-T037.
- **Notes**: Use `Arc` for shared dependencies since they are sent across `tokio::spawn` boundaries. The `CancellationToken` is cheap to clone. Each spawned task should log its name at startup (`info!("Mentions loop started")`).

### Subtask T035 -- Graceful shutdown

- **Purpose**: Ensure the agent responds to OS termination signals cleanly, completing in-progress work and saving state before exiting. This prevents data corruption and partial posts.
- **Steps**:
  1. In the runtime orchestrator (or a dedicated shutdown module), register signal handlers:
     a. `tokio::signal::ctrl_c()` on all platforms.
     b. On Unix: `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM.
     c. Use `tokio::select!` to await whichever signal fires first.
  2. When a signal is received:
     a. Call `cancellation_token.cancel()` to signal all loops to stop.
     b. Log `info!("Shutting down gracefully...")`.
     c. Drop the posting queue sender to signal the consumer to drain and exit.
     d. Await all `JoinHandle`s with `tokio::time::timeout(Duration::from_secs(30), ...)`.
     e. If timeout exceeded, log `warn!("Shutdown timeout exceeded, forcing exit")` and call `std::process::exit(1)`.
  3. Each automation loop must check `cancellation_token.is_cancelled()` at the top of each iteration and break out of its loop when cancelled.
  4. Ensure in-progress posting (if a tweet is mid-post) completes before the posting queue consumer exits. The consumer should drain remaining items in the channel after cancellation.
  5. Save any pending state to the database (e.g., updated `since_id` for mentions).
- **Files**: `crates/replyguy-core/src/automation/mod.rs` (integrated into Runtime)
- **Parallel?**: No -- tightly coupled with T034 (runtime orchestrator).
- **Notes**: On Windows, SIGTERM is not available; `ctrl_c()` alone is sufficient. Use `#[cfg(unix)]` for the SIGTERM handler. The 30-second timeout is a safety net; most loops should exit within a few seconds once cancelled. Test by sending SIGTERM to the process and verifying clean exit.

### Subtask T036 -- Posting queue

- **Purpose**: Serialize all posting actions through a single channel so that concurrent loops do not create race conditions when posting to X API. The queue also enforces per-post delays and handles rate limiting centrally.
- **Steps**:
  1. Define the `PostAction` enum in `crates/replyguy-core/src/automation/mod.rs` (or a dedicated `posting_queue.rs`):
     ```rust
     pub enum PostAction {
         Reply { tweet_id: String, content: String },
         Tweet { content: String },
         ThreadTweet { content: String, in_reply_to: String },
     }
     ```
  2. Create a bounded `tokio::sync::mpsc::channel::<PostAction>(100)` in the runtime orchestrator. The sender (`mpsc::Sender<PostAction>`) is cloned and given to each loop. The receiver (`mpsc::Receiver<PostAction>`) is consumed by a single consumer task.
  3. Implement the consumer task as an async function:
     a. Loop: `while let Some(action) = receiver.recv().await`.
     b. For each action, check rate limits via `SafetyGuard` before posting.
     c. If rate limited, log a warning and park the action (sleep until rate limit resets, then retry).
     d. Post to X API via `XApiClient` (call `reply_to_tweet`, `post_tweet`, etc. depending on variant).
     e. Add a configurable minimum delay between posts (from `limits.min_action_delay_seconds`) using `tokio::time::sleep`.
     f. Log the result (success or failure) to the action log.
  4. All automation loops must send `PostAction`s to this channel instead of calling the X API directly.
  5. When the channel sender is dropped (during shutdown), the consumer drains remaining items and exits.
- **Files**: `crates/replyguy-core/src/automation/mod.rs` or `crates/replyguy-core/src/automation/posting_queue.rs`
- **Parallel?**: Yes -- can be developed alongside T037.
- **Notes**: Use a bounded channel (capacity ~100) with `send().await` to apply backpressure. If the channel is full, the sending loop will pause, which is the desired behavior. Add a `tokio::time::timeout` on send (e.g., 10 seconds) to detect deadlocks. Consider including a `oneshot::Sender<Result<String>>` in each `PostAction` variant so the sender can await the result (e.g., the posted tweet ID).

### Subtask T037 -- Periodic status summary

- **Purpose**: Provide users with a periodic heartbeat showing what the agent has been doing, so they can confirm it is operating correctly without enabling verbose logging.
- **Steps**:
  1. Define `StatusReporter` struct in `crates/replyguy-core/src/automation/status_reporter.rs` (or inline in `mod.rs`) with fields: `pool: Pool<Sqlite>`, `interval: Duration`.
  2. Implement async method `run(cancel: CancellationToken, scheduler: LoopScheduler)`:
     a. Each iteration: query `action_log` for counts grouped by `action_type` since the last report timestamp.
     b. Format the summary: `"Last {interval}: {N} tweets scored, {M} replies sent, {K} tweets posted. Next cycle in {T}."`.
     c. Print using `info!` (so it appears at default log level or above).
     d. Await `scheduler.tick()`.
  3. Only spawn this task if `config.logging.status_interval_seconds > 0`. If 0, do not spawn.
  4. The status interval can be overridden via the `--status-interval` CLI flag on `replyguy run`.
  5. Use a dedicated `LoopScheduler` instance with `interval = Duration::from_secs(status_interval_seconds)`.
- **Files**: `crates/replyguy-core/src/automation/status_reporter.rs` or `crates/replyguy-core/src/automation/mod.rs`
- **Parallel?**: Yes -- can be developed alongside T036.
- **Notes**: Keep the DB queries lightweight -- a simple `SELECT action_type, COUNT(*) FROM action_log WHERE created_at > ? GROUP BY action_type` is sufficient. The "Next cycle in" part can be estimated from the scheduler's interval. If no actions occurred since last report, print "Last {interval}: No activity."

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Deadlock in posting queue | All loops stall, no posts sent | Use bounded channel (capacity ~100) with timeout on send. Monitor channel fullness at `warn!` level. |
| Missed shutdown signals | Agent continues running after user expects exit | Register signal handlers as early as possible in the startup sequence, before spawning loops. |
| Loop panic crashing entire runtime | Agent exits unexpectedly, losing state | Each `tokio::spawn` returns a `JoinHandle` -- check for `JoinError` (which wraps panics). Log the panic and continue running other loops. Optionally wrap loop bodies with `std::panic::catch_unwind`. |
| Jitter range misconfiguration | min_delay > max_delay causes panic in `gen_range` | Validate at config load time that `min_action_delay_seconds <= max_action_delay_seconds`. Swap values if inverted. |
| Posting queue consumer dies | All pending posts lost | If the consumer task exits unexpectedly, the runtime should detect it (JoinHandle completes) and either restart the consumer or initiate shutdown. |

## Review Guidance

- Verify that all spawned tasks check `CancellationToken` at the top of each loop iteration.
- Confirm the posting queue uses a bounded channel and handles backpressure gracefully.
- Check that SIGTERM handling is gated behind `#[cfg(unix)]` for cross-platform compatibility.
- Ensure the status reporter query is efficient and does not lock the database during normal operation.
- Validate that the scheduler's jitter computation cannot panic (min <= max).
- Confirm all `JoinHandle`s are collected and awaited during shutdown.
- Review that the `PostAction` enum covers all posting scenarios (reply, tweet, thread tweet).
- Check that the consumer drains the channel on shutdown rather than dropping pending actions.

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
- 2026-02-22T01:14:13Z – claude-opus – shell_pid=75157 – lane=doing – Assigned agent via workflow command
- 2026-02-22T01:20:38Z – claude-opus – shell_pid=75157 – lane=for_review – Ready for review: LoopScheduler with jitter, PostingQueue with bounded MPSC and drain-on-shutdown, StatusReporter with ActionCounts, Runtime with CancellationToken and signal handling. 62 tests pass, clippy clean, fmt clean.
- 2026-02-22T01:21:12Z – claude-opus – shell_pid=75157 – lane=done – Merged to main. 255 tests pass.
