# Session 10 Handoff — Instrumentation & Success Metrics

**Date:** 2026-03-22
**Session:** 10 of 11
**Status:** Complete

---

## What Changed

Typed telemetry events for Hook Miner and Forge, backend allowlist expansion, and a concrete instrumentation plan for post-release evaluation.

| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-server/src/routes/telemetry.rs` | **Modified** | Replaced single `EVENT_PREFIX` with `ALLOWED_PREFIXES` array (`backlink.`, `hook_miner.`, `forge.`). Updated validation and error message. Added 4 new tests. |
| `dashboard/src/lib/analytics/hookMinerFunnel.ts` | **Created** | 7 typed event helpers + `sanitizePathStem()` utility for the `hook_miner.*` and `forge.*` namespaces |
| `dashboard/tests/unit/hookMinerFunnel.test.ts` | **Created** | 15 tests covering event contracts, path sanitization, and privacy guarantees |
| `docs/roadmap/hook-miner-forge-loop/instrumentation-plan.md` | **Created** | Event catalog, adoption/fallback/sync metrics, privacy checklist, log query examples |
| `docs/roadmap/hook-miner-forge-loop/session-10-handoff.md` | **Created** | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D71 | `ALLOWED_PREFIXES` array instead of single `EVENT_PREFIX` | Extensible for future namespaces. One-line change to add a new feature area. |
| D72 | Separate `hookMinerFunnel.ts` (not extend `backlinkFunnel.ts`) | Hook Miner/Forge are distinct features with different event schemas. Keeps files focused and under size limits. |
| D73 | `sanitizePathStem()` strips directories and extensions | Prevents full file paths from leaking into telemetry. Tested with both Unix and Windows path separators. |
| D74 | Properties are structural metadata only | No raw note content, frontmatter values, or full paths. Error reasons are categorical strings, not stack traces. |
| D75 | Telemetry remains logging-only (no DB writes) | Matches existing `backlink.*` design. Structured logs aggregated by external tools. |
| D76 | Events use snake_case with dot-separated namespace | Matches existing `backlink.*` convention. |

---

## Quality Gates

```
cargo fmt --all && cargo fmt --all --check    Pass
RUSTFLAGS="-D warnings" cargo test --workspace Pass (6148 passed, 0 failed)
cargo clippy --workspace -- -D warnings        Pass
npm --prefix dashboard run check               Pass (0 errors, 0 warnings)
npm --prefix dashboard run test:unit:run        Pass (994 passed)
```

---

## Test Coverage Added

### Backend (Rust)

| Test | What It Validates |
|------|-------------------|
| `hook_miner_prefix_accepted` | `hook_miner.angles_shown` event returns 204 |
| `forge_prefix_accepted` | `forge.sync_succeeded` event returns 204 |
| `mixed_namespace_batch_accepted` | Batch with `backlink.*`, `hook_miner.*`, and `forge.*` events returns 204 |
| `unknown_prefix_rejected` | `other.event` returns 400 with updated error message |

### Frontend (TypeScript)

| Test | What It Validates |
|------|-------------------|
| `sanitizePathStem` (5 cases) | Unix paths, Windows paths, extensions, bare stems, multiple dots |
| `trackAnglesShown` | Correct event name and properties; path sanitization |
| `trackAngleSelected` | `angle_kind` property |
| `trackFallbackOpened` | `reason` property |
| `trackForgePromptShown` | `local_eligible` property |
| `trackForgeEnabled` | `enabled_from` property |
| `trackForgeSyncSucceeded` | Count properties |
| `trackForgeSyncFailed` | `reason` and `stage` properties |
| Privacy: path separators | No `source_path_stem` contains `/` or `\` |
| Privacy: no content leak | No event has `content`, `body`, `raw_text`, or `frontmatter` property |

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Events defined but not yet wired into UI components | Medium | Helpers are exported and tested. Wiring into Angle cards, Forge sync triggers, and approval store is integration work for Session 11 or a follow-up PR. |
| `PerformancePercentiles` computation not yet implemented | Medium | Deferred from Session 09. The `run_forge_sync()` function accepts percentiles as a parameter. Session 11 should add the DB query. |
| `run_forge_sync()` not yet called from analytics loop | Medium | Session 09 deferred this. The config field `analytics_sync_enabled` exists. Session 11 should wire the call. |
| `setPendingAnalyticsSyncPrompt()` not called from approval store | Medium | The function exists and is exported (Session 09). Session 11 should call it from the `ActionPerformed` WebSocket handler. |
| Telemetry lost if logs aren't aggregated | Low | Acceptable for v1. Structured logging is the established pattern. |

---

## Required Inputs for Session 11

Session 11 should wire everything together and close remaining integration gaps.

**Must read:**
- `crates/tuitbot-core/src/automation/analytics_loop.rs` — where to call `run_forge_sync()`
- `crates/tuitbot-core/src/automation/watchtower/loopback/sync.rs` — `run_forge_sync()` signature
- `dashboard/src/lib/stores/approval.ts` — where to call `setPendingAnalyticsSyncPrompt()` on publish success
- `dashboard/src/lib/analytics/hookMinerFunnel.ts` — event helpers to wire into UI components

**Must implement:**
1. Compute `PerformancePercentiles` from `tweet_performance` table (p50, p90, count >= 10)
2. Wire `run_forge_sync()` call after analytics loop iteration, reading `analytics_sync_enabled` from source config
3. Call `setPendingAnalyticsSyncPrompt()` from the approval store's `ActionPerformed` handler when publish succeeds for eligible content
4. Wire `trackAnglesShown`, `trackAngleSelected`, `trackFallbackOpened` into Hook Miner UI components
5. Wire `trackForgePromptShown`, `trackForgeEnabled` into AnalyticsSyncPrompt and settings toggle
6. Wire `trackForgeSyncSucceeded`, `trackForgeSyncFailed` into sync completion handlers

**Must preserve:**
- `run_forge_sync()` takes `analytics_sync_enabled` as parameter (don't change to reading from DB internally)
- All existing loopback, telemetry, and analytics tests pass
- `sanitizePathStem()` is always called before including paths in events
- Prompt state management functions are stable exports

---

## Architecture Context

```
Session 06: Forge data contract (frontmatter schema + thread contract)
        |
        v
Session 07: Thread publish normalization
        |
        v
Session 08: Forge sync engine (run_forge_sync, update_entry_analytics)
        |
        v
Session 09: Settings UI + consent prompt
        |
        v
Session 10: Instrumentation & success metrics (this session)
  |- ALLOWED_PREFIXES: backlink., hook_miner., forge.
  |- 7 typed event helpers in hookMinerFunnel.ts
  |- sanitizePathStem() privacy utility
  |- 15 frontend tests + 4 backend tests
  |- instrumentation-plan.md with metrics playbook
        |
        v
Session 11: Integration wiring
  -> Wire events into UI components
  -> Wire run_forge_sync() into analytics loop
  -> Compute PerformancePercentiles
  -> Trigger prompt from publish events
```
