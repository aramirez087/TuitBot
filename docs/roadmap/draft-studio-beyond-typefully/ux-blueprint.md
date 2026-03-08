# Draft Studio UX Blueprint

## Layout

Draft Studio is a three-zone layout that fills the app content area:

```
┌─────────────────────────────────────────────────────────┐
│  Sidebar  │  Draft Rail (260px)  │  Composer + Inspector │
│  (existing)│                     │  (flex: 1)            │
│           │  [Active] [Sched] [Archive]                  │
│           │  ┌──────────────┐   │  ┌─────────────────┐  │
│           │  │ Draft 1 ●    │   │  │ ComposeWorkspace │  │
│           │  │ Draft 2      │   │  │ (bound to        │  │
│           │  │ Draft 3      │   │  │  selected draft)  │  │
│           │  │ ...          │   │  │                   │  │
│           │  └──────────────┘   │  └─────────────────┘  │
│           │  [+ New Draft]      │                        │
└─────────────────────────────────────────────────────────┘
```

- **Draft Rail** (left): 260px fixed width, scrollable list of drafts grouped by status tab.
- **Composer Surface** (center): Full `ComposeWorkspace` bound to the selected draft's server record.
- **Inspector** (right, inside ComposeWorkspace): Existing inspector rail with schedule, voice, AI, revisions.

## Screen States

### 1. Empty State

First visit to `/drafts` with no drafts. The rail shows the status tabs (Active selected by default) and an empty area. The composer zone shows a centered empty state:

- Illustration or icon (PenLine)
- "Start writing" heading
- "Create your first draft to begin composing." description
- Primary CTA: "New Draft" button
- Secondary CTA: "AI Generate" button (sparkle icon)

Pressing `N` from anywhere on the page creates a new draft.

### 2. Draft Rail

The rail is a scrollable vertical list with three tab buttons at the top:

| Tab | Filter | Count badge |
|-----|--------|-------------|
| Active | `status = 'draft'` | Number of active drafts |
| Scheduled | `status = 'scheduled'` | Number of scheduled items |
| Archive | `archived_at IS NOT NULL` | Number of archived items |

Each draft entry in the rail shows:
- **Title or preview**: `title` field if set, otherwise first ~60 characters of content
- **Content type badge**: "tweet" or "thread" as a small uppercase label
- **Relative timestamp**: "2m ago", "yesterday", "Mar 3"
- **Sync indicator**: Small dot — green (saved), pulsing (saving), orange (offline/local-only)
- **Selected state**: Highlighted background when this draft is active in the composer

The rail has a "New Draft" button at the bottom (or top, depending on scroll position).

Draft entries are sorted by `updated_at DESC` within each tab (most recently edited first).

### 3. New Draft

Creating a new draft:
1. User clicks "New Draft" button or presses `N` from the rail.
2. Frontend sends `POST /api/drafts` with `content_type: "tweet"` and `content: ""` (blank).
3. Server creates a `scheduled_content` row with `status = 'draft'`, returns `{ id }`.
4. Frontend selects the new draft in the rail. URL updates to `/drafts?id={newId}`.
5. Composer hydrates from the new (empty) record. Cursor focuses in the textarea.
6. User starts typing. Debounced PATCH saves begin after 1.5 seconds of inactivity.

The blank draft appears at the top of the Active tab immediately (optimistic insert).

### 4. Draft Selection

Clicking a draft in the rail:
1. If the current draft has unsaved changes, flush them with an immediate PATCH.
2. Update `selectedId` in the `draftStudio` store.
3. URL updates to `/drafts?id={newSelectedId}` (replaceState, not pushState — avoid polluting browser history).
4. Composer re-hydrates from the newly selected draft's server record.
5. The rail highlights the new selection.

If the user navigates to `/drafts?id=42` directly (bookmark, deep link), the rail auto-selects draft 42 and the composer loads it.

### 5. Composer Surface

The center zone renders `ComposeWorkspace` with a new `draftId` prop. Changes from the current component-local state model:

| Aspect | Current | Draft Studio |
|--------|---------|-------------|
| Initial state | Empty or localStorage recovery | Hydrated from server record |
| Save mechanism | localStorage only (500ms debounce) | Server PATCH (1.5s debounce) + localStorage fallback (500ms) |
| Submit action | `api.content.compose()` creates new row | Status transition on existing draft row |
| Mode/content type | Component-local `$state` | Read from `draft.content_type`, synced back on change |

The composer header shows a **sync badge** (see section 7).

All existing ComposeWorkspace features are preserved: inspector, preview overlay, AI assist, command palette, keyboard shortcuts, thread flow, media upload.

### 6. Thread Editing

Same `ThreadFlowLane` experience as today. Thread blocks are stored as `ThreadBlocksPayload` in the draft's `content` column. The debounced PATCH serializes blocks to the same format.

Thread validation runs client-side on every edit (as today) and server-side on schedule/publish transitions.

### 7. Sync Badge

A small status indicator in the composer header area:

| State | Display | Condition |
|-------|---------|-----------|
| Saved | "Saved" with checkmark, subtle text | Server confirmed last PATCH, no pending changes |
| Saving | "Saving..." with spinner | PATCH in flight |
| Unsaved | "Unsaved changes" | Local edits not yet sent (within debounce window) |
| Offline | "Offline — local backup" with warning icon | Server unreachable, localStorage fallback active |
| Conflict | "Newer version on server" with refresh action | Server `updated_at` is newer than last known (rare, multi-tab) |

Typefully does not show save status. This is a key differentiator.

### 8. Schedule

Scheduling a draft:
1. User opens the inspector's schedule section (or presses the schedule shortcut).
2. Picks a time from the TimePicker (preferred slots or custom datetime).
3. Frontend sends `POST /api/drafts/{id}/schedule` with `scheduled_for`.
4. Server updates `status = 'scheduled'` and `scheduled_for` on the row.
5. Draft moves from Active tab to Scheduled tab in the rail.
6. Sync badge shows "Scheduled for {time}".

Unscheduling:
1. User clicks "Unschedule" in the inspector or rail context menu.
2. Frontend sends `POST /api/drafts/{id}/unschedule`.
3. Server resets `status = 'draft'` and `scheduled_for = NULL`.
4. Draft moves from Scheduled tab back to Active tab.

This is a single-click operation, not a modal confirmation. The transition is reversible.

### 9. Archive

Archiving a draft:
1. User presses `Delete`/`Backspace` on a selected draft in the rail, or clicks the archive action.
2. Frontend sends `POST /api/drafts/{id}/archive`.
3. Server sets `archived_at = datetime('now')`.
4. Draft disappears from Active tab, appears in Archive tab.
5. If the archived draft was selected, the composer clears or selects the next draft.

No confirmation dialog for archive — it's non-destructive and reversible.

### 10. Restore

From the Archive tab:
1. User selects an archived draft and clicks "Restore" or presses `R`.
2. Frontend sends `POST /api/drafts/{id}/restore`.
3. Server sets `archived_at = NULL`.
4. Draft moves from Archive tab to Active tab.

### 11. Keyboard Navigation

#### Rail Context

| Key | Action |
|-----|--------|
| `Arrow Up` / `Arrow Down` | Navigate between drafts in the rail |
| `Enter` | Select the focused draft (load into composer) |
| `N` | Create a new draft |
| `Delete` / `Backspace` | Archive the focused draft |
| `R` | Restore (Archive tab only) |
| `D` | Duplicate the focused draft |
| `Tab` | Move focus from rail to composer |

#### Composer Context

All existing `ComposeWorkspace` shortcuts are preserved:

| Key | Action |
|-----|--------|
| `Cmd+Shift+Enter` | Submit (publish or schedule) |
| `Cmd+K` | Command palette |
| `Cmd+I` | Toggle inspector |
| `Cmd+Shift+P` | Toggle preview overlay |
| `Cmd+J` / `Cmd+Shift+J` | AI improve (inline assist) |
| `Cmd+Shift+N` | Switch to tweet mode |
| `Cmd+Shift+T` | Switch to thread mode |
| `Cmd+Enter` | Convert tweet to thread (tweet mode) / Split card (thread mode) |
| `Cmd+V` | Paste (with image support in Tauri) |
| `Escape` | Focus cascade: palette -> inspector -> rail |

#### Global

| Key | Action |
|-----|--------|
| `Cmd+K` | Command palette (extended with draft actions: archive, duplicate, new) |
| `Escape` | Focus cascade: command palette -> preview overlay -> inspector (mobile) -> from rail back to composer |

### 12. Mobile Behavior

On screens narrower than 768px:

- The rail collapses to a **top picker bar**: a horizontal scrollable row showing draft titles/previews as chips. Tapping a chip selects it.
- Below the picker bar, the full-width composer renders.
- The tab selector (Active/Scheduled/Archive) becomes a dropdown or segmented control above the picker bar.
- "New Draft" is a floating action button (bottom-right).
- Swipe left on a draft chip to archive.
- The inspector remains a bottom drawer (existing mobile behavior).

The three-zone layout is desktop-only (>= 768px). Mobile is a stacked single-column layout with the picker at the top.

## Typefully Differentiators

### Explicit Sync State
**Typefully**: No visible save indicator. Users guess whether their content is saved.
**Draft Studio**: Sync badge shows "Saved 2s ago" / "Saving..." / "Offline — local backup" in the composer header. Saving state is never ambiguous.

### Safer AI Undo
**Typefully**: No undo after AI rewrite. Content is permanently replaced.
**Draft Studio**: Every AI action (improve, generate, from-notes) snapshots before replacement. 10-second undo banner appears immediately. Beyond the banner, revision history stores snapshots accessible from the inspector.

### Revision Restore
**Typefully**: No revision history. No way to see or restore previous versions.
**Draft Studio**: `content_revisions` table stores snapshots on meaningful events: AI rewrite, schedule, unschedule, manual "save revision" action. The inspector shows a timeline of revisions. Click any revision to preview; click "Restore" to revert.

### Frictionless Lifecycle Transitions
**Typefully**: Separates drafts and scheduled posts into different views. Moving between them requires multiple clicks and sometimes re-entering data.
**Draft Studio**: One table, one UI. Status transitions (`draft -> scheduled -> posted`) happen on the same row. Schedule is one click. Unschedule is one click. The draft stays in the same composer — only the rail tab changes.

### Keyboard-First Navigation
**Typefully**: Arrow keys don't navigate drafts. No shortcut to create a new draft. Limited command palette.
**Draft Studio**: Full arrow-key navigation in the rail. `N` creates a new draft. `Enter` selects. `Delete` archives. 16+ composer shortcuts. Extended command palette with draft actions.

## Information Architecture

```
/drafts                    → Draft Studio (main workspace)
/drafts?id=42              → Draft Studio with draft 42 selected
/                          → Home (redirects to /drafts or shows analytics, configurable)
/content                   → Calendar (compose action creates draft, navigates to /drafts)
```

The sidebar nav item "Drafts" links to `/drafts`. The sidebar nav item "Home" behavior depends on the `homeSurface` setting:
- If `homeSurface === 'composer'`: redirects to `/drafts` (Draft Studio replaces the home composer)
- If `homeSurface === 'analytics'`: shows AnalyticsHome as today

## Component Inventory

| Component | Status | Notes |
|-----------|--------|-------|
| `DraftStudioShell.svelte` | New | Three-zone layout container |
| `DraftRail.svelte` | New | Draft list with tabs, selection, keyboard nav |
| `DraftRailEntry.svelte` | New | Individual draft item in the rail |
| `DraftSyncBadge.svelte` | New | Save status indicator |
| `DraftEmptyState.svelte` | New | Empty state for no-drafts and no-selection |
| `DraftMobilePicker.svelte` | New | Mobile top picker bar |
| `ComposeWorkspace.svelte` | Modified | Add `draftId` prop, server hydration, debounced PATCH |
| `composerAutosave.ts` | Modified | Draft-ID-scoped keys, downgrade to crash-recovery role |
| `draftStudio.ts` (store) | New | Collection state, selected ID, filters, sync status |
| `/drafts/+page.svelte` | Replaced | Renders `DraftStudioShell` instead of flat card list |
| `/+page.svelte` | Modified | Redirect to `/drafts` when homeSurface is composer |
| `/content/+page.svelte` | Modified | Compose action creates draft, navigates to `/drafts` |
| `Sidebar.svelte` | Unchanged | Already links to `/drafts` |
