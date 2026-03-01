# Session 1 Handoff: Benchmark & Charter

## Session Summary

Audited the current composer (9 component files, ~3,000 lines), benchmarked against Typefully's UX model, and produced a charter, benchmark analysis, and UI architecture document that lock the design direction for sessions 2–5.

## Decisions Made

### D1: No mode tabs — content determines tweet vs. thread
The "Tweet" / "Thread" tab bar in `ComposerShell.svelte` (lines 105–124) is removed from the UI. A draft with one segment is a tweet; adding a separator makes it a thread. `ComposeModal` still tracks `mode` internally for submission logic (the `ComposeRequest.content_type` field), but the mode is derived from block count rather than a user toggle.

**Rationale**: Typefully proved this model works. Forcing users to pick a mode before writing creates unnecessary friction and a premature commitment.

### D2: Unibody editor replaces card-per-tweet model
`ThreadComposer.svelte` (427 lines) renders each tweet as a bordered card with its own textarea, gutter, drag handle, char counter, and action row. This is replaced by `ThreadFlowEditor.svelte` — a continuous writing surface where tweet separators are the only visual boundary between segments.

**Rationale**: The card model is the single biggest source of visual noise. A 5-tweet thread shows 5 bordered boxes with 5 sets of controls. The unibody model shows one writing surface with 4 thin separator lines.

### D3: Inline preview toggle replaces side-by-side layout
The `compose-layout` grid in `ComposeModal.svelte` (lines 391–405) uses a 50/50 split giving half the modal to a preview column. This is replaced by a toggle that swaps the canvas between edit and preview modes in the same space.

**Rationale**: The side-by-side layout wastes half the width in tweet mode (a single preview card fills 10% of the preview column). In thread mode it has some value, but the preview column doesn't scroll-sync with the editor, so users editing tweet 8 might see tweet 1 in the preview.

### D4: Floating submit replaces footer
The 5-element footer (AI Assist, Notes, spacer, Cancel, Submit) is eliminated. Submit becomes a floating pill button inside the canvas. AI Assist and Notes move to the command palette and the inspector rail.

**Rationale**: The footer competes with the writing surface for attention. The submit button is the only action that deserves persistent visibility; everything else is secondary.

### D5: Inspector rail for secondary controls
Schedule (TimePicker), voice context (VoiceContextPanel), and from-notes (FromNotesPanel) move to a collapsible right-side rail toggled by Cmd+I. This keeps the writing canvas clean while making power features one shortcut away.

**Rationale**: These controls are important but not constantly needed. Typefully uses a similar sidebar approach. Moving them out of the main writing flow reduces chrome density.

### D6: ThreadBlock[] contract preserved
The unibody editor emits the same `ThreadBlock[]` shape that `ThreadComposer` emits today. `ComposeModal`'s submission logic, autosave format, and API contract are unchanged.

**Rationale**: Minimizes blast radius. The change is purely presentational — the data model and all downstream logic are untouched.

### D7: Incremental session execution
Each session changes one architectural layer:
- Session 2: Shell chrome (header, footer, layout) — editor internals untouched
- Session 3: Editor model (unibody + separators) — shell already done
- Session 4: Secondary controls (inspector rail) — editor and shell already done
- Session 5: Validation only — no source changes unless fixing regressions

**Rationale**: Isolates regressions. If the unibody editor (Session 3) has issues, the shell (Session 2) is already stable and doesn't need to be re-done.

## Scope Locked for Session 2

### In Scope
1. Create `ComposerHeaderBar.svelte`: close button, preview toggle, focus-mode toggle. No title, no date, no tabs.
2. Create `ComposerCanvas.svelte`: layout wrapper with `canvas-main` slot and future `canvas-inspector` slot.
3. Refactor `ComposerShell.svelte`: strip header and footer; delegate to HeaderBar + Canvas. Keep backdrop, modal container, recovery banner, and focus-mode fullscreen logic.
4. Update `ComposeModal.svelte`: remove mode-tab props from ComposerShell, remove the 50/50 grid layout, add floating submit button. Keep all state management, autosave, and submission logic unchanged.
5. Remove visible mode tabs. Keep `mode` state in ComposeModal, derive it from content (single text = tweet, blocks present = thread). The mode switching via Cmd+Shift+N/T and command palette actions remains functional for explicit mode setting.

### Out of Scope (Session 2)
- Changing `ThreadComposer` or `TweetEditor` internals (Session 3)
- Moving VoiceContextPanel, TimePicker, or FromNotesPanel to inspector (Session 4)
- Unibody editor (Session 3)
- Auto-split (Session 3)
- Mobile inspector drawer (Session 4)

## Required Inputs for Session 2

### Files to Read
- `dashboard/src/lib/components/composer/ComposerShell.svelte` — current shell to refactor
- `dashboard/src/lib/components/ComposeModal.svelte` — orchestrator to update
- `dashboard/src/app.css` — design tokens for styling new components
- `docs/roadmap/composer-ui-typefully-plus/ui-architecture.md` — component specs

### Design Reference
- `ComposerHeaderBar` spec in `ui-architecture.md` → "New Component Specifications" section
- `ComposerCanvas` spec in `ui-architecture.md` → "New Component Specifications" section
- Floating submit button CSS pattern in `ui-architecture.md` → "New CSS Patterns" section

### Constraints
- `ComposerShell.svelte` currently at 516 lines → must end ≤ 500 lines (CLAUDE.md limit), ideally ~200 with logic delegated to subcomponents
- No changes to `ThreadComposer.svelte`, `TweetEditor.svelte`, or any Rust code
- Preserve all keyboard shortcuts (Cmd+K, Cmd+Enter, Cmd+Shift+F, Cmd+J, Cmd+Shift+N/T, Escape)
- Preserve autosave, recovery banner, focus mode, and command palette behavior
- All existing tests must pass

## Open Questions for Future Sessions

### Q1: Contenteditable vs. Stacked Textareas (Session 3)
The unibody editor spec prefers `contenteditable` with block-level children for true continuous editing. If `contenteditable` proves too fragile (IME issues, selection bugs, mobile keyboard problems), the fallback is stacked `<textarea>` elements with visual connectors. Session 3 should prototype both and decide early.

### Q2: Separator Drag Behavior (Session 3)
When a user drags a separator, does the text above/below the separator move with it (reordering the segments)? Or does the separator itself move, changing which text is in which segment? The Typefully model uses the former (drag = reorder segments). This needs validation in Session 3's implementation.

### Q3: Inspector Width on Small Desktops (Session 4)
The inspector is spec'd at 280px width. On a 1024px-wide viewport with a 640px modal, the inspector would leave only 360px for the writing canvas. Should the inspector collapse to icons-only below a threshold, or should the modal width expand to accommodate it?

### Q4: Preview Mode Editing (Session 4)
When the user is in preview mode and clicks on a tweet preview card, should it switch to edit mode with the cursor at that tweet? Typefully doesn't support this, but it would be a natural interaction.

## Risk Register

| # | Risk | Severity | Owner | Status |
|---|------|----------|-------|--------|
| R1 | Unibody editor cursor management complexity | High | Session 3 | Open — contenteditable fallback defined |
| R2 | Breaking existing autosave format | Medium | Session 3 | Mitigated — ThreadBlock[] contract preserved |
| R3 | Mobile responsive regression | Medium | All sessions | Open — test at 640px/768px each session |
| R4 | Scope creep | Medium | All sessions | Mitigated — explicit session boundaries |
| R5 | ComposerShell exceeds 500-line limit | Low | Session 2 | Mitigated — decomposition planned |
| R6 | Mode removal confuses existing users | Low | Session 2 | Mitigated — Cmd+Shift+N/T still works; command palette has mode actions |

## Code Changes in This Session

None. This session produced documentation only.

## Next Session

**Session 2: Composer Shell Redesign** — Create `ComposerHeaderBar.svelte` and `ComposerCanvas.svelte`, refactor `ComposerShell.svelte` to ~200 lines, remove mode tabs, add floating submit button, switch from 50/50 grid to full-width canvas with inline preview toggle.
