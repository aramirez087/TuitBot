# Draft Studio — Workspace Shell

## Overview

The Draft Studio shell replaces the old `/drafts` CRUD card list with a two-zone layout (three-zone once the right details panel ships in Session 07):

```
┌──────────┬──────────────────────────────────┐
│  Rail    │  Composer Surface                │
│  260px   │  (flex 1fr)                      │
│          │                                  │
│ Tabs     │  Selected draft preview          │
│ Draft    │  or                              │
│ List     │  Empty state                     │
│          │                                  │
│ [+ New]  │                                  │
└──────────┴──────────────────────────────────┘
```

## Route State

- **Selected draft:** `?id=42` query parameter. Updated via `history.replaceState` to avoid SvelteKit re-navigation and fade transitions.
- **Tab:** In-memory only (`active` | `scheduled` | `archive`). Resets to `active` on navigation. Not bookmarkable by design.
- **Reload-safe:** On mount, `initFromUrl()` reads the `?id=` param and restores the selection after collection loads.

## Store (`draftStudio.svelte.ts`)

Svelte 5 rune-based module store using `$state` and `$derived` at module scope (requires `.svelte.ts` extension for rune compilation).

### State

| Field | Type | Default |
|-------|------|---------|
| `collection` | `DraftSummary[]` | `[]` |
| `archivedCollection` | `DraftSummary[]` | `[]` |
| `selectedId` | `number \| null` | `null` |
| `tab` | `'active' \| 'scheduled' \| 'archive'` | `'active'` |
| `loading` | `boolean` | `true` |
| `archiveLoaded` | `boolean` | `false` |
| `error` | `string \| null` | `null` |
| `syncStatus` | sync status enum | `'saved'` |

### Derived

- `activeDrafts` — drafts with `status === 'draft'`, sorted by `updated_at` desc
- `scheduledDrafts` — drafts with `status === 'scheduled'`, sorted desc
- `currentTabDrafts` — whichever tab is active
- `selectedDraft` — matching `selectedId` in both collections
- `tabCounts` — `{ active, scheduled, archive }`

### Actions

- `loadDrafts()` — fetches non-archived drafts
- `selectDraft(id)` — sets selection + updates URL
- `setTab(tab)` — switches tab, lazy-loads archive on first visit
- `createDraft()` — POST, optimistic insert, auto-select
- `archiveDraft(id)` — archive + move between collections
- `restoreDraft(id)` — restore + move between collections
- `duplicateDraft(id)` — duplicate + reload + select
- `initFromUrl(url)` — read `?id=` on mount
- `reset()` — clear all state (account switch)

## Components

### `DraftStudioShell.svelte`

Orchestrator. Mounts the rail and composer zone in a CSS grid. Listens for `ACCOUNT_SWITCHED_EVENT`. Handles the conditional rendering of empty states, loading, error banner, and draft preview.

### `DraftRail.svelte`

Left rail with three sections:
1. **Tab bar** — Active/Scheduled/Archive with counts and accent underline
2. **Scrollable draft list** — `DraftRailEntry` components
3. **New Draft button** — sticky footer

### `DraftRailEntry.svelte`

Individual draft item showing title/preview, relative timestamp, content type badge, and scheduled badge when applicable. Uses `aria-current` for accessibility.

### `DraftEmptyState.svelte`

Two variants:
- `no-drafts` — "Start writing" with create button
- `no-selection` — "Select a draft" with create button

## Layout Strategy

The shell uses negative margins (`margin: -24px -32px`) to cancel the `.main-content` padding from the app layout, allowing the studio to fill edge-to-edge. This is scoped to the `.studio-shell` class and only affects the `/drafts` route.

## Account Switching

The shell listens for `ACCOUNT_SWITCHED_EVENT` on mount. When fired, it calls `reset()` then `loadDrafts()` to refresh with the new account's drafts.

## Remaining Polish (Future Sessions)

- Session 05: Bind `ComposeWorkspace` with server hydration and autosave
- Session 06: Keyboard navigation in rail, focus management
- Session 07: Right details panel (metadata, tags)
- Session 10: Mobile responsive layout (`DraftMobilePicker`)
