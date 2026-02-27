# Session Execution Map: Sessions 02-08

**Date:** 2026-02-27
**Reference:** `charter.md` for architecture decisions, `ui-gap-audit.md` for gap IDs.

---

## Session 02: Data Model & API Contract

**Goal:** Typed thread block schema with stable IDs, backwards-compatible compose/draft endpoints.

**Gaps addressed:** G-01 (data layer), G-03 (data layer)

### Files to Modify

| File | Change |
|------|--------|
| `crates/tuitbot-server/src/routes/content/compose.rs` | Add `blocks: Option<Vec<ThreadBlock>>` to `ComposeRequest`. When present, validate block ordering, per-block character limits, and per-block media paths. Fall back to `content` string when `blocks` is absent. Update `compose()` handler to normalize both formats into internal representation |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Add `blocks` support to `CreateDraftRequest` and `EditDraftRequest`. Draft edit accepts either `content` or `blocks`. Draft list response includes parsed `blocks` when content is structured |
| `crates/tuitbot-core/src/content/mod.rs` | Add `ThreadBlock` struct: `{id: String, text: String, media_paths: Vec<String>, order: u32}`. Add `validate_thread_blocks()` function |
| `dashboard/src/lib/api.ts` | Add `ThreadBlock` interface: `{id: string, text: string, media_paths: string[], order: number}`. Update `ComposeRequest` with optional `blocks` field. Add typed response for draft with blocks |

### Files to Create

| File | Purpose |
|------|---------|
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Contract tests: legacy compose (string content), new compose (blocks), mixed validation, backwards compatibility, block ID uniqueness, ordering validation |

### Quality Gates

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
```

### Exit Criteria

- Backend accepts `blocks` field for thread compose and draft endpoints
- Legacy `content` string flows continue to work (no regressions)
- Block IDs survive roundtrip: compose with blocks, read back, IDs match
- Per-block media paths validated (each block's media respects 4-image / 1-GIF/video limits)
- Contract tests pass for all new and legacy flows
- TypeScript types updated and type-check passes

### Dependencies

- None (first implementation session)

---

## Session 03: Thread Composer Foundation UI

**Goal:** Visual tweet-card composer with edit + preview side-by-side, auto-save recovery.

**Gaps addressed:** G-01, G-04, G-06

### Files to Create

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/ThreadComposer.svelte` | Card-based thread editor. Each card: textarea, character counter, card number, remove button, drag handle placeholder (non-functional yet). Props: `blocks: ThreadBlock[]`, callbacks for add/remove/update. Emits block array changes to parent. Max 400 lines |
| `dashboard/src/lib/components/TweetPreview.svelte` | Read-only tweet card render. Props: `text: string`, `media_paths: string[]`, `index: number`, `total: number`, `handle: string`. Renders avatar placeholder, handle, text, media grid (1-4 images or video), relative timestamp. Max 200 lines |

### Files to Modify

| File | Change |
|------|--------|
| `dashboard/src/lib/components/ComposeModal.svelte` | Refactor thread mode to delegate to `ThreadComposer`. Keep tweet mode inline. Add auto-save to localStorage (debounced 500ms). Add recovery prompt on open. Add side-panel preview layout (editor left, preview right). Reduce to ~350 lines |
| `dashboard/src/routes/(app)/content/+page.svelte` | Wire updated ComposeModal (no API changes, just updated component interface) |
| `dashboard/src/routes/(app)/drafts/+page.svelte` | Wire ComposeModal for draft editing (open compose with draft content pre-filled) |

### Quality Gates

```bash
cd dashboard && npm run check
```

### Exit Criteria

- Thread compose shows card-based editor with per-card textarea and character counter
- Live preview panel renders tweet cards alongside editor
- Auto-save fires on keystroke (verify via localStorage inspection)
- Recovery prompt appears when opening compose with unsaved content
- Existing tweet compose mode works identically to current (no regression)
- All thread validation prevents invalid submits (empty cards, single-card thread, over-limit)
- ComposeModal stays under 400 lines

### Dependencies

- Session 02 (ThreadBlock type in api.ts, blocks field in API)

---

## Session 04: Reorder & Media Placement

**Goal:** Drag-and-drop + keyboard reorder, per-tweet media, power actions (duplicate, split, merge).

**Gaps addressed:** G-02, G-03, G-10, G-11

### Files to Create

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/MediaSlot.svelte` | Per-tweet media attachment zone. Props: `media: AttachedMedia[]`, `canAttach: boolean`, callbacks for add/remove. Supports file picker button and drag-drop zone. Renders thumbnails. Max 200 lines |

### Files to Modify

| File | Change |
|------|--------|
| `dashboard/src/lib/components/ThreadComposer.svelte` | Add reorder: drag handle per card (HTML5 native drag-and-drop), keyboard reorder (Alt+Up/Down). Add power actions: duplicate card (Cmd+D), split at cursor (Cmd+Shift+S), merge with next (Cmd+Shift+M). Integrate MediaSlot per card. Update block ordering on every operation |
| `dashboard/src/lib/components/ComposeModal.svelte` | Wire media upload to per-tweet assignment in thread mode. Keep existing media flow for tweet mode |

### Quality Gates

```bash
cd dashboard && npm run check
```

### Exit Criteria

- Drag-and-drop reorder works: grab handle, drag to new position, drop. Block order persisted
- Keyboard reorder works: Alt+Up moves card up, Alt+Down moves card down. Focus follows card
- Duplicate: Cmd+D on focused card creates identical card below with new UUID
- Split: Cmd+Shift+S splits card at cursor into two cards
- Merge: Cmd+Shift+M merges focused card with card below
- Per-tweet media: each card has its own media slot, media follows card on reorder
- Media drag-drop: dropping files onto a card's media slot uploads and attaches
- Reordered thread submits correctly (block order field determines tweet sequence)

### Dependencies

- Session 03 (ThreadComposer exists, card-based layout)

---

## Session 05: Distraction-Free Writing Mode

**Goal:** Full-viewport compose, command palette, enhanced AI assist, keyboard shortcut system.

**Gaps addressed:** G-05, G-07, G-08, G-09

### Files to Create

| File | Purpose |
|------|---------|
| `dashboard/src/lib/components/CommandPalette.svelte` | Cmd+K action palette. Fuzzy substring search over actions. Categories: Compose, Thread, Mode, AI. Each action: label, shortcut hint, handler. Rendered as overlay within ComposeModal. Max 300 lines |
| `dashboard/src/lib/utils/shortcuts.ts` | Keyboard shortcut registry. Functions: `registerShortcut(combo, handler)`, `unregisterShortcut(combo)`, `handleKeyEvent(event)`. Supports Mac (Cmd) and Windows (Ctrl) modifiers. Prevents conflicts with browser defaults. Max 100 lines |

### Files to Modify

| File | Change |
|------|--------|
| `dashboard/src/lib/components/ComposeModal.svelte` | Add focus mode toggle (Cmd+Shift+F): full-viewport state, hide modal chrome. Integrate CommandPalette. Register all shortcuts via `shortcuts.ts`. Add Cmd+Enter to submit, Cmd+J for AI assist |
| `dashboard/src/lib/components/ThreadComposer.svelte` | Integrate command palette actions for thread operations. Add inline AI assist: select text + Cmd+J sends selection to `/api/assist/improve`. Tab/Shift+Tab to navigate between cards |

### Quality Gates

```bash
cd dashboard && npm run check
```

### Exit Criteria

- Focus mode: Cmd+Shift+F toggles full-viewport compose. All functionality preserved in focus mode
- Command palette: Cmd+K opens palette. Search filters actions. Selecting action executes it. Escape closes palette
- Shortcuts: Cmd+Enter submits, Cmd+J triggers AI assist, Cmd+D duplicates, Cmd+Shift+S splits, Cmd+Shift+M merges, Alt+Up/Down reorders, Cmd+Shift+F toggles focus mode. Total: >= 15 shortcuts
- Inline AI: select text in textarea, Cmd+J improves selected text only (not entire tweet)
- Tab/Shift+Tab moves focus between thread cards
- Shortcuts do not conflict with browser defaults in Tauri or Chrome

### Dependencies

- Session 04 (power actions exist, media slots exist)

---

## Session 06: Responsive & Accessible Polish

**Goal:** Mobile layouts, keyboard-only flows, WCAG AA contrast, reduced motion, ARIA coverage.

**Gaps addressed:** G-04 (mobile preview), G-05 (accessibility), all accessibility scorecard metrics

### Files to Modify

| File | Change |
|------|--------|
| `dashboard/src/lib/components/ComposeModal.svelte` | Mobile breakpoints (< 768px: full viewport). Focus trap: Tab cycles within modal. `role="dialog"`, `aria-label`, `aria-modal="true"`. Focus returns to trigger on close. Remove `svelte-ignore a11y_*` directives (line 256) |
| `dashboard/src/lib/components/ThreadComposer.svelte` | Mobile card layout (stacked, larger touch targets >= 44px). ARIA: `aria-live="polite"` on character counters, `aria-label` on buttons. Drag handle accessible via keyboard |
| `dashboard/src/lib/components/TweetPreview.svelte` | Responsive preview: on mobile, toggle between edit and preview (not side-by-side). `aria-label` on preview region |
| `dashboard/src/lib/components/CommandPalette.svelte` | Mobile layout: full-width. Focus trap within palette. `role="dialog"`, `aria-label` |
| `dashboard/src/lib/components/MediaSlot.svelte` | Touch targets >= 44px. `aria-label` on upload button and remove buttons |
| `dashboard/src/app.css` | Focus indicator tokens (`:focus-visible` ring). `prefers-reduced-motion` media query to disable all transitions/animations. Verify contrast ratios on all design tokens |

### Quality Gates

```bash
cd dashboard && npm run check
```

### Exit Criteria

- Mobile (< 768px): compose modal is full viewport, cards are stacked, touch targets >= 44px
- Focus trap: Tab never escapes modal, cycles through all interactive elements
- Focus return: closing modal returns focus to the button/element that opened it
- ARIA: `role="dialog"`, `aria-modal="true"`, `aria-label` on modal. `aria-live` on character counters and error messages
- Contrast: all text/background pairs meet 4.5:1 (WCAG AA). Verified with contrast checker tool
- Reduced motion: `prefers-reduced-motion` disables all CSS transitions and animations
- No `svelte-ignore a11y_*` directives remain in any modified component
- Axe audit: zero critical or serious violations on compose modal in all states

### Dependencies

- Session 05 (all interactive elements exist, command palette exists)

---

## Session 07: Docs & Adoption Readiness

**Goal:** Documentation reflects shipped UX, migration notes for existing users, keyboard shortcut reference.

**Gaps addressed:** Documentation alignment

### Files to Create

| File | Purpose |
|------|---------|
| `docs/roadmap/typefully-composer-ui-parity/shortcut-cheatsheet.md` | Complete keyboard shortcut reference. Organized by category (Compose, Thread, Navigation, AI, Mode). Each entry: action, Mac shortcut, Windows shortcut, description |

### Files to Modify

| File | Change |
|------|--------|
| `docs/composer-mode.md` | Update with: thread composer workflow (card-based editor), preview panel, keyboard shortcuts, command palette, focus mode, auto-save recovery, per-tweet media. Reflect current UI accurately |

### Quality Gates

```bash
# Run all gates if any code is touched during doc updates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
```

### Exit Criteria

- `composer-mode.md` accurately describes the shipped UI (no references to old textarea-array behavior)
- Shortcut cheatsheet lists all registered shortcuts with correct key combinations
- No stale documentation references to removed UI patterns

### Dependencies

- Session 06 (all UI work complete, shortcuts finalized)

---

## Session 08: Final Validation & Go/No-Go

**Goal:** End-to-end validation of all requirements, superiority scorecard evaluation, go/no-go verdict.

### Files to Create

| File | Purpose |
|------|---------|
| `docs/roadmap/typefully-composer-ui-parity/traceability-matrix.md` | Maps every gap (G-01 through G-11) to implementation evidence (file, line, test). Pass/fail verdict per gap |
| `docs/roadmap/typefully-composer-ui-parity/superiority-scorecard-final.md` | Final scorecard with measured values for every metric. Evidence (screenshots, keystroke counts, timing data, audit results). Overall verdict |
| `docs/roadmap/typefully-composer-ui-parity/final-go-no-go-report.md` | Executive summary: what shipped, what didn't, overall verdict (go/no-go), known limitations, recommended follow-up work |

### Quality Gates

```bash
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check
```

### Exit Criteria

- All 6 critical gaps (G-01 through G-06) verified as implemented with evidence
- All 5 important gaps (G-07 through G-11) verified as implemented with evidence
- Superiority scorecard shows >= Typefully in all 4 dimensions, clear win in >= 3
- All quality gates pass
- Go/no-go verdict issued with rationale

### Dependencies

- Session 07 (all implementation and documentation complete)

---

## Dependency Graph

```
Session 01 (Charter)
    |
Session 02 (Data Model & API)
    |
Session 03 (Thread Composer UI)
    |
Session 04 (Reorder & Media)
    |
Session 05 (Focus Mode & Palette)
    |
Session 06 (Responsive & A11y)
    |
Session 07 (Docs)
    |
Session 08 (Validation)
```

Sessions are strictly sequential. Each session depends on the prior session's output. No sessions can run in parallel.

---

## File Creation/Modification Summary

| Session | Files Created | Files Modified | Total |
|---------|--------------|----------------|-------|
| 02 | 1 (contract tests) | 4 (compose.rs, drafts.rs, content/mod.rs, api.ts) | 5 |
| 03 | 2 (ThreadComposer, TweetPreview) | 3 (ComposeModal, content page, drafts page) | 5 |
| 04 | 1 (MediaSlot) | 2 (ThreadComposer, ComposeModal) | 3 |
| 05 | 2 (CommandPalette, shortcuts.ts) | 2 (ComposeModal, ThreadComposer) | 4 |
| 06 | 0 | 6 (ComposeModal, ThreadComposer, TweetPreview, CommandPalette, MediaSlot, app.css) | 6 |
| 07 | 1 (shortcut-cheatsheet.md) | 1 (composer-mode.md) | 2 |
| 08 | 3 (traceability, scorecard-final, go-no-go) | 0 | 3 |
| **Total** | **10** | **18** | **28** |
