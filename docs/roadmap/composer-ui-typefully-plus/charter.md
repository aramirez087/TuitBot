# Composer UI Charter: Thread-First Unibody Redesign

## Problem Statement

Tuitbot's composer works — submission, scheduling, AI assist, autosave, and approval flow are all solid. But the compose experience feels heavy. Drafting a 5-tweet thread requires interacting with 5 separate bordered card components, each with its own textarea, gutter number, drag handle, character counter, and action row. The modal chrome (header with title + date, mode tabs, 5-element footer) adds visual noise before the user types a single character.

The result is that composing in Tuitbot feels like filling out a form, not writing. Typefully — the benchmark — makes composing feel like drafting in a note app that happens to produce tweets and threads. We need to close this gap without losing our existing advantages (deeper AI integration, voice context, command palette, local-first architecture).

## Vision

Transform the composer from a modal-with-form-fields into a **thread-first writing surface** where:

- Writing starts immediately with zero chrome overhead
- Tweet boundaries are fluid separators, not rigid card borders
- A single tweet is simply a thread of one — same editor, same flow
- Preview is inline, not a competing side column
- Power features (AI, voice, scheduling) are accessible but hidden until needed
- The experience is keyboard-native for power users and mouse-friendly for everyone

## Design Principles

### 1. Writing First
The compose surface should feel like a note-taking app. Every pixel of chrome must justify its presence against the question: "does this help the user write?" If not, it should be hidden, contextual, or removed.

### 2. Progressive Disclosure
Start with a clean writing surface. Thread separators appear when the user adds them. AI, voice, and scheduling live in a collapsible inspector rail toggled by keyboard or button. The complexity is there but never forced.

### 3. Content Determines Mode
No "tweet vs. thread" selector. The user writes; adding a separator creates a thread. Removing all separators collapses to a tweet. The composer handles both without the user needing to decide upfront.

### 4. Keyboard-Native
Every thread operation (add separator, navigate, reorder, split, merge, preview, submit) has a keyboard shortcut. The command palette (Cmd+K) is the power-user's primary entry point. Mouse affordances exist but don't clutter the writing surface.

### 5. Mobile-Ready
The inspector rail collapses to a drawer on mobile. The writing surface uses the full viewport. Touch targets meet 44px minimums. The experience degrades gracefully without losing core functionality.

### 6. Preserve What Works
Autosave, submission, scheduling, approval flow, and API contracts remain unchanged. The `ThreadBlock[]` data model is preserved — the unibody editor emits the same shape. This is a presentation-layer overhaul, not a data-model rewrite.

## Success Criteria

### Quantitative
- **Fewer visible chrome elements**: Current composer shows ~12 fixed UI elements (header, date, 2 tabs, voice panel toggle, editor pane, preview pane, AI button, notes button, cancel button, submit button, schedule section). Target: 5 or fewer visible by default (close button, preview toggle, focus toggle, writing canvas, floating submit).
- **Reduced interaction cost for thread creation**: Current path to create a 3-tweet thread: click "Thread" tab → type in textarea 1 → click "Add tweet" → type in textarea 2 → click "Add tweet" → type in textarea 3. That's 5 interactions (1 tab click + 2 "add tweet" clicks + 2 textarea focus changes). Target: type text → press Cmd+Enter to insert separator → keep typing → press Cmd+Enter → keep typing. That's 2 interactions (2 keyboard shortcuts), never leaving the writing flow.
- **Same feature coverage**: Every feature available today (submit, schedule, AI assist, AI improve, from-notes, voice context, media attachment, autosave, recovery, command palette, focus mode) remains fully functional.

### Qualitative
- The composer feels like a writing tool, not a form
- Thread drafting flows continuously without card-to-card context switching
- Power users can compose a 10-tweet thread without touching the mouse
- New users can compose a single tweet without being confused by thread controls

## Non-Goals

- **No new backend APIs** — The composer consumes existing endpoints. No Rust changes needed unless a frontend change reveals a missing contract, in which case it's scoped as a narrow, documented exception.
- **No new content types** — Threads and tweets only. No polls, quote-tweets-as-content, or other X content types.
- **No analytics changes** — The analytics pipeline and display are unrelated to composer UX.
- **No mobile-native UI** — The responsive web UI serves mobile via Tauri/browser. No native iOS/Android.
- **No multi-account support** — Single-account compose only.
- **No real-time collaboration** — Local-first single-user tool.

## Session Roadmap

| Session | Title | Scope | Key Deliverables |
|---------|-------|-------|-----------------|
| **1** (this session) | Benchmark & Charter | Audit, benchmark, design direction, session planning | `charter.md`, `benchmark-notes.md`, `ui-architecture.md`, `session-01-handoff.md` |
| **2** | Composer Shell Redesign | Strip chrome: minimal header, remove tabs, floating submit, wider canvas | `ComposerHeaderBar.svelte`, `ComposerCanvas.svelte`, refactored `ComposerShell.svelte` |
| **3** | Thread Interactions & Media | Unibody editor with separators, auto-split, Cmd+Enter flow, inline media | `ThreadFlowEditor.svelte`, `ThreadSeparator.svelte` |
| **4** | Inspector Actions & Polish | Collapsible inspector rail for schedule/voice/notes, command palette updates | `ComposerInspector.svelte`, refined `ComposerCanvas.svelte` |
| **5** | Validation & Release | Full CI, regression audit, go/no-go report | `release-readiness.md` |

## Risk Summary

| Risk | Severity | Mitigation |
|------|----------|------------|
| Unibody editor cursor management complexity | High | Use contenteditable with block children; fall back to visually-connected textareas if fragile |
| Breaking existing autosave format | Medium | Keep `ThreadBlock[]` localStorage format; unibody editor parses/emits blocks |
| Mobile responsive regression | Medium | Test at 640px and 768px breakpoints every session |
| Scope creep across sessions | Medium | Each session has explicit scope; extras documented as scope cuts |
| ComposerShell exceeds 500-line limit | Low | Session 2 decomposes into HeaderBar + Canvas subcomponents |
