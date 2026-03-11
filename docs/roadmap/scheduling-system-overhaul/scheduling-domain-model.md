# Scheduling Domain Model

_Established in Session 02_

---

## Timestamp Contract

All `scheduled_for` values are stored as **UTC ISO-8601 with trailing `Z`**.

```
Format: YYYY-MM-DDTHH:MM:SSZ
Example: 2026-03-10T18:00:00Z
```

### Why UTC?

- SQLite's `datetime('now')` returns UTC.
- The posting engine compares `scheduled_for <= datetime('now')` — both sides must be UTC for correct comparison.
- Storing in UTC eliminates DST ambiguity and makes cross-timezone queries trivial.

## Account Timezone

The **account timezone** (`ScheduleConfig.timezone`, e.g., `"America/New_York"`) is the canonical user-facing timezone for display and input.

- Users see dates/times in their account timezone.
- Users pick dates/times in their account timezone.
- The system converts to UTC before sending to the server.
- The browser timezone is **never** used for scheduling logic.

## Responsibility Boundaries

| Layer | Responsibility |
|-------|---------------|
| **Frontend** | Convert user-selected date/time from account timezone to UTC before API call. Use `buildScheduledFor()` from `timezone.ts`. |
| **Backend** | Validate timestamp format, reject past timestamps (with 5-min grace), normalize to UTC with `Z` suffix. Use `scheduling::validate_and_normalize()`. |
| **Storage** | Store `scheduled_for` as `TEXT` column. All values are UTC with `Z`. |
| **Posting engine** | Compare `scheduled_for <= datetime('now')`. Both sides are UTC, so comparison is correct. |
| **Display** | Convert stored UTC back to account timezone using `formatInAccountTz()` or `toAccountTzParts()`. |

## Accepted Input Formats

The backend `normalize_scheduled_for()` function accepts:

1. **UTC with Z** (preferred): `"2026-03-10T14:00:00Z"` → stored as-is
2. **Bare string** (backward compat): `"2026-03-10T14:00:00"` → treated as UTC, `Z` appended
3. **Offset string**: `"2026-03-10T14:00:00+05:30"` → converted to UTC `"2026-03-10T08:30:00Z"`

## Past-Schedule Rejection

Timestamps more than **5 minutes** (300 seconds) in the past are rejected with a `400 Bad Request` error. This grace period accounts for:

- Network latency between client and server
- Clock skew between client and server
- Brief delays in form submission

## Reschedule Semantics

Rescheduling is a **single atomic operation** via `PATCH /api/drafts/:id/reschedule`. It:

1. Validates the new timestamp
2. Creates a revision snapshot (trigger_kind: `"reschedule"`)
3. Updates `scheduled_for` in a single SQL UPDATE (only if status = `'scheduled'`)
4. Logs an activity entry with `from` and `to` times

There is no intermediate "unscheduled" state — the item stays `scheduled` throughout.

## Data Migration Note

Existing `scheduled_for` values in the database are timezone-ambiguous bare strings generated before this contract was established. They were effectively UTC (server generated UTC, compose handler converted browser-local to UTC). The new contract makes all new values explicitly UTC with `Z`. A data migration to normalize existing values is planned for Session 6.
