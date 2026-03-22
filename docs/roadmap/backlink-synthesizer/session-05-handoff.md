# Session 05 Handoff: Draft Insertion & Suggestion Controls

**Date:** 2026-03-21
**Branch:** `epic/backlink-synthesizer`

## What Changed

Implemented the draft insertion model: users can now accept related-note suggestions into specific thread slots, see what changed, and undo individual insertions without regenerating the entire draft.

### Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `dashboard/src/lib/stores/draftInsertStore.ts` | Pure utility functions for insert history management | ~105 |
| `dashboard/src/lib/components/composer/SlotTargetPanel.svelte` | Slot targeting UI for applying suggestions to specific blocks | ~180 |
| `dashboard/tests/unit/draftInsertStore.test.ts` | 21 unit tests for insert store utilities | ~170 |
| `dashboard/tests/unit/SlotTargetPanel.test.ts` | 9 component tests for slot targeting | ~155 |
| `docs/roadmap/backlink-synthesizer/draft-insertion-model.md` | Decision doc: insertion model, undo stack, provenance | ~100 |
| `docs/roadmap/backlink-synthesizer/session-05-handoff.md` | This file | ~120 |

### Files Modified

| File | Change |
|------|--------|
| `dashboard/src/lib/api/types.ts` | Added `DraftInsert`, `DraftInsertState` types |
| `dashboard/src/lib/components/composer/VaultSelectionReview.svelte` | Added `threadBlocks`, `mode`, `insertState`, `oninsert`, `onundoinsert` props; renders `SlotTargetPanel` when draft exists and neighbors accepted |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Added `draftInsertState`, `handleSlotInsert()`, `handleUndoInsert()`, `handleUndoInsertById()`, `hasPendingInsertUndo()`; passes insert props to InspectorContent |
| `dashboard/src/lib/components/composer/InspectorContent.svelte` | Added `threadBlocks`, `insertState`, `onslotinsert`, `onundoinsert` props; passes through to FromVaultPanel |
| `dashboard/src/lib/components/composer/FromVaultPanel.svelte` | Added `threadBlocks`, `insertState`, `onslotinsert`, `onundoinsert` props; passes through to VaultSelectionReview |
| `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` | Updated `handleUndo()` to prefer insert-level undo; wired `onundo` to ComposerCanvas |
| `dashboard/src/lib/components/composer/ThreadFlowCard.svelte` | Added `inserts` and `onundoinsert` props; renders insert badges with undo buttons below textarea |
| `dashboard/src/lib/components/composer/CitationChips.svelte` | Added `graphInserts` and `onundoinsert` props; renders "Related notes:" section with purple-accented chips |
| `dashboard/src/lib/components/composer/ComposerCanvas.svelte` | Added `onundo` prop; undo banner now includes clickable "Undo" button |
| `dashboard/tests/unit/VaultSelectionReview.test.ts` | Added 5 new tests for slot targeting integration |

## Decisions Made

1. **Contextual inserts over prompt mutations.** Accepted suggestions produce visible, discrete text changes via `api.assist.improve()` rather than invisibly modifying the LLM prompt and regenerating everything. Users see what changed and where.

2. **Block ID as stable identifier.** `DraftInsert.blockId` uses `ThreadBlock.id` (UUID) rather than slot index, so inserts survive block reordering.

3. **Per-insert undo stack.** LIFO history with `popInsert()` and `undoInsertById()`. Insert undo takes priority over workspace-level undo (full snapshot restore) in the `handleUndo()` chain.

4. **SlotTargetPanel extracted as separate component.** Keeps VaultSelectionReview under the 400-line Svelte limit (~430 lines with slot panel inline vs ~270 lines with extraction).

5. **No backend changes required.** All insertion logic is frontend-only. The backend already accepts `provenance: Vec<ProvenanceRef>` with `edge_type`/`edge_label`, which the insert flow appends to incrementally.

6. **Purple accent for graph-sourced citations.** CitationChips uses a purple tint for "Related notes:" section to visually distinguish from primary "Based on:" citations (blue accent).

7. **Undo banner is now actionable.** ComposerCanvas undo banner includes a clickable "Undo" button rather than being purely informational. Works for both insert undos and workspace undos.

## Verification

```
npm --prefix dashboard run check          0 errors, 0 warnings
npm --prefix dashboard run test:unit:run  833 passed, 0 failed (43 files)
cargo fmt --all --check                   clean
cargo clippy --workspace -- -D warnings   no warnings
RUSTFLAGS="-D warnings" cargo test --workspace  all passed
```

Test breakdown for new code:
- `draftInsertStore.test.ts`: 21 passed (pure utility functions)
- `SlotTargetPanel.test.ts`: 9 passed (component rendering, interactions, a11y)
- `VaultSelectionReview.test.ts`: 53 passed (48 existing + 5 new slot targeting tests)

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `api.assist.improve()` may produce text that doesn't fit slot's role/length | Medium | Context prompt includes slot label: "This is the [Opening hook]..." — LLM adapts style accordingly |
| Multiple inserts to same block create compounding text changes | Low | Each insert captures `previousText` at the moment of application; undo restores that exact state |
| Thread block reorder after insert could confuse users about which slot was modified | Low | `DraftInsert.blockId` tracks the stable block ID; `slotLabel` is cosmetic and could be re-derived |
| Insert undo + workspace undo interaction could confuse users | Low | Clear UX: undo message distinguishes "Reverted [slot label]" vs "Content replaced." |
| ThreadFlowCard and CitationChips `inserts` prop not yet wired from ThreadFlowLane | Medium | Props are defined and tested; parent wiring from ThreadFlowLane needs to pass `inserts` per block. Currently badges require manual prop passing at the ThreadFlowLane level. |
| SlotTargetPanel slot selector on mobile may be small | Low | Touch-friendly sizing via `@media (pointer: coarse)` breakpoints |

## Required Inputs for Session 6

1. **This session's output:**
   - `draftInsertStore.ts` — pure utility module for insert history
   - `SlotTargetPanel.svelte` — slot targeting component
   - Updated `ComposerInspector.svelte` with insert state management
   - Updated `ThreadFlowCard.svelte` with insert badge rendering
   - Updated `CitationChips.svelte` with graph provenance section

2. **Key integration points for next session:**
   - Wire `inserts` prop per-block through `ThreadFlowLane` → `ThreadFlowCard` (currently prop is defined but not connected from parent)
   - Wire `graphInserts` prop through ComposerCanvas → CitationChips (currently prop defined but not connected)
   - Consider adding visual feedback (animation) when an insert is applied
   - Test with real vault data end-to-end

3. **UX polish items deferred:**
   - Animation for insert application (currently instant text swap)
   - Keyboard navigation within SlotTargetPanel
   - Reordering multiple inserts on the same block
   - Slot label live-updating when thread blocks are reordered

4. **Decisions to carry forward:**
   - Insert undo takes priority over workspace undo
   - Graph citations use purple accent color
   - `buildInsert()` generates UUID and timestamp automatically
   - MAX_GRAPH_FRAGMENTS_PER_NOTE = 3 per neighbor in prompt (from Session 4)
