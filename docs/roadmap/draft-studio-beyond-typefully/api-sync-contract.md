# Draft Studio API Sync Contract

## Overview

The Draft Studio API lives under `/api/drafts` and provides 13 endpoints for managing drafts in the studio workspace. These are additive to the existing `/api/content/drafts` endpoints, which remain unchanged for backward compatibility.

## Endpoints

### Collection

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/drafts` | List drafts with optional filters |
| `GET` | `/api/drafts/:id` | Get a single draft (full content) |
| `POST` | `/api/drafts` | Create a blank or seeded draft |

### Autosave

| Method | Path | Description |
|--------|------|-------------|
| `PATCH` | `/api/drafts/:id` | Autosave content with conflict detection |

### Metadata

| Method | Path | Description |
|--------|------|-------------|
| `PATCH` | `/api/drafts/:id/meta` | Update title and notes |

### Workflow Transitions

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/drafts/:id/schedule` | Transition draft → scheduled |
| `POST` | `/api/drafts/:id/unschedule` | Transition scheduled → draft |
| `POST` | `/api/drafts/:id/archive` | Soft-delete (set archived_at) |
| `POST` | `/api/drafts/:id/restore` | Restore from archive |
| `POST` | `/api/drafts/:id/duplicate` | Clone into a new draft |

### Revisions & Activity

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/drafts/:id/revisions` | List revision snapshots |
| `POST` | `/api/drafts/:id/revisions` | Create a manual revision |
| `GET` | `/api/drafts/:id/activity` | List activity log entries |

## Request/Response Shapes

### GET /api/drafts

**Query parameters:**
- `status` — `"draft"`, `"scheduled"`, or `"all"` (default: all non-archived)
- `search` — substring match on title and content preview
- `tag` — tag ID filter (reserved, not yet implemented)
- `archived` — `"true"` to show only archived items

**Response:** `DraftSummary[]`

```json
[{
    "id": 1,
    "title": "My Draft",
    "content_type": "tweet",
    "content_preview": "First 60 chars of content...",
    "status": "draft",
    "scheduled_for": null,
    "archived_at": null,
    "updated_at": "2026-03-08 12:00:00",
    "created_at": "2026-03-08 11:00:00"
}]
```

### GET /api/drafts/:id

**Response:** Full `ScheduledContent` object (same shape as legacy drafts list items, plus `title`, `notes`, `archived_at`).

### POST /api/drafts

**Request body (all fields optional):**
```json
{
    "content_type": "tweet",
    "content": "Initial content",
    "source": "manual",
    "title": "My Draft"
}
```

Defaults: `content_type = "tweet"`, `content = " "` (space placeholder), `source = "manual"`.

**Response:** `{ "id": number, "updated_at": string }`

### PATCH /api/drafts/:id (Autosave)

**Request body:**
```json
{
    "content": "Updated content",
    "content_type": "tweet",
    "updated_at": "2026-03-08 12:00:00"
}
```

The `updated_at` field is the client's last known server timestamp. The server compares this against the DB row's `updated_at`:
- **Match:** Update succeeds → `200 { "id": number, "updated_at": string }`
- **Mismatch:** Stale write → `409 { "error": "stale_write", "server_updated_at": string }`

**Autosave is side-effect free:**
- No revision created
- No activity logged
- Only updates `content`, `content_type`, and `updated_at`

### PATCH /api/drafts/:id/meta

**Request body:**
```json
{
    "title": "New Title",
    "notes": "Internal notes"
}
```

**Response:** Full `ScheduledContent` object.

### POST /api/drafts/:id/schedule

**Request body:**
```json
{ "scheduled_for": "2026-12-31T23:59:59" }
```

**Response:** `{ "id": number, "status": "scheduled", "scheduled_for": string }`

Side effects: Creates a revision (trigger: `schedule`), logs `scheduled` activity.

### POST /api/drafts/:id/unschedule

**Request body:** empty `{}`

**Response:** `{ "id": number, "status": "draft" }`

Side effects: Creates a revision (trigger: `unschedule`), logs `unscheduled` activity.

### POST /api/drafts/:id/archive

**Request body:** empty `{}`

**Response:** `{ "id": number, "archived_at": string }`

### POST /api/drafts/:id/restore

**Request body:** empty `{}`

**Response:** `{ "id": number }`

### POST /api/drafts/:id/duplicate

**Request body:** empty `{}`

**Response:** `{ "id": number }` (new draft's ID)

### GET /api/drafts/:id/revisions

**Response:** `ContentRevision[]`

```json
[{
    "id": 1,
    "content_id": 42,
    "account_id": "...",
    "content": "snapshot content",
    "content_type": "tweet",
    "trigger_kind": "schedule",
    "created_at": "2026-03-08 12:00:00"
}]
```

### POST /api/drafts/:id/revisions

**Request body:**
```json
{ "trigger_kind": "manual" }
```

**Response:** `{ "id": number }` (revision ID)

### GET /api/drafts/:id/activity

**Response:** `ContentActivity[]`

```json
[{
    "id": 1,
    "content_id": 42,
    "account_id": "...",
    "action": "created",
    "detail": null,
    "created_at": "2026-03-08 11:00:00"
}]
```

## Error Codes

| Status | Condition |
|--------|-----------|
| 400 | Invalid request body, wrong status for transition |
| 404 | Draft not found |
| 409 | Stale write during autosave (conflict detection) |

## Content Preview Truncation

The `content_preview` field in `DraftSummary` is generated server-side:
- For tweets: first 60 characters of content, trimmed
- For threads: first block's text extracted from JSON, then truncated to 60 chars
- Truncated previews end with `...`

## Backward Compatibility

The legacy `/api/content/drafts` routes are unchanged:
- `GET /api/content/drafts` — still works
- `POST /api/content/drafts` — still works (requires content, creates no activity)
- `PATCH /api/content/drafts/:id` — still works (requires content or blocks)
- `DELETE /api/content/drafts/:id` — still works
- `POST /api/content/drafts/:id/schedule` — still works (no revision)
- `POST /api/content/drafts/:id/publish` — still works

The frontend `api.drafts.*` namespace continues to call legacy endpoints. The new `api.draftStudio.*` namespace targets the new endpoints. Migration happens as UI components switch to the studio workspace.

## Frontend Client

The `api.draftStudio` namespace in `dashboard/src/lib/api/client.ts` covers all 13 endpoints with typed request/response shapes. New types are in `dashboard/src/lib/api/types.ts`:

- `DraftSummary` — list item shape
- `AutosaveResponse` — `{ id, updated_at }`
- `StaleWriteError` — `{ error: 'stale_write', server_updated_at }`
- `ContentRevision` — revision snapshot
- `ContentActivity` — activity log entry
