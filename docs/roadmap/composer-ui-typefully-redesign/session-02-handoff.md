# Session 02 Handoff — Live Canvas Surface

## What Changed

Seven component files modified to transform the compose surface from a form-like input into a live-post canvas.

| File | Changes |
|------|---------|
| `TweetEditor.svelte` | Borderless textarea (transparent bg, no border, 15px/1.4 font), conditional char counter (hidden when <= 240), icon-only media attach button with touch target, removed attach-hint text, tightened media grid margin |
| `ThreadFlowCard.svelte` | 15px/1.4 font on textarea, smaller spine dot (8px, 1.5px border, repositioned to -21px), conditional char counter (hidden when <= 240), reduced between-zone height (12px) and plus circle (14px), smaller Plus icon (10px) |
| `ThreadFlowLane.svelte` | Reduced padding-left from 40px to 32px, lane spine moved from left: 19px to left: 15px, spine made semi-transparent (60% opacity via color-mix) |
| `HomeComposerHeader.svelte` | Removed schedule pill entirely (Calendar import, onschedule prop, formatTime function, schedule-pill CSS), reduced header padding from 10px 20px to 8px 16px, removed redundant mobile padding override |
| `ComposeWorkspace.svelte` | Removed ThreadPreviewRail import, removed inline preview section (template + CSS), removed onschedule prop from HomeComposerHeader invocation |
| `ComposerTipsTray.svelte` | Reduced padding from 8px 20px to 6px 16px, removed redundant mobile padding override |
| `ComposerPromptCard.svelte` | Reduced margin from 16px 20px to 12px 16px, removed redundant mobile media query |

**No Rust or backend changes.**

## Decisions Confirmed

### D1: Textarea stays borderless with no focus ring
Confirmed from Session 1 plan. The textarea has `outline: none` on focus and no replacement border. The cursor and placeholder text provide sufficient affordance. Thread mode's spine-dot focus color still gives visual feedback. Over-limit state uses `box-shadow: inset 2px 0 0 var(--color-danger)` as a subtle left-edge indicator.

### D2: Character counter threshold at 240
Both TweetEditor and ThreadFlowCard now hide the character counter when count is <= 240. The conditional also checks `overLimit` as a safety net. This gives users ~40 characters of warning before the 280 limit.

### D3: Schedule pill removed, not hidden
The schedule pill is deleted from the component, not conditionally hidden. Schedule is accessible via the inspector panel (Cmd+I) and command palette (Cmd+K). The publish pill's label still dynamically shows "Schedule" when a time is selected via the inspector.

### D4: Inline preview section removed
The ThreadPreviewRail is no longer rendered inline. The `previewCollapsed` state and `togglePreview()` function remain wired to Cmd+Shift+P and the preview button — they toggle state but have no visible effect until the PreviewOverlay is added in Session 3.

### D5: Tips and prompt components retained
Both ComposerTipsTray and ComposerPromptCard are kept with minor padding adjustments to match the new 16px horizontal rhythm. They don't compete with the canvas — tips show between header and canvas (auto-dismiss), prompt shows below canvas when empty.

## Transitional States

### Preview toggle (Cmd+Shift+P)
The shortcut and button still toggle `previewCollapsed` state, and the button shows active/inactive styling. However, no preview panel renders. This is intentional — Session 3 drops in `PreviewOverlay.svelte` that reads this same state.

### openScheduleInInspector function
The function remains in ComposeWorkspace even though it's no longer passed as a prop to HomeComposerHeader. It will be useful for the command palette's schedule action.

### sortedPreviewBlocks, hasPreviewContent, tweetMediaPreviewMap
These derived values remain in ComposeWorkspace. They're still used by `hasPreviewContent` (which gates the preview button's active state) and will be consumed by the PreviewOverlay in Session 3.

## Quality Gates

| Check | Result |
|-------|--------|
| `npm --prefix dashboard run check` | 0 errors, 6 pre-existing warnings (none from our changes) |
| `cargo fmt --all && cargo fmt --all --check` | Clean |
| `cargo clippy --workspace -- -D warnings` | Clean |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | All tests pass |

## Contract Preservation

| Contract | Status |
|----------|--------|
| `ThreadBlock[]` shape | Unchanged — no modifications to block operations |
| `ComposeRequest` shape | Unchanged — `buildComposeRequest()` not touched |
| `onsubmit(data)` callback | Unchanged |
| Autosave `{ mode, tweetText, blocks, timestamp }` | Unchanged |
| Modal entry: `ComposeModal` props | Unchanged |
| Home entry: `+page.svelte` embedded workspace | Unchanged |
| `api.content.compose()` / `api.content.schedule()` | Unchanged |
| `api.assist.improve()` / `api.assist.thread()` | Unchanged |
| `api.media.upload()` | Unchanged (only the attach button UI changed) |

## Known Limitations

1. **No preview overlay** — The full-screen preview doesn't exist yet. Pressing Cmd+Shift+P toggles state but nothing renders. Session 3 resolves this.
2. **Spine dot alignment** — The arithmetic positions the 8px dot centered on the spine at left: 15px within a 32px gutter. If visual testing reveals a 1px drift, adjust `spine-dot left` in ThreadFlowCard.

## Session 03 Inputs

### Starting Files
- `docs/roadmap/composer-ui-typefully-redesign/charter.md`
- `docs/roadmap/composer-ui-typefully-redesign/ui-architecture.md`
- This handoff document

### Primary Task: PreviewOverlay
Create `dashboard/src/lib/components/composer/PreviewOverlay.svelte` — a full-screen overlay that renders the draft as it would appear on X. Wire it to the existing `previewCollapsed` state (rename to `previewMode`).

### Files to Create/Modify
| File | Action |
|------|--------|
| `PreviewOverlay.svelte` | **Create** — full-screen overlay, reads `sortedPreviewBlocks`, `tweetText`, `tweetMediaPreviewMap` from ComposeWorkspace |
| `ComposeWorkspace.svelte` | Rename `previewCollapsed` to `previewMode`, render `<PreviewOverlay>` conditionally, pass draft state as props |
| `ComposerHeaderBar.svelte` | Update preview button to open overlay instead of toggling inline panel |
| `TweetPreview.svelte` | Review for reuse inside PreviewOverlay |

### Key Design Constraints for Session 3
- Overlay is **read-only** — no editing capability
- Escape and a visible close button dismiss the overlay
- The overlay renders on top of both embedded and modal composer
- Focus trap: keyboard focus stays within the overlay while open
- Accessible: `role="dialog"`, `aria-modal="true"`, focus returns to editor on close
