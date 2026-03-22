# Session 04 Handoff: Ghostwriter Entry Flow & Graph Suggestion UX

**Date:** 2026-03-21
**Branch:** `epic/backlink-synthesizer`

## What Changed

Integrated graph-aware retrieval into the Ghostwriter compose flow with suggestion cards, accept/dismiss interactions, a session-level synthesis toggle, and provenance tracking for accepted neighbors.

### Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `dashboard/src/lib/components/composer/GraphSuggestionCards.svelte` | Suggestion card component with loading, available, empty, not-indexed, fallback states | ~230 |
| `dashboard/tests/unit/GraphSuggestionCards.test.ts` | 21 unit tests for the new component | ~210 |
| `docs/roadmap/backlink-synthesizer/ghostwriter-entry-flow.md` | Interaction model documentation | ~120 |
| `docs/roadmap/backlink-synthesizer/session-04-handoff.md` | This file | ~100 |

### Files Modified

| File | Change |
|------|--------|
| `dashboard/src/lib/api/types.ts` | Added `NeighborItem`, `GraphState` types; extended `VaultSelectionResponse` with `graph_neighbors`/`graph_state`; extended `ProvenanceRef` with `edge_type`/`edge_label` |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Added graph suggestion integration: acceptedNeighbors, dismissedNodeIds, synthesisEnabled state; synthesis toggle; GraphSuggestionCards rendering; neighbor provenance in ongenerate |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Extended `handleGenerateFromVault` to accept `neighborProvenance` parameter; enriches `ProvenanceRef[]` with `edge_type`/`edge_label` for neighbor nodes |
| `dashboard/src/lib/components/composer/InspectorContent.svelte` | Updated `ongeneratefromvault` type to include optional `neighborProvenance` parameter |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Updated `ongenerate` type to include optional `neighborProvenance` parameter |
| `dashboard/tests/unit/VaultSelectionReview.test.ts` | Added 11 graph-related tests; updated 2 existing assertions for new ongenerate signature |

## Decisions Made

1. **No backend changes needed.** Session 3's `GetSelectionResponse` already includes `graph_neighbors` and `graph_state`. Frontend just consumes them.

2. **`@const` in `{#each}` for dynamic icon.** Svelte 5 deprecates `<svelte:component>` in runes mode. Used `{@const ReasonIcon = getReasonIcon(reason)}` inside `{#each}` block, then `<ReasonIcon size={10} />`.

3. **Session-scoped state, not localStorage.** `acceptedNeighbors`, `dismissedNodeIds`, and `synthesisEnabled` are component state that resets each compose session. This aligns with the "fail open" principle.

4. **Neighbor provenance uses `reason` as `edge_type` and `reason_label` as `edge_label`.** This maps cleanly to the backend columns added in Session 2. `linked_note` → edge_type, "linked note" → edge_label.

5. **Empty neighbors with `available` state falls through to empty message.** The `no_related_notes` and `available` (empty) states show the same message. This prevents a visual gap if the backend returns `available` with zero neighbors.

6. **GraphSuggestionCards is extracted as separate component.** Keeps VaultSelectionReview under the 400-line Svelte limit (~200 lines for the new component, ~200 lines for VaultSelectionReview).

7. **Intent actions are single-action per card.** Each card shows one action button mapped from its `intent` field. This avoids cluttering small cards with multiple action buttons.

## Verification

```
npm --prefix dashboard run check          ✅ 0 errors, 0 warnings
npm --prefix dashboard run test:unit:run  ✅ 798 passed, 0 failed (41 files)
cargo fmt --all --check                   ✅ clean
cargo clippy --workspace -- -D warnings   ✅ no warnings
RUSTFLAGS="-D warnings" cargo test --workspace  ✅ all passed
```

Test breakdown for new code:
- `GraphSuggestionCards.test.ts`: 21 passed (states, interactions, a11y)
- `VaultSelectionReview.test.ts`: 48 passed (11 new graph tests + 37 existing)

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| GraphSuggestionCards shimmer animation may stutter on low-end devices | Low | `prefers-reduced-motion` media query disables animations |
| Accepted neighbor node IDs may not resolve in `resolve_composer_rag_context()` if vault changed since graph expansion | Low | Backend already handles missing node IDs gracefully (fail open) |
| `intent` classification from Session 3 is heuristic-based | Low | Defaults to "Use as context" for unrecognized intents. Refinable with LLM classification later |
| Score dot opacity scale (0-10) may not match actual score distributions | Low | Visual hint only, not a decision gate. Can be calibrated with real data |

## Required Inputs for Session 5

Session 5 ("Compose Integration & Polish") needs:

1. **This session's output:**
   - `GraphSuggestionCards.svelte` — standalone component for suggestion cards
   - Updated `VaultSelectionReview.svelte` with synthesis toggle and accept/dismiss
   - Updated `ComposerInspector.svelte` with provenance enrichment
   - TypeScript types: `NeighborItem`, `GraphState` in `types.ts`

2. **Key integration points for next session:**
   - Verify accepted neighbor IDs reach `resolve_composer_rag_context()` end-to-end
   - Consider adding neighbor context as distinct prompt section (not mixed with primary selection)
   - Test with real vault data (requires Obsidian plugin connected to running server)

3. **UX polish items deferred to Session 5:**
   - Animation for card dismiss (currently instant)
   - Keyboard navigation within suggestion cards
   - Re-ordering accepted neighbors by role
   - Visual indicator when a neighbor has already been accepted (checkmark or highlight)

4. **Decisions to carry forward:**
   - MAX_GRAPH_FRAGMENTS_PER_NOTE = 3 per neighbor in prompt
   - Graph UI shows `reason_label` and `intent` visually
   - Suggestions are visible but not auto-included (explicit accept required)
