# Draft Studio Charter

## Problem Statement

Tuitbot has four independent compose/draft surfaces, each with its own state management, producing a fragmented writing experience:

1. **Home Composer** (`dashboard/src/routes/(app)/+page.svelte`): Full `ComposeWorkspace` embedded on `/`. State is component-local `$state` with localStorage autosave. Submits via `api.content.compose()` — creates a `scheduled_content` row directly, bypassing drafts entirely. If you close the tab, only localStorage recovery can save you.

2. **Drafts CRUD Page** (`dashboard/src/routes/(app)/drafts/+page.svelte`): A flat card-list page with inline textarea editing. No `ComposeWorkspace` integration. No autosave, no undo, no inspector, no preview, no keyboard shortcuts.

3. **Calendar Compose Modal** (`dashboard/src/routes/(app)/content/+page.svelte`): Opens `ComposeModal` -> `ComposeWorkspace` as a dialog. Same local-only state as home. No draft record created until final submit.

4. **Backend Draft Storage** (`crates/tuitbot-core/src/storage/scheduled_content.rs`): Drafts are rows in `scheduled_content` with `status = 'draft'`. Schema has: `id, content_type, content, scheduled_for, status, posted_tweet_id, source, created_at, updated_at, qa_*, account_id`. No title, notes, tags, archive flag, or revision history columns.

The result: users get two disconnected writing experiences (rich ComposeWorkspace with 16 shortcuts vs basic textarea), ephemeral compose state with no server backing until final submit, and no way to "open" a draft into the full composer.

## Goals

### G1: One Canonical Draft Workspace
`/drafts` becomes the canonical writing workspace — a three-zone layout (draft rail + composer + inspector). Home becomes a shortcut into Draft Studio, not a parallel composer. Calendar compose creates a new draft and opens Draft Studio.

### G2: Server-Backed Drafts as Single Source of Truth
Every compose action creates or selects a server draft record. Autosave writes to server with debounced PATCH (1.5s). localStorage is crash recovery only — a safety net, not the primary store.

### G3: Explicit, Visible, Reversible Transitions
Draft-to-scheduled-to-posted transitions are explicit status changes visible in the UI. Schedule/unschedule is a single click. Archive/restore replaces hard delete. Every transition is reversible except posting.

### G4: Safe AI and Undo
Every AI action snapshots before replacement. 10-second undo banner on all AI rewrites. Revision history stores snapshots on meaningful events (AI rewrite, schedule, manual save) for longer-term recovery.

### G5: Beat Typefully on Clarity, Safety, and Speed
Exceed Typefully on: sync state visibility, AI undo safety, revision restore, keyboard-first navigation, and frictionless draft lifecycle transitions.

## Locked Defaults

| Decision | Default | Rationale |
|----------|---------|-----------|
| One canonical draft workspace | `/drafts` route | Eliminates fragmentation. Home becomes a shortcut into Draft Studio. |
| Server-backed draft records | Every compose action creates or selects a server draft | No more ephemeral component state. |
| No independent home-composer state | Home links to `/drafts` or creates a new draft there | Prevents the split where home compose and drafts are disconnected. |
| Extend `scheduled_content` table | Add columns, not a parallel table | Preserves existing draft/schedule/post lifecycle. Status field already covers `draft -> scheduled -> posted -> cancelled`. |
| Additive, reversible migrations | New columns nullable or with defaults | No data loss risk. Old queries still work. |
| Local autosave as crash recovery only | Debounced server saves (1.5s), localStorage as fallback | Server is truth. Local recovery banner appears only when server state is stale. |
| Archive instead of hard delete | New `archived_at` column, status = 'archived' | Soft delete with restore capability. Existing `cancelled` status preserved for scheduled items. |
| Revision snapshots on meaningful events | `content_revisions` table, triggered by AI rewrite, schedule, unschedule | Not on every keystroke — only on events where rollback has clear value. |

## Non-Goals

### NG1: Don't Remove ComposeWorkspace
`ComposeWorkspace` becomes the composer surface inside Draft Studio. It gains a `draftId` prop to hydrate from server and debounced PATCH saves. The component is reused, not replaced.

### NG2: Don't Break Calendar Compose
Calendar compose creates a new draft and navigates to `/drafts?id={newId}`. The `ComposeModal` wrapper may be preserved as a thin redirect layer during transition.

### NG3: Don't Change Existing API Contracts
`ComposeRequest`, `ThreadBlock[]`, `ThreadBlocksPayload` — all preserved. New endpoints are additive. Existing `api.content.compose()` and `api.drafts.*` continue to work.

### NG4: Don't Introduce contenteditable
The editor remains `<textarea>`. The visual polish from the composer redesign charter (borderless, 15px font, conditional counters) applies within Draft Studio.

### NG5: Don't Change Tauri Bridge or Desktop Chrome
The sidecar launch, `localhost:3001` connection, and Tauri bridge are unchanged.

### NG6: Don't Create a Parallel Table
All draft/scheduled/posted content lives in `scheduled_content`. New columns are additive. New supporting tables (`content_revisions`, `content_tags`, `content_tag_assignments`, `content_activity`) reference `scheduled_content.id`.

## Relationship to Composer Redesign

The existing `docs/roadmap/composer-ui-typefully-redesign/` charter (Sessions S1-S5) focused on visual polish: borderless textareas, lighter chrome, dedicated preview overlay, shortcut safety. Draft Studio is a larger scope that subsumes those visual changes:

- **Visual polish** (redesign S2-S3): Incorporated into Draft Studio naturally — the composer surface inside Draft Studio benefits from the same restyling.
- **Shortcut safety** (redesign S4): Already partially shipped (undo snapshots exist in ComposeWorkspace). Draft Studio extends this with revision history.
- **Preview overlay** (redesign S3): Shipped as `ComposerPreviewSurface.svelte`. Preserved in Draft Studio.

Draft Studio sessions 02-11 do not re-implement the redesign work. They build on it.

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Compose surfaces | 4 independent (home, drafts, calendar, modal) | 1 canonical (`/drafts`) |
| Server-backed drafts | Only when explicitly saving on drafts page | Always — every keystroke debounce-saves |
| Crash recovery | localStorage only, 7-day TTL | Server truth + localStorage fallback |
| Undo after AI rewrite | 10-second banner (home composer only) | 10-second banner everywhere + revision history |
| Draft-to-scheduled transition | Create new row via compose endpoint | Status change on existing row |
| Archive/restore | Hard delete (status = 'cancelled') | Soft archive with restore |
| Keyboard navigation in draft list | None | Full arrow-key nav, Enter to select, N to create |
