# Scheduling API Contract

_Established in Session 02_

---

All scheduling-related endpoints validate timestamps using `tuitbot_core::scheduling::validate_and_normalize()`. The contract:

- **Input**: Any of the three accepted formats (UTC with Z, bare string, offset string)
- **Output**: Always UTC with Z suffix
- **Rejection**: Past timestamps (>5 min) return `400 Bad Request`
- **Rejection**: Unparseable strings return `400 Bad Request`

---

## Endpoints

### `POST /api/content/compose`

Unified compose endpoint. The `scheduled_for` field is optional.

```json
{
  "content_type": "tweet",
  "content": "Hello world",
  "scheduled_for": "2026-03-10T18:00:00Z"  // optional
}
```

**Validation**: `scheduled_for` is validated before any branching (approval mode, scheduling, or direct post). Invalid timestamps are rejected even if the content would have been routed to the approval queue.

### `POST /api/content/drafts/:id/schedule`

Legacy draft scheduling endpoint.

```json
{ "scheduled_for": "2026-03-10T18:00:00Z" }
```

**Validation**: Validates and normalizes. Returns normalized timestamp in response.

### `POST /api/drafts/:id/schedule`

Draft Studio scheduling endpoint. Transitions draft → scheduled.

```json
{ "scheduled_for": "2026-03-10T18:00:00Z" }
```

**Validation**: Validates and normalizes. Creates revision snapshot and activity log.

### `PATCH /api/drafts/:id/reschedule` (NEW)

Atomic reschedule. Changes the scheduled time without unscheduling first.

```json
{ "scheduled_for": "2026-03-10T20:00:00Z" }
```

**Precondition**: Item must be in `scheduled` status.

**Response**:
```json
{
  "id": 42,
  "status": "scheduled",
  "scheduled_for": "2026-03-10T20:00:00Z"
}
```

**Side effects**:
- Creates a revision snapshot with `trigger_kind: "reschedule"`
- Logs activity with `from` and `to` timestamps

### `POST /api/drafts/:id/unschedule`

Transitions scheduled → draft. Clears `scheduled_for`. No timestamp validation needed.

### `PATCH /api/content/scheduled/:id`

Edit a scheduled content item. The `scheduled_for` field is optional.

```json
{
  "content": "Updated content",
  "scheduled_for": "2026-03-10T20:00:00Z"
}
```

**Validation**: If `scheduled_for` is provided, it is validated and normalized.

### `GET /api/content/calendar?from=...&to=...`

Calendar range query. The `from` and `to` parameters are ISO-8601 strings. Currently not timezone-aware (planned for Session 3).

---

## Frontend Client Methods

### `api.draftStudio.reschedule(id, scheduledFor)`

```typescript
reschedule: (id: number, scheduledFor: string) =>
    request<{ id: number; status: string; scheduled_for: string }>(
        `/api/drafts/${id}/reschedule`,
        { method: 'PATCH', body: JSON.stringify({ scheduled_for: scheduledFor }) }
    )
```

### `buildScheduledFor(date, time, timezone)`

Converts a user-selected date/time in the account timezone to a UTC ISO-8601 string.

```typescript
import { buildScheduledFor } from '$lib/utils/timezone';

// User picks March 10, 2:00 PM in New York (EDT, UTC-4)
const utc = buildScheduledFor("2026-03-10", "14:00", "America/New_York");
// Returns: "2026-03-10T18:00:00Z"
```
