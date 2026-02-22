---
work_package_id: WP06
title: Scoring Engine
lane: "done"
dependencies:
- WP01
base_branch: 001-replyguy-autonomous-x-growth-assistant-WP01
base_commit: 54b47b462601a1e58d0222f08ae0a65ca3068a1d
created_at: '2026-02-22T01:07:48.586281+00:00'
subtasks: [T029, T030, T031, T032]
phase: Phase 1 - Core Features
assignee: ''
agent: "claude-opus"
shell_pid: "70426"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP06 -- Scoring Engine

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

- Implement four independent scoring signal functions (keyword relevance, follower score, recency score, engagement rate) as pure functions with configurable max values.
- Build a `ScoringEngine` that combines all signals into a total score (0-100) with a configurable threshold for the REPLY/SKIP verdict.
- Implement formatted CLI output that shows the total score, per-signal breakdown, and verdict for use by the `replyguy score` command.
- Implement the `replyguy score <tweet_id>` CLI command that fetches a tweet, scores it, and displays the formatted result.
- Scoring is purely heuristic -- no LLM calls. All weights are configurable via the `[scoring]` section of `config.toml`.
- All scoring functions return consistent, deterministic results for the same inputs.
- No `unwrap()` in any production code path.
- All public items have `///` doc comments.

---

## Context & Constraints

### Reference Documents

- **Constitution**: `.kittify/memory/constitution.md` -- Rust 1.75+, no `unwrap()`, thiserror in library, cargo clippy/fmt/audit gates.
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- project structure, module dependency graph (`scoring` depends on config and x_api::types).
- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-005 (score 0-100), FR-006 (threshold-based reply decision), User Story 7 (score command).
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 4 (scoring engine design, signal descriptions, weight ranges).
- **CLI Contract**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/contracts/cli-interface.md` -- `replyguy score <TWEET_ID>` command and output format.
- **Data Model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- `ScoringWeights` config entity, `DiscoveredTweet` entity fields.
- **Tasks**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/tasks.md` -- WP06 summary, signal weights (0-40, 0-20, 0-15, 0-25).

### Architectural Constraints

- Scoring code lives in `crates/replyguy-core/src/scoring/`.
- CLI command code lives in `crates/replyguy-cli/src/commands/score.rs`.
- Scoring signals are pure functions -- they take input data and return a `f32` score. No side effects, no I/O, no database access.
- The `ScoringEngine` depends on `ScoringConfig` from WP01 and `Tweet`/`User`/`PublicMetrics` types from WP04.
- All max values for signals are configurable. The defaults are: keyword_relevance=40, follower_count=20, recency=15, engagement_rate=25 (totaling 100).
- The threshold (default 70) determines the REPLY/SKIP verdict.

### Dependencies

- **WP01** must be complete: config types (including `ScoringConfig` with `threshold`, `keyword_relevance_max`, `follower_count_max`, `recency_max`, `engagement_rate_max`).
- **WP04** must be complete: X API types (`Tweet`, `User`, `PublicMetrics`, `UserMetrics`) and `XApiClient` trait (for the `score` CLI command to fetch tweets).

---

## Subtasks & Detailed Guidance

### Subtask T029 -- Scoring Signals

- **Purpose**: Implement four independent, pure scoring functions that each evaluate one dimension of a tweet's reply-worthiness. These are the building blocks of the scoring engine.
- **Steps**:
  1. Create `crates/replyguy-core/src/scoring/signals.rs`.
  2. Implement `keyword_relevance(tweet_text: &str, keywords: &[String], max_score: f32) -> f32`:
     - Convert `tweet_text` to lowercase for case-insensitive matching.
     - For each keyword in `keywords`, check if the lowercased tweet text contains the lowercased keyword.
     - Weight matches by specificity: multi-word keywords (containing a space) score 2x compared to single-word keywords. For example, if keywords are `["mac", "menu bar apps"]` and the tweet contains both, "mac" contributes 1 point and "menu bar apps" contributes 2 points.
     - Normalize the weighted match count to the 0-`max_score` range:
       - Compute `max_possible_weight = sum of weights for all keywords` (1 per single-word, 2 per multi-word).
       - Score = `(matched_weight / max_possible_weight) * max_score`.
       - Clamp to `0.0..=max_score`.
     - If `keywords` is empty, return 0.0.
  3. Implement `follower_score(follower_count: u64, max_score: f32) -> f32`:
     - Use a logarithmic scale to map follower count to a score:
       - 0 followers = 0.0
       - 100 followers = 25% of `max_score`
       - 1,000 followers = 50% of `max_score`
       - 10,000 followers = 75% of `max_score`
       - 100,000+ followers = `max_score`
     - Implementation: use `log10(max(follower_count, 1))` mapped to the 0-`max_score` range.
       - `score = (log10(follower_count.max(1) as f64) / 5.0) * max_score as f64` (since log10(100000) = 5.0).
       - Clamp to `0.0..=max_score`.
     - Return as `f32`.
  4. Implement `recency_score(tweet_created_at: &str, max_score: f32) -> f32`:
     - Parse `tweet_created_at` as an ISO-8601 timestamp using `chrono`.
     - Calculate the age of the tweet from the current time (`chrono::Utc::now()`).
     - Map age to score using these brackets:
       - 0-5 minutes: `max_score` (100%)
       - 5-30 minutes: 80% of `max_score`
       - 30-60 minutes: 50% of `max_score`
       - 1-6 hours: 25% of `max_score`
       - 6+ hours: 0.0
     - Use linear interpolation within each bracket for smoother scores (e.g., 15 minutes = 90% rather than a hard 80%).
     - If the timestamp fails to parse, log a warning and return 0.0 (do not panic).
  5. Implement `engagement_rate(metrics: &PublicMetrics, follower_count: u64, max_score: f32) -> f32`:
     - Calculate the engagement rate: `rate = (likes + retweets + replies) as f64 / max(follower_count, 1) as f64`.
     - Compare to a baseline engagement rate of ~1.5% (0.015).
     - Map to score:
       - Rate >= 5% (0.05): `max_score` (100%).
       - Rate 0% to 5%: linearly scale from 0 to `max_score`.
       - Implementation: `score = (rate / 0.05).min(1.0) * max_score`.
     - Clamp to `0.0..=max_score`.
  6. Add `///` doc comments on every function explaining the signal, its inputs, scale, and rationale.
- **Files**: `crates/replyguy-core/src/scoring/signals.rs`
- **Parallel?**: Yes -- this subtask has no dependencies within WP06 and can start as soon as WP04 types are available.
- **Notes**:
  - All functions are pure: same inputs always produce the same outputs (except `recency_score` which depends on current time -- consider accepting a `now: DateTime<Utc>` parameter for testability).
  - For `recency_score`, accepting `now` as a parameter instead of calling `Utc::now()` internally makes the function deterministically testable. The public API can have a convenience wrapper that passes `Utc::now()`.
  - The engagement baseline of 1.5% comes from research.md. This is the average engagement rate on X; tweets above 5% are considered high-engagement.
  - Edge case: a tweet with 0 followers should not divide by zero. Use `max(follower_count, 1)` in both `follower_score` and `engagement_rate`.
  - Edge case: `PublicMetrics` fields that are 0 should produce a 0 engagement rate, not an error.

### Subtask T030 -- ScoringEngine

- **Purpose**: Combine all four scoring signals into a unified `ScoringEngine` that produces a total score and per-signal breakdown for any tweet, with a configurable threshold for the reply decision.
- **Steps**:
  1. Create `crates/replyguy-core/src/scoring/mod.rs`.
  2. Define the `TweetScore` struct:
     ```rust
     #[derive(Debug, Clone)]
     pub struct TweetScore {
         /// Total score (0-100), clamped.
         pub total: f32,
         /// Keyword relevance signal score.
         pub keyword_relevance: f32,
         /// Author follower count signal score.
         pub follower: f32,
         /// Tweet recency signal score.
         pub recency: f32,
         /// Engagement rate signal score.
         pub engagement: f32,
         /// Whether the total score meets the configured threshold.
         pub meets_threshold: bool,
     }
     ```
  3. Define the `ScoringEngine` struct:
     ```rust
     pub struct ScoringEngine {
         config: ScoringConfig,
         keywords: Vec<String>,
     }
     ```
     - `config` comes from the `[scoring]` section of config.toml (via WP01).
     - `keywords` is the combined list of `product_keywords` and `competitor_keywords` from the `BusinessProfile`.
  4. Constructor: `new(config: ScoringConfig, keywords: Vec<String>) -> Self`.
  5. Implement `score_tweet(&self, tweet: &Tweet, author: &User) -> TweetScore`:
     - Call each signal function from T029:
       - `keyword_relevance(tweet.text, &self.keywords, self.config.keyword_relevance_max)`
       - `follower_score(author.public_metrics.followers_count, self.config.follower_count_max)`
       - `recency_score(tweet.created_at, self.config.recency_max)`
       - `engagement_rate(tweet.public_metrics, author.public_metrics.followers_count, self.config.engagement_rate_max)`
     - Compute `total = keyword_relevance + follower + recency + engagement`.
     - Clamp `total` to `0.0..=100.0`.
     - Set `meets_threshold = total >= self.config.threshold as f32`.
     - Return the `TweetScore`.
  6. Re-export `signals` module and `TweetScore` from `mod.rs`.
- **Files**: `crates/replyguy-core/src/scoring/mod.rs`
- **Parallel?**: Depends on T029 (needs signal functions).
- **Notes**:
  - The `keywords` field combines both `product_keywords` and `competitor_keywords` from the business profile. The caller is responsible for merging them before constructing the engine.
  - The total is clamped to 100 even though the max values sum to 100 by default. This protects against configuration changes where users increase individual max values beyond their intended budget.
  - The `ScoringConfig` struct must provide: `threshold` (u32), `keyword_relevance_max` (f32), `follower_count_max` (f32), `recency_max` (f32), `engagement_rate_max` (f32). These come from WP01.
  - Consider adding a `ScoringEngine::score_tweet_with_time(&self, tweet, author, now)` variant for testability, similar to the `recency_score` approach from T029.

### Subtask T031 -- Score Formatting

- **Purpose**: Implement display formatting for `TweetScore` so the CLI can show a human-readable score breakdown with per-signal details and a REPLY/SKIP verdict.
- **Steps**:
  1. Add display methods to `TweetScore` in `crates/replyguy-core/src/scoring/mod.rs` (or a dedicated `crates/replyguy-core/src/scoring/display.rs` if preferred).
  2. Implement `TweetScore::format_breakdown(&self, config: &ScoringConfig, tweet: &Tweet, author: &User, matched_keywords: &[String]) -> String`:
     - Format the output to match the CLI contract in `contracts/cli-interface.md`:
       ```
       Tweet: "{truncated_text}..." by @{author_username} ({formatted_followers} followers)
       Score: {total}/100
         Keyword relevance:  {keyword_relevance}/{keyword_relevance_max}  (matched: {matched_keywords_list})
         Author reach:       {follower}/{follower_count_max}  ({formatted_followers} followers, log scale)
         Recency:            {recency}/{recency_max}  (posted {age_description} ago)
         Engagement rate:    {engagement}/{engagement_rate_max}  ({rate}% engagement vs 1.5% baseline)
       Verdict: {REPLY|SKIP} (threshold: {threshold})
       ```
  3. Implement helper formatting functions:
     - `format_follower_count(count: u64) -> String`: format as "1.2K", "45.3K", "1.2M" etc. for readability.
     - `format_tweet_age(created_at: &str) -> String`: format as "12 minutes", "2 hours", "1 day" etc.
     - `truncate_text(text: &str, max_len: usize) -> String`: truncate tweet text for display (e.g., first 50 chars + "...").
  4. Implement `find_matched_keywords(tweet_text: &str, keywords: &[String]) -> Vec<String>`:
     - Return the subset of keywords that actually matched the tweet text (case-insensitive).
     - This is used for the display only -- the actual scoring uses the weighted count from T029.
  5. Consider implementing `std::fmt::Display` for `TweetScore` as a simpler alternative for logging contexts.
- **Files**: `crates/replyguy-core/src/scoring/mod.rs` (or `crates/replyguy-core/src/scoring/display.rs`)
- **Parallel?**: Yes -- can proceed alongside T029-T030 since it only needs the `TweetScore` struct definition.
- **Notes**:
  - The score values in the formatted output should be rendered as integers (no decimal places) for cleanliness: `35/40` not `35.2/40.0`.
  - The follower count formatting should use standard abbreviations: values < 1000 show as-is, 1K-999K with one decimal, 1M+ with one decimal.
  - The tweet text should be truncated to approximately 50 characters for the display header line, with "..." appended if truncated.
  - The engagement rate percentage should be displayed with one decimal place (e.g., "4.2%").
  - The verdict should be visually prominent. Consider uppercase "REPLY" or "SKIP".
  - The `format_breakdown` method takes extra parameters (config, tweet, author, matched_keywords) because `TweetScore` itself only stores numeric scores, not the source data.

### Subtask T032 -- CLI `replyguy score` Command

- **Purpose**: Wire up the scoring engine to a CLI command that lets users manually evaluate any tweet's score, providing a way to understand and tune the scoring weights.
- **Steps**:
  1. Create `crates/replyguy-cli/src/commands/score.rs`.
  2. Define the CLI subcommand (integrating with the Clap structure from WP01):
     ```rust
     #[derive(Parser)]
     pub struct ScoreCommand {
         /// The X tweet ID to score
         pub tweet_id: String,
     }
     ```
  3. Implement the `execute` function for `ScoreCommand`:
     - Load configuration (config.toml + env overrides + CLI overrides).
     - Initialize the `XApiClient` with the stored access token (from token manager, WP04).
     - Call `client.get_tweet(&tweet_id)` to fetch the tweet.
     - Extract the author from the response's `includes.users` (the tweet's `author_id` matches a user in includes).
     - If the author is not found in includes, attempt `client.get_me()` or handle gracefully with a warning and default `User` with 0 metrics.
     - Construct the `ScoringEngine` with scoring config and merged keywords (product_keywords + competitor_keywords).
     - Call `engine.score_tweet(&tweet, &author)` to get the `TweetScore`.
     - Find matched keywords for display.
     - Call `score.format_breakdown(...)` and print the result to stdout.
  4. Handle errors:
     - Tweet not found (404 or empty response): print "Tweet not found: {tweet_id}" to stderr, exit code 1.
     - API error (rate limited, auth expired): print descriptive error to stderr, exit code 1.
     - Auth expired specifically: suggest running `replyguy auth` to re-authenticate.
     - Config error: print config issue to stderr, exit code 1.
  5. Register the `score` subcommand in the main CLI dispatch (in `main.rs` or `commands/mod.rs`).
- **Files**: `crates/replyguy-cli/src/commands/score.rs`, `crates/replyguy-cli/src/commands/mod.rs`
- **Parallel?**: Depends on T030 (ScoringEngine), T031 (formatting), and WP04 (XApiClient). This is the final integration subtask.
- **Notes**:
  - The `replyguy score` command is a diagnostic/utility tool (User Story 7, Priority P3). It helps users understand how the scoring engine evaluates tweets and tune their weights.
  - The tweet ID is a positional argument, not a flag: `replyguy score 1234567890`.
  - The `get_tweet` call must include `expansions=author_id` and `user.fields=username,public_metrics` to get the author data in the same request.
  - If the user's tokens are expired, the command should attempt a refresh before giving up. If refresh fails, print a clear message.
  - The command does not modify any state (no database writes, no posting). It is read-only.
  - Exit code 0 for success, 1 for any error (matching the CLI contract).

---

## Test Strategy

- **Unit tests** for each scoring signal (T029):
  - `keyword_relevance`: test with 0 keywords, single-word match, multi-word match, no matches, all matches, case insensitivity.
  - `follower_score`: test with 0, 100, 1000, 10000, 100000, 1000000 followers. Verify logarithmic scale.
  - `recency_score`: test with tweets from 1 min ago, 15 min, 45 min, 3 hours, 12 hours. Use injected `now` for determinism.
  - `engagement_rate`: test with 0 engagement, average engagement (1.5%), high engagement (5%+), 0 followers edge case.
- **Unit tests** for `ScoringEngine` (T030):
  - Test that total = sum of individual signals.
  - Test that total is clamped to 0-100.
  - Test `meets_threshold` with scores above and below threshold.
  - Test with default config values.
- **Unit tests** for formatting (T031):
  - Test follower count formatting: 500 -> "500", 1200 -> "1.2K", 45300 -> "45.3K", 1200000 -> "1.2M".
  - Test tweet age formatting: 30 seconds -> "30 seconds", 12 minutes -> "12 minutes", 3 hours -> "3 hours".
  - Test text truncation at boundaries.
- **Integration tests** for the CLI score command (T032):
  - Use wiremock to mock the X API `get_tweet` endpoint.
  - Verify the command prints the expected formatted output.
  - Verify error handling for missing tweets and API errors.
- **Test commands**:
  ```bash
  cargo test -p replyguy-core --lib scoring
  cargo test -p replyguy-cli --lib commands::score
  cargo test -p replyguy-core --test integration -- scoring
  ```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Scoring accuracy depends on weight tuning | High | Medium | The `replyguy score` command lets users test individual tweets and adjust weights. Default weights come from research.md analysis. |
| Missing author data (no followers) | Medium | Low | Default to 0 followers if author data is unavailable. Log a warning. `follower_score` and `engagement_rate` handle 0 followers gracefully with `max(follower_count, 1)`. |
| Timestamp parsing failures | Low | Low | `recency_score` returns 0.0 on parse failure with a logged warning. Does not panic or propagate error. |
| Recency score depends on system clock | Low | Low | Accept `now` parameter for testability. In production, use `chrono::Utc::now()`. Time skew is unlikely to be significant for 5-minute to 6-hour brackets. |
| Config weights that do not sum to 100 | Medium | Low | Clamp total to 0-100 regardless of individual max values. Document that default weights sum to 100. |
| Tweet text in non-ASCII languages | Medium | Low | Keyword matching uses Unicode-aware lowercase (`str::to_lowercase()`). Keyword relevance works for any UTF-8 text. |

---

## Review Guidance

- Verify all four scoring signal functions are pure (no side effects, no I/O) and handle edge cases (0 values, empty inputs, parse failures).
- Verify `follower_score` uses true logarithmic scaling: 100 followers maps to ~25% of max, 1000 to ~50%, 10000 to ~75%, 100000 to ~100%.
- Verify `recency_score` brackets match the specification: 0-5 min = max, 5-30 min = 80%, 30-60 min = 50%, 1-6 hr = 25%, 6+ hr = 0.
- Verify `engagement_rate` correctly uses `likes + retweets + replies` (not impressions or bookmarks) and the 5% ceiling.
- Verify the `ScoringEngine` clamps the total score to 0-100 and correctly computes `meets_threshold`.
- Verify the formatted output matches the CLI contract exactly (field alignment, units, verdict format).
- Verify the CLI score command handles all error cases: tweet not found, API error, auth expired, config error.
- Verify no `unwrap()` calls in any production code path.
- Verify all public types and functions have `///` doc comments.
- Verify signal functions accept testability parameters (e.g., `now` for recency) or have testable wrappers.

---

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
- 2026-02-22T01:07:48Z – claude-opus – shell_pid=70426 – lane=doing – Assigned agent via workflow command
- 2026-02-22T01:13:36Z – claude-opus – shell_pid=70426 – lane=for_review – Ready for review: Scoring engine with 4 signals (keyword relevance, follower count, recency, engagement rate), ScoringEngine combiner, TweetData/TweetScore types, formatting helpers, Display impl. 74 tests pass, clippy clean, fmt clean. T032 (CLI score command) deferred to WP10 CLI integration.
- 2026-02-22T01:14:03Z – claude-opus – shell_pid=70426 – lane=done – Merged to main. 221 tests pass. Scoring engine fully implemented.
