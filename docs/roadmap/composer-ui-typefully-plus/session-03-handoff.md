# Session 03 Handoff — Thread Splitting & Flow Lane

## What Changed

### New Files

1. **`dashboard/src/lib/utils/threadOps.ts`** (~210 lines)
   - Pure, stateless utility functions for all thread block operations
   - Functions: `createDefaultBlocks`, `normalizeOrder`, `sortBlocks`, `addBlock`, `addBlockAfter`, `removeBlock`, `updateBlockText`, `updateBlockMedia`, `moveBlock`, `moveBlockToIndex`, `duplicateBlock`, `splitBlockAt`, `mergeWithNext`, `mergeWithPrevious`, `splitFromPaste`, `validateThread`
   - Word-boundary snapping on split (±10 chars), media limit enforcement on merge (max 4)
   - No DOM access, no Svelte imports — testable in isolation

### Modified Files

2. **`dashboard/src/lib/components/composer/ThreadFlowLane.svelte`** (137 → ~310 lines)
   - **Role change**: From pure pass-through to smart thread orchestrator (replaced ThreadComposer)
   - Owns all thread state: `blocks`, `focusedBlockId`, `draggingBlockId`, `dropTargetBlockId`, `reorderAnnouncement`, `mergeError`, `assistingBlockId`
   - All block operations delegate to `threadOps.ts` pure functions
   - Keyboard handler updated: `Cmd+Enter` = split at cursor (was `Cmd+Shift+Enter`)
   - New paste handler: intercepts multi-paragraph paste into empty blocks
   - Exports same API as former ThreadComposer: `getBlocks`, `setBlocks`, `handleInlineAssist`, `handlePaletteAction`
   - **Visual**: Persistent left spine line (`1px`, `--color-border-subtle`) connecting blocks, with 40px left gutter on desktop
   - **Mobile**: Spine and gutter hidden at ≤640px via media query
   - Removed: "Add tweet" button at bottom (replaced by between-block affordance in ThreadFlowCard)

3. **`dashboard/src/lib/components/composer/ThreadFlowCard.svelte`** (295 → ~290 lines)
   - **Visual overhaul**: Removed `border-left: 2px solid` boxed card styling
   - Added spine dot marker (10px circle, centered on spine line), changes color on focus/over-limit
   - Added between-block "+" affordance (18px circle button, fades in on hover, aligned to spine)
   - Unified separator: removed `border-bottom`, reduced height from 28px to 24px
   - New props: `onaddafter` (insert block after), `onpaste` (paragraph-aware paste)
   - Paste handler on textarea: intercepts when block is empty and paste contains `\n\n`
   - IME guard: `e.isComposing` check on keydown passthrough
   - Updated placeholder text: "Start writing..." / "Continue..." (was "Start your thread..." / "Tweet N...")
   - Mobile fallback: thin accent bar instead of spine dot at ≤640px

4. **`dashboard/src/lib/components/composer/ComposeWorkspace.svelte`** (741 → ~745 lines)
   - Import swap: `ThreadComposer` → `ThreadFlowLane`
   - Ref swap: `threadComposerRef` → `threadFlowRef`
   - Keyboard handler: `Cmd+Enter` now mode-aware (tweet=submit, thread=propagate to lane for split)
   - Added `Cmd+Shift+Enter` as universal submit shortcut
   - Template: renders `ThreadFlowLane` directly instead of `ThreadComposer`

5. **`dashboard/src/lib/components/CommandPalette.svelte`** (345 → ~345 lines)
   - `allActions` converted from `const` to `$derived` for mode-reactive shortcut display
   - Language updates: "Add tweet card" → "Add post below", "Duplicate card" → "Duplicate post", "Move card up/down" → "Move post up/down"
   - `submit` action shows `cmd+shift+enter` in thread mode, `cmd+enter` in tweet mode
   - `split` action shows `cmd+enter` (was `cmd+shift+s`)

6. **`dashboard/src/lib/utils/shortcuts.ts`** (120 → ~123 lines)
   - Updated `SHORTCUT_CATALOG` labels: "card" → "post" throughout
   - Added `cmd+shift+enter` entry for thread submit
   - Updated `cmd+enter` label to clarify dual behavior

7. **`dashboard/src/lib/components/ThreadComposer.svelte`** (434 → ~35 lines)
   - Converted to thin delegate wrapper around ThreadFlowLane
   - Preserves the same exported API for backward compatibility
   - No longer imported by ComposeWorkspace (dead code, kept for reference)

## Architecture Decisions

### D1: Cmd+Enter splits, Cmd+Shift+Enter submits (in thread mode)

In tweet mode, `Cmd+Enter` remains submit. In thread mode, `Cmd+Enter` is now the fast split shortcut (two keys instead of three). Submit in thread mode uses `Cmd+Shift+Enter` or the button.

**Implementation**: ComposeWorkspace's window-level keydown handler checks mode. For tweet mode, it calls `handleSubmit()`. For thread mode, it does NOT `preventDefault()` — the event propagates down to ThreadFlowLane's card-level handler which catches it with `e.stopPropagation()` and splits. `Cmd+Shift+Enter` always calls `handleSubmit()` regardless of mode.

### D2: threadOps.ts extraction

All block manipulation logic (split, merge, reorder, validate, paste-split) moved to pure functions in `threadOps.ts`. This eliminates:
- Inline logic duplication between ThreadComposer and ThreadFlowLane
- DOM coupling in business logic (cursor position is passed as a parameter, not read from DOM)
- Testing friction (functions can be unit-tested without Svelte)

### D3: ThreadFlowLane absorbs ThreadComposer

ThreadFlowLane went from a passive pass-through (137 lines) to the smart orchestrator (310 lines). ThreadComposer is now a thin wrapper. This eliminates:
- The dual-state sync via `$effect` on `initialBlocks` between ComposeWorkspace and ThreadComposer
- The `threadComposerRef?.handlePaletteAction()` indirection chain
- Two-level rendering (ComposeWorkspace → ThreadComposer → ThreadFlowLane → ThreadFlowCard)

Now it's: ComposeWorkspace → ThreadFlowLane → ThreadFlowCard

### D4: Paragraph-aware paste auto-split

Pasting text with `\n\n` into an empty block splits into multiple blocks. Conditions:
- Target block must be empty (prevents disrupting in-progress editing)
- Paste must contain `\n\n` (paragraph breaks)
- Must produce ≥2 non-empty paragraphs
- Each paragraph is trimmed, empty ones discarded
- Media stays on the original block
- Cursor lands at end of last new block

### D5: Spine visual — persistent left line with dot markers

- 1px vertical line in `--color-border-subtle` runs full height of the lane
- 10px dot (circle with 2px border) at top of each writing area, centered on spine
- Dot color: `--color-border-subtle` → `--color-accent` on focus → `--color-danger` on over-limit
- Between-block "+" circles appear on hover, also centered on spine
- Mobile (≤640px): spine + dots hidden, thin 1px accent bar on left edge instead

### D6: Double-empty-line auto-split deferred

Decided to defer this to Session 4. The edge cases (intent detection, undo interaction, IME interruption, user confusion) are significant and the paste-split + Cmd+Enter already provide fast splitting.

## Quality Gate Results

| Check | Result |
|-------|--------|
| `cd dashboard && npm run check` | Pass (0 errors, 6 pre-existing warnings) |
| `cd dashboard && npm run build` | Pass |
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (all tests) |
| `cargo clippy --workspace -- -D warnings` | Pass |

## Exit Criteria Status

| Criterion | Status |
|-----------|--------|
| Three-post thread can be written and reordered without "add tweet" button | Done — between-block "+" and keyboard shortcuts |
| Cmd+Enter splits at cursor in thread mode | Done |
| Backspace at offset 0 merges with previous | Done |
| Alt+Up/Alt+Down reorder blocks | Done |
| Paste multi-paragraph text creates separate blocks | Done — when target block is empty |
| Media attachment per block works | Done — unchanged |
| Autosave and recovery work with thread data | Done — ThreadBlock[] format unchanged |
| Submit serializes correctly | Done — ComposeRequest path unchanged |
| Spine visual renders on desktop, hides on mobile | Done |
| Command palette shows updated language | Done — "post" instead of "card" |
| Split, merge, reorder, media, and validation work without breaking autosave or submit | Done |
| Lane looks and feels like a writing surface, not a form builder | Done — no boxed cards, no borders, continuous spine |

## Cursor Management Edge Cases (documented)

### Working behaviors

1. **Split at cursor**: Uses textarea's `selectionStart`. Snaps to word boundary within ±10 chars. Both halves must be non-empty after trim, or it adds an empty block instead.
2. **Merge cursor placement**: After merge, `requestAnimationFrame` → DOM query → `setSelectionRange(joinPoint)`. Join point = length of first block + separator.
3. **Focus after add/split/duplicate**: `requestAnimationFrame` + `document.querySelector('[data-block-id="..."] textarea')`. Cursor goes to position 0.
4. **Paste split**: `e.preventDefault()` stops browser paste. Text distributed across blocks. Cursor lands at end of last new block.
5. **Tab navigation**: Wraps at boundaries (Tab at last = no-op, Shift+Tab at first = no-op).
6. **IME guard**: `e.isComposing` check prevents split shortcuts from firing during IME composition.

### Known edge cases for follow-up

1. **Undo/Redo**: Browser's Ctrl+Z doesn't understand multi-block splits. After split, undo restores new block text but doesn't re-merge. Custom undo stack needed.
2. **Textarea auto-resize**: `rows={3}` is fixed. Long text scrolls within the textarea. Should auto-resize based on `scrollHeight`.
3. **Paste with media**: Only text paste is intercepted. Image paste from clipboard is not handled (separate feature).
4. **Selection spanning blocks**: Not possible with separate `<textarea>` elements. Non-issue.

## Scope Cuts

| Feature | Reason | Target |
|---------|--------|--------|
| Double-empty-line auto-split | Edge case complexity, UX confusion risk | Session 4 candidate |
| Textarea auto-resize | Visual improvement, not blocking core flow | Session 4 |
| Custom undo stack for thread ops | Complex, browser undo partially works | Session 4+ |
| `HomeComposerHeader` action cluster | Separate design task | Session 4 |
| Avatar images on spine dots | Requires API integration, calmer with dots | Session 4+ |

## Inputs for Session 04

Session 04 should read:

1. `ThreadFlowLane.svelte` — the new smart orchestrator
2. `ThreadFlowCard.svelte` — updated card with spine dot and between-zone
3. `threadOps.ts` — pure thread operations
4. `ComposeWorkspace.svelte` — keyboard handler changes
5. `home-surface-plan.md` — remaining acceptance criteria
6. `ui-architecture.md` — HomeComposerHeader spec

### Recommended Session 04 focus areas

1. **Textarea auto-resize**: Listen to `input` events, adjust textarea height based on `scrollHeight`. Critical for writing comfort.
2. **HomeComposerHeader action cluster**: Replace floating submit pill with the warm Schedule / cool Publish cluster per the charter.
3. **Double-empty-line auto-split**: Now that Cmd+Enter split works, this is a progressive enhancement. Guard with `e.isComposing`, add visual feedback.
4. **Getting-started tips**: Progressive disclosure module for empty state on home surface.
5. **Cmd+N interception**: Global shortcut to start new compose from anywhere in the app.
