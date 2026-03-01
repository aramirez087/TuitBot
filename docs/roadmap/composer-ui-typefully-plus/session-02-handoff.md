# Session 02 Handoff — Shared ComposeWorkspace & Home Shell

## What Changed

### New Files

1. **`dashboard/src/lib/components/home/AnalyticsHome.svelte`** (~175 lines)
   - 1:1 extraction of the analytics dashboard from `+page.svelte`
   - Self-contained lifecycle: `onMount` → `loadDashboard` + `startAutoRefresh`, `onDestroy` → `stopAutoRefresh`
   - Zero behavioral changes from original

2. **`dashboard/src/lib/components/composer/ComposeWorkspace.svelte`** (~410 lines)
   - Extracted from `ComposeModal.svelte` — contains all compose state (25 `$state` declarations), derived values, handlers, and template body
   - Accepts `embedded` prop that controls two render modes:
     - `embedded=false` (modal): wraps content in `ComposerShell` + `ComposerHeaderBar`, supports focus mode, Escape closes
     - `embedded=true` (full-page): renders bare workspace, no shell/header, no focus mode, Escape only dismisses sub-panels
   - Inspector content defined once as a Svelte snippet, rendered in both desktop (`ComposerCanvas` inspector slot) and mobile (`ComposerInspector` drawer)
   - Recovery banner moved here from `ComposerShell` — workspace owns all recovery state
   - Self-reset after submit in embedded mode (since component doesn't unmount)
   - Uses `onMount`/`onDestroy` for lifecycle instead of the old `$effect` on `open` prop

### Modified Files

3. **`dashboard/src/lib/components/ComposeModal.svelte`** (679 → ~40 lines)
   - Thin wrapper: `{#if open}` guard, `triggerElement` tracking for focus restoration, prop forwarding to `ComposeWorkspace` with `embedded=false`
   - External API unchanged — same props: `open`, `prefillTime`, `prefillDate`, `schedule`, `onclose`, `onsubmit`

4. **`dashboard/src/lib/components/composer/ComposerShell.svelte`** (151 → ~85 lines)
   - Removed: `showRecovery`, `onrecover`, `ondismissrecovery` props and recovery banner HTML/CSS
   - Kept: `open`, `focusMode`, `inspectorOpen`, `onclose`, `children` — pure structural modal (backdrop + dialog + focus trap)

5. **`dashboard/src/routes/(app)/+page.svelte`** (193 → ~60 lines)
   - Composer-first home: loads `home_surface` preference via `persistGet`, defaults to `'composer'`
   - `'composer'` surface: renders `ComposeWorkspace` with `embedded=true` inside an 860px max-width centered lane
   - `'analytics'` surface: renders `AnalyticsHome` (extracted analytics dashboard)
   - `loaded` flag prevents flash while async preference resolves
   - Loads schedule config from API for the workspace's time picker

## Architecture Decisions

### D1: ComposeWorkspace conditionally renders ComposerShell
The workspace owns both modal chrome (shell + header) and compose state. When `embedded=false`, it wraps everything in `ComposerShell` and renders `ComposerHeaderBar`. When `embedded=true`, it renders a bare `.embedded-workspace` div. This avoids cross-component state sharing between the modal wrapper and the compose surface.

### D2: Recovery banner in ComposeWorkspace, not ComposerShell
Since ComposeWorkspace manages `showRecovery` and `recoveryData`, it renders the banner directly. This works identically in both modal (inside shell children slot) and full-page (at top of workspace) contexts.

### D3: Inspector snippet deduplication
The inspector content (Schedule, Voice, AI sections) is defined once as a `{#snippet inspectorContent()}` in ComposeWorkspace and rendered in both the desktop `ComposerCanvas` inspector slot and the mobile `ComposerInspector` drawer. This eliminates the 47-line duplication that existed in ComposeModal.

### D4: Lifecycle via onMount/onDestroy instead of $effect on open
The old `$effect(() => { if (open) ... })` pattern is replaced by:
- Modal: `ComposeModal` uses `{#if open}` to mount/unmount `ComposeWorkspace`, so each open is a fresh `onMount`
- Full-page: `ComposeWorkspace` mounts once with the route; `onMount` checks recovery and initializes state

### D5: Self-reset in embedded mode
After successful submit, the embedded workspace explicitly resets all state (text, blocks, mode, media, timers) since the component doesn't unmount like it does in modal context.

### D6: Autosave key is shared
Both modal and full-page use `tuitbot:compose:draft`. A draft started on the home surface can be recovered from the modal and vice versa. Only one surface is ever active at a time.

## Quality Gate Results

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace -- -D warnings` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (all tests) |
| `cd dashboard && npm run check` | Pass (0 errors, 6 pre-existing warnings) |
| `cd dashboard && npm run build` | Pass |

## Exit Criteria Status

| Criterion | Status |
|-----------|--------|
| Home route opens to full-page compose surface | Done |
| Modal compose flow still works and reuses shared workspace | Done — ComposeModal delegates to ComposeWorkspace |
| Old analytics home content exists via AnalyticsHome.svelte | Done |

## Open Issues / Deferred Items

| Item | Reason | Target Session |
|------|--------|----------------|
| `HomeComposerHeader` action cluster (Schedule/Publish pills) | Needs own design. Floating submit pill works for now. | Session 3+ |
| Avatar spine on ThreadFlowLane/ThreadFlowCard | Visual enhancement, not blocking | Session 3+ |
| Getting-started tips module | Progressive disclosure, not core | Session 4+ |
| `Cmd+N` interception on home route | Currently non-functional globally | Session 4+ |
| `home_surface` Settings toggle | Need to wire Settings page | Session 5 |
| Sidebar label "Dashboard" → "Home" | Cosmetic | Session 5 |

## Inputs for Session 03

Session 03 should read:

1. `ComposeWorkspace.svelte` — the shared workspace (understand embedded vs modal paths)
2. `+page.svelte` — home route structure
3. `ComposerCanvas.svelte` — submit pill that will be replaced by HomeComposerHeader
4. `ComposerHeaderBar.svelte` — modal-only header, reference for home header design
5. `home-surface-plan.md` — acceptance criteria AC2 (avatar spine), AC3 (action cluster)
6. `ui-architecture.md` — target HomeComposerHeader component spec
