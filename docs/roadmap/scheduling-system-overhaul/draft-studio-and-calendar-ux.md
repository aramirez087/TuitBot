# Draft Studio & Calendar UX — Scheduling Consistency

_Session 04 · 2026-03-10_

---

## Overview

This document describes how the scheduling UX was unified across Compose, Draft Studio, and the Content Calendar so all three surfaces express scheduling with shared rules, matching copy, and timezone-correct behavior.

## Shared Scheduling Primitives

### Components

| Component | Location | Used By |
|-----------|----------|---------|
| `SchedulePicker.svelte` | `$lib/components/` | Composer `InspectorContent`, `DraftScheduleSection`, `ScheduleComposerSheet` |
| `ScheduleComposerSheet.svelte` | `$lib/components/composer/` | `HomeComposerHeader`, `ComposerCanvas` |

### Timezone Utilities (`$lib/utils/timezone.ts`)

| Function | Purpose |
|----------|---------|
| `buildScheduledFor(date, time, tz)` | Convert picker date+time in account tz → UTC ISO-8601 string for server |
| `formatInAccountTz(utcIso, tz, opts)` | Format UTC timestamp for display in account timezone |
| `toAccountTzParts(utcIso, tz)` | Extract `{ date, time }` parts from UTC timestamp in account tz for form inputs |
| `nowInAccountTz(tz)` | Get current `{ date, time }` in account timezone |

### Timezone Contract

- **All `scheduled_for` values sent to the server** are UTC ISO-8601 with Z suffix.
- **All user-facing display** uses `formatInAccountTz()` with the account timezone from `ScheduleConfig.timezone`.
- **Browser timezone is never used** for scheduling logic — only the IANA timezone from the account config.
- **Calendar date boundaries** for API queries are computed by converting account-tz midnight boundaries to UTC via `buildScheduledFor()`.

## Calendar Timezone Alignment

### Problem

The calendar store (`calendar.ts`) used browser-local `Date` methods (`getFullYear`, `getMonth`, `getDate`, `getHours`) for:
- Computing query range boundaries (`getDateRange()`)
- Grouping items by date/time slot in `CalendarWeekView`
- Determining "today" for highlighting

This caused items to appear in the wrong day column when the account timezone differed from the browser timezone (e.g., user in EST viewing at 11pm, server in UTC).

### Solution

1. **Calendar store**: `getDateRange()` now uses `buildScheduledFor(dateYMD, "00:00", tz)` to compute UTC boundaries aligned to account-tz midnight.
2. **CalendarWeekView**: Item grouping (`itemsByDate`, `itemsBySlot`) now uses `toAccountTzParts(item.timestamp, timezone)` instead of `new Date(item.timestamp).getHours()`.
3. **"Today" boundary**: Uses `nowInAccountTz(timezone).date` instead of browser-local `new Date()`.
4. **Layout grid dates**: Still use browser `Date` objects since they're midnight-anchored representative dates for grid positioning, not timestamps.

### `accountTimezone` Derived Store

```ts
export const accountTimezone = derived(schedule, ($s) => $s?.timezone ?? 'UTC');
```

Exposed from `calendar.ts` and consumed by `+page.svelte` (content page) to thread timezone to calendar views.

## Atomic Reschedule

### Problem

`draftStudio.svelte.ts` `rescheduleDraft()` performed sequential `unschedule()` → `schedule()` calls, which was racy (item could be picked up by the scheduler between calls).

### Solution

Uses the atomic `PATCH /api/drafts/:id/reschedule` endpoint (added in Session 02/03). Single server call:
1. Validates new time
2. Creates revision snapshot with trigger `"reschedule"`
3. Updates `scheduled_for` directly (no intermediate "draft" state)
4. Logs `"rescheduled"` activity with `{ from, to }` detail

## ContentItem Affordance Matrix

| Condition | Edit (content) | Reschedule | Unschedule | Cancel |
|-----------|:---:|:---:|:---:|:---:|
| Scheduled, source=manual | Yes | Yes | Yes | Yes |
| Scheduled, source=assist | No | Yes | Yes | Yes |
| Scheduled, source=discovery | No | Yes | Yes | Yes |
| Posted | No | No | No | No |
| Pending | No | No | No | No |

- **Reschedule**: Opens Draft Studio with item selected (same as Edit — user reschedules via `SchedulePicker` in the details panel).
- **Unschedule**: Calls `api.draftStudio.unschedule(id)`, reloads calendar. Falls back to `api.content.cancelScheduled(id)` for legacy items.
- **Scheduled time display**: Shows timezone-aware time with abbreviation (e.g., "2:00 PM EST") in expanded ContentItem.

## Prefill Flow

```
Calendar slot click
  → buildScheduledFor(date, time) produces account-tz-naive "YYYY-MM-DDTHH:MM:00"
  → URL param: ?prefill_schedule=2026-03-12T14:00:00
  → DraftStudioShell reads param, passes to DraftDetailsPanel
  → DraftScheduleSection: toAccountTzParts() extracts date/time for SchedulePicker
  → User confirms/adjusts → SchedulePicker triggers onschedule
  → handleSchedule: buildScheduledFor(date, time, timezone) → UTC ISO → server
```

## History Panel Enhancements

- **Trigger meta**: Added `reschedule: "Before Reschedule"` entry
- **Action meta**: Added `rescheduled: "Rescheduled"` entry
- **Activity detail**: Shows timezone-aware timestamps for `scheduled_for` and `from`/`to` (reschedule) with timezone abbreviation
- **Timezone prop**: Threaded from `DraftStudioShell` via `$scheduleStore.timezone`

## Metadata Section

- Shows scheduled time as "Mar 12, 2026, 2:00 PM EST" instead of browser-local "3/12/2026"
- Uses `formatInAccountTz()` with `timeZoneName: 'short'` option
