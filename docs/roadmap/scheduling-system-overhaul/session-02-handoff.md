# Session 02 Handoff

_Completed 2026-03-10_

---

## What Was Done

1. **Created `scheduling.rs` validation module** (`tuitbot-core/src/scheduling.rs`) — shared contract for validating and normalizing `scheduled_for` timestamps. Accepts UTC, bare, and offset formats. Rejects past timestamps with configurable grace period (default 5 minutes). 14 unit tests.

2. **Wired validation into all 4 ingest paths**:
   - `compose.rs` — `persist_content()` and `compose_thread_blocks_flow()` both validate before branching on approval mode
   - `draft_studio.rs` — `schedule_studio_draft()` validates before scheduling
   - `drafts.rs` — `schedule_draft()` validates before scheduling
   - `scheduled.rs` — `edit_scheduled()` validates when `scheduled_for` is provided

3. **Added atomic reschedule**:
   - Storage: `reschedule_draft_for()` in `scheduled_content/mod.rs`
   - Handler: `reschedule_studio_draft()` in `draft_studio.rs`
   - Route: `PATCH /api/drafts/{id}/reschedule` registered in `lib.rs`
   - Creates revision snapshot + activity log with from/to times

4. **Created frontend timezone utilities** (`dashboard/src/lib/utils/timezone.ts`):
   - `buildScheduledFor(date, time, timezone)` — converts account-tz to UTC
   - `formatInAccountTz(utcIso, timezone)` — UTC to display string
   - `toAccountTzParts(utcIso, timezone)` — UTC to form input parts
   - `nowInAccountTz(timezone)` — current date/time in account tz

5. **Updated `composeHandlers.ts`** — uses `buildScheduledFor()` instead of browser-local `Date.setHours()` + `.toISOString().replace('Z','')`. Added optional `timezone` field to `BuildComposeRequestOpts`.

6. **Added `reschedule` to API client** — `api.draftStudio.reschedule(id, scheduledFor)`

7. **Fixed UTC format in fallback scheduling** — `chrono::Utc::now().format()` now uses `%Y-%m-%dT%H:%M:%SZ` (with Z) instead of bare format.

8. **Added tests**:
   - 14 unit tests in `scheduling.rs` (format parsing, past rejection, grace period, edge cases)
   - 2 storage tests in `scheduled_content/tests.rs` (reschedule success, non-scheduled returns false)
   - 5 compose contract tests (valid UTC, bare ISO, offset, past rejected, garbage rejected)
   - 5 Draft Studio tests (valid UTC schedule, past rejected, atomic reschedule, revision creation, activity logging)

9. **Documentation**: scheduling domain model, API contract, this handoff.

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Validate `scheduled_for` before approval_mode branching in compose | Ensures invalid timestamps are rejected regardless of whether content routes to approval queue. Without this, past/garbage timestamps would silently pass through approval mode. |
| D2 | 5-minute grace period for past rejection | Balances usability (clock skew, network latency) with correctness (don't allow obviously-past schedules). Configurable via `DEFAULT_GRACE_SECONDS`. |
| D3 | Bare strings treated as UTC for backward compat | Existing data and clients send bare strings. Breaking them would be a regression. The contract normalizes all to Z-suffixed. |
| D4 | `BuildComposeRequestOpts.timezone` is optional | Backward compatible — callers that don't pass it fall back to UTC. TypeScript compiler won't break existing callsites. |

## Files Created

| File | Purpose |
|------|---------|
| `crates/tuitbot-core/src/scheduling.rs` | Shared scheduling validation/normalization |
| `dashboard/src/lib/utils/timezone.ts` | Frontend timezone conversion utilities |
| `docs/roadmap/scheduling-system-overhaul/scheduling-domain-model.md` | Domain model documentation |
| `docs/roadmap/scheduling-system-overhaul/scheduling-api-contract.md` | API contract documentation |
| `docs/roadmap/scheduling-system-overhaul/session-02-handoff.md` | This handoff |

## Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/lib.rs` | Added `pub mod scheduling;` |
| `crates/tuitbot-core/src/storage/scheduled_content/mod.rs` | Added `reschedule_draft_for()` |
| `crates/tuitbot-core/src/storage/scheduled_content/tests.rs` | Added reschedule tests |
| `crates/tuitbot-server/src/lib.rs` | Registered `/drafts/{id}/reschedule` route |
| `crates/tuitbot-server/src/routes/content/mod.rs` | Re-exported `reschedule_studio_draft` |
| `crates/tuitbot-server/src/routes/content/compose.rs` | Early validation in `persist_content` and `compose_thread_blocks_flow`, UTC format fix |
| `crates/tuitbot-server/src/routes/content/draft_studio.rs` | Added `reschedule_studio_draft` handler, validation in `schedule_studio_draft` |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Validation in `schedule_draft` |
| `crates/tuitbot-server/src/routes/content/scheduled.rs` | Validation in `edit_scheduled` |
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Added 5 scheduling validation tests |
| `crates/tuitbot-server/tests/draft_studio_api_tests.rs` | Added 5 reschedule tests, updated existing test for Z-normalized timestamp |
| `dashboard/src/lib/utils/composeHandlers.ts` | Uses `buildScheduledFor()`, added `timezone` opt |
| `dashboard/src/lib/api/client.ts` | Added `draftStudio.reschedule` method |

## Session 03 Mission

**Replace the two scheduling UIs** (`TimePicker` and `DraftScheduleSection`) with a shared `SchedulePicker` component that uses the new timezone utilities.

### Starting Files

| File | Action | What to do |
|------|--------|------------|
| `dashboard/src/lib/components/composer/TimePicker.svelte` | Replace | Rewrite using `timezone.ts` utilities, receive `schedule.timezone` |
| `dashboard/src/lib/components/drafts/DraftScheduleSection.svelte` | Replace | Rewrite using `timezone.ts` utilities, or merge into shared SchedulePicker |
| `dashboard/src/lib/stores/calendar.ts` | Modify | Use account timezone for calendar range queries (`getDateRange`, `formatDateISO`) |
| `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` | Modify | Load ScheduleConfig and thread timezone to scheduling components |
| `dashboard/src/lib/components/calendar/ComposeWorkspace.svelte` | Modify | Pass `timezone` from ScheduleConfig to `buildComposeRequest` |

### Key Context for Session 03

- `timezone.ts` is ready with all conversion utilities
- `composeHandlers.ts` accepts optional `timezone` in opts but callers don't pass it yet
- `DraftStudioShell` doesn't load `ScheduleConfig` — needs to use the `schedule` store from `calendar.ts` or load it on mount
- The `schedule` store in `calendar.ts` already fetches `ScheduleConfig` including timezone
- `ComposeWorkspace.svelte:66` receives `schedule: ScheduleConfig | null` as a prop

## Risks Carried Forward

- **Existing bare timestamps in DB**: All current `scheduled_for` values are timezone-ambiguous. New values are correct; data migration is Session 6.
- **Frontend callers don't pass timezone yet**: `ComposeWorkspace` and `DraftStudioShell` need to thread `schedule.timezone` to the compose handler. This is Session 3 work.
- **Calendar queries still UTC-based**: `calendar.ts` range computation still uses browser-local time. Needs timezone-aware boundaries in Session 3.
- **Two scheduling UIs still exist**: `TimePicker` and `DraftScheduleSection` have different datetime construction. Unification is Session 3.
