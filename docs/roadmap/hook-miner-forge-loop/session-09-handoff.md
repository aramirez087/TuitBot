# Session 09 Handoff — Forge Settings & Prompt UX

**Date:** 2026-03-22
**Session:** 09 of 11
**Status:** Complete

---

## What Changed

Settings, UI toggles, consent prompt, and copy that make Forge analytics sync understandable and consent-driven.

| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/src/config/types.rs` | **Modified** | Added `analytics_sync_enabled: bool` to `ContentSourceEntry` with `#[serde(default)]` (false). Added roundtrip test. |
| `crates/tuitbot-core/src/automation/watchtower/mod.rs` | **Modified** | Added `analytics_sync_enabled` to local_fs `config_json` serialization |
| `crates/tuitbot-core/src/automation/watchtower/tests.rs` | **Modified** | Added `analytics_sync_enabled: false` to all 4 `ContentSourceEntry` constructors |
| `crates/tuitbot-core/src/config/tests/migrations.rs` | **Modified** | Added `analytics_sync_enabled: false` to all 13 `ContentSourceEntry` constructors |
| `crates/tuitbot-core/src/config/tests/deployment.rs` | **Modified** | Added `analytics_sync_enabled: false` to all 3 `ContentSourceEntry` constructors |
| `dashboard/src/lib/api/types.ts` | **Modified** | Added `analytics_sync_enabled: boolean` to content source type |
| `dashboard/src/lib/stores/settings.ts` | **Modified** | Added prompt state management: `analyticsSyncPromptDismissed`, `pendingAnalyticsSyncPrompt`, dismiss/reset/set/clear helpers with localStorage backing |
| `dashboard/src/routes/(app)/settings/SourceTogglesSection.svelte` | **Created** | Extracted Watch + Loop Back + Analytics Sync toggles + Google Drive notice |
| `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte` | **Modified** | Uses `SourceTogglesSection`, added `analytics_sync_enabled` to updateSource, calls `resetAnalyticsSyncPrompt()` on mount |
| `dashboard/src/lib/components/settings/AnalyticsSyncPrompt.svelte` | **Created** | One-time consent banner with Enable/Dismiss actions |
| `dashboard/src/routes/(app)/activity/+page.svelte` | **Modified** | Renders `AnalyticsSyncPrompt` when conditions met, navigates to settings on enable |
| `dashboard/tests/unit/analyticsSyncPrompt.test.ts` | **Created** | 8 tests for prompt state management lifecycle |
| `docs/configuration.md` | **Modified** | Documented `analytics_sync_enabled` field in Local Folder table |
| `config.example.toml` | **Modified** | Added `analytics_sync_enabled = false` to local folder example |
| `docs/roadmap/hook-miner-forge-loop/settings-and-copy-notes.md` | **Created** | Copy decisions and UX rationale |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D61 | `analytics_sync_enabled` is per-source on `ContentSourceEntry`, not global | Matches `loop_back_enabled` pattern. Users with multiple vaults may want sync on one but not another. |
| D62 | Default `false` via `#[serde(default)]` | Opt-in per epic contract. No behavioral change for existing users. |
| D63 | Prompt is a banner on Activity page, not a modal | Modals block workflow. Banner respects user agency. Activity is the natural post-publish destination. |
| D64 | "Not now" persists to localStorage, reset when user visits Settings > Content Sources | Balances not-spamming with re-discoverability. `onMount` reset means revisiting settings gives a fresh opportunity. |
| D65 | Analytics sync toggle only visible when `loop_back_enabled` is true | Analytics sync is a superset of publish writeback. Showing it independently would be confusing. |
| D66 | Google Drive gets informational notice, not error | Users shouldn't feel misconfigured. Explains capability boundary. |
| D67 | Extracted toggles to `SourceTogglesSection.svelte` | ContentSourcesSection was at 397 lines. Adding analytics toggle + notice would exceed 400-line limit. Extraction brings parent to 380. |
| D68 | localStorage for prompt state, not DB | Prompt state is UI-only concern. No cross-device sync needed. |
| D69 | Updated Loop Back copy from "coming soon" to accurate description | Writeback is now implemented. Old copy was misleading. |
| D70 | `run_forge_sync()` parameter-based contract preserved | Per Session 08 handoff. Config wiring happens at the call site, not inside the sync engine. |

---

## Quality Gates

```
cargo fmt --all && cargo fmt --all --check    Pass
RUSTFLAGS="-D warnings" cargo test --workspace Pass (648 passed, 0 failed)
cargo clippy --workspace -- -D warnings        Pass
npm --prefix dashboard run check               Pass (0 errors, 0 warnings)
npm --prefix dashboard run test:unit:run        Pass (979 passed)
```

---

## Test Coverage Added

| Test | File | What It Validates |
|------|------|-------------------|
| `content_source_entry_deserialize_defaults` (updated) | types.rs | `analytics_sync_enabled` defaults to false |
| `content_source_entry_analytics_sync_roundtrip` | types.rs | Field serializes/deserializes correctly |
| `defaults to false when localStorage is empty` | analyticsSyncPrompt.test.ts | Dismissed store initializes correctly |
| `dismissAnalyticsSyncPrompt sets store and localStorage` | analyticsSyncPrompt.test.ts | Dismiss persists |
| `resetAnalyticsSyncPrompt clears store and localStorage` | analyticsSyncPrompt.test.ts | Reset clears |
| `setPendingAnalyticsSyncPrompt sets store and localStorage` | analyticsSyncPrompt.test.ts | Pending flag persists |
| `clearPendingAnalyticsSyncPrompt clears store and localStorage` | analyticsSyncPrompt.test.ts | Pending flag clears |
| `dismiss then reset produces fresh prompt opportunity` | analyticsSyncPrompt.test.ts | Full lifecycle |
| `pending and dismissed are independent` | analyticsSyncPrompt.test.ts | Orthogonal state |

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Prompt trigger requires explicit `setPendingAnalyticsSyncPrompt()` call from publish flow | Medium | Not wired to WebSocket events in this session. The function exists and is exported. Session 10 or a future PR should call it from the approval store's `ActionPerformed` handler. |
| `PerformancePercentiles` computation not yet implemented | Medium | Session 08 deferred this. The `run_forge_sync()` function accepts percentiles as a parameter. Session 10 should add the DB query. |
| Settings page `onMount` reset could conflict with rapid navigation | Low | Acceptable for v1. The reset is idempotent and only clears a localStorage flag. |
| No end-to-end test for prompt → settings navigation | Low | Manual testing recommended. The navigation uses `window.location.href` which is standard browser behavior. |

---

## Required Inputs for Session 10

Session 10 wires the sync engine into the analytics loop and adds the percentile computation.

**Must read:**
- `crates/tuitbot-core/src/automation/watchtower/loopback/sync.rs` — `run_forge_sync()` signature
- `crates/tuitbot-core/src/automation/analytics_loop.rs` — where to call `run_forge_sync()`
- `crates/tuitbot-core/src/config/types.rs` — `ContentSourceEntry.analytics_sync_enabled`

**Must implement:**
- Compute `PerformancePercentiles` from `tweet_performance` table (p50, p90, count >= 10)
- Wire `run_forge_sync()` call after analytics loop iteration
- Read `analytics_sync_enabled` from first source's config at the call site
- Call `setPendingAnalyticsSyncPrompt()` from the approval store's `ActionPerformed` handler when publish succeeds for eligible content

**Must preserve:**
- `run_forge_sync()` takes `analytics_sync_enabled` as parameter (don't change to reading from DB internally)
- All existing loopback tests pass
- The publish writeback path (`write_metadata_to_file`) remains unchanged
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
Session 09: Settings UI + consent prompt (this session)
  |- analytics_sync_enabled on ContentSourceEntry
  |- SourceTogglesSection (extracted, 3 toggles)
  |- AnalyticsSyncPrompt (banner on Activity page)
  |- Prompt state management (localStorage-backed)
  |- Updated copy for loop back vs analytics sync
  |- Google Drive unsupported notice
        |
        v
Session 10: Sync wiring + percentile computation
  -> Wire run_forge_sync() into analytics loop
  -> Compute PerformancePercentiles from DB
  -> Trigger pending prompt from publish events
```
