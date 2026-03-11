# Composer Scheduling UX

_Session 3 — 2026-03-10_

---

## Component Hierarchy

```
ComposeWorkspace (orchestrator — owns scheduledDate, selectedTime, accountTimezone)
├── HomeComposerHeader (embedded mode)
│   └── ScheduleComposerSheet (popover, opened from clock icon)
│       └── SchedulePicker (shared)
├── ComposerCanvas (modal mode)
│   └── ScheduleComposerSheet (popover, opened from clock icon)
│       └── SchedulePicker (shared)
└── InspectorContent
    └── SchedulePicker (shared, inline in inspector)

DraftStudioShell → DraftDetailsPanel → DraftScheduleSection
    └── SchedulePicker (shared, inline in panel)
```

## State Machine

ComposeWorkspace manages two pieces of schedule state:
- `scheduledDate: string | null` — "YYYY-MM-DD" from SchedulePicker
- `selectedTime: string | null` — "HH:MM" from SchedulePicker

A post is "scheduled" when both are non-null. The UTC ISO-8601 `scheduledFor` is derived from these two values plus `accountTimezone` using `buildScheduledFor()`.

### CTA State Matrix

| canPublish | isScheduled | submitting | Primary CTA | Secondary |
|-----------|------------|-----------|-------------|-----------|
| true | false | false | "Publish" | Clock icon (opens schedule sheet) |
| true | true | false | "Schedule · Mar 12, 2:00 PM EST" | "Publish now" icon |
| false | false | false | "Save to Calendar" | Clock icon (opens schedule sheet) |
| false | true | false | "Schedule · Mar 12, 2:00 PM EST" | — |
| any | false | true | "Posting..." | disabled |
| any | true | true | "Scheduling..." | disabled |

## Shared SchedulePicker Component

`dashboard/src/lib/components/SchedulePicker.svelte`

### Props
- `timezone: string` — IANA timezone from ScheduleConfig
- `preferredTimes: string[]` — Quick-select time slots
- `selectedDate: string | null` — Currently selected date (YYYY-MM-DD)
- `selectedTime: string | null` — Currently selected time (HH:MM)
- `scheduledFor: string | null` — UTC ISO for display (scheduled state)
- `status: 'draft' | 'scheduled' | 'posted'` — Current content status
- `compact: boolean` — Smaller sizing for inspector panel

### Events
- `onschedule(date, time)` — User picked a date+time
- `onunschedule()` — User cleared the schedule

### Features
- **Timezone badge**: Shows timezone name and abbreviation (e.g., "New York (EST)")
- **Date + time inputs**: Native `<input type="date">` and `<input type="time">`
- **Preferred time pills**: Quick-select buttons from ScheduleConfig.preferred_times
- **Next free slot**: Finds next preferred time after now, or first slot tomorrow
- **Clear button**: Removes schedule selection
- **Scheduled state display**: Shows formatted scheduled time with unschedule action

## ScheduleComposerSheet

`dashboard/src/lib/components/composer/ScheduleComposerSheet.svelte`

A popover that wraps SchedulePicker for the composer context:
- Positioned absolutely below the trigger button
- Closes on Escape, click outside
- On mobile (< 640px): renders as a bottom sheet (full-width, slides up)
- Auto-closes after schedule selection

## Timezone Flow

1. `ScheduleConfig.timezone` is the canonical user-facing timezone
2. SchedulePicker displays times in account timezone
3. User picks date + time in account timezone context
4. `buildScheduledFor(date, time, timezone)` converts to UTC ISO-8601
5. `buildComposeRequest()` sends UTC `scheduled_for` to server
6. Display uses `formatInAccountTz()` for human-readable labels

## Keyboard Access

- Tab to clock icon button in header → Enter to open schedule sheet
- Tab through date input, time input, preferred time pills
- Enter on time input to set custom time
- Escape to close schedule sheet
- Cmd+Shift+Enter to submit (publish or schedule)

## DraftScheduleSection Integration

DraftScheduleSection now wraps SchedulePicker and accepts `timezone` and `preferredTimes` props from DraftStudioShell (which loads ScheduleConfig via `loadSchedule()`). All datetime construction uses `buildScheduledFor()` instead of browser-local `new Date()`.
