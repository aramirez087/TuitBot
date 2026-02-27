# Session 04 — Reorder & Media Placement

**Date:** 2026-02-27
**Session:** 04 — Reorder, Media, Power Actions
**Status:** Complete

---

## Objective

Deliver best-in-class structural control and media choreography that outperforms Typefully's thread editor. Users can reorder tweet cards (drag + keyboard), perform power actions (duplicate/split/merge), and attach media per tweet card with inline validation.

---

## Deliverables

### New Files

| File | Lines | Purpose |
|------|-------|---------|
| `dashboard/src/lib/components/MediaSlot.svelte` | ~175 | Self-contained per-tweet media attachment component with upload, thumbnails, drag-drop file receive, remove, and validation |

### Modified Files

| File | Lines | Change Summary |
|------|-------|----------------|
| `dashboard/src/lib/components/ThreadComposer.svelte` | 351→540 | DnD reorder, keyboard reorder (Alt+Arrow), power actions (duplicate/split/merge), per-card toolbar, MediaSlot integration, ARIA live announcements |
| `dashboard/src/lib/components/ComposeModal.svelte` | 922→926 | Per-block media flattening into top-level `media_paths` on thread submit |

### Unchanged Files (confirmed)

| File | Reason |
|------|--------|
| `dashboard/src/lib/components/TweetPreview.svelte` | Already supports `mediaPaths` rendering with grid layouts |
| `dashboard/src/lib/api.ts` | `ThreadBlock.media_paths` and `ComposeRequest.blocks` already typed |
| Rust crates | No backend changes; per-block media already validated server-side |

---

## Features Implemented

### 1. Drag-and-Drop Reorder

- HTML5 native DnD API on grip handle (GripVertical icon)
- Grip handle: `draggable="true"`, `cursor: grab/grabbing`
- Visual feedback: `.dragging` class (opacity 0.5), `.drop-target` class (accent dashed border)
- On drop: block moves to target position, order fields renormalized 0..N
- State: `draggingBlockId` and `dropTargetBlockId` tracked in component

### 2. Keyboard Reorder

- **Alt+ArrowUp**: Move focused card up one position
- **Alt+ArrowDown**: Move focused card down one position
- Focus follows the moved card via `requestAnimationFrame`
- ARIA live region announces "Tweet moved to position N"
- No-op at boundaries (first/last card)

### 3. Power Actions

#### Duplicate (Cmd+D or toolbar button)
- Creates new block with `crypto.randomUUID()`, copies text and media_paths
- Inserts immediately after current card
- Focus moves to new card

#### Split at Cursor (Cmd+Shift+S or toolbar button)
- Reads `selectionStart` from focused textarea
- Snaps to nearest word boundary within 10 characters
- Guards against empty splits (both halves must have content)
- Original keeps media; new block starts with empty media_paths
- Focus moves to second (new) block

#### Merge with Next (Cmd+Shift+M or toolbar button)
- Combines current block text with next block (joined by `\n`)
- Media from both blocks combined (guarded at max 4)
- If combined media > 4: inline error shown for 3 seconds, merge aborted
- Merge button hidden when only 2 cards remain (minimum thread size)
- Cursor placed at the join point after merge

### 4. Per-Tweet Media Attachment (MediaSlot)

- Each tweet card has an inline MediaSlot below the textarea
- Upload via file picker or drag-and-drop files onto the slot
- Compact thumbnail grid (48x48px per thumb) with remove buttons
- Validation: max 4 media per slot, GIF/video exclusivity, file size limits
- Upload uses same `api.media.upload(file)` as ComposeModal
- `onmediachange` callback updates parent block's `media_paths`

### 5. Per-Block Media in Submit Payload

- `ComposeRequest.blocks[i].media_paths` carries per-block paths
- Top-level `media_paths` set to flattened union for legacy compatibility
- Server's `compose_thread_blocks_flow` already reads per-block media

### 6. Validation & Boundary Handling

- `validationErrors` extended to flag blocks with >4 media
- `canSubmit` checks both character limits and media limits
- Reorder preserves media (entire block object moves)
- Duplicate deep-copies media_paths array
- Split: original keeps media, new block starts empty
- Merge: combines media with >4 guard
- Order normalization (contiguous 0..N) on every structural mutation

---

## Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D4-1 | HTML5 native DnD (no library) | Zero dependencies. Adequate for vertical list reorder. Tauri WebView supports fully. |
| D4-2 | Alt+Arrow for keyboard reorder | Avoids conflict with text selection (Shift+Arrow), cursor (Arrow), word-jump (Cmd+Arrow). Works on Mac and Windows. |
| D4-3 | Power actions in card toolbar | Discoverable on hover/focus. Keyboard shortcuts as secondary access. |
| D4-4 | MediaSlot as self-contained component | Isolates upload complexity from ThreadComposer. Reusable for future inline media. |
| D4-5 | `role="listitem"` on tweet cards | Resolves a11y warnings for DnD handlers on div elements. |
| D4-6 | `role="region"` on MediaSlot | Resolves a11y warning for drag-drop zone. |

---

## Superiority Assessment vs Typefully

| Dimension | Typefully | Tuitbot (after Session 04) | Winner |
|-----------|-----------|---------------------------|--------|
| Structural control | Click-to-reorder only | DnD + keyboard + duplicate/split/merge | Tuitbot |
| Media per tweet | Basic attach | Inline per-card with drag-drop zone, validation | Tuitbot |
| Writing speed | No split/merge | Split at cursor, merge adjacent, duplicate | Tuitbot |
| Feedback clarity | Basic counters | Per-card char counter + media count validation + ARIA announcements | Tuitbot |
| Accessibility | Limited keyboard support | Full keyboard reorder, ARIA live regions, focus management | Tuitbot |

---

## Quality Gate Results

| Check | Status |
|-------|--------|
| `npm run check` | 0 errors, 5 warnings (all pre-existing) |
| `npm run build` | Success |
| No Rust changes | Rust CI gates not applicable |
