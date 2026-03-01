# Composer UI Charter: Composer-First Home Experience

## Problem Statement

Tuitbot's composer works — submission, scheduling, AI assist, autosave, and approval flow are all solid. But two structural problems prevent it from competing with Typefully:

1. **The compose experience feels heavy.** Drafting a 5-tweet thread requires interacting with 5 separate bordered card components, each with its own textarea, gutter number, drag handle, character counter, and action row. The modal chrome adds visual noise before the user types a single character.

2. **The app opens to the wrong surface.** Tuitbot's home route (`/`) is an analytics dashboard. Writing requires opening a modal via `Cmd+N`. Typefully opens directly to a full-page compose surface — zero clicks between launch and writing. This is the single largest workflow gap.

The result is that composing in Tuitbot feels like filling out a form inside a metrics tool, not writing inside a growth co-pilot. We need to close both gaps without losing our existing advantages (deeper AI integration, voice context, command palette, local-first architecture).

## Vision

Transform the application from an analytics-first dashboard into a **composer-first home experience** where:

- **The app opens to writing, not metrics.** The composer is the default home surface at `/`. Analytics is a secondary view, selectable via a `home_surface` preference in Settings.
- **A persisted UI preference controls the home surface.** `home_surface: 'composer' | 'analytics'` stored via `persistGet/persistSet` (Tauri plugin-store). Fresh installs default to `'composer'`. Users who prefer the analytics dashboard can switch back in Settings.
- Writing starts immediately with zero chrome overhead — the full-page canvas has a blinking cursor ready
- Tweet boundaries are fluid separators, not rigid card borders
- A single tweet is simply a thread of one — same editor, same flow
- Preview is inline, not a competing side column
- Power features (AI, voice, scheduling) are accessible but hidden until needed
- The experience is keyboard-native for power users and mouse-friendly for everyone
- The compose orchestration logic is shared between the full-page home surface and the modal (accessible from other routes via `Cmd+N`)

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

- **No new backend APIs for UI preferences** — The `home_surface` preference uses the existing `persistGet/persistSet` path (Tauri plugin-store → `ui-state.json`). No Rust changes needed for preference storage.
- **No new backend APIs for compose** — The composer consumes existing endpoints. No Rust changes needed unless a frontend change reveals a missing contract, in which case it's scoped as a narrow, documented exception.
- **No new content types** — Threads and tweets only. No polls, quote-tweets-as-content, or other X content types.
- **No analytics pipeline changes** — The analytics data layer and API are unrelated to composer UX. The analytics dashboard UI is extracted into its own component but not redesigned.
- **No mobile-native UI** — The responsive web UI serves mobile via Tauri/browser. No native iOS/Android.
- **No multi-account support** — Single-account compose only.
- **No real-time collaboration** — Local-first single-user tool.

## Session Roadmap

### Phase 1: Modal Composer Overhaul (Sessions 1–5, shipped)

| Session | Title | Status | Key Deliverables |
|---------|-------|--------|-----------------|
| **1** | Benchmark & Charter | Shipped | `charter.md`, `benchmark-notes.md`, `ui-architecture.md`, `session-01-handoff.md` |
| **2** | Composer Shell Redesign | Shipped | `ComposerHeaderBar.svelte`, `ComposerCanvas.svelte`, refactored `ComposerShell.svelte` |
| **3** | Thread Flow Components | Shipped | `ThreadFlowLane.svelte`, `ThreadFlowCard.svelte` |
| **4** | Inspector & Polish | Shipped | `ComposerInspector.svelte`, `ThreadPreviewRail.svelte` |
| **5** | Validation & Release | Shipped | `release-readiness.md` |

### Phase 2: Composer-First Home Surface (Sessions 6–9)

| Session | Title | Scope | Key Deliverables |
|---------|-------|-------|-----------------|
| **6** (this session) | Home Surface Charter | Benchmark Typefully home surface, architecture planning, acceptance criteria | `charter.md` update, `benchmark-notes.md` update, `ui-architecture.md` update, `home-surface-plan.md`, `session-01-handoff.md` |
| **7** | ComposeWorkspace Extraction | Extract shared compose orchestrator from `ComposeModal.svelte`; modal delegates to shared workspace | `ComposeWorkspace.svelte`, updated `ComposeModal.svelte` |
| **8** | Full-Page Composer Home | Build the home composer surface with action cluster, avatar spine, tips | `HomeComposerSurface.svelte`, `HomeComposerHeader.svelte`, updated `+page.svelte` |
| **9** | Settings Override & Polish | `home_surface` preference in Settings, analytics extraction, responsive QA, sidebar label update | Updated settings page, `AnalyticsDashboard.svelte`, updated `Sidebar.svelte` |

## Risk Summary

### Phase 1 Risks (resolved)

| Risk | Severity | Outcome |
|------|----------|---------|
| Unibody editor cursor management complexity | High | Resolved: used connected textareas (`ThreadFlowCard`) instead of `contenteditable` |
| Breaking existing autosave format | Medium | Resolved: `ThreadBlock[]` format preserved |
| Mobile responsive regression | Medium | Resolved: tested at 640px and 768px breakpoints |
| ComposerShell exceeds 500-line limit | Low | Resolved: decomposed into HeaderBar + Canvas + Inspector subcomponents |

### Phase 2 Risks (active)

| Risk | Severity | Mitigation |
|------|----------|------------|
| `ComposeWorkspace` extraction breaks existing modal | High | Session 7 must maintain `ComposeModal`'s external API (`open`, `onclose`, `onsubmit` props). Extract state and handlers into `ComposeWorkspace.svelte`, then have `ComposeModal` delegate to it. Run full CI after extraction. |
| Autosave conflict between modal and home surface | Medium | Use the same `tuitbot:compose:draft` localStorage key for both surfaces. The home composer and modal should not coexist on the same route — home replaces the need for modal on `/`. When navigating away, autosave persists; opening modal from another page uses the same recovery flow. |
| `home_surface` preference not loading fast enough | Medium | `persistGet` is async (Tauri store load). The `+page.svelte` must render `composer` as the synchronous default while the preference resolves, avoiding layout shift. |
| Thread flow components not designed for full-page width | Low | `ThreadFlowLane` and `ThreadFlowCard` are already width-flexible. The 760–860px centered lane is achieved via `max-width` on the parent container. |
| Settings page line count | Low | Adding a home-surface toggle is ~20 lines. Can go in an existing section or a new "Appearance" section. Page stays well under 400 lines. |
| Sidebar nav label confusion | Low | "Dashboard" nav item changes to "Home". ~5-line change to `Sidebar.svelte`. |
