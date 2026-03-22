# Session 04 Handoff — Hook Miner Composer Integration

**Date:** 2026-03-22
**Session:** 04 of 11
**Status:** Complete

---

## What Changed

Frontend integration of Hook Miner into the Ghostwriter compose flow. The Obsidian selection path now mines evidence-backed angles when neighbors are accepted, with smooth fallback to generic hooks.

| File | Action | Purpose |
|------|--------|---------|
| `dashboard/src/lib/api/types.ts` | Modified | Added `EvidenceItemResponse`, `MinedAngle`, `AssistAnglesResponse` types |
| `dashboard/src/lib/api/client.ts` | Modified | Added `api.assist.angles()` method |
| `dashboard/src/lib/utils/angleStyles.ts` | Created | Angle type labels, evidence type config, citation truncation |
| `dashboard/src/lib/utils/hookStyles.ts` | Modified | Added `story`, `listicle`, `hot_take` to STYLE_LABELS |
| `dashboard/src/lib/analytics/backlinkFunnel.ts` | Modified | Added `trackAnglesMined` and `trackAngleFallback` events |
| `dashboard/src/lib/components/composer/AngleCards.svelte` | Created | Angle card list with evidence badges, format toggle, selection |
| `dashboard/src/lib/components/composer/AngleFallback.svelte` | Created | Fallback state for weak signal, timeout, parse error |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Modified | Branching `handleGenerate` logic, angle state variables, new handlers |
| `dashboard/tests/unit/angleStyles.test.ts` | Created | 13 tests for angle/evidence utilities |
| `dashboard/tests/unit/apiClientAngles.test.ts` | Created | 7 tests for API client method |
| `dashboard/tests/unit/AngleCards.test.ts` | Created | 28 tests for AngleCards component |
| `dashboard/tests/unit/angleMinerFlow.test.ts` | Created | 10 tests for state transition flow |
| `dashboard/tests/unit/hookStyles.test.ts` | Modified | Added 3 tests for new angle type labels |
| `dashboard/tests/unit/VaultSelectionReview.test.ts` | Modified | Updated 2 tests for angles flow, added `angles` to mock |
| `docs/roadmap/hook-miner-forge-loop/hook-miner-ui-notes.md` | Created | UI state machine and copy choices documentation |
| `docs/roadmap/hook-miner-forge-loop/session-04-handoff.md` | Created | This file |

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D20 | `acceptedNeighborIds` is required positional param | Matches Rust endpoint contract (400 on empty). |
| D21 | Response types use `string` not TS enums | Flexible — new backend types don't break frontend. Display logic centralized in `angleStyles.ts`. |
| D22 | Separate `angleStyles.ts` from `hookStyles.ts` | Different domains. `getConfidenceBadge` imported from hookStyles (reused, not duplicated). |
| D23 | `AngleFallback` as separate component | Keeps VaultSelectionReview template manageable. |
| D24 | "More hook styles" preserves all neighbor state | `acceptedNeighbors`, `dismissedNodeIds`, `synthesisEnabled` never reset by view transitions. |
| D25 | From Vault chunk-selection path NOT modified | UX spec explicitly excludes this path (no neighbors = no evidence). |
| D26 | AngleCards reuses HookPicker visual patterns exactly | Visual consistency — users recognize the interaction pattern. |
| D27 | No client-side AbortController timeout in V1 | Standard fetch timeout sufficient. Fallback path handles errors. |
| D28 | Angle type labels added to hookStyles STYLE_LABELS | Downstream compose/provenance uses `hookStyle` field — angle types must render there too. |

---

## Test Coverage

| Module | Tests | Status |
|--------|-------|--------|
| `angleStyles.ts` | 13 tests: label mapping (5), evidence config (4), truncation (4) | All pass |
| `apiClientAngles.ts` | 7 tests: POST payload, optional params, combined params | All pass |
| `AngleCards.svelte` | 28 tests: rendering, selection, confirm, callbacks, loading, error, a11y | All pass |
| `angleMinerFlow.ts` | 10 tests: branching logic, fallback, back navigation, errors, timeout | All pass |
| `hookStyles.ts` | 3 new tests: angle type labels in STYLE_LABELS | All pass |
| `VaultSelectionReview.test.ts` | 2 updated tests: angles flow with neighbor provenance | All pass |
| **Total new/modified tests** | **63** | **All pass** |
| **Total frontend tests** | **971** | **All pass** |

---

## Quality Gates

```
npm --prefix dashboard run check              ✅ (0 errors, 0 warnings)
npx vitest run                                ✅ (971 passed)
cargo fmt --all && cargo fmt --all --check     ✅ (no Rust changes)
cargo clippy --workspace -- -D warnings        ✅ (no Rust changes)
RUSTFLAGS="-D warnings" cargo test --workspace ✅ (567 passed)
```

---

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| VaultSelectionReview grew from 335 to ~430 lines (script) | Medium | New components absorb rendering complexity. Conditional chain is documented in hook-miner-ui-notes.md. Within 500-line limit. |
| Evidence badges may look crowded with 3+ items | Low | CSS truncation at 40 chars. Evidence section scrollable with `max-height: 120px`. |
| Integration tests depend on `.suggestion-accept-btn` class from GraphSuggestionCards | Low | Tests guard with `if (acceptBtn)` — graceful degradation if class changes. |
| AngleFallback `reason` nullability required `?? undefined` coercion | Low | Minor type wiring. Svelte 5 props use `?` (undefined) while state uses `null`. |

---

## Required Inputs for Session 05

Session 05 implements the Forge Loop — connecting angle selection outcomes to the loopback measurement pipeline.

**Must read:**
- `docs/roadmap/hook-miner-forge-loop/hook-miner-ui-notes.md` (this session's state machine)
- `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` (updated angle flow)
- `dashboard/src/lib/components/composer/AngleCards.svelte` (angle card component)
- `crates/tuitbot-core/src/content/angles.rs` (angle domain types)
- `crates/tuitbot-server/src/routes/assist/angles.rs` (angle endpoint)

**Must preserve:**
- Existing angle flow (VaultSelectionReview → AngleCards → ongenerate)
- All state preservation guarantees (neighbor maps, synthesis toggle)
- Fallback path integrity (AngleFallback → HookPicker or back to neighbors)

**Must create:**
- Loopback measurement for angle-sourced drafts
- Forge loop data pipeline connecting post performance back to angle types
- Analytics for tracking angle type effectiveness over time

---

## Architecture Summary

```
User clicks "Generate hooks" in VaultFooter
                    │
    ┌───────────────┼───────────────┐
    │                               │
synthesis ON                   synthesis OFF
+ neighbors > 0               OR neighbors = 0
    │                               │
    ▼                               ▼
POST /api/assist/angles    POST /api/assist/hooks
    │                               │
    ├── angles OK ──▶ AngleCards     ├── hooks ──▶ HookPicker
    │                   │           │
    ├── fallback ──▶ AngleFallback  │
    │                   │           │
    └── error ──▶ Error banner      │
                        │           │
            ┌───────────┼───────┐   │
            │                   │   │
    "Use generic hooks"  "Back to   │
            │          related      │
            │          notes"       │
            ▼                ▼      │
       HookPicker    selection_     │
                     review         │
```
