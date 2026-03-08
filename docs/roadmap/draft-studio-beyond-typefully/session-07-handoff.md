# Session 07 Handoff

## What Changed

### Backend (Rust)

**`ScheduledContent` struct** (`crates/tuitbot-core/src/storage/scheduled_content/mod.rs`)
- Added `pub source: String` field. Maps from the existing `source` column in `scheduled_content` table (default `'manual'`).

**`tags.rs`** (`crates/tuitbot-core/src/storage/scheduled_content/tags.rs`)
- Added `list_draft_tags_for(pool, account_id, content_id)` — fetches tags assigned to a specific draft via JOIN on `content_tag_assignments`.

**`draft_studio.rs`** (`crates/tuitbot-server/src/routes/content/draft_studio.rs`)
- Added `source: String` to server `DraftSummary` struct and `to_summary()` mapping.

**`draft_tags.rs`** (new, `crates/tuitbot-server/src/routes/content/draft_tags.rs`)
- 5 handler functions: `list_account_tags`, `create_account_tag`, `list_draft_tags`, `assign_draft_tag`, `unassign_draft_tag`.
- Kept in a separate file to stay under the 500-line limit on `draft_studio.rs`.

**Route registration** (`crates/tuitbot-server/src/lib.rs`)
- `GET/POST /api/tags` — account tag CRUD
- `GET /api/drafts/{id}/tags` — list draft tags
- `POST/DELETE /api/drafts/{id}/tags/{tag_id}` — assign/unassign

### Frontend Types & API Client

**`types.ts`**
- Added `source: string` to `DraftSummary`
- Added `ContentTag` interface (id, account_id, name, color)

**`client.ts`**
- Added `draftStudio.tags(id)`, `draftStudio.assignTag(id, tagId)`, `draftStudio.unassignTag(id, tagId)`
- Added `tags.list()`, `tags.create(name, color?)`

### Store (`draftStudio.svelte.ts`)
- New state: `searchQuery`, `sortBy`, `tagFilter`, `accountTags`, `selectedDraftTags`
- New derived: `currentTabDrafts` now applies search filter and sort before rendering
- New actions: `setSearchQuery`, `setSortBy`, `setTagFilter`, `loadTags`, `loadSelectedDraftTags`, `assignTag`, `unassignTag`, `createAndAssignTag`
- New getters: `getSearchQuery`, `getSortBy`, `getTagFilter`, `getAccountTags`, `getSelectedDraftTags`
- `loadDrafts` passes `tag` param when `tagFilter` is set
- `createDraft` includes `source: 'manual'` in optimistic summary
- `reset` clears all new state

### New Components

**`DraftFilterBar.svelte`** (~170 lines)
- Search input with debounced 300ms callback
- Sort dropdown (updated/created/title/scheduled)
- Tag filter dropdown with color dots
- Clear filters button when any filter is active
- Exports `focusSearch()` for the `/` keyboard shortcut

**`DraftDetailsPanel.svelte`** (~300 lines)
- Inline title editing (auto-save on blur/Enter, debounced 800ms)
- Expandable notes textarea (auto-save on blur, debounced 800ms)
- Tag management: assigned tags as removable pills, inline picker with create-new option
- Read-only metadata: type, source, status, created, updated, scheduled
- Ready state indicator (green/amber dot)
- Close button sets `detailsPanelOpen = false`

### Modified Components

**`DraftRailItem.svelte`**
- Source badge: shows "AI" for assist, "Disc" for discovery, hidden for manual
- Ready indicator: small colored dot (green if content >10 chars, amber otherwise)

**`DraftRail.svelte`**
- Integrates `DraftFilterBar` above the list
- New props: `searchQuery`, `sortBy`, `tagFilter`, `accountTags`, `onsearch`, `onsort`, `ontagfilter`
- `/` key focuses the search input

**`DraftStudioShell.svelte`**
- 3-column grid when details panel is open (`260px 1fr 280px`), 2-column when closed
- Wires details panel with meta update, tag assign/unassign, tag create callbacks
- Loads tags on mount and on account switch
- Loads selected draft tags when selection changes
- `Cmd+Shift+D` toggles details panel
- Responsive: details panel hides below 1024px

**`shortcuts.ts`**
- Added `cmd+shift+d` (Toggle details panel) and `/` (Focus search) to `SHORTCUT_CATALOG`

### Documentation
- `metadata-and-filters.md`: organization model, tag architecture, filter/sort design, non-goals

## Key Decisions

1. **Details panel as third column**: Chosen over modal or drawer because it keeps metadata visible while writing without blocking the composer. The `auto` column approach cleanly collapses when the panel is closed.

2. **Tags fetched separately per draft**: Avoids a JOIN on every list call. Tags are only needed when viewing a specific draft, not when scanning the rail. The `?tag=` filter param reloads from server since assignments aren't on the summary.

3. **Search is client-side, tag filter is server-side**: The collection is small enough for client-side search. Tag filter requires server because tag assignments aren't on the summary.

4. **Source badge hidden for "manual"**: Most drafts are manual. Only "AI" and "Disc" badges add information. This avoids visual noise.

5. **Ready state is a simple heuristic**: Content preview > 10 characters. No validation gate — just a visual signal. This is intentionally low-friction.

6. **Tag handlers in separate file**: `draft_studio.rs` was 565 lines. Adding ~110 lines of tag handlers would exceed the 500-line limit. Split into `draft_tags.rs`.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `source` field missing on old rows without DB column | Low | Medium | Column has DEFAULT 'manual' from migration. `SELECT *` includes it. All existing rows have the default. |
| Details panel 3-column layout squeezes composer on narrow screens | Medium | Low | Panel hides below 1024px. On 1024-1400px the composer still gets ~480px+. |
| Tag picker dropdown can overflow the rail width | Low | Low | Picker is inside the details panel (280px wide), not the rail. |
| `DraftStudioShell.svelte` at ~400 lines (close to 400 limit) | Medium | Low | Currently ~395 lines. If it grows further, extract details-zone wiring into a helper component. |
| `loadSelectedDraftTags` called on every selection change (extra API call) | Low | Low | Single lightweight query. Only fires when selection changes. |

## Exact Inputs for Session 08

1. Read `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — schedule integration target
2. Read `dashboard/src/lib/stores/draftStudio.svelte.ts` — schedule/unschedule actions
3. Read `crates/tuitbot-server/src/routes/content/draft_studio.rs` — schedule/unschedule handlers
4. Read `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` — schedule picker integration
5. Read `dashboard/src/routes/(app)/calendar/+page.svelte` — calendar integration target
6. Implement schedule picker in studio (date/time selection, schedule/unschedule buttons)
7. Wire studio schedule action to server endpoint
8. Add calendar link or redirect from studio scheduled tab
9. Run: `npm --prefix dashboard run check`, full Rust CI
