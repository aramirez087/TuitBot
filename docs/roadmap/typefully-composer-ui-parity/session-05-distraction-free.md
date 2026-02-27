# Session 05 — Distraction-Free Writing Mode

**Date:** 2026-02-27
**Session:** 05 — Focus Mode, Command Palette, AI Assist, Shortcuts
**Status:** Complete

---

## Overview

Session 05 delivers a distraction-free writing system that prioritizes keyboard efficiency over pointer-heavy interactions. Five capabilities were added:

1. **Focus mode** — full-viewport compose layout via `Cmd+Shift+F`
2. **Command palette** — `Cmd+K` fuzzy-search action launcher
3. **Keyboard shortcut registry** — centralized shortcut matching utility
4. **Inline AI assist** — `Cmd+J` to improve selected text via `/api/assist/improve`
5. **"From notes" helper** — local text input that transforms rough notes into compose content
6. **Tab/Shift+Tab card navigation** — navigate between thread cards without mouse

---

## Architecture

### Shortcut Registry (`shortcuts.ts`)

A stateless utility module with three core functions:

- `matchEvent(e, combo)` — checks if a `KeyboardEvent` matches a combo string like `"cmd+shift+f"`
- `formatCombo(combo)` — returns platform-appropriate display strings (e.g., `⌘⇧F` on Mac, `Ctrl+Shift+F` on Windows)
- `isMac()` — platform detection with caching

The combo format uses `+`-separated tokens: modifiers (`cmd`, `shift`, `alt`) followed by a key name matching `event.key`. `cmd` maps to `metaKey` on Mac, `ctrlKey` elsewhere.

A `SHORTCUT_CATALOG` constant exports all shortcuts with labels and categories for the CommandPalette.

**Design decision:** Stateless utilities rather than a global registry avoids lifecycle management (register/unregister on mount/destroy). Each component checks its own shortcuts.

### Command Palette (`CommandPalette.svelte`)

An overlay rendered inside the modal (not a portal) at z-index 10 relative to the modal.

**Props:** `open`, `mode`, `onclose`, `onaction(actionId: string)`

**Design:**
- Absolutely positioned within the modal with semi-transparent backdrop
- 13 actions organized into 4 categories: Mode, Compose, AI, Thread
- Thread-specific actions filtered out when `mode === 'tweet'`
- Search via case-insensitive `includes()` on label and category
- Keyboard navigation: Arrow keys for selection, Enter to execute, Escape to close
- Each action shows icon, label, and optional shortcut hint from `formatCombo()`
- ARIA: `role="dialog"`, `role="listbox"`, `role="option"`, `aria-activedescendant`

**Action dispatch:** Parent receives `onaction(id)` and maps to handler. CommandPalette is stateless.

### Focus Mode

CSS-only implementation via `.modal.focus-mode` class on the existing modal div:

```css
.modal.focus-mode {
  width: 100vw;
  max-width: 100vw;
  height: 100vh;
  max-height: 100vh;
  border-radius: 0;
  display: flex;
  flex-direction: column;
}
```

No routing, state transfer, or new components needed. Toggle button in header shows `Maximize2`/`Minimize2` icons.

### Inline AI Assist

**Tweet mode:** Gets textarea selection via `selectionStart`/`selectionEnd`. If text is selected, only that portion is sent to `/api/assist/improve` and the result replaces the selection. If no selection, the entire tweet text is improved.

**Thread mode:** Delegates to `ThreadComposer.handleInlineAssist()` which operates on the focused card's textarea selection using the same logic.

Visual feedback: Thread cards show `.assisting` class (accent border, reduced opacity, pointer-events disabled) during API calls.

### From Notes Helper

A collapsible section in the modal body with:
- Textarea for pasting rough notes
- Generate button that calls `api.assist.improve()` (tweet mode) or `api.assist.thread()` (thread mode)
- Confirmation dialog if existing content will be overwritten
- Accessible via command palette (`ai-from-notes` action) or footer button

No backend changes needed — uses existing endpoints with context parameter.

### Tab Navigation

Added to `handleCardKeydown` in ThreadComposer:
- `Tab` moves focus to next card's textarea
- `Shift+Tab` moves to previous card
- Boundary behavior: stays on first/last card (no wrap-around)
- Only intercepted when no other modifiers (Alt, Cmd, Ctrl) are held

---

## Complete Shortcut Reference

| Combo | Label | Category | Scope |
|-------|-------|----------|-------|
| `Cmd+Enter` | Submit / Post | Compose | always |
| `Cmd+Shift+F` | Toggle focus mode | Mode | always |
| `Cmd+K` | Open command palette | Mode | always |
| `Cmd+J` | AI improve selection | AI | always |
| `Escape` | Close (layered) | Mode | always |
| `Cmd+Shift+N` | Switch to tweet mode | Mode | always |
| `Cmd+Shift+T` | Switch to thread mode | Mode | always |
| `Alt+ArrowUp` | Move card up | Thread | thread |
| `Alt+ArrowDown` | Move card down | Thread | thread |
| `Cmd+D` | Duplicate card | Thread | thread |
| `Cmd+Shift+S` | Split at cursor | Thread | thread |
| `Cmd+Shift+M` | Merge with next | Thread | thread |
| `Tab` | Next card | Thread | thread |
| `Shift+Tab` | Previous card | Thread | thread |

### Escape Priority (layered)

1. Command palette open → close palette
2. From notes open → close from-notes section
3. Focus mode active → exit focus mode
4. Otherwise → close modal

---

## File Changes

### New Files

| File | Lines | Purpose |
|------|-------|---------|
| `dashboard/src/lib/utils/shortcuts.ts` | 104 | Stateless keyboard shortcut matching and formatting |
| `dashboard/src/lib/components/CommandPalette.svelte` | 236 | Command palette overlay with fuzzy search |

### Modified Files

| File | Change Summary |
|------|----------------|
| `dashboard/src/lib/components/ThreadComposer.svelte` | +Tab/Shift+Tab navigation, +`handleInlineAssist()` export, +`handlePaletteAction()` export, +`.assisting` CSS, +`assistingBlockId` state |
| `dashboard/src/lib/components/ComposeModal.svelte` | +Focus mode (state, CSS, toggle button), +CommandPalette integration, +enhanced keydown handler, +inline AI assist, +from-notes section, +footer notes button |

---

## Superiority Assessment

| Dimension | Typefully | Tuitbot (after S05) | Winner |
|-----------|-----------|---------------------|--------|
| Writing speed | Basic editor, no AI | Focus mode + AI improve + from-notes + Cmd+Enter | **Tuitbot** |
| Structural control | Click-to-reorder | DnD + keyboard + power actions + Tab nav + command palette | **Tuitbot** |
| Feedback clarity | Basic counters | Per-card counters + live preview + ARIA + assist state | **Tuitbot** |
| Accessibility | Limited keyboard support | 14 shortcuts + command palette + Tab nav + focus management | **Tuitbot** |
