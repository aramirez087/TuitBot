# Draft Studio Technical Architecture

## Data Model

### Current `scheduled_content` Schema

```sql
CREATE TABLE scheduled_content (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,                            -- 'tweet' or 'thread'
    content TEXT NOT NULL,                                 -- text or ThreadBlocksPayload JSON
    scheduled_for TEXT,                                    -- ISO-8601 or NULL
    status TEXT NOT NULL DEFAULT 'scheduled',              -- scheduled | posted | cancelled | draft
    posted_tweet_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    source TEXT NOT NULL DEFAULT 'manual',                 -- manual | assist | discovery
    qa_report TEXT DEFAULT '{}',
    qa_hard_flags TEXT DEFAULT '[]',
    qa_soft_flags TEXT DEFAULT '[]',
    qa_recommendations TEXT DEFAULT '[]',
    qa_score REAL DEFAULT 0,
    account_id TEXT NOT NULL DEFAULT '00000000-...'
);
```

### Migration: Additive Columns on `scheduled_content`

```sql
-- Migration: 20260308000022_draft_studio_schema.sql

-- Optional title for drafts (shown in rail, not required)
ALTER TABLE scheduled_content ADD COLUMN title TEXT DEFAULT NULL;

-- Free-form notes field (internal context, not posted)
ALTER TABLE scheduled_content ADD COLUMN notes TEXT DEFAULT NULL;

-- Soft-delete timestamp (NULL = not archived)
ALTER TABLE scheduled_content ADD COLUMN archived_at TEXT DEFAULT NULL;
```

All columns are nullable with defaults. Existing queries (`SELECT *`) will return the new columns with NULL/default values. No existing query uses explicit column lists that would break.

Impact on existing storage functions in `scheduled_content.rs`:
- `ScheduledContent` struct gains `title: Option<String>`, `notes: Option<String>`, `archived_at: Option<String>`.
- All `SELECT *` queries automatically pick up new columns via `sqlx::FromRow`.
- No existing `INSERT` or `UPDATE` queries reference these columns, so they get defaults.
- `list_drafts_for` needs a filter update: `WHERE status = 'draft' AND archived_at IS NULL` to exclude archived drafts from the active list.

### New Table: `content_revisions`

```sql
CREATE TABLE content_revisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL,
    content TEXT NOT NULL,                  -- snapshot of content at this point
    content_type TEXT NOT NULL,             -- 'tweet' or 'thread'
    trigger TEXT NOT NULL,                  -- 'ai_rewrite' | 'schedule' | 'unschedule' | 'manual'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_content_revisions_content_id ON content_revisions(content_id);
```

Revisions are created on meaningful events only (not on every autosave keystroke). Trigger values:
- `ai_rewrite`: Before an AI improve/generate/from-notes action replaces content.
- `schedule`: When a draft transitions to scheduled status.
- `unschedule`: When a scheduled item returns to draft status.
- `manual`: User explicitly clicks "Save revision" in the inspector.

### New Table: `content_tags`

```sql
CREATE TABLE content_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT DEFAULT NULL,
    UNIQUE(account_id, name)
);
```

### New Table: `content_tag_assignments`

```sql
CREATE TABLE content_tag_assignments (
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES content_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (content_id, tag_id)
);
```

### New Table: `content_activity`

```sql
CREATE TABLE content_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL,
    action TEXT NOT NULL,                   -- 'created' | 'edited' | 'ai_rewrite' | 'scheduled' |
                                            -- 'unscheduled' | 'archived' | 'restored' | 'posted'
    detail TEXT DEFAULT NULL,               -- optional JSON with action-specific metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_content_activity_content_id ON content_activity(content_id);
```

## API Surface

### New Endpoints

| Method | Path | Description | Session |
|--------|------|-------------|---------|
| `GET` | `/api/drafts` | List drafts with optional filters (`status`, `tag`, `search`, `archived`) | 03 |
| `GET` | `/api/drafts/:id` | Get a single draft by ID | 03 |
| `POST` | `/api/drafts` | Create a blank draft (returns `{ id }`) | 03 |
| `PATCH` | `/api/drafts/:id` | Autosave: update content, content_type | 03 |
| `PATCH` | `/api/drafts/:id/meta` | Update title, notes, tags | 03 |
| `POST` | `/api/drafts/:id/schedule` | Transition draft -> scheduled | 03 |
| `POST` | `/api/drafts/:id/unschedule` | Transition scheduled -> draft | 03 |
| `POST` | `/api/drafts/:id/archive` | Set `archived_at`, remove from active list | 03 |
| `POST` | `/api/drafts/:id/restore` | Clear `archived_at`, return to active list | 03 |
| `POST` | `/api/drafts/:id/duplicate` | Clone draft content into a new draft row | 03 |
| `GET` | `/api/drafts/:id/revisions` | List revision snapshots for a draft | 03 |
| `POST` | `/api/drafts/:id/revisions` | Create a manual revision snapshot | 03 |
| `GET` | `/api/drafts/:id/activity` | List activity log entries for a draft | 03 |

### Modified Endpoints

| Method | Path | Change | Session |
|--------|------|--------|---------|
| `GET` | `/api/content/drafts` | Deprecated in favor of `/api/drafts`. Kept as alias during transition. | 03 |
| `PATCH` | `/api/content/drafts/:id` | Deprecated in favor of `/api/drafts/:id`. Kept as alias. | 03 |

### Autosave PATCH Contract

```typescript
// Client sends (debounced 1.5s after last keystroke):
PATCH /api/drafts/:id
{
    content: string,           // tweet text or ThreadBlocksPayload JSON
    content_type: "tweet" | "thread"
}

// Server responds:
200 { id: number, updated_at: string }
// or
409 { error: "stale_write", server_updated_at: string }
```

The server compares `updated_at` to detect stale writes. The client sends `If-Unmodified-Since` or an `updated_at` field; if the server's row is newer, it returns 409. The client can then re-fetch and merge or overwrite.

## Frontend Architecture

### New Store: `draftStudio.ts`

```typescript
// dashboard/src/lib/stores/draftStudio.ts

interface DraftSummary {
    id: number;
    title: string | null;
    content_type: string;
    content_preview: string;    // first ~60 chars of content
    status: string;
    scheduled_for: string | null;
    archived_at: string | null;
    updated_at: string;
    created_at: string;
}

interface DraftStudioState {
    collection: DraftSummary[];
    selectedId: number | null;
    tab: 'active' | 'scheduled' | 'archive';
    filters: {
        search: string;
        tags: number[];
        type: 'all' | 'tweet' | 'thread';
    };
    syncStatus: 'saved' | 'saving' | 'unsaved' | 'offline' | 'conflict';
    loading: boolean;
    error: string | null;
}
```

The store uses Svelte 5 runes (`$state`, `$derived`). It exposes actions:
- `loadDrafts()`: Fetch collection from server, filtered by current tab/filters.
- `selectDraft(id)`: Set selectedId, update URL.
- `createDraft()`: POST to server, add to collection, select.
- `archiveDraft(id)`: POST archive, move in collection.
- `restoreDraft(id)`: POST restore, move in collection.
- `duplicateDraft(id)`: POST duplicate, add to collection, select.
- `setSyncStatus(status)`: Update sync badge.

### New Components

| Component | File | Responsibility |
|-----------|------|---------------|
| `DraftStudioShell` | `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` | Three-zone layout. Reads `draftStudio` store. Renders rail + composer + inspector. |
| `DraftRail` | `dashboard/src/lib/components/drafts/DraftRail.svelte` | Tab bar, scrollable draft list, keyboard navigation, "New Draft" button. |
| `DraftRailEntry` | `dashboard/src/lib/components/drafts/DraftRailEntry.svelte` | Single draft item: preview, badge, timestamp, sync dot. |
| `DraftSyncBadge` | `dashboard/src/lib/components/drafts/DraftSyncBadge.svelte` | Save status indicator with icon and text. |
| `DraftEmptyState` | `dashboard/src/lib/components/drafts/DraftEmptyState.svelte` | Empty state for no-drafts and no-selection. |
| `DraftMobilePicker` | `dashboard/src/lib/components/drafts/DraftMobilePicker.svelte` | Mobile horizontal chip picker. |

### Modified Components

| Component | Change |
|-----------|--------|
| `ComposeWorkspace.svelte` | New optional `draftId` prop. When present: hydrate from server, debounced PATCH on changes, sync badge integration. When absent: existing behavior (home/modal). |
| `composerAutosave.ts` | Draft-ID-scoped localStorage keys (`tuitbot:compose:draft:{id}`). Downgraded to crash-recovery: only reads on mount if server fetch fails. |
| `/drafts/+page.svelte` | Complete replacement: renders `DraftStudioShell` instead of flat card list. |
| `/+page.svelte` | When `homeSurface === 'composer'`: redirect to `/drafts`. Analytics mode unchanged. |
| `/content/+page.svelte` | "Compose" action creates a draft via API, then navigates to `/drafts?id={newId}`. |

### State Flow

```
URL: /drafts?id=42
          │
          ▼
    draftStudio store (module-level $state)
    ├── collection: DraftSummary[]       ← GET /api/drafts
    ├── selectedId: 42                   ← from URL param
    ├── tab: 'active'                    ← user selection
    ├── filters: { search: '', ... }     ← user input
    ├── syncStatus: 'saved'              ← from PATCH responses
    │
    DraftStudioShell
    ├── DraftRail
    │   ├── reads collection, selectedId
    │   ├── emits selectDraft(id), createDraft(), archiveDraft(id)
    │   └── keyboard: ↑↓ = navigate, Enter = select, N = new, Del = archive
    │
    ├── ComposeWorkspace (draftId=42)
    │   ├── onMount: GET /api/drafts/42 → hydrate mode, tweetText, threadBlocks
    │   ├── $effect: debounced PATCH /api/drafts/42 (1.5s) on content change
    │   ├── localStorage fallback: tuitbot:compose:draft:42 (500ms, crash recovery)
    │   ├── DraftSyncBadge: reads syncStatus from store
    │   └── All existing features: inspector, preview, AI, shortcuts
    │
    └── (Inspector is inside ComposeWorkspace, unchanged)
```

### Autosave Flow

```
User types in ComposeWorkspace
         │
         ├─── 500ms debounce ──→ localStorage (crash recovery)
         │                        key: tuitbot:compose:draft:{id}
         │
         └─── 1500ms debounce ─→ PATCH /api/drafts/{id}
                                  ├── 200 OK → syncStatus = 'saved'
                                  ├── 409 Conflict → syncStatus = 'conflict'
                                  └── Network error → syncStatus = 'offline'
                                       └── localStorage remains as fallback
```

On page load:
1. Fetch draft from server (`GET /api/drafts/{id}`).
2. Check localStorage for crash recovery data (`tuitbot:compose:draft:{id}`).
3. If localStorage timestamp > server `updated_at`: show recovery banner (same UX as today).
4. Otherwise: hydrate from server, clear localStorage.

## Phased Session Plan

### Session 02: Draft Domain and Schema

**Rust changes:**
- New migration file: `20260308000022_draft_studio_schema.sql`
- Add columns to `scheduled_content`: `title`, `notes`, `archived_at`
- Create tables: `content_revisions`, `content_tags`, `content_tag_assignments`, `content_activity`
- Update `ScheduledContent` struct with new `Option<String>` fields
- Update `list_drafts_for` query to filter `archived_at IS NULL`
- Add new storage functions: `archive_draft_for`, `restore_draft_for`, `duplicate_draft_for`, `update_draft_meta_for`
- Add revision storage functions: `insert_revision_for`, `list_revisions_for`
- Add activity storage functions: `insert_activity_for`, `list_activity_for`
- Add tag storage functions: `create_tag_for`, `list_tags_for`, `assign_tag_for`, `unassign_tag_for`

**Tests:** All existing `scheduled_content` tests pass against migrated schema. New tests for archive/restore/duplicate/revision/activity functions.

**Dependencies:** None (first implementation session).

### Session 03: Draft API and Sync Contract

**Rust changes:**
- New route module: `crates/tuitbot-server/src/routes/content/draft_studio.rs`
- Register all new endpoints in router
- Implement autosave PATCH with `updated_at` conflict detection
- Implement schedule/unschedule transitions with revision snapshots
- Implement archive/restore
- Implement duplicate
- Implement revision and activity list endpoints
- Deprecation aliases for old `/api/content/drafts` paths

**Frontend changes:**
- Update `dashboard/src/lib/api/client.ts` with new `draftStudio` API namespace
- New methods: `fetchDrafts`, `fetchDraft`, `createDraft`, `autosaveDraft`, `updateDraftMeta`, `scheduleDraft`, `unscheduleDraft`, `archiveDraft`, `restoreDraft`, `duplicateDraft`, `fetchRevisions`, `fetchActivity`

**Tests:** API integration tests for all endpoints. Conflict detection test.

**Dependencies:** Session 02 (schema must be in place).

### Session 04: Draft Studio Shell and Selection

**Frontend changes:**
- New `draftStudio.ts` store
- New `DraftStudioShell.svelte`: three-zone layout
- New `DraftRail.svelte`: tab bar, draft list, selection
- New `DraftRailEntry.svelte`: draft item display
- New `DraftEmptyState.svelte`: empty state
- Replace `/drafts/+page.svelte` content with `DraftStudioShell`
- URL-based selection (`/drafts?id=42`)

**Tests:** `npm run check` passes. Manual: rail renders, selection works, URL updates.

**Dependencies:** Session 03 (API must be available).

### Session 05: Composer Binding and Server Autosave

**Frontend changes:**
- `ComposeWorkspace.svelte`: add optional `draftId` prop
- When `draftId` is set: hydrate from `GET /api/drafts/{id}`, set up debounced PATCH
- New `DraftSyncBadge.svelte`: save status indicator
- Modify `composerAutosave.ts`: draft-ID-scoped keys, downgrade to crash recovery
- Recovery banner logic: compare localStorage timestamp vs server `updated_at`

**Tests:** `npm run check` passes. Manual: typing saves to server, sync badge updates, crash recovery works.

**Dependencies:** Session 04 (shell must render composer).

### Session 06: Rail Keyboard and Multi-Draft Actions

**Frontend changes:**
- Full keyboard navigation in `DraftRail` (arrow keys, Enter, N, Delete, R, D, Tab)
- Focus management: rail <-> composer transitions
- Command palette extensions: "New Draft", "Archive Draft", "Duplicate Draft"
- Bulk selection in rail (Shift+click, Cmd+click) for bulk archive

**Tests:** `npm run check` passes. Manual: keyboard navigation works, command palette shows draft actions.

**Dependencies:** Session 05 (composer must be bound to drafts).

### Session 07: Metadata, Filters, and Organization

**Frontend changes:**
- Title editing: inline editable field in rail or composer header
- Tag management UI: create tags, assign to drafts, filter by tag
- Search input in rail header: filters drafts by content/title substring
- Filter by content type (all/tweet/thread)
- Sort options (recently edited, created date, scheduled date)

**Tests:** `npm run check` passes. Manual: tags create/assign/filter, search works.

**Dependencies:** Session 06 (rail must support keyboard nav for filter interactions).

### Session 08: Scheduling, Queue, and Calendar Flow

**Frontend changes:**
- Schedule from Draft Studio inspector: sets `scheduled_for`, transitions status
- Unschedule: returns to draft status
- Calendar integration: clicking a time slot in `/content` creates a draft with prefilled schedule, navigates to `/drafts?id={id}`
- Scheduled tab in rail shows scheduled drafts with countdown

**Rust changes (if needed):** Calendar queries may need updating to include draft-studio-created items.

**Dependencies:** Session 07 (metadata support needed for rich scheduled views).

### Session 09: Activity History and Restore

**Frontend changes:**
- Revision timeline in inspector panel: list of snapshots with trigger type and timestamp
- Preview a revision: overlay showing the content at that point
- Restore from revision: replaces current content with revision snapshot (with undo)
- Activity log: chronological list of actions (created, edited, AI rewrite, scheduled, etc.)

**Rust changes:** Revision creation hooks in schedule/unschedule/AI-rewrite endpoints.

**Dependencies:** Session 08 (schedule/unschedule must create revisions).

### Session 10: Entrypoints and Rollout

**Frontend changes:**
- Home page (`/`): when `homeSurface === 'composer'`, redirect to `/drafts`
- Calendar compose creates draft and navigates to `/drafts?id={id}`
- Sidebar: update "Home" link behavior based on surface setting
- Feature flag: `draft_studio_enabled` in settings store. When disabled, old behavior preserved.
- Mobile layout: `DraftMobilePicker` for narrow screens

**Dependencies:** Sessions 04-09 (all features must be in place).

### Session 11: Validation and Launch Readiness

**Tasks:**
- Full QA pass: all flows (create, edit, schedule, unschedule, archive, restore, duplicate)
- Accessibility audit: ARIA labels, focus management, keyboard nav, screen reader announcements
- Mobile testing: responsive layout, touch targets, swipe gestures
- Performance profiling: draft list with 100+ items, autosave under load
- Remove feature flag (or keep as opt-out escape hatch)
- Update `docs/composer-mode.md` with Draft Studio documentation

**Dependencies:** Session 10 (all entrypoints must route correctly).

## Migration Safety

### Existing Queries Affected by Schema Changes

All queries in `scheduled_content.rs` use `SELECT *` via `sqlx::FromRow`. Adding nullable columns with defaults is safe — SQLx maps NULL to `Option<T>`.

| Function | Query Pattern | Impact |
|----------|--------------|--------|
| `get_by_id_for` | `SELECT * WHERE id = ? AND account_id = ?` | Safe: new columns returned as NULL |
| `get_in_range_for` | `SELECT * WHERE ... ORDER BY ...` | Safe: no filter on new columns |
| `get_due_items_for` | `SELECT * WHERE status = 'scheduled' AND ...` | Safe: no filter on new columns |
| `list_drafts_for` | `SELECT * WHERE status = 'draft' AND account_id = ?` | Needs update: add `AND archived_at IS NULL` |
| `insert_for` | `INSERT ... (account_id, content_type, content, scheduled_for)` | Safe: new columns get defaults |
| `insert_draft_for` | `INSERT ... (account_id, content_type, content, status, source)` | Safe: new columns get defaults |
| `update_draft_for` | `UPDATE ... SET content = ? WHERE ...` | Safe: doesn't touch new columns |
| `delete_draft_for` | `UPDATE ... SET status = 'cancelled' WHERE ...` | Safe: doesn't touch new columns |
| `update_status_for` | `UPDATE ... SET status = ?, posted_tweet_id = ? WHERE ...` | Safe: doesn't touch new columns |
| `update_content_for` | `UPDATE ... SET content = ?, scheduled_for = ? WHERE ...` | Safe: doesn't touch new columns |
| `update_qa_fields_for` | `UPDATE ... SET qa_* WHERE ...` | Safe: doesn't touch new columns |

### Rollback Strategy

If a migration causes issues:
1. The new columns can be dropped with `ALTER TABLE scheduled_content DROP COLUMN title` etc. (SQLite 3.35+).
2. New tables can be dropped entirely.
3. No existing data is modified by the migration.

The migration is additive and behaviorally reversible.

## Performance Considerations

### Autosave Debounce
- Server PATCH: 1500ms debounce prevents excessive writes. At ~40 WPM typing, this means ~1 save every 1.5 seconds, not every keystroke.
- localStorage: 500ms debounce (unchanged from current). Acts as crash buffer between server saves.

### Draft Collection Loading
- `GET /api/drafts` should return lightweight summaries (id, title, content_preview, status, timestamps) rather than full content for all drafts.
- The server truncates `content` to ~60 characters for the `content_preview` field.
- Full content is loaded only for the selected draft via `GET /api/drafts/:id`.

### Revision Storage
- Revisions store full content snapshots (not diffs). For typical tweet-length content (<280 chars), this is negligible.
- Thread content (multiple blocks as JSON) is larger but still small (<10KB per revision).
- A draft with 50 revisions: ~500KB total. Acceptable for SQLite.
