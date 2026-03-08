# Session 08 Handoff

## What Changed

### Store (`draftStudio.svelte.ts`)
- Added `scheduleDraft(id, scheduledFor)`: Calls `api.draftStudio.schedule()`, updates collection status/scheduled_for, switches to Scheduled tab.
- Added `unscheduleDraft(id)`: Calls `api.draftStudio.unschedule()`, resets status to `draft`, clears `scheduled_for`, switches to Active tab.
- Added `rescheduleDraft(id, scheduledFor)`: Unschedules then re-schedules in sequence. Updates collection on success.

### DraftDetailsPanel (`DraftDetailsPanel.svelte`)
- New props: `onschedule`, `onunschedule`, `onreschedule`, `onduplicate`.
- **Draft status**: Date/time picker with "Schedule" button. Defaults to tomorrow's next round hour.
- **Scheduled status**: Shows formatted scheduled time. "Unschedule" and "Reschedule" buttons. Reschedule toggles inline date/time inputs.
- **Posted status**: "Duplicate as draft" button.
- New state: `scheduleDate`, `scheduleTime`, `showReschedule`, `scheduling`.
- Schedule inputs sync from `draftSummary.scheduled_for` via `$effect` on draft change.

### DraftStudioShell (`DraftStudioShell.svelte`)
- `handleDraftSubmit()` now accepts `ComposeRequest` and schedules if `scheduled_for` is present (Inspector TimePicker → Cmd+Shift+Enter flow).
- New handlers: `handleSchedule`, `handleUnschedule`, `handleReschedule`, `handleDuplicateFromDetails`.
- All schedule handlers call `fetchDraft(id)` after success to refresh hydration with updated status.
- New props passed to `DraftDetailsPanel`: `onschedule`, `onunschedule`, `onreschedule`, `onduplicate`.

### Calendar Integration (`content/+page.svelte`)
- Added `handleEditScheduled(id)` which calls `goto('/drafts?id=${id}')`.
- Wired `onedit={handleEditScheduled}` on `CalendarWeekView` — clicking "Edit" on a calendar item navigates to Draft Studio with that item selected.

### Documentation
- `scheduling-flow.md`: State machine, server validations, integration points, two compose paths (studio vs calendar), race conditions, edge cases.

## Key Decisions

1. **Schedule UI in Details Panel, not Inspector**: The Inspector's TimePicker is designed for the calendar compose flow (submit clears state). For studio drafts, the Details Panel is the natural home — it shows status-specific controls and doesn't clear the composer on action.

2. **Reschedule = unschedule + schedule**: Server requires `status = 'draft'` to schedule. So reschedule must unschedule first. If the second call fails, the draft safely falls back to draft status. No content loss.

3. **Calendar quick-compose unchanged**: Calendar compose continues via the legacy `api.content.compose()` path. Scheduled items still appear in both the calendar and Draft Studio's Scheduled tab (same table). Unifying compose paths is deferred to Session 10.

4. **`handleDraftSubmit` accepts `ComposeRequest`**: This enables the Inspector TimePicker → Cmd+Shift+Enter scheduling flow as a secondary path. Without `scheduled_for`, the submit is a no-op (publish-now deferred to a later session).

5. **Default schedule time is tomorrow + 1 hour**: When opening the schedule picker for a draft, it defaults to a practical future time rather than "now" (which would schedule in the past within seconds).

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `DraftStudioShell.svelte` at ~490 lines (over 400-line Svelte guideline) | Medium | Low | ~115 lines are `<style>`. Script is ~240 lines. Extract schedule handlers to a utility if it grows further. |
| Reschedule race (unschedule succeeds, schedule fails) | Low | Low | Draft falls back to `draft` status safely. User sees it in Active tab and can retry. |
| Calendar compose creates items without draft identity | Medium | Low | By design. These items still appear in Studio Scheduled tab. Session 10 can unify paths. |
| `handleDraftSubmit` no-op without `scheduled_for` | Low | Low | Publish-now is a future session deliverable. Users can schedule from the Details Panel. |
| Date/time picker timezone handling | Medium | Medium | Uses local timezone via `new Date()` constructor. Server stores UTC. This matches existing Inspector TimePicker behavior. |

## Exact Inputs for Session 09

1. Read `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte` — revision timeline integration target
2. Read `dashboard/src/lib/stores/draftStudio.svelte.ts` — revision/activity store actions
3. Read `crates/tuitbot-server/src/routes/content/draft_studio.rs` — revision/activity endpoints
4. Read `crates/tuitbot-core/src/storage/scheduled_content/revisions.rs` — revision storage layer
5. Implement revision timeline component showing snapshots with preview/restore
6. Add activity log to details panel or a dedicated tab
7. Run: `npm --prefix dashboard run check`, full Rust CI
