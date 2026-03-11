# Telemetry, Copy & Accessibility Notes

_Session 06 — 2026-03-10_

---

## Telemetry Event Inventory

All events use the existing `trackFunnel()` pattern from `dashboard/src/lib/analytics/funnel.ts`. Events are logged as structured JSON to `console.info` with prefix `[tuitbot:funnel]`. No backend event store is needed yet — this matches the onboarding telemetry pattern (21 existing events).

### Event Table

| Event | Component | Trigger | Properties | Purpose |
|-------|-----------|---------|------------|---------|
| `schedule:created` | ComposeWorkspace | `handleSubmit` when `data.scheduled_for` is set | `mode`, `has_approval`, `timezone` | Measure schedule adoption from composer |
| `schedule:created` | DraftScheduleSection | `handleSchedule` for new schedule | `mode: 'draft-studio'`, `timezone` | Measure schedule adoption from Draft Studio |
| `compose:publish-now` | ComposeWorkspace | `handleSubmit` when publishing (no schedule, `canPublish`) | `mode`, `has_approval` | Measure direct publish vs schedule ratio |
| `compose:save-draft` | ComposeWorkspace | `handleSubmit` when saving (no schedule, no publish) | `mode` | Track save-to-calendar fallback |
| `compose:submit-error` | ComposeWorkspace | Error catch in `handleSubmit` | `error_type`, `mode` | Identify error patterns and confusion |
| `schedule:time-selected` | SchedulePicker | `selectPreferredTime` or `selectCustom` | `context`, `source`, `timezone` | Preferred-time vs custom split; identify which surfaces are used |
| `schedule:reschedule` | DraftScheduleSection / ContentItem | Reschedule handler / button click | `source: 'draft-studio' \| 'calendar'`, `timezone` (draft-studio only) | Measure reschedule frequency and surface |
| `schedule:unschedule` | DraftScheduleSection / ContentItem | Unschedule handler / button click | `source: 'draft-studio' \| 'calendar'` | Measure unschedule frequency |
| `schedule:approval-bridge` | ApprovalCard | Approve button click when `item.scheduled_for` is present | `has_scheduled_for: true` | Track how often scheduled items pass through approval |

### Decision: Console-Only Telemetry

Console-only telemetry (no backend event store) was chosen to match the existing onboarding pattern. This keeps the change set minimal and avoids adding new API endpoints or database tables. When a backend event store is needed, the `trackFunnel()` function can be extended to also `POST` events to an API endpoint.

---

## Copy Changes Inventory

| Location | Before | After | Rationale |
|----------|--------|-------|-----------|
| `DraftScheduleSection.svelte` — posted state label | "Actions" | "Post Actions" | More descriptive for screen readers and visual clarity |
| `ComposeWorkspace.svelte` — submit error | Raw error message | "Couldn't schedule post: {error}" / "Couldn't publish: {error}" / "Couldn't save draft: {error}" | User-friendly context prefix helps users understand which action failed |
| `ComposeWorkspace.svelte` — undo message | "Published." / "Saved to calendar." | "Scheduled." / "Published." / "Saved to calendar." (schedule-aware) | Distinct feedback for scheduled vs published vs saved |

---

## Accessibility Improvements Inventory

### Screen-Reader Announcements (`aria-live`)

| Component | Region | Trigger | Announcement |
|-----------|--------|---------|-------------|
| `ComposeWorkspace.svelte` | Existing `.sr-only` region | After successful scheduled submit | "Post scheduled for {time}" |
| `ComposeWorkspace.svelte` | Existing `.sr-only` region | After successful publish | "Post published" |
| `ComposeWorkspace.svelte` | Existing `.sr-only` region | After save to calendar | "Post saved to calendar" |
| `DraftScheduleSection.svelte` | New `.sr-only` region | After schedule/reschedule | "Draft scheduled for {time}" / "Schedule updated" |
| `DraftScheduleSection.svelte` | New `.sr-only` region | After unschedule | "Schedule removed" |
| `ContentItem.svelte` | New `.sr-only` region | After reschedule/unschedule button click | "Item rescheduled" / "Item unscheduled" |
| `ApprovalCard.svelte` | New `.sr-only` region | After approve action | "Item approved" / "Item approved and scheduled" |

### ARIA Attributes

| Component | Attribute | Value | Purpose |
|-----------|-----------|-------|---------|
| `SchedulePicker.svelte` | `aria-label` on container | "Schedule picker" | Landmark identification |
| `SchedulePicker.svelte` | `aria-describedby` on date/time inputs | Points to `#tz-info` timezone badge | Links timezone context to inputs |
| `SchedulePicker.svelte` | `context` prop | `'composer' \| 'draft-studio' \| 'calendar'` | Telemetry context (default: `'composer'`) |
| `ContentItem.svelte` | `aria-expanded` on container | Dynamic boolean | Communicates expand/collapse state |
| `ContentItem.svelte` | `aria-label` on container | "{type}: {preview}" | Content identification for screen readers |
| `ApprovalCard.svelte` | `aria-label` on card | "{typeLabel} approval item" | Card identification |
| `ApprovalStats.svelte` | `role="status"` + `aria-label` on stats bar | "Approval queue statistics" | Live region for stats updates |
| `EmptyState.svelte` | `role="status"` on container | — | Screen readers announce empty states |
| `DraftScheduleSection.svelte` | `aria-label` on section | "Schedule section" | Section identification |

---

## Dead Code Check

Searched for:
- `TimePicker` references — none found (was removed in Session 03)
- Unused scheduling imports in modified files — none found
- Residual inline date/time inputs in DraftScheduleSection — none (all delegated to SchedulePicker)
- Scheduling-related `// TODO` or `// FIXME` comments — none requiring resolution this session
