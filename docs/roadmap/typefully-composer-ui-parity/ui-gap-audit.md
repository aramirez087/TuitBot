# UI Gap Audit: Typefully vs. Tuitbot Composer

**Date:** 2026-02-27
**Baseline:** Typefully web app (current public release)
**Subject:** Tuitbot dashboard composer (`ComposeModal.svelte`, 787 lines)

---

## Current State Summary

### Tuitbot Composer (ComposeModal.svelte)

- **Layout:** 520px fixed-width modal overlay with backdrop click to close
- **Modes:** Tweet (single textarea) and Thread (array of textareas)
- **Character counting:** Weighted length via `tweetWeightedLen()` with visual indicator
- **Media:** File picker for images/GIF/video, preview thumbnails, tweet mode only
- **Scheduling:** TimePicker component with preferred time slots from schedule config
- **AI Assist:** Single button — generates new tweet or improves existing text via `/api/assist/*`
- **Thread operations:** Add/remove parts only. No reorder, no duplicate, no split/merge
- **Keyboard shortcuts:** Escape to close. No others
- **Preview:** None — raw textarea with character count
- **Auto-save:** None — content lost on modal close
- **Accessibility:** Two `svelte-ignore a11y_*` directives suppressing warnings. No ARIA roles, no focus trap, no keyboard navigation beyond Escape

### Tuitbot Drafts Page (drafts/+page.svelte)

- **Layout:** Separate page with card list, inline editing
- **Actions:** Edit, AI Improve, Schedule (datetime-local input), Publish, Delete
- **Creation:** New Draft button or AI Generate
- **Connection to ComposeModal:** None — drafts are a separate workflow

---

## Gap Analysis

### Critical Gaps (Must Fix for Parity)

These gaps represent features that Typefully users expect as baseline. Without them, the Tuitbot composer feels like a prototype.

#### G-01: Thread Editor Paradigm

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Editor type | Inline document editor with `---` separator creating new tweets | Discrete `<textarea>` array with sequential numbering | Card-based editor: each tweet is a visual card with its own textarea, character counter, and media slot |
| Visual structure | Tweet boundaries visible inline with separator lines | Numbered textareas stacked vertically | Cards with clear visual separation, drag handles, numbering, and action buttons per card |
| State management | Single document model with parsed tweet boundaries | Array of strings (`threadParts: string[]`) | Array of `ThreadBlock` objects with stable UUIDs, text, media, and order fields |

**Session:** 03
**Priority:** P0

#### G-02: Thread Reordering

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Drag-and-drop | Full card drag-and-drop reordering | Not supported | HTML5 native drag-and-drop with dedicated drag handle zone per card |
| Keyboard reorder | Not prominent | Not supported | `Alt+Up` / `Alt+Down` to move focused card. Primary reorder path |
| Visual feedback | Card lifts and placeholder shows drop position | N/A | Drag ghost, drop indicator line, smooth animation on drop |

**Session:** 04
**Priority:** P0

#### G-03: Per-Tweet Media in Threads

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Attachment scope | Per-tweet media attachment in threads | Tweet mode only (`mode === 'tweet' && canAttachMore` guard at line 347) | Per-card media slot in ThreadComposer with file picker and drag-drop upload |
| Media moves with card | Yes — media follows tweet on reorder | N/A | Media is part of the `ThreadBlock` object; reordering a card moves its media |
| Media constraints | Per-tweet: 4 images or 1 GIF/video | Same limits but only for single tweets | Same per-tweet limits applied per card |

**Session:** 04
**Priority:** P0

#### G-04: Live WYSIWYG Preview

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Preview | High-fidelity inline preview with profile pic, handle, timestamps | No preview — raw textarea with character count only | Side-panel `TweetPreview.svelte` showing tweet cards with avatar placeholder, handle, text, media grid, and relative timestamp |
| Preview scope | Inline (editing and preview are the same view) | N/A | Side panel (editor left, preview right). On mobile: toggle between edit and preview |
| Preview accuracy | Close to actual X rendering | N/A | Approximate — focuses on text length, media grid layout, thread structure. Not pixel-matching X |

**Session:** 03
**Priority:** P0

#### G-05: Keyboard Shortcuts

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Submit | Cmd+Enter | Not supported | Cmd+Enter to submit (schedule or post) |
| AI assist | Cmd+J | Not supported | Cmd+J to trigger AI assist (selection-based when text selected, full-tweet when not) |
| Command palette | Cmd+K | Not supported | Cmd+K to open command palette with search and categories |
| Thread navigation | Tab between tweets | Not supported | Tab/Shift+Tab to move focus between thread cards |
| Close | Escape | Escape (line 218-220) | Escape (preserve existing) |
| Total shortcuts | ~12 | 1 (Escape) | 15+ |

**Session:** 05
**Priority:** P0

#### G-06: Auto-Save / Recovery

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Auto-save | Continuous — every change persisted immediately | None — all content lost on modal close (line 66-79 resets state on open) | localStorage auto-save debounced at 500ms. Recovery prompt on next compose open. Clear on successful submit. 7-day TTL |
| Draft persistence | Seamless — closing browser and reopening retains all drafts | ComposeModal has no draft integration; Drafts page is separate | Recovery prompt offers: restore, discard, or save as draft |

**Session:** 03
**Priority:** P0

---

### Important Gaps (Needed for Superiority)

These features go beyond parity and establish Tuitbot as the better tool.

#### G-07: Command Palette

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Trigger | Cmd+K | Not supported | Cmd+K (Mac) / Ctrl+K (Win) |
| Actions | Basic set (format, schedule, AI) | N/A | Compose actions (submit, schedule, AI assist), thread actions (add, duplicate, split, merge, reorder), mode actions (focus mode, preview toggle), navigation |
| Search | Fuzzy search over actions | N/A | Substring search with highlighted matches |
| Hotkey hints | Shows keyboard shortcuts per action | N/A | Right-aligned shortcut hints on each action row |

**Session:** 05
**Priority:** P1

#### G-08: Distraction-Free / Focus Mode

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Full-screen writing | Available — hides nav, shows only editor | Not supported — 520px modal is the only layout | Full-viewport modal state via Cmd+Shift+F. Hides sidebar, header, modal chrome. Shows editor + preview only |
| Toggle | Button in editor chrome | N/A | Cmd+Shift+F keyboard shortcut and button in modal header |

**Session:** 05
**Priority:** P1

#### G-09: Inline AI Assist on Selection

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Selection-based AI | Select text → context menu → improve/rephrase/expand | Single "AI Improve" button that replaces entire tweet text (line 230-235) | Cmd+J with text selected → improve selection only. Without selection → improve full tweet. Uses existing `/api/assist/improve` endpoint |
| Context awareness | Hook detection, voice learning | Basic — sends draft text to improve endpoint | Sends selected text or full text as `draft` param with surrounding context as `context` param |

**Session:** 05
**Priority:** P1

#### G-10: Media Drag-and-Drop Upload

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Upload method | Drag files onto editor area | File picker button only (line 349) | Drag-and-drop zone on each media slot + existing file picker as fallback |
| Visual feedback | Drop zone highlights on drag over | N/A | Border highlight, file type icon, "Drop to attach" label |

**Session:** 04
**Priority:** P1

#### G-11: Power Actions (Duplicate, Split, Merge)

| Aspect | Typefully | Tuitbot Current | Target |
|--------|-----------|-----------------|--------|
| Duplicate | Not prominent | Not supported | Cmd+D: duplicate focused card (copy text + media, insert below) |
| Split | Not prominent | Not supported | Cmd+Shift+S: split card at cursor position into two cards |
| Merge | Not prominent | Not supported | Cmd+Shift+M: merge focused card with the card below (concatenate text) |

**Session:** 04
**Priority:** P1

---

### Nice-to-Have (Post-Initiative)

These features are valuable but not required for the superiority claim. They can be addressed in follow-up work.

| # | Feature | Typefully | Tuitbot Current | Notes |
|---|---------|-----------|-----------------|-------|
| N-01 | Emoji picker | Integrated | Not supported | Browser native emoji picker is adequate for now |
| N-02 | AI alt text for images | Available | Not supported | Requires new AI endpoint; accessibility benefit |
| N-03 | Auto thread splitting from long text | Paste long text → auto-split at 280-char boundaries | Not supported | UX convenience; not core to thread composition |
| N-04 | Auto tweet numbering (1/N) | Configurable | Not supported | Can be added as a compose option |
| N-05 | Calendar drag-drop rescheduling | Drag scheduled items to new time slots | Not supported | Calendar page enhancement, not compose UX |
| N-06 | Draft tags and search | Tags + full-text search | Not supported | Drafts page enhancement |
| N-07 | Thread templates | Save/load thread structures | Not supported | Power user feature |
| N-08 | Undo/redo history | Standard | Browser-native textarea undo only | Would require custom undo stack |
| N-09 | GIF search integration | GIPHY search | Not supported | Requires third-party API integration |
| N-10 | Link preview cards | Preview how links will render | Not supported | Requires URL metadata fetching |

---

### Explicitly Out of Scope

| # | Feature | Reason |
|---|---------|--------|
| X-01 | Cross-posting (LinkedIn, Threads, Bluesky, Mastodon) | Platform strategy decision — Tuitbot is X-focused |
| X-02 | Voice memo to draft | Requires speech-to-text integration outside initiative scope |
| X-03 | AI voice learning / Ghostwriter engine | Excluded by operator rules; separate initiative |
| X-04 | Collaborative draft editing / sharing / locking | Multi-user feature — Tuitbot is single-user local-first |
| X-05 | Native app / Raycast extension | Desktop delivery stays Tauri |
| X-06 | Polls / photo tags / community posts | X API features outside tweet/thread/media scope |
| X-07 | Filesystem ingestion / RAG | Excluded by operator rules; separate initiative |
| X-08 | Background seed systems | Excluded by operator rules; separate initiative |
| X-09 | Watchtower monitoring | Excluded by operator rules; separate initiative |

---

## Summary

| Category | Count |
|----------|-------|
| Critical gaps (P0) | 6 |
| Important gaps (P1) | 5 |
| Nice-to-have (post-initiative) | 10 |
| Out of scope | 9 |

All 6 critical gaps and all 5 important gaps are addressed across Sessions 02-06. The initiative delivers parity on critical gaps and superiority on important gaps, establishing Tuitbot as the better compose experience for power users who value keyboard-first workflows and structural control.
