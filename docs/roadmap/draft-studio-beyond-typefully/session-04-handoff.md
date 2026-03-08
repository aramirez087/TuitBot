# Session 04 Handoff

## What Changed

### New store: `draftStudio.svelte.ts`

Svelte 5 rune-based module store at `dashboard/src/lib/stores/draftStudio.svelte.ts`. Manages:

- Two collections: `collection` (active + scheduled) and `archivedCollection` (lazy-loaded on first Archive tab visit)
- URL-based selection via `?id=N` query parameter, updated with `history.replaceState`
- Tab state (active/scheduled/archive) — in-memory only
- Loading, error, and sync status tracking
- Actions: load, select, create, archive, restore, duplicate, reset

### New components: Draft Studio shell

| Component | Path | Purpose |
|-----------|------|---------|
| `DraftStudioShell` | `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` | Two-zone grid layout (rail + composer surface), orchestrates store and renders conditional states |
| `DraftRail` | `dashboard/src/lib/components/drafts/DraftRail.svelte` | Tab bar + scrollable draft list + new draft button |
| `DraftRailEntry` | `dashboard/src/lib/components/drafts/DraftRailEntry.svelte` | Single draft item with title, timestamp, badges |
| `DraftEmptyState` | `dashboard/src/lib/components/drafts/DraftEmptyState.svelte` | Empty state variants (no drafts / no selection) |

### Modified files

| File | Change |
|------|--------|
| `dashboard/src/routes/(app)/drafts/+page.svelte` | Replaced entire old CRUD view with `DraftStudioShell` mount point |
| `dashboard/src/lib/components/Sidebar.svelte` | Renamed nav label from "Drafts" to "Draft Studio" |

### New documentation

- `docs/roadmap/draft-studio-beyond-typefully/workspace-shell.md` — Shell behavior, store design, layout strategy

## Key Decisions

1. **Svelte 5 rune store (`.svelte.ts`)** — Uses module-level `$state`/`$derived` instead of Svelte 4 `writable`/`derived`. This is forward-looking and avoids `get()` wrappers. Getter functions are used for state that needs to be read reactively in components (since module-level `$state` variables need function access to trigger reactivity through component boundaries).

2. **`history.replaceState` for URL updates** — Avoids `goto()` which triggers SvelteKit navigation and the `{#key $page.url.pathname}` fade transition in the app layout. The URL is read once on mount via `initFromUrl()`, then managed imperatively.

3. **Tab is not in URL** — The tab (active/scheduled/archive) resets to "active" on every visit. Bookmarking a specific tab is not useful for drafts.

4. **Lazy archive loading** — The archive collection is only fetched when the user first clicks the Archive tab. This avoids unnecessary API calls on initial load.

5. **Negative margin layout** — The shell cancels the app layout's `padding: 24px 32px` with `margin: -24px -32px` to go full-bleed. This is scoped to `.studio-shell` and only affects `/drafts`.

6. **Composer surface is preview-only** — Session 04 shows a styled read-only preview of the selected draft. The full `ComposeWorkspace` binding with autosave is Session 05 scope.

7. **Sidebar label updated** — Changed from "Drafts" to "Draft Studio" to signal the new experience.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 05 | Composer binding | `ComposeWorkspace` draftId prop, server hydration, debounced autosave PATCH |
| 06 | Keyboard and bulk | Rail keyboard nav, focus management, command palette, bulk archive |
| 07 | Metadata and filters | Title editing, tags UI, search, content type filter, sort options, right details panel |
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Module-level `$state` reactivity across component boundaries | Medium | Medium | Getter functions (`getSelectedId()`, `isLoading()`, etc.) are used for primitive state reads in component templates. Derived values like `currentTabDrafts` and `selectedDraft` work directly since they're `$derived`. If reactivity issues appear in Session 05, can wrap in a class-based store. |
| `history.replaceState` and `$page.url` diverge | Low | Low | `$page.url` is only read on initial mount. All subsequent reads use store state. |
| Old drafts API still mounted at `/api/content/drafts` | None | None | Both endpoints coexist. The old page was the only consumer of `api.drafts.*`; it's now replaced. Legacy endpoints can be removed when ready. |
| Draft preview truncation from `content_preview` | Low | Low | `DraftSummary.content_preview` is server-truncated. The full content loads in Session 05 via `api.draftStudio.get(id)`. |

## Exact Inputs for Session 05

1. Read `dashboard/src/lib/stores/draftStudio.svelte.ts` — the store to extend with autosave
2. Read `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — replace preview with composer
3. Read `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` — existing composer to bind
4. Read `dashboard/src/lib/api/client.ts` — `api.draftStudio.get()` and `api.draftStudio.autosave()` methods
5. Read `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md` — autosave conflict protocol
6. Bind `ComposeWorkspace` to accept a `draftId` prop for server hydration
7. Implement debounced autosave (PATCH with `updated_at` conflict detection)
8. Handle 409 stale write responses with user-facing conflict resolution
9. Add `syncStatus` visual indicator in the shell
10. Run: `npm --prefix dashboard run check`
