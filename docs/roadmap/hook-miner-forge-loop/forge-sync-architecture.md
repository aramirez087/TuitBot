# Forge Sync Architecture

**Date:** 2026-03-22
**Session:** 08
**Status:** Active

---

## Overview

The Forge sync engine enriches Obsidian vault source notes with social performance analytics. It runs as an opt-in post-processing step after the analytics loop measures tweet performance, writing aggregated metrics back into the YAML frontmatter of notes that originated published content.

---

## Data Flow

```
tweet_performance table
        │
        ▼
┌─────────────────────┐
│  run_forge_sync()    │  For each tweet with measured performance:
│  (orchestrator)      │
└─────────────────────┘
        │
        ├─── get_original_tweet_id_by_tweet_id()  → original_tweets.id
        │
        ├─── get_primary_source_for_tweet()        → (relative_path, source_type, base_path)
        │    (provenance: entity_type='original_tweet', source_role='primary_selection')
        │
        ├─── Gate: source_type == "local_fs"?      → skip non-local
        │
        ├─── Is thread root?
        │    ├── Yes → get_thread_tweet_ids_by_root_for() → child IDs
        │    │         get_tweet_performances_for()       → all metrics
        │    │         aggregate_thread_metrics()          → summed counts, weighted score
        │    └── No  → use single tweet metrics directly
        │
        └─── update_entry_analytics(path, tweet_id, analytics, percentiles)
             ├── Parse frontmatter, find entry by tweet_id
             ├── Update analytics fields in-place
             ├── recompute_summaries() → tuitbot_* summary keys
             └── Write file back
```

---

## Sync Trigger Model

The sync engine is designed to be called after each analytics loop iteration. Session 09 will wire the trigger; this session provides the engine.

```
analytics_loop iteration
        │
        ▼
    tweet_performance rows upserted
        │
        ▼
    run_forge_sync(pool, account_id, enabled, percentiles)
        │
        ▼
    ForgeSyncSummary { tweets_synced, threads_synced, ... }
```

The `analytics_sync_enabled` flag is passed as a boolean parameter. When `false`, the function returns immediately with an empty summary.

---

## Fail-Open Semantics

The sync engine never fails the overall iteration due to individual note errors:

| Scenario | Behavior | Counter |
|----------|----------|---------|
| File doesn't exist on disk | Skip, no error | `files_not_found` |
| Entry not in frontmatter | Skip, no error | `entries_not_found` |
| Source type is not `local_fs` | Skip, no error | `non_local_skipped` |
| File write I/O error | Log warning, skip | `files_not_found` |
| No provenance link found | Skip silently | — |
| No `original_tweets` row found | Skip silently | — |

The `ForgeSyncSummary` provides visibility into what was skipped and why.

---

## Thread Aggregation

Thread metrics are aggregated per the rules in `forge-thread-contract.md`:

- **Count metrics** (impressions, likes, retweets, replies): summed across root + all children
- **Engagement rate**: computed from aggregated counts, not averaged
- **Performance score**: impression-weighted average across tweets with metrics

Child tweet IDs are resolved via:
1. The `child_tweet_ids` field on the existing frontmatter entry (preferred)
2. Fallback: `get_thread_tweet_ids_by_root_for()` from `thread_tweets` table

---

## Non-Local Source Handling

Only `local_fs` source types receive frontmatter writeback. This is enforced at the provenance lookup level:

- `google_drive` sources: skipped with `non_local_skipped` counter
- `cloud` sources: skipped with `non_local_skipped` counter
- Unknown source types: skipped with `non_local_skipped` counter

This matches the existing loopback path behavior and preserves the trust boundary: Forge only writes to files the user controls locally.

---

## Module Structure

```
loopback/
├── mod.rs    — LoopBackEntry, TuitbotFrontMatter, write_metadata_to_file (publish path)
├── sync.rs   — update_entry_analytics, recompute_summaries, aggregate_thread_metrics,
│               run_forge_sync (analytics sync path)
└── tests.rs  — All loopback and sync tests
```

The split from `loopback.rs` → `loopback/` satisfies the 500-line file limit while keeping the publish and sync paths colocated under the same module.

---

## Relationship to Other Sessions

| Session | Provides | This Session Uses |
|---------|----------|-------------------|
| 06 | Forge frontmatter contract | Entry schema, summary fields, write semantics |
| 06 | Forge thread contract | Aggregation rules, matching rules |
| 07 | Thread publish normalization | `child_tweet_ids`, `persist_thread_records`, `get_thread_tweet_ids_by_root_for` |
| 09 | Settings UI + config wiring | `analytics_sync_enabled` flag (this session accepts as parameter) |
