# Mode & Approval Matrix

_Session 05 — 2026-03-10_

This document defines how scheduling behavior interacts with operating mode (Autopilot vs Composer) and approval policy (on vs off). It is the canonical reference for all mode combinations.

## Matrix

| Mode | Approval | `scheduled_for` | Result |
|------|----------|-----------------|--------|
| Autopilot | Off | None | Autonomous posting via discovery loop |
| Autopilot | Off | Some(time) | Manual compose → `scheduled_content` at time |
| Autopilot | On | None | Autonomous → `approval_queue` → immediate post on approve |
| Autopilot | On | Some(time) | Manual compose → `approval_queue` with time → `scheduled_content` on approve |
| Composer | Off | None | Manual compose → direct post (if `can_post`) or `scheduled_content` at now |
| Composer | Off | Some(time) | Manual compose → `scheduled_content` at time |
| Composer | On | None | Manual compose → `approval_queue` → immediate post on approve |
| Composer | On | Some(time) | Manual compose → `approval_queue` with time → `scheduled_content` on approve |

## Key Invariants

1. **Scheduling intent survives approval.** When a user sets `scheduled_for` and approval is on, the timestamp is stored on the `approval_queue` row. On approval, the system creates a `scheduled_content` entry at the intended time instead of posting immediately.

2. **Autopilot does not suppress manual scheduling.** The `approval_mode` toggle and the autopilot discovery loop are orthogonal to the manual compose/schedule flow. A user can always compose and schedule content manually regardless of autopilot state.

3. **The "scheduled" status prevents double-posting.** When an approval item bridges to `scheduled_content`, its status transitions to `"scheduled"` (not `"approved"`). The posting engine queries `status = 'approved'` only, so scheduled items are excluded from immediate posting.

4. **Expired schedules fall back to immediate post.** If `scheduled_for` is in the past at the time of approval (user waited too long), the item is approved normally for immediate posting. No error is raised.

## Status Lifecycle

```
pending → approved → posted      (no schedule, immediate)
pending → approved → failed      (posting error)
pending → scheduled              (has future scheduled_for, bridges to scheduled_content)
pending → rejected               (user rejects)
pending → expired                (auto-expiry after configured hours)
```

## API Response Changes

### Compose endpoints (`POST /api/content/compose`)

When `approval_mode` is on and `scheduled_for` is provided, the response includes:
```json
{
  "status": "queued_for_approval",
  "id": 42,
  "scheduled_for": "2026-03-15T14:00:00Z"
}
```

### Approve endpoint (`POST /api/approval/:id/approve`)

When the item has a future `scheduled_for`:
```json
{
  "status": "scheduled",
  "id": 42,
  "scheduled_content_id": 99,
  "scheduled_for": "2026-03-15T14:00:00Z"
}
```

When the item has no schedule or a past schedule:
```json
{
  "status": "approved",
  "id": 42
}
```

### Stats endpoint (`GET /api/approval/stats`)

Now includes `scheduled` count:
```json
{
  "pending": 5,
  "approved": 12,
  "rejected": 3,
  "failed": 1,
  "scheduled": 2
}
```

## UI Behavior

- **ApprovalCard** shows a blue scheduling badge ("Scheduled for Mar 15, 2:00 PM EST") on items with `scheduled_for`.
- **ApprovalStats** shows "N scheduled" count when > 0.
- **Settings** copy explains that scheduled posts retain their target time through approval.
- **Approval page** subtitle mentions scheduled items posting at their intended time.

## Telemetry Coverage

Each matrix row is instrumented via the scheduling funnel events (Session 06). See `telemetry-and-copy-notes.md` for the full event inventory.

| Matrix Row | Telemetry Events |
|------------|-----------------|
| No Schedule + No Approval | `compose:publish-now` |
| Schedule + No Approval | `schedule:created`, `schedule:time-selected` |
| No Schedule + Approval | `compose:publish-now` |
| Schedule + Approval | `schedule:created`, `schedule:approval-bridge` (on approve) |
| Reschedule (any surface) | `schedule:reschedule` (source: `draft-studio` or `calendar`) |
| Unschedule (any surface) | `schedule:unschedule` (source: `draft-studio` or `calendar`) |
| Submit error (any path) | `compose:submit-error` |
