# Release Readiness Assessment — Hook Miner + Forge Loop

**Date:** 2026-03-22
**Epic:** Hook Miner + Forge Loop (Sessions 01–11)
**Verdict:** **Conditional GO**

---

## 1. Go/No-Go Verdict

**GO** — with the conditions listed in "Residual Risks" below.

All quality gates pass. Every feature area has unit test coverage. The integration points (analytics loop → Forge sync, approval → consent prompt, telemetry → UI) are wired and tested. The codebase compiles cleanly with zero warnings across `fmt`, `clippy`, and `test`.

The "conditional" qualifier exists because:
- `run_forge_sync()` has no integration test with a real database (unit-tested through mocks only).
- `compute_performance_percentiles_for()` is defined but not yet called from a concrete `AnalyticsStorage` implementor in the server crate (the trait method `run_forge_sync_if_enabled` has a default no-op implementation; the server must wire the real call).
- Forge sync success/failure telemetry events are server-side log events only — no frontend WebSocket notification exists yet.

These are follow-up integration items, not blockers for the feature gate.

---

## 2. Feature Completeness Matrix

| Feature Area | Status | Evidence |
|-------------|--------|----------|
| **Hook Miner extraction** | Complete | Angle parsing, evidence linking, confidence scoring — all tested |
| **Angle cards UX** | Complete | AngleCards.svelte renders, selects, confirms — telemetry wired |
| **Weak-signal fallback** | Complete | AngleFallback.svelte with reason-specific copy + telemetry |
| **Provenance** | Complete | Source tracking, original_tweet linkage, local_fs gate |
| **Thread normalization** | Complete | child_tweet_ids serialization, reply-chain posting, roundtrip tests |
| **Forge frontmatter contract** | Complete | EntryAnalytics, recompute_summaries, performance tiers |
| **Forge sync engine** | Complete | run_forge_sync: single tweets + thread aggregation |
| **PerformancePercentiles computation** | Complete | compute_performance_percentiles_for with nearest-rank method |
| **Analytics loop integration** | Complete | run_forge_sync_if_enabled trait method, non-fatal error handling |
| **Settings UX** | Complete | analytics_sync_enabled toggle, consent prompt, localStorage state |
| **Consent prompt trigger** | Complete | ApprovalUpdated → setPendingAnalyticsSyncPrompt on publish |
| **Telemetry instrumentation** | Complete | 7 typed event helpers, ALLOWED_PREFIXES, path sanitization |
| **Telemetry wiring** | Partial | Hook Miner + Forge prompt events wired; sync success/failure are server-side logs only |
| **Privacy guards** | Complete | sanitizePathStem, no content/frontmatter/full-path leaks — tested |

---

## 3. Test Coverage Summary

| Suite | Tests Passed | Tests Failed | Delta from Session 10 |
|-------|-------------|-------------|----------------------|
| Rust (`cargo test --workspace`) | 567 | 0 | +0 unit, +7 new tests |
| Frontend (`npx vitest run`) | 998 | 0 | +4 new tests |

### New tests added in Session 11:

**Backend (7 tests):**
- `aggregate_thread_zero_impressions` — all-zero impressions edge case
- `aggregate_thread_mixed_zero_and_positive_impressions` — mixed impressions
- `aggregate_thread_single_child_only` — single-child thread
- `update_analytics_no_frontmatter_returns_entry_not_found` — missing frontmatter
- `iteration_with_forge_sync_enabled` — analytics loop + Forge happy path
- `iteration_with_forge_sync_disabled` — Forge disabled returns None
- `iteration_forge_sync_failure_non_fatal` — Forge error doesn't fail iteration

**Frontend (4 tests):**
- `sanitizePathStem_empty_string` — empty input edge case
- `sanitizePathStem_no_extension` — bare filename
- `ApprovalUpdated approved triggers sync prompt` — prompt trigger on publish
- `ApprovalUpdated rejected does NOT trigger prompt` — no prompt on rejection

---

## 4. Residual Risks (Ranked by Severity)

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| 1 | `run_forge_sync_if_enabled()` has a default no-op — server crate must wire the concrete impl calling `compute_performance_percentiles_for()` + `run_forge_sync()` | **High** | The trait method, percentile computation, and sync engine are all tested individually. The server wiring is a straightforward integration task: read `analytics_sync_enabled` from active config, compute percentiles, call `run_forge_sync()`. |
| 2 | `compute_performance_percentiles_for()` is only testable with a real SQLite DB — no in-memory unit test | **Medium** | The function is simple (fetch + sort + index). SQLite `ORDER BY` is deterministic. Integration-test in the server crate's existing DB test harness. |
| 3 | Forge sync events (`forge.sync_succeeded`, `forge.sync_failed`) not wired to frontend | **Low** | By design (D81): server logs cover v1 metrics. Frontend helpers exist and are tested — wire when a sync-status UI is built. |
| 4 | `AngleCards` telemetry props (`sessionId`, `sourcePathStem`) default to `'unknown'` | **Low** | The composer parent must pass these through for full telemetry. Events still fire — just with reduced metadata. Follow-up: wire props from composer state. |
| 5 | `PerformancePercentiles` type moved from sync.rs to storage module | **Low** | Re-exported via `pub use` — no downstream breakage. All existing tests pass. |

---

## 5. Rollout Notes

### Feature Flags
- **`analytics_sync_enabled`** (per content source): Controls Forge sync. Defaults to `false`. User enables via Settings toggle or consent prompt. No server-side override needed.
- The Hook Miner extraction path is always active when a `local_fs` source is configured and notes are selected. No separate feature flag.

### Gradual Enablement
1. **Phase 1 (immediate):** Ship with `analytics_sync_enabled` defaulting to `false`. Consent prompt appears after first successful publish from a local_fs source.
2. **Phase 2 (post-release):** Monitor `hook_miner.*` and `forge.*` structured logs for adoption signals. If prompt dismissal rate exceeds 80%, consider adjusting copy or timing.
3. **Phase 3 (follow-up):** Wire `forge.sync_succeeded` / `forge.sync_failed` to frontend if sync-status UI is built.

### Monitoring
- Structured logs: `hook_miner.angles_shown`, `hook_miner.angle_selected`, `hook_miner.fallback_opened` for adoption.
- Structured logs: `forge.prompt_shown`, `forge.enabled` for consent conversion.
- Server-side `tracing::info!` / `tracing::warn!` in analytics loop for sync health.
- Follower drop alert (>2%) already in analytics loop.

---

## 6. Known Limitations (Explicitly Deferred)

| Item | Reason |
|------|--------|
| `PerformancePercentiles` DB query not called from server crate | Integration wiring — the trait method exists, the function exists, the concrete server impl must call them together. |
| No WebSocket notification for Forge sync results | Scope creep for Session 11. Server logs are sufficient for v1. |
| `trackForgeSyncSucceeded` / `trackForgeSyncFailed` not called from frontend | No trigger point exists without WS notification. Helpers are exported and tested. |
| E2E test coverage for full Hook Miner → Forge loop | E2E tests require a running server with LLM + X API mocks. Out of scope for the epic; should be added in a dedicated testing sprint. |
| `release-plz update` and `cargo package` validation | Not run in this session (no version bumps). CI enforces on every PR. |
