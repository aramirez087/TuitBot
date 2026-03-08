# Scheduling Flow — Draft Studio

## State Machine

```
draft ←→ scheduled → posted
  ↕          ↕
archive   archive
```

### Transitions

| From | To | Action | Server Endpoint | Side Effects |
|------|----|--------|-----------------|--------------|
| draft | scheduled | Schedule | `POST /api/drafts/{id}/schedule` | Creates revision snapshot, logs activity |
| scheduled | draft | Unschedule | `POST /api/drafts/{id}/unschedule` | Creates revision snapshot, logs activity |
| scheduled | scheduled | Reschedule | Unschedule + Schedule | Two revisions created (atomic from user's perspective) |
| scheduled | posted | Auto-post | Background worker | Status update, posted_at set |
| any | archive | Archive | `POST /api/drafts/{id}/archive` | Sets archived_at |
| archive | draft | Restore | `POST /api/drafts/{id}/restore` | Clears archived_at, resets status to draft |

### Server Validations

- **Schedule**: Requires `status = 'draft'`. Returns 400 if already scheduled or posted.
- **Unschedule**: Requires `status = 'scheduled'`. Returns 400 if draft or posted.
- **Both**: Validate the draft exists and belongs to the account.

## Integration Points

### Details Panel (Primary)
The Details Panel in Draft Studio is the primary scheduling interface:
- **Draft status**: Shows date/time picker + "Schedule" button
- **Scheduled status**: Shows scheduled time, "Unschedule" and "Reschedule" buttons
- **Posted status**: Shows "Duplicate as draft" button

### Inspector TimePicker (Secondary)
The ComposeWorkspace Inspector already has a TimePicker. When a user selects a time and hits Cmd+Shift+Enter, the shell's `handleDraftSubmit` checks for `scheduled_for` in the `ComposeRequest` and calls `scheduleDraft`.

### Calendar View
- Scheduled items from Draft Studio appear in the calendar via the shared `scheduled_content` table
- Clicking "Edit" on a calendar item navigates to `/drafts?id={id}`, opening it in Draft Studio
- Calendar auto-refreshes on `ContentScheduled` WebSocket events

## Two Compose Paths

### Studio Path (Canonical)
1. Create draft in Draft Studio
2. Write content in the embedded composer
3. Pick date/time in the Details Panel
4. Click "Schedule" → status transitions to `scheduled`
5. Draft persists with full identity (title, tags, revisions, notes)

### Calendar Quick-Compose Path (Legacy)
1. Click a time slot in the calendar
2. ComposeModal opens with prefilled time
3. Submit creates a new `scheduled_content` row via `api.content.compose()`
4. No draft identity — no title, tags, revisions, or notes

Both paths write to the same `scheduled_content` table, so items appear in both the calendar and the Draft Studio's Scheduled tab.

## Race Conditions

### Concurrent edits from multiple tabs
- **Autosave conflict**: `updated_at` version check catches stale writes. The studio shows a conflict banner with "Use mine" / "Reload" options.
- **Schedule while editing**: Schedule is a status-only operation (doesn't touch content), so it won't conflict with content autosave.

### Schedule + immediate edit
- After scheduling, the draft moves to the Scheduled tab. The user can still select and edit it. Editing a scheduled draft's content is allowed — only the status gate prevents double-scheduling.

### Reschedule atomicity
- Reschedule calls unschedule then schedule sequentially. If schedule fails after unschedule succeeds, the draft returns to `draft` status. The user can retry. No content is lost.

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Schedule draft with empty content | Allowed — server doesn't validate content on schedule. Details panel shows amber "Not ready" indicator. |
| Schedule archived draft | Not possible — archived drafts have `archived_at` set and won't appear in the active/scheduled views. Restore first. |
| Edit scheduled draft content | Allowed — content autosave works regardless of status. |
| View posted draft | Read-only in composer. "Duplicate as draft" available in Details Panel. |
| Calendar quick-compose item in Studio | Appears in Scheduled tab. Can be edited (navigates with full draft identity if the item has one). |
| WebSocket `ContentScheduled` event | Calendar auto-refreshes. Studio collection reloaded if viewing the same account. |
