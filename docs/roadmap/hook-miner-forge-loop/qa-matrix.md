# QA Matrix — Hook Miner + Forge Loop Epic

**Date:** 2026-03-22
**Session:** 11 (final)

---

## Backend Tests (Rust)

### Hook Miner Extraction

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `parse_tuitbot_entries` | loopback/tests | Pass | Parses tuitbot YAML entries from frontmatter |
| `parse_no_tuitbot_key` | loopback/tests | Pass | Returns empty vec for notes without tuitbot key |
| `split_no_front_matter` | loopback/tests | Pass | Handles notes with no YAML block |
| `split_with_front_matter` | loopback/tests | Pass | Correctly separates YAML from body |
| `split_no_closing_delimiter` | loopback/tests | Pass | Tolerates unclosed frontmatter |

### Provenance & Loopback

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `write_new_file` | loopback/tests | Pass | Creates frontmatter on new file |
| `write_existing_frontmatter` | loopback/tests | Pass | Appends to existing frontmatter |
| `idempotent` | loopback/tests | Pass | Duplicate writes don't create duplicates |
| `multiple_tweets` | loopback/tests | Pass | Multiple entries in one note |

### Thread Normalization

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `thread_entry_serializes_child_tweet_ids` | loopback/tests | Pass | child_tweet_ids roundtrip through YAML |
| `thread_entry_without_child_ids_omits_field` | loopback/tests | Pass | No empty arrays in output |
| `thread_entry_roundtrip` | loopback/tests | Pass | Full thread entry survives write-read |
| `thread_entry_idempotent_by_root_tweet_id` | loopback/tests | Pass | Dedup on root tweet ID |
| `aggregate_thread_metrics_empty` | loopback/tests | Pass | Empty vec returns None |
| `aggregate_thread_metrics_weighted_score` | loopback/tests | Pass | Impression-weighted average |
| `aggregate_thread_metrics_partial_children` | loopback/tests | Pass | Root-only thread works |
| `aggregate_thread_zero_impressions` | loopback/tests | Pass | All-zero → None engagement/score |
| `aggregate_thread_mixed_zero_and_positive` | loopback/tests | Pass | Zero-impression children excluded from weighted avg |
| `aggregate_thread_single_child_only` | loopback/tests | Pass | Single child still produces valid analytics |
| `update_analytics_thread_aggregation` | loopback/tests | Pass | Aggregated thread analytics written to frontmatter |

### Forge Sync

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `update_analytics_single_tweet` | loopback/tests | Pass | Analytics fields written correctly |
| `update_analytics_preserves_other_frontmatter` | loopback/tests | Pass | Non-tuitbot YAML preserved |
| `update_analytics_entry_not_found` | loopback/tests | Pass | Missing tweet_id → EntryNotFound |
| `update_analytics_file_not_found` | loopback/tests | Pass | Missing file → FileNotFound |
| `update_analytics_idempotent` | loopback/tests | Pass | Repeated updates produce same result |
| `update_analytics_no_frontmatter` | loopback/tests | Pass | File without frontmatter → EntryNotFound |
| `recompute_summaries_best_post` | loopback/tests | Pass | Picks highest-impression entry |
| `recompute_summaries_tie_break_by_date` | loopback/tests | Pass | Latest published_at wins ties |
| `recompute_summaries_no_impressions` | loopback/tests | Pass | No impressions → performance "none" |
| `recompute_summaries_performance_tiers` | loopback/tests | Pass | high/medium/low/none tier assignment |

### Analytics Loop + Forge Integration

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `iteration_snapshots_followers` | analytics_loop | Pass | Follower count stored |
| `iteration_measures_replies` | analytics_loop | Pass | Reply metrics fetched and stored |
| `iteration_measures_tweets` | analytics_loop | Pass | Tweet metrics fetched and stored |
| `iteration_detects_follower_drop` | analytics_loop | Pass | >2% drop triggers alert |
| `iteration_with_forge_sync_enabled` | analytics_loop | Pass | forge_synced=true when sync returns data |
| `iteration_with_forge_sync_disabled` | analytics_loop | Pass | forge_synced=false when sync returns None |
| `iteration_forge_sync_failure_non_fatal` | analytics_loop | Pass | Iteration succeeds despite Forge error |

### Telemetry Route

| Test Name | Module | Pass/Fail | Notes |
|-----------|--------|-----------|-------|
| `hook_miner_prefix_accepted` | telemetry | Pass | `hook_miner.angles_shown` → 204 |
| `forge_prefix_accepted` | telemetry | Pass | `forge.sync_succeeded` → 204 |
| `mixed_namespace_batch_accepted` | telemetry | Pass | Multi-namespace batch → 204 |
| `unknown_prefix_rejected` | telemetry | Pass | `other.event` → 400 |

**Backend total: 567 tests passed, 0 failed**

---

## Frontend Tests (TypeScript/Vitest)

### Hook Miner Funnel Events

| Test Name | File | Pass/Fail | Notes |
|-----------|------|-----------|-------|
| `sanitizePathStem strips Unix dirs` | hookMinerFunnel.test | Pass | `/a/b/c.md` → `c` |
| `sanitizePathStem strips Windows dirs` | hookMinerFunnel.test | Pass | `C:\a\b.md` → `b` |
| `sanitizePathStem removes extension` | hookMinerFunnel.test | Pass | `note.md` → `note` |
| `sanitizePathStem bare stem` | hookMinerFunnel.test | Pass | `note` → `note` |
| `sanitizePathStem multiple dots` | hookMinerFunnel.test | Pass | `a.b.md` → `a.b` |
| `sanitizePathStem empty string` | hookMinerFunnel.test | Pass | `''` → `''` |
| `sanitizePathStem no extension` | hookMinerFunnel.test | Pass | `README` → `README` |
| `trackAnglesShown properties` | hookMinerFunnel.test | Pass | Correct event name + props |
| `trackAnglesShown sanitizes paths` | hookMinerFunnel.test | Pass | Full paths stripped |
| `trackAngleSelected angle_kind` | hookMinerFunnel.test | Pass | Correct event + angle_kind |
| `trackFallbackOpened reason` | hookMinerFunnel.test | Pass | Correct reason prop |
| `trackForgePromptShown local_eligible` | hookMinerFunnel.test | Pass | Correct prompt event |
| `trackForgeEnabled enabled_from` | hookMinerFunnel.test | Pass | Correct enable source |
| `trackForgeSyncSucceeded counts` | hookMinerFunnel.test | Pass | Count properties correct |
| `trackForgeSyncFailed reason+stage` | hookMinerFunnel.test | Pass | Error metadata correct |
| `privacy: no path separators` | hookMinerFunnel.test | Pass | No `/` or `\` in stems |
| `privacy: no content leak` | hookMinerFunnel.test | Pass | No forbidden keys |

### Approval Store + Forge Prompt Trigger

| Test Name | File | Pass/Fail | Notes |
|-----------|------|-----------|-------|
| `ApprovalUpdated approved triggers sync prompt` | wsevents.test | Pass | setPendingAnalyticsSyncPrompt called |
| `ApprovalUpdated rejected does NOT trigger prompt` | wsevents.test | Pass | setPendingAnalyticsSyncPrompt not called |

### Analytics Sync Prompt

| Test Name | File | Pass/Fail | Notes |
|-----------|------|-----------|-------|
| `renders prompt title` | analyticsSyncPrompt.test | Pass | "Enable Analytics Sync?" |
| `renders privacy text` | analyticsSyncPrompt.test | Pass | Local-only language |
| `calls onEnable on click` | analyticsSyncPrompt.test | Pass | Enable callback fires |
| `calls onDismiss on click` | analyticsSyncPrompt.test | Pass | Dismiss callback fires |

### Settings Store

| Test Name | File | Pass/Fail | Notes |
|-----------|------|-----------|-------|
| `setPendingAnalyticsSyncPrompt sets true` | settings.test | Pass | localStorage write |
| `dismissAnalyticsSyncPrompt persists` | settings.test | Pass | Dismissed state persists |
| `clearPendingAnalyticsSyncPrompt resets` | settings.test | Pass | Clears pending state |

**Frontend total: 998 tests passed, 0 failed**

---

## Consistency Checks

| Area | Check | Status |
|------|-------|--------|
| Event naming | `hookMinerFunnel.ts` matches `instrumentation-plan.md` | Pass |
| Settings copy | `AnalyticsSyncPrompt.svelte` matches `settings-and-copy-notes.md` | Pass |
| Config field | `analytics_sync_enabled` consistent across Rust + TS | Pass |
| Telemetry route | `ALLOWED_PREFIXES` includes `hook_miner.`, `forge.` | Pass |
| Re-export | `PerformancePercentiles` moved to storage, re-exported from sync | Pass |

---

## Coverage Summary

| Gate | Threshold | Status |
|------|-----------|--------|
| `cargo fmt --all --check` | Clean | Pass |
| `cargo clippy --workspace -- -D warnings` | No warnings | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | 567 pass | Pass |
| `npm run check` | 0 errors | Pass |
| `npx vitest run` | 998 pass | Pass |
