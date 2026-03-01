# Session 05 Handoff — Home Surface Preference & Settings

## What Changed

### New Files

1. **`dashboard/src/lib/stores/homeSurface.ts`** (~35 lines)
   - Centralized reactive store for the home surface preference (`'composer' | 'analytics'`)
   - Exports `homeSurface` (read-only store), `homeSurfaceReady` (derived boolean), `loadHomeSurface()`, `setHomeSurface()`
   - Defaults to `'composer'` synchronously to eliminate flash-of-wrong-surface
   - All mutations go through `setHomeSurface()` to ensure persistence is always written

2. **`dashboard/src/routes/(app)/settings/WorkspaceSection.svelte`** (~130 lines)
   - New settings section for "Default Landing Page" preference
   - Radio-card UI with two options: "Composer home" (with "Recommended" badge) and "Analytics overview"
   - Instant-save behavior — changes persist immediately via `setHomeSurface()`, no dependency on the backend settings Save/Discard flow
   - Uses `SettingsSection` wrapper matching all existing settings sections
   - Hint text: "Takes effect on next visit to the home page"

### Modified Files

3. **`dashboard/src/lib/stores/persistence.ts`** (43 → ~60 lines)
   - Added `localStorage` fallback when not running in Tauri (browser-only dev)
   - Keys prefixed with `tuitbot:ui:` to avoid collisions
   - Cached `useFallback` flag prevents re-attempting Tauri import on every call
   - All existing callers (`sidebar_collapsed`, `home_tips_dismissed`) gain browser persistence with zero code changes

4. **`dashboard/src/routes/(app)/settings/+page.svelte`** (345 → ~350 lines)
   - Added `LayoutDashboard` import from lucide-svelte
   - Added `WorkspaceSection` import
   - Added `{ id: 'workspace', label: 'Workspace', icon: LayoutDashboard }` as first entry in `sections` nav
   - Renders `<WorkspaceSection />` first in the `.sections` div

5. **`dashboard/src/routes/(app)/+page.svelte`** (68 → ~68 lines)
   - Replaced inline `HomeSurface` type and `persistGet` call with `homeSurface` store imports
   - Uses `$homeSurface` and `$homeSurfaceReady` for reactive rendering
   - Reactive via store subscription — if preference changes in Settings, home route updates automatically

6. **`dashboard/src/lib/components/Sidebar.svelte`** (384 lines, 2 lines changed)
   - Renamed home nav item from "Dashboard" to "Home"
   - Swapped icon from `LayoutDashboard` to `Home` (house icon)

## Architecture Decisions

### D1: localStorage fallback in persistence.ts (not a separate system)

Enhanced the existing `persistGet`/`persistSet` module with localStorage fallback instead of creating a parallel `browserStorage.ts`. All callers gain browser persistence automatically. Keys use `tuitbot:ui:` prefix to avoid collisions with other localStorage usage (autosave uses `tuitbot:compose:draft`, theme uses `tuitbot-theme`).

### D2: Instant-save in WorkspaceSection (no Save bar dependency)

The workspace preference is a UI-only setting stored in plugin-store/localStorage. It does not participate in the server settings Save/Discard flow which validates and writes to `config.toml`. Mixing them would require backend schema changes for a frontend-only preference. Instant-save matches how `sidebar_collapsed` and theme preferences already work.

### D3: Centralized store module vs inline state

A centralized `homeSurface.ts` store enables reactive propagation: when `setHomeSurface()` is called in Settings, the store updates immediately. When the user navigates to `/`, the home route reads from the already-updated store — no flash of wrong surface, no stale state.

### D4: Radio cards instead of select dropdown

Two options is ideal for radio-style cards — each gets a label, description, and visual treatment. A `<select>` would hide options behind a click and provide no room for the "Recommended" badge. Cards are consistent with the app's design quality bar.

### D5: Sidebar rename from "Dashboard" to "Home"

The default surface is now the composer, not a dashboard. "Dashboard" implies analytics. "Home" is surface-agnostic and accurate regardless of the user's preference. The `Home` icon (house) is more appropriate than `LayoutDashboard` (grid).

### D6: WorkspaceSection renders inside `$draft` gate

The workspace section renders inside the existing `{:else if $draft}` block for layout consistency. If the backend settings API fails, the entire settings page shows an error. This is acceptable because: (a) the error state has a "Retry" button, (b) if the backend is unreachable, the user has bigger problems, (c) rendering it outside would require duplicating loading/error state handling.

## Quality Gate Results

| Check | Result |
|-------|--------|
| `cd dashboard && npm run check` | Pass (0 errors, 6 pre-existing warnings) |
| `cd dashboard && npm run build` | Pass |
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | Pass (all tests) |
| `cargo clippy --workspace -- -D warnings` | Pass |

## Exit Criteria Status

| Criterion | Status |
|-----------|--------|
| Fresh installs land on composer home by default | Done — `homeSurface` store defaults to `'composer'` |
| Changing the setting reliably flips home surface | Done — reactive store updates both Settings UI and home route |
| Preference persists in desktop and browser dev flows | Done — Tauri plugin-store (production) + localStorage fallback (browser dev) |
| Direct path to analytics preserved when composer is default | Done — Settings toggle + sidebar "Home" link renders either surface |
| No flicker, redirect loops, or stale state | Done — synchronous default + `homeSurfaceReady` gate + reactive store |

## Known Issues

### ComposeWorkspace at 852 lines

The file remains at 852 lines from Session 04. The 500-line guideline for component files is exceeded. Recommended extractions:
- Extract `inspectorContent` snippet → `ComposerInspectorContent.svelte` (~80 lines saved)
- Extract recovery banner → small component (~30 lines saved)
- Extract submit handler + AI handlers → utility module (~60 lines saved)

### WorkspaceSection inside backend settings gate

If the backend settings API fails to load, the workspace preference UI is hidden along with all other settings. Since the workspace preference doesn't depend on the backend, it could theoretically render independently. Low impact — backend failure is rare and affects all settings equally.

## Scope Cuts

| Feature | Reason | Target |
|---------|--------|--------|
| ComposeWorkspace extraction (852 → <500 lines) | Separate refactoring concern | Session 06+ |
| Sparkles icon button wiring | Requires inline assist invoke path from header | Session 06+ |
| `Cmd+N` interception on home route | Lower priority | Session 06+ |
| Avatar images on spine dots | Needs backend change | Future |
| Double-empty-line auto-split | UX edge cases unresolved | Future |
| Preview as side-by-side rail | Layout rework | Future |
| Custom undo stack for thread ops | Complex browser undo interaction | Future |

## Inputs for Session 06

Session 06 should read:
1. `ComposeWorkspace.svelte` — 852 lines, needs extraction to bring under 500
2. `HomeComposerHeader.svelte` — Sparkles icon button needs wiring to AI assist
3. `home-surface-plan.md` — remaining acceptance criteria
4. `ui-architecture.md` — overall architecture reference

### Recommended Session 06 focus areas

1. **ComposeWorkspace extraction**: Split inspector content, recovery banner, and submit/AI handlers into separate modules
2. **Sparkles button**: Wire the AI icon in HomeComposerHeader to trigger inline assist
3. **`Cmd+N`**: Global shortcut to start new compose from anywhere
4. **Preview rail mode**: Side-by-side preview option for wide screens
5. **Sidebar analytics shortcut**: Consider adding a direct "Analytics" nav item or sub-link for quick access when composer is default
