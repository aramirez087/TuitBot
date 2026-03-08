# Draft Studio Data Model

## Overview

Draft Studio extends the existing `scheduled_content` table with three additive columns and introduces four supporting tables for revisions, tags, and activity tracking. All changes are additive and behaviorally reversible.

Migration: `20260308000100_draft_studio_foundation.sql`

## `scheduled_content` (extended)

| Column | Type | Default | New? | Description |
|--------|------|---------|------|-------------|
| `id` | INTEGER PK | AUTOINCREMENT | | Internal ID |
| `content_type` | TEXT NOT NULL | | | `'tweet'` or `'thread'` |
| `content` | TEXT NOT NULL | | | Tweet text or ThreadBlocksPayload JSON |
| `scheduled_for` | TEXT | NULL | | ISO-8601 timestamp or NULL |
| `status` | TEXT NOT NULL | `'scheduled'` | | `scheduled` / `posted` / `cancelled` / `draft` |
| `posted_tweet_id` | TEXT | NULL | | X tweet ID after posting |
| `created_at` | TEXT NOT NULL | `datetime('now')` | | Creation timestamp |
| `updated_at` | TEXT NOT NULL | `datetime('now')` | | Last modification timestamp |
| `source` | TEXT NOT NULL | `'manual'` | | `manual` / `assist` / `discovery` |
| `qa_report` | TEXT | `'{}'` | | Full QA report JSON |
| `qa_hard_flags` | TEXT | `'[]'` | | QA hard flags JSON |
| `qa_soft_flags` | TEXT | `'[]'` | | QA soft flags JSON |
| `qa_recommendations` | TEXT | `'[]'` | | QA recommendations JSON |
| `qa_score` | REAL | `0` | | QA score (0-100) |
| `account_id` | TEXT NOT NULL | sentinel UUID | | Account isolation |
| `title` | TEXT | NULL | Yes | Draft title for rail display |
| `notes` | TEXT | NULL | Yes | Free-form scratchpad (not posted) |
| `archived_at` | TEXT | NULL | Yes | Soft-delete timestamp; NULL = active |

### Lifecycle States

```
                 ┌──────────┐
   create ──────>│  draft    │
                 └────┬─────┘
                      │ schedule_draft_for
                      v
                 ┌──────────┐
                 │ scheduled │
                 └────┬─────┘
                      │ post
                      v
                 ┌──────────┐
                 │  posted   │
                 └──────────┘

   Any status ──> cancelled  (via cancel/delete)
```

Archive is orthogonal to status: `archived_at IS NOT NULL` hides the item from active lists regardless of status. The `list_drafts_for` query filters `AND archived_at IS NULL`.

## `content_revisions`

Stores content snapshots at meaningful lifecycle events.

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| `id` | INTEGER PK | AUTOINCREMENT | Revision ID |
| `content_id` | INTEGER NOT NULL | | FK -> `scheduled_content.id` (CASCADE) |
| `account_id` | TEXT NOT NULL | | Account ID |
| `content` | TEXT NOT NULL | | Content snapshot |
| `content_type` | TEXT NOT NULL | | `'tweet'` or `'thread'` |
| `trigger_kind` | TEXT NOT NULL | | `ai_rewrite` / `schedule` / `unschedule` / `manual` |
| `created_at` | TEXT NOT NULL | `datetime('now')` | Snapshot timestamp |

Index: `idx_content_revisions_content_id` on `content_id`.

**Note:** The technical architecture doc uses `trigger` as the column name, but `TRIGGER` is a SQL reserved word in SQLite. We use `trigger_kind` instead. The Rust struct field matches: `trigger_kind: String`.

## `content_tags`

User-defined tags for organizing content.

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| `id` | INTEGER PK | AUTOINCREMENT | Tag ID |
| `account_id` | TEXT NOT NULL | | Account ID |
| `name` | TEXT NOT NULL | | Tag name |
| `color` | TEXT | NULL | Optional hex color |

Constraint: `UNIQUE(account_id, name)` — tag names are unique per account.

## `content_tag_assignments`

Many-to-many join between content and tags.

| Column | Type | Description |
|--------|------|-------------|
| `content_id` | INTEGER NOT NULL | FK -> `scheduled_content.id` (CASCADE) |
| `tag_id` | INTEGER NOT NULL | FK -> `content_tags.id` (CASCADE) |

Primary key: `(content_id, tag_id)`.

## `content_activity`

Chronological log of content lifecycle events.

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| `id` | INTEGER PK | AUTOINCREMENT | Activity ID |
| `content_id` | INTEGER NOT NULL | | FK -> `scheduled_content.id` (CASCADE) |
| `account_id` | TEXT NOT NULL | | Account ID |
| `action` | TEXT NOT NULL | | Event type (see below) |
| `detail` | TEXT | NULL | Optional JSON metadata |
| `created_at` | TEXT NOT NULL | `datetime('now')` | Event timestamp |

Index: `idx_content_activity_content_id` on `content_id`.

### Activity action values

`created` | `edited` | `ai_rewrite` | `scheduled` | `unscheduled` | `archived` | `restored` | `posted`

## Compatibility Rules

### Existing functions unaffected

All existing `SELECT *` queries via `sqlx::FromRow` automatically pick up the three new nullable columns. No existing `INSERT` or `UPDATE` references these columns, so they receive defaults.

### Modified function

`list_drafts_for` now includes `AND archived_at IS NULL` to exclude archived drafts from the active list. Since `archived_at` defaults to NULL on all existing rows, this filter is a no-op until the first archive action.

### Rollback strategy

1. New columns can be dropped: `ALTER TABLE scheduled_content DROP COLUMN title` (SQLite 3.35+)
2. New tables can be dropped entirely
3. No existing data is modified by the migration

## Storage Functions

### Draft Studio operations (`drafts.rs`)

| Function | Signature | Description |
|----------|-----------|-------------|
| `archive_draft_for` | `(pool, account_id, id) -> Result<bool>` | Set `archived_at`; returns false if already archived |
| `restore_draft_for` | `(pool, account_id, id) -> Result<bool>` | Clear `archived_at`; returns false if not archived |
| `duplicate_draft_for` | `(pool, account_id, id) -> Result<Option<i64>>` | Clone content into new draft; returns new ID or None |
| `update_draft_meta_for` | `(pool, account_id, id, title, notes) -> Result<bool>` | Update title and notes |
| `list_archived_drafts_for` | `(pool, account_id) -> Result<Vec<ScheduledContent>>` | List archived items |

### Revision operations (`revisions.rs`)

| Function | Signature | Description |
|----------|-----------|-------------|
| `insert_revision_for` | `(pool, account_id, content_id, content, content_type, trigger_kind) -> Result<i64>` | Create snapshot |
| `list_revisions_for` | `(pool, account_id, content_id) -> Result<Vec<ContentRevision>>` | List by ID desc |

### Activity operations (`activity.rs`)

| Function | Signature | Description |
|----------|-----------|-------------|
| `insert_activity_for` | `(pool, account_id, content_id, action, detail) -> Result<i64>` | Log event |
| `list_activity_for` | `(pool, account_id, content_id) -> Result<Vec<ContentActivity>>` | List by ID desc |

### Tag operations (`tags.rs`)

| Function | Signature | Description |
|----------|-----------|-------------|
| `create_tag_for` | `(pool, account_id, name, color) -> Result<i64>` | Create tag |
| `list_tags_for` | `(pool, account_id) -> Result<Vec<ContentTag>>` | List by name |
| `assign_tag_for` | `(pool, content_id, tag_id) -> Result<()>` | Assign (idempotent) |
| `unassign_tag_for` | `(pool, content_id, tag_id) -> Result<bool>` | Remove assignment |
