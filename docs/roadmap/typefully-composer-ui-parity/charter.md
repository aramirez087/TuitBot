# Typefully Composer UI Parity — Project Charter

**Status:** Active
**Created:** 2026-02-27
**Owner:** Engineering
**Initiative:** Ship a composer UX that is measurably better than Typefully in writing speed, structural control, feedback clarity, and accessibility — while preserving all existing API compatibility.

---

## Executive Summary

Tuitbot's current compose experience is a 520px modal with discrete textareas, a file-picker for media, and a single AI Assist button. Typefully offers an inline document editor with drag-and-drop reordering, WYSIWYG preview, command palette, and focus mode. This initiative closes the gap and establishes superiority in four measurable dimensions: writing speed, structural control, feedback clarity, and accessibility. The scope is strictly UI parity — no new AI engines, no cross-platform posting, no collaborative editing.

---

## Scope

### In Scope

- **ComposeModal refactor**: Extract thread editing into `ThreadComposer.svelte`, keep tweet mode inline.
- **Thread block model**: Stable block IDs (UUID), structured JSON storage, backwards-compatible API.
- **Live preview**: `TweetPreview.svelte` side panel showing tweet cards with avatar, handle, text, media grid, timestamp.
- **Thread operations**: Reorder (drag-and-drop + keyboard), duplicate, split, merge.
- **Per-tweet media**: Media attachment zones on each thread card.
- **Distraction-free mode**: Full-viewport compose, toggle via `Cmd+Shift+F`.
- **Command palette**: `CommandPalette.svelte` with `Cmd+K` trigger and declarative action registry.
- **Keyboard shortcuts**: Full coverage — reorder, submit, AI assist, duplicate, split, navigate between cards.
- **Auto-save**: localStorage debounced at 500ms, recovery prompt on next open.
- **Inline AI assist**: Select text and improve via `Cmd+J`, using existing `/api/assist/improve`.
- **Responsive layout**: Mobile breakpoints, focus trap, ARIA landmarks, WCAG AA contrast.
- **Backend contract updates**: Additive `blocks` field on compose/draft endpoints for thread ordering and per-tweet media.

### Non-Goals

These are explicitly excluded from this initiative:

1. **Ghostwriter engine** — Voice learning, hook detection, custom AI prompts, writing style adaptation. The AI Assist feature uses existing `/api/assist/*` endpoints only.
2. **Filesystem ingestion / RAG pipeline** — No document ingestion, no retrieval-augmented generation, no knowledge base building.
3. **Background seed systems** — No automated content seeding, no autonomous scheduling based on AI recommendations.
4. **Watchtower monitoring** — No real-time competitor tracking, no automated alerts.
5. **Cross-posting** — No LinkedIn, Threads, Bluesky, or Mastodon support. X (Twitter) only.
6. **Collaborative editing** — No multi-user draft editing, no draft sharing, no draft locking.
7. **Polls, photo tags, community posts** — X-specific features beyond tweets, threads, and media.
8. **Native Mac app or Raycast extension** — Desktop delivery remains Tauri-based.

---

## Architecture Decisions

### A-1: ThreadComposer as Separate Component

**Decision:** Extract thread editing from `ComposeModal.svelte` into a dedicated `ThreadComposer.svelte` component.

**Rationale:** ComposeModal is 787 lines (exceeding the 400-line Svelte limit). Thread editing adds card management, reorder, per-tweet media, and preview — another 400+ lines. ThreadComposer owns all thread-specific state. ComposeModal orchestrates mode switching (tweet vs. thread) and delegates to ThreadComposer for thread mode.

**Impact:** `ComposeModal.svelte` shrinks to ~350 lines. Thread logic is isolated and testable.

### A-2: Stable Block IDs with UUID

**Decision:** Each tweet block in a thread gets a client-generated UUID. Backend validates ordering by accepting `blocks: [{id, text, media_paths, order}]`.

**Rationale:** Without stable IDs, reordering requires array index tracking which breaks on concurrent edits and makes optimistic UI updates fragile. UUIDs enable: (a) reorder by updating `order` field without re-indexing, (b) per-block media references that survive reorder, (c) future extensibility for collaborative editing.

**Impact:** New `ThreadBlock` TypeScript type in `api.ts`. Backend `ComposeRequest` adds optional `blocks` field.

### A-3: Structured JSON for Thread Data

**Decision:** Thread content stored as structured JSON (`blocks` array of objects) rather than `\n---\n`-joined strings.

**Rationale:** Current thread storage (`compose.rs` line 104) joins tweets with `\n---\n`, losing per-tweet metadata. Structured JSON preserves block IDs, ordering, and per-block media paths. Backwards-compatible: server accepts both legacy `content` (JSON string array) and new `blocks` field.

**Impact:** Server `ComposeRequest` struct gains `blocks: Option<Vec<ThreadBlock>>`. Thread validation checks either field. No database migration required — content column stores the serialized form.

### A-4: Side-Panel Preview (Not Inline WYSIWYG)

**Decision:** Live preview renders in a side panel via `TweetPreview.svelte`, not as inline WYSIWYG editing.

**Rationale:** Typefully uses inline WYSIWYG via contenteditable. A side panel is simpler to implement, avoids contenteditable complexity (cursor management, paste handling, undo history), keeps editing in native `<textarea>` (reliable input handling), and provides clear visual separation between authoring and preview. This is a deliberate UX differentiation — the editor stays fast and reliable while preview stays high-fidelity.

**Impact:** New `TweetPreview.svelte` component. Purely presentational — receives text, media, and metadata as props, renders tweet-card UI, no backend calls.

### A-5: Command Palette with Cmd+K

**Decision:** Implement `CommandPalette.svelte` triggered by `Cmd+K` (Mac) / `Ctrl+K` (Windows). Actions registered declaratively.

**Rationale:** Typefully has a command palette for common actions. Ours will be keyboard-first with search, categories, and hotkey hints. Actions registered via a simple array — no global store pollution. The palette is mounted inside ComposeModal and only active when compose is open.

**Impact:** New `CommandPalette.svelte` component. New `shortcuts.ts` utility for keyboard shortcut registration and handling.

### A-6: Distraction-Free Mode via Modal State

**Decision:** Distraction-free mode is a full-viewport state of `ComposeModal` (not a separate route). Toggle via `Cmd+Shift+F` or button.

**Rationale:** A separate route would require URL management and state transfer. A modal state change is simpler — hide sidebar, header, and modal chrome; show only editor and preview. The modal already handles backdrop click and Escape; focus mode just changes the layout constraints.

**Impact:** ComposeModal gains a `focusMode` state. CSS toggles between 520px and full-viewport layout.

### A-7: localStorage Auto-Save with Debounce

**Decision:** Auto-save draft content to localStorage on every keystroke (debounced 500ms). Recovery prompt on next compose open if unsaved content exists.

**Rationale:** Backend auto-save requires new API endpoints and creates state management complexity (draft status, conflict resolution). localStorage is zero-backend-cost, instant, and recoverable. Clear on successful submit. Cap at 10 recovery slots. TTL of 7 days.

**Impact:** No backend changes. ComposeModal checks localStorage on open, prompts recovery if data exists.

---

## API Compatibility

The existing `POST /api/content/compose` contract is preserved:

```json
{
  "content_type": "tweet" | "thread",
  "content": "string",
  "scheduled_for": "ISO 8601 timestamp (optional)",
  "media_paths": ["string array (optional)"]
}
```

New fields are additive only:

```json
{
  "content_type": "thread",
  "content": "[\"tweet 1\", \"tweet 2\"]",
  "blocks": [
    {"id": "uuid", "text": "tweet 1", "media_paths": [], "order": 0},
    {"id": "uuid", "text": "tweet 2", "media_paths": ["path/to/img.jpg"], "order": 1}
  ],
  "scheduled_for": "2026-03-01T10:00:00",
  "media_paths": ["path/to/img.jpg"]
}
```

When `blocks` is present, it takes precedence over `content` for thread payloads. When absent, the legacy `content` field is used. The `media_paths` top-level field remains for tweet mode and backwards compatibility.

---

## Success Criteria

Superiority must be demonstrated in all 4 dimensions, with clear wins in at least 3:

| Dimension | Target |
|-----------|--------|
| Writing Speed | 5-tweet thread composable in < 3 minutes |
| Structural Control | Reorder + Duplicate + Split + Merge, all keyboard-accessible |
| Feedback Clarity | Live preview + character count + inline validation errors |
| Accessibility | Full keyboard navigation, ARIA landmarks, 4.5:1 contrast, focus trapping |

See `superiority-scorecard.md` for detailed metrics and measurement methods.

---

## Timeline

| Session | Focus | Deliverables |
|---------|-------|-------------|
| 01 | Charter & Gap Audit | This document, gap audit, scorecard, execution map, handoff |
| 02 | Data Model & API Contract | Thread block schema, backwards-compatible endpoints, contract tests |
| 03 | Thread Composer Foundation | ThreadComposer.svelte, TweetPreview.svelte, ComposeModal refactor |
| 04 | Reorder & Media Placement | Drag-and-drop, keyboard reorder, per-tweet media, power actions |
| 05 | Focus Mode & Command Palette | Full-viewport compose, Cmd+K palette, inline AI assist, shortcuts |
| 06 | Responsive & Accessible Polish | Mobile layouts, keyboard-only flows, WCAG AA, animations |
| 07 | Docs & Adoption Readiness | Updated docs, shortcut cheatsheet, migration notes |
| 08 | Final Validation & Go/No-Go | End-to-end validation, scorecard evaluation, verdict |

See `session-execution-map.md` for detailed file targets and dependencies per session.
