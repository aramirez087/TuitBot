# Revision History & Restore

## Revision Model

Every meaningful content lifecycle event creates a **revision snapshot** — a frozen copy of the content at that point in time.

### When Revisions Are Created

| Trigger | `trigger_kind` | Who Creates |
|---|---|---|
| Manual checkpoint | `manual` | User (via API) |
| Before scheduling | `schedule` | Server (schedule endpoint) |
| Before unscheduling | `unschedule` | Server (unschedule endpoint) |
| AI rewrite | `ai_rewrite` | Client (before applying AI result) |
| Before restore | `pre_restore` | Server (restore endpoint) |

Autosave does **not** create revisions. This is intentional: autosave fires frequently and would create noise. Meaningful snapshots are reserved for structural lifecycle events.

### Schema

```
content_revisions
├── id            (PK)
├── content_id    (FK → scheduled_content)
├── account_id    (scoping)
├── content       (full content snapshot)
├── content_type  (tweet / thread)
├── trigger_kind  (manual / schedule / ai_rewrite / pre_restore / ...)
└── created_at    (timestamp)
```

## Restore Safeguards

Restoring from a revision is a **non-lossy** operation:

1. **Pre-restore snapshot**: Before overwriting content, the server creates a `pre_restore` revision capturing the current state.
2. **Confirmation UI**: The frontend shows a two-click confirm flow. First click shows "Current state will be saved first." Second click executes.
3. **Activity log**: Every restore is logged as a `revision_restored` activity with the source `from_revision_id`.
4. **Atomic execution**: Snapshot + content update + activity log happen in a single server request, preventing partial state from autosave races.

### Restore Flow

```
User clicks "Restore" on revision R
  → UI shows confirmation message
User clicks "Confirm"
  → POST /api/drafts/:id/revisions/:rev_id/restore
    → Server: get current content C
    → Server: insert pre_restore revision with C
    → Server: update content to R.content
    → Server: insert revision_restored activity
    → Server: return updated draft
  → Client: re-hydrate composer with restored content
  → Client: reload revisions + activity
```

## Retention

Currently unbounded — all revisions are kept indefinitely. SQLite handles thousands of rows per draft efficiently. Revisions are listed newest-first and the UI renders all of them.

Future considerations:
- Age-based cleanup (e.g., keep 90 days)
- Count-based limit (e.g., keep last 50 per draft)
- Pagination for drafts with many revisions

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/drafts/:id/revisions` | List all revisions (newest first) |
| `POST` | `/api/drafts/:id/revisions` | Create manual revision |
| `POST` | `/api/drafts/:id/revisions/:rev_id/restore` | Restore content from revision |
| `GET` | `/api/drafts/:id/activity` | List activity log (newest first) |

## AI Change Visibility

Revisions with `trigger_kind = 'ai_rewrite'` and activities with `action = 'ai_rewrite'` are visually distinguished in the UI:
- Purple left border accent
- Sparkle icon
- "AI Rewrite" label

This makes AI-generated changes clearly identifiable and reversible.
