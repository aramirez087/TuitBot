# Session 06 Handoff — Forge Data Contract & Frontmatter Schema

**Date:** 2026-03-22
**Session:** 06 of 11
**Status:** Complete

---

## What Changed

Documentation-only session. Three specification documents created, zero code changes.

| File | Action | Purpose |
|------|--------|---------|
| `docs/roadmap/hook-miner-forge-loop/forge-frontmatter-contract.md` | Created | Canonical spec for `tuitbot:` entry schema, note-level summary fields, write semantics, idempotency rules, and YAML examples |
| `docs/roadmap/hook-miner-forge-loop/forge-thread-contract.md` | Created | Thread-specific addendum: aggregation rules, matching by root tweet ID, lifecycle phases, edge cases |
| `docs/roadmap/hook-miner-forge-loop/session-06-handoff.md` | Created | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D37 | 8 additive optional fields on `LoopBackEntry`: `child_tweet_ids`, `impressions`, `likes`, `retweets`, `replies`, `engagement_rate`, `performance_score`, `synced_at` | Maps 1:1 to `tweet_performance` table columns. All `Option` with `skip_serializing_if` for backward compat. `PartialEq` only (no `Eq`) due to `f64` fields. |
| D38 | 4 top-level frontmatter summary keys: `tuitbot_social_performance`, `tuitbot_best_post_impressions`, `tuitbot_best_post_url`, `tuitbot_last_synced_at` | Top-level keys (not inside `tuitbot:` array) enable simple Obsidian Dataview queries. `tuitbot_` prefix avoids namespace collisions. Read/written via `TuitbotFrontMatter.other` mapping, no struct changes needed. |
| D39 | Forge sync uses match-and-update (not append). New function `update_entry_analytics()` alongside existing `write_metadata_to_file()`. | Separation of concerns: publish path is append-only and fast; sync path is match-and-update with summary recomputation. Forge never creates entries (prevents orphan analytics for non-Tuitbot tweets). |
| D40 | Thread = one entry with aggregated metrics. Root tweet ID is canonical `tweet_id`. `child_tweet_ids` array excludes root. | One entry per outcome. Users see thread performance as a single unit. Per-tweet breakdown stays in dashboard analytics. Sum for counts, impression-weighted average for scores. |
| D41 | Performance tiers are account-relative percentiles (90th = high, 50th = medium, <50th = low, no data = none). Minimum 10 historical posts required. | Absolute thresholds would be meaningless across accounts of different sizes. 10-post minimum prevents misleading tiers from small samples. |
| D42 | Summary fields always reflect the highest-impression entry. Ties broken by latest `published_at`. | Matches user intent: "how well did my best post from this note do?" not "what's the average across all posts." |

---

## Contract Summary

### Entry fields (total: 14)

```
tweet_id         (String, required)     — existing
url              (String, required)     — existing
published_at     (String, required)     — existing
type             (String, required)     — existing
status           (String?, optional)    — existing
thread_url       (String?, optional)    — existing
child_tweet_ids  (String[]?, optional)  — NEW, threads only
impressions      (Integer?, optional)   — NEW, from X API
likes            (Integer?, optional)   — NEW, from X API
retweets         (Integer?, optional)   — NEW, from X API
replies          (Integer?, optional)   — NEW, from X API
engagement_rate  (Float?, optional)     — NEW, computed
performance_score(Float?, optional)     — NEW, from tweet_performance
synced_at        (String?, optional)    — NEW, system clock
```

### Write operations

| Operation | When | Who Calls | Behavior |
|-----------|------|-----------|----------|
| `write_metadata_to_file()` | Publish | Approval poster | Append-only, analytics = null |
| `update_entry_analytics()` | Sync | Forge sync job | Match by tweet_id, update analytics, recompute summaries |

### Thread aggregation

| Metric | Method |
|--------|--------|
| Impressions, likes, retweets, replies | Sum across root + children |
| Engagement rate | `(sum_likes + sum_retweets + sum_replies) / sum_impressions × 100` |
| Performance score | Impression-weighted average of per-tweet scores |

---

## Quality Gates

No code changes — documentation-only session.

- All YAML examples are syntactically valid (verified manually).
- Entry schema is backward-compatible with current `LoopBackEntry` (all new fields optional).
- Summary fields use the existing `#[serde(flatten)] other` mapping (no struct changes needed).
- Thread contract matches `thread_tweets` table schema (tweet_id, position ordering).
- Aggregation rules are deterministic given any valid `tuitbot:` array.
- Performance tier computation is well-defined for all input ranges.

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `LoopBackEntry` loses `Eq` derive when `f64` fields are added | Low | Only `PartialEq` is needed. No current code depends on `Eq` for `LoopBackEntry`. Test comparisons use field-by-field asserts. |
| `child_tweet_ids` may not be available at publish time (approval poster doesn't chain replies yet) | Medium | Session 07 fixes thread posting. Until then, `child_tweet_ids` is populated during first Forge sync from `thread_tweets` table. |
| YAML serialization order may differ between writes | Low | `serde_yaml` with `Mapping` preserves insertion order. Existing `TuitbotFrontMatter` already handles this. |
| `tuitbot:` array cap at 20 entries could lose historical data | Low | 20 entries per note is generous. Oldest entries are removed — their data persists in the DB (`original_tweets`, `tweet_performance`). |
| Percentile computation requires sufficient historical data | Low | Default to `"none"` tier until >= 10 posts. Percentiles computed from `tweet_performance` table, not frontmatter. |
| Forge sync for threads requires `thread_tweets` lookup if `child_tweet_ids` is missing | Low | Documented as fallback path. Query is simple: `SELECT tweet_id FROM thread_tweets WHERE thread_id = ... ORDER BY position`. |

---

## Required Inputs for Session 07

Session 07 fixes thread posting in the approval poster (reply-chain posting) and normalizes thread entity provenance.

**Must read:**
- `forge-frontmatter-contract.md` (this session's contract — entry schema, write semantics)
- `forge-thread-contract.md` (this session's contract — thread aggregation, matching rules)
- `crates/tuitbot-core/src/automation/approval_poster.rs` (current publish flow — `post_tweet()` routes all thread_tweets as standalone)
- `crates/tuitbot-core/src/storage/threads.rs` (thread/thread_tweets tables)
- `current-state-audit.md` Section 7 (thread infrastructure gaps)

**Must fix:**
- Approval poster `thread_tweet` action type must post as reply to previous tweet in chain (not standalone)
- `propagate_provenance()` must copy links to `thread` entity type for thread posts
- `execute_loopback_for_provenance()` must populate `child_tweet_ids` when writing thread entries

**Must preserve:**
- Existing `write_metadata_to_file()` append-only behavior
- All 6 existing `LoopBackEntry` fields and their current semantics
- Provenance propagation to `original_tweet` entity type (additive, not replacement)
- `source_role` semantics (`primary_selection` receives Forge writeback)

---

## Architecture Context

```
Session 05: Provenance fields (angle_kind, signal_kind, signal_text, source_role)
        │
        ▼
Session 06: Forge data contract (this session)
  ├── forge-frontmatter-contract.md (entry schema + summaries)
  └── forge-thread-contract.md (thread aggregation)
        │
        ▼
Session 07: Thread posting fix + thread provenance
  → Approval poster chains reply tweets
  → Propagates provenance to thread entity type
  → Populates child_tweet_ids at publish time
        │
        ▼
Session 08: Forge sync implementation
  → update_entry_analytics() function
  → Summary recomputation logic
  → Thread aggregation from tweet_performance
```
