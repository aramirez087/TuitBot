# Session 03 — Thread Composer Foundation UI

**Date:** 2026-02-27
**Status:** Complete

---

## Overview

Session 03 builds the premium thread composer foundation — a two-pane WYSIWYG editing experience with card-based thread composition and live preview. This is the core UI delivery that exceeds Typefully's thread editor by providing real-time visual feedback, per-card validation, and auto-save/recovery.

## Components Built

### TweetPreview.svelte (New)

Stateless tweet card renderer used in the side-panel preview.

**Props:**
- `text: string` — tweet content
- `mediaPaths: string[]` — media file paths for image grid
- `index: number` — zero-based position in thread
- `total: number` — total tweets in thread
- `handle?: string` — display handle (defaults to `@you`)

**Features:**
- Avatar placeholder + handle header
- Whitespace-preserving text display with word wrapping
- Media grid layout: 1 (full), 2 (50/50), 3 (2+1), 4 (2x2)
- Thread connector line between cards
- Proper ARIA labels (`article` with descriptive label)

### ThreadComposer.svelte (New)

Card-based thread editor with per-card character counting and validation.

**Props:**
- `initialBlocks?: ThreadBlock[]` — seed data for editing existing threads
- `onchange: (blocks: ThreadBlock[]) => void` — called on every mutation
- `onvalidchange: (valid: boolean) => void` — submit-readiness signal

**Internal State:**
- `blocks: ThreadBlock[]` — managed internally, synced from parent via ID-based change detection
- `focusedBlockId: string | null` — active card highlight

**Features:**
- Add/remove/edit tweet cards with stable UUIDs via `crypto.randomUUID()`
- Per-card character counter using `tweetWeightedLen()` (URL-aware)
- Warning state at 260+ chars, error state at 280+
- Minimum 2 blocks enforced (remove button hidden when exactly 2)
- Validation summary with real-time error messages
- Drag handle placeholder (visual only — Session 04 activates DnD)
- Focus tracking with accent border highlight
- Thread connector lines between cards
- Full ARIA: `role="region"`, per-textarea labels, `aria-live` counters

**Exported Methods:**
- `getBlocks(): ThreadBlock[]` — read current state
- `setBlocks(blocks: ThreadBlock[])` — programmatic update

### ComposeModal.svelte (Refactored)

Refactored from 787 lines to integrate the new two-pane thread mode while preserving all existing tweet composition behavior.

**New Capabilities:**
- Two-pane layout in thread mode: editor (left) + live preview (right)
- Uses `ThreadBlock[]` instead of `string[]` for thread state
- Constructs `ComposeRequest` with `blocks` field for API integration
- Auto-save to `localStorage` with 500ms debounce and 7-day TTL
- Draft recovery prompt on modal open if unsaved content detected
- Responsive: collapses to single-column below 768px
- AI Assist generates blocks with proper UUIDs when in thread mode
- Proper accessibility: `role="dialog"`, `aria-modal`, `aria-label`, tablist semantics on mode tabs

**Removed:**
- `threadParts: string[]` — replaced by `threadBlocks: ThreadBlock[]`
- `addThreadPart`, `removeThreadPart`, `updateThreadPart` — delegated to ThreadComposer
- Old thread-specific CSS (`.thread-compose`, `.thread-part`, etc.)

## Pages Updated

### content/+page.svelte

- `handleCompose` callback typed to accept `ComposeRequest` (from `$lib/api`)
- Seamless passthrough to `composeContent()` which already accepts `ComposeRequest`

### drafts/+page.svelte

- Imports `parseThreadContent`, `isBlocksPayload` from `$lib/api`
- Detects blocks-format drafts and renders numbered block previews instead of raw JSON
- Both the main display and the scheduling view handle blocks format
- Added `.thread-preview-compact`, `.thread-block-preview`, `.block-num` CSS

## Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D-1 | Two-pane via CSS grid within modal | Avoids routing complexity. Modal already manages backdrop/focus/escape. Simple `grid-template-columns: 1fr 1fr` with 768px breakpoint. |
| D-2 | ThreadComposer owns block state, parent owns submit | Self-contained and testable. Parent reads latest blocks on submit via `onchange` callback. |
| D-3 | Client-generated UUIDs via `crypto.randomUUID()` | Matches Session 02 API contract. Available in all modern browsers and Tauri WebView. |
| D-4 | Auto-save to localStorage with namespaced key | Zero backend cost, instant. `tuitbot:compose:draft` key with 7-day TTL. |
| D-5 | ID-based sync for parent-to-child block updates | Prevents infinite loops: child edits produce blocks with same IDs, so sync skips. AI-generated blocks have new IDs, triggering sync. |
| D-6 | ComposeModal sends both `content` (legacy) and `blocks` (new) | Backward compatibility: existing publish workflow reads `content` as `["tweet1","tweet2"]`. Blocks provide richer structure for future use. |
| D-7 | Responsive single-column below 768px | Two-pane is not viable on mobile. Preview pane stacks below editor with border separator. |

## Superiority vs Typefully

| Dimension | Typefully | Tuitbot (Session 03) | Winner |
|-----------|-----------|---------------------|--------|
| Writing speed | Tab between tweets, no preview while typing | Side-by-side preview updates as you type | Tuitbot |
| Structural control | Reorder via drag (good) | Card numbers + placeholder (Session 04 adds DnD) | Tie |
| Feedback clarity | Character count per tweet, no visual warning states | Per-card counter with 260+ warning + 280+ error + validation summary | Tuitbot |
| Accessibility | Basic keyboard | `role="dialog"`, `aria-modal`, `aria-live` counters, tablist, labeled textareas | Tuitbot |
| Draft safety | Server-side autosave | Client-side autosave (500ms) + recovery prompt on reopen | Tuitbot |

## Quality Gates

| Gate | Result |
|------|--------|
| `npm run check` | 0 errors, 5 warnings (all pre-existing) |
| `npm run build` | Success |
| No Rust changes | N/A — frontend-only session |
