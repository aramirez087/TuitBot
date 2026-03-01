# Session 01 Handoff — Home Surface Charter

## What Changed

### Updated Documents

1. **`benchmark-notes.md`** — Added "Typefully's Home Surface Model" section documenting that Typefully opens directly to a full-page compose surface (no analytics intermediary). Added updated feature comparison table row for home surface gap. Added Pattern #5 "Compose-First Home" to the patterns-to-emulate list.

2. **`charter.md`** — Renamed from "Thread-First Unibody Redesign" to "Composer-First Home Experience". Updated problem statement to include the home-surface gap alongside the heavy-modal problem. Updated vision to explicitly include composer-first default, `home_surface` preference, and shared compose orchestration. Added Phase 2 session roadmap (Sessions 6–9). Updated non-goals to cover backend API exclusion for preferences. Added Phase 2 risk summary.

3. **`ui-architecture.md`** — Updated component hierarchy to reflect shipped Phase 1 state (actual component names, line counts). Added target structure for Phase 2 showing dual-surface home renderer and shared `ComposeWorkspace`. Added "Home Surface Architecture" section with: dual-surface renderer pseudocode, `ComposeWorkspace` extraction spec (state list, handler list, props interface, `embedded` mode), `home_surface` preference data flow, `HomeComposerHeader` action cluster design, avatar spine specification, and full-page responsive breakpoints. Updated files-affected tables for Phase 2 sessions.

### New Documents

4. **`home-surface-plan.md`** — Six acceptance criteria (AC1–AC6) with testable assertions for: full-page dark canvas, centered thread lane with avatar spine, top-right action cluster, progressive disclosure tips, analytics as alternate surface, and persisted `home_surface` preference. Includes layout specification (desktop/tablet/mobile diagrams), component map (reused/modified/new), interaction specification (keyboard shortcuts, CTA hierarchy), preference system design, and progressive disclosure rules.

5. **`session-01-handoff.md`** — This document.

## Files Session 02 Must Extract or Create First

Session 02 corresponds to **Session 7** in the charter's Phase 2 roadmap. The work must proceed in this order:

### 1. `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` (CREATE)

**Priority**: Critical — everything else depends on this extraction.

Extract from `ComposeModal.svelte` (679 lines). The workspace encapsulates:
- All `$state()` declarations (lines 40–64 of ComposeModal)
- All handler functions: `handleSubmit`, `handleInlineAssist`, `handleAiAssist`, `handleGenerateFromNotes`, `handleUndo`, `autoSave`, `clearAutoSave`, `checkRecovery`, `recoverDraft`, `dismissRecovery`, `toggleInspector`, `togglePreview`, `toggleFocusMode`, `handlePaletteAction`
- The template body from line 365 (`ComposerHeaderBar`) through line 542 (end of `CommandPalette` block), minus the `ComposerShell` wrapper and `ComposerHeaderBar`
- All `<style>` blocks from ComposeModal

**Props interface**:
```typescript
{
  schedule: ScheduleConfig | null;
  onsubmit: (data: ComposeRequest) => void;
  onclose?: () => void;
  prefillTime?: string | null;
  prefillDate?: Date | null;
  embedded?: boolean;
}
```

**Key design choice**: The `embedded` prop controls differences between modal and full-page contexts:
- `embedded = false` (default): focus mode available, Escape closes modal, recovery delegates to shell
- `embedded = true`: no focus mode, Escape only dismisses sub-panels, recovery banner inline

### 2. `ComposeModal.svelte` (MODIFY — reduce to ~80 lines)

After extraction, `ComposeModal` becomes a thin wrapper:
```svelte
ComposerShell (backdrop + dialog + recovery)
  └── ComposerHeaderBar (close, preview, inspector, focus)
      └── ComposeWorkspace (all compose logic, embedded=false)
```

**Critical**: The external API must not change. These props must still work identically:
- `open`, `prefillTime`, `prefillDate`, `schedule`, `onclose`, `onsubmit`

### 3. Verify existing modal flow works

Before proceeding to Session 8 (home surface), run the full CI checklist:
- `cargo fmt --all && cargo fmt --all --check`
- `cd dashboard && npm run check`
- `cd dashboard && npm run build`
- Manual smoke test: open compose modal from sidebar, write a tweet, write a thread, schedule, AI assist, autosave/recover

## Architectural Risks

### Risk 1: ComposeWorkspace extraction breaks autosave (Severity: High)

**Problem**: `ComposeModal` currently manages the autosave lifecycle tied to the `open` prop (line 147–167). When `open` changes to `true`, it resets state and checks for recovery. When the modal closes, autosave state is cleared.

**Mitigation**: The `ComposeWorkspace` needs its own lifecycle management:
- In modal context: reset on mount (parent controls mounting via `{#if open}`), clear on unmount
- In full-page context: check recovery on mount, autosave continuously, never clear on unmount (the page doesn't "close")
- The `embedded` prop distinguishes these behaviors

### Risk 2: Duplicate inspector content between modal and full-page (Severity: Medium)

**Problem**: The inspector sections (Schedule, Voice, AI) are currently defined as snippet content inside `ComposeModal.svelte` (lines 424–471 for desktop, duplicated at 484–530 for mobile). Moving this into `ComposeWorkspace` means the inspector template is shared, but the mobile drawer (`ComposerInspector.svelte`) is currently rendered by `ComposeModal` outside of `ComposerCanvas`.

**Mitigation**: `ComposeWorkspace` should own both the desktop inspector (via `ComposerCanvas` inspector snippet) and the mobile drawer (via `ComposerInspector`). The workspace detects `isMobile` and renders the appropriate variant.

### Risk 3: `home_surface` async load causes flash (Severity: Medium)

**Problem**: `persistGet` is async. If the preference is `'analytics'` but we render `'composer'` as the sync default, users who prefer analytics will see a brief flash of the composer before the analytics dashboard appears.

**Mitigation**: Use a loading state that renders neither surface until the preference resolves. The loading state can be a minimal skeleton or the sidebar-only layout for ~50ms while the Tauri store loads. In practice, the Tauri store loads very quickly (< 10ms for local JSON), so this flash is unlikely to be perceptible.

### Risk 4: `Cmd+N` conflict on home route (Severity: Low)

**Problem**: `+layout.svelte` dispatches `tuitbot:compose` on `Cmd+N`, which opens the compose modal. On the home route with the composer surface active, opening a modal over an already-visible composer is confusing.

**Mitigation**: The home route should intercept `Cmd+N` when the composer surface is active:
- If already on `/` with composer surface: focus the first textarea (no modal)
- If on any other route: open the modal as usual

## Open Questions (resolved during planning)

All questions from the plan were resolved with concrete decisions:

1. **Where does the home-surface preference live?** → `persistGet/persistSet` with key `'home_surface'`, same pattern as `sidebar_collapsed`. No backend API.

2. **Should the full-page composer have focus mode?** → No. The full-page surface IS the focused writing experience. Focus mode in the modal makes the modal go full-screen; the home surface is already full-page.

3. **Should the avatar spine be on by default?** → Yes for full-page context (`avatarSpine` prop on `ThreadFlowLane`); no for modal context (keeps the simpler accent bar).

4. **Where in Settings does the preference go?** → Either a new "Appearance" section or within "Business Profile". Decision deferred to Session 9 based on available vertical space.

## Inputs for Session 02

Session 02 should read these files before making any changes:

1. `ComposeModal.svelte` — The primary extraction source (679 lines)
2. `composer/ComposerCanvas.svelte` — To understand the inspector snippet interface
3. `composer/ComposerInspector.svelte` — Mobile drawer behavior
4. `composer/ComposerHeaderBar.svelte` — Header props that ComposeWorkspace must relay
5. `stores/persistence.ts` — `persistGet/persistSet` API
6. `ui-architecture.md` — Target component hierarchy and ComposeWorkspace spec
7. `home-surface-plan.md` — Acceptance criteria (especially AC1, AC6)
