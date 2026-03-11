# Session 01 Handoff

_Completed 2026-03-10_

---

## What Was Done

1. Audited every user-facing scheduling entry point across the compose flow, Draft Studio, and calendar surfaces, documenting component chains, API paths, backend handlers, and storage operations.

2. Identified 9 concrete gaps (G1-G9) blocking first-class scheduling, with code-level references.

3. Researched competitive UX patterns from Typefully (queue-based scheduling, next free slot), Buffer (per-account timezone, calendar grid), and Planable (approval-aware scheduling, visual preview).

4. Defined the epic charter with problem statement, user stories, goals, non-goals, success metrics, and rollout risks.

5. Created a session-by-session implementation map (Sessions 2-6) with exact files, tests, and dependencies.

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Account timezone as canonical for all datetime ops | Eliminates G1 (timezone bugs). Browser timezone is irrelevant; `ScheduleConfig.timezone` is the source of truth. |
| D2 | Explicit `action` field in ComposeRequest | Eliminates G2 (implicit intent, silent fallback). No more inferring publish vs schedule from `scheduled_for` presence. |
| D3 | Atomic reschedule via `PATCH /api/drafts/:id/reschedule` | Eliminates G3 (non-atomic two-call reschedule). Single API call, single revision snapshot, no intermediate state. |
| D4 | Shared `SchedulePicker` component replaces both `TimePicker` and `DraftScheduleSection` | Eliminates G5 (two incompatible UIs). One component, one datetime construction path, one set of bugs to fix. |
| D5 | Calendar queries include account timezone | Eliminates G6 (timezone-naive queries). Server computes correct date range boundaries. |

## Session 02 Mission

**Establish the timezone foundation**: Create the shared `timezone.ts` utility, update backend to validate/normalize `scheduled_for`, and propagate `ScheduleConfig.timezone` to all scheduling surfaces.

### Starting Files

| File | Action | What to do |
|------|--------|------------|
| `dashboard/src/lib/utils/timezone.ts` | Create | `toAccountTz()`, `fromAccountTz()`, `formatInAccountTz()`, `buildScheduledFor()` using `Intl.DateTimeFormat` |
| `dashboard/src/lib/utils/composeHandlers.ts` | Modify | Replace lines 36-41 (`new Date(targetDate)` + `setHours` + `toISOString().replace('Z','')`) with `buildScheduledFor()` from timezone.ts |
| `dashboard/src/lib/components/drafts/DraftScheduleSection.svelte` | Modify | Replace lines 28-29 (mixed UTC/local) and line 49 (`buildIso()`) with timezone-aware construction |
| `dashboard/src/lib/stores/calendar.ts` | Modify | Replace `formatDateISO()` (line 87-89) and `getDateRange()` (lines 91-109) with timezone-aware boundary computation |
| `crates/tuitbot-server/src/routes/content/compose.rs` | Modify | Add `scheduled_for` validation in `persist_content()` — parse, validate, normalize to UTC with annotation |
| `crates/tuitbot-server/src/routes/content/calendar.rs` | Modify | Add optional `timezone` query param to `CalendarQuery`, use for range computation |

### Key Context for Session 02

- `ScheduleConfig.timezone` is already returned by `/api/content/schedule` (see `calendar.rs:156-173`) and stored in the `schedule` writable store (`calendar.ts:8`). It just needs to be threaded through to datetime construction.
- `ComposeWorkspace.svelte:66` receives `schedule: ScheduleConfig | null` as a prop. The timezone is available at `schedule?.timezone`.
- The `TimePicker` already receives `schedule` and could pass `schedule.timezone` to a utility function.
- `DraftStudioShell.svelte` does not currently receive `ScheduleConfig` — it will need to load it or receive it.
- Frontend timezone conversion should use `Intl.DateTimeFormat` with `timeZone` option — no external library needed.

## Decisions to Honor

- Do not change the `scheduled_for` column type in Session 2. Additive only.
- Do not remove `TimePicker` or `DraftScheduleSection` in Session 2. That's Session 3.
- Do not add the `action` field to `ComposeRequest` in Session 2. That's Session 4.
- Backend changes must be backward-compatible: bare strings without timezone still accepted.
- All timezone utility functions must be pure functions with unit tests.

## Open Questions

1. **Default timezone for accounts without one configured**: Should we default to UTC or attempt to detect from the user's browser on first use? Recommendation: Default to UTC with a prompt to set timezone in settings.

2. **Posting engine timezone handling**: The posting engine (`get_due_items_for`) compares `scheduled_for <= datetime('now')`. When we add timezone awareness, should the engine convert `scheduled_for` to UTC before comparison, or store everything in UTC and convert only for display? Recommendation: Store in UTC, convert for display. Defer detailed design to Session 6.

3. **ScheduleConfig availability in Draft Studio**: `DraftStudioShell` doesn't currently load ScheduleConfig. Options: (a) load it in onMount, (b) pass it down from the parent page, (c) use a global store. Recommendation: Use the existing `schedule` store from `calendar.ts` and load it in `DraftStudioShell.onMount()`.

## Risks Carried Forward

- **Existing bare datetimes in DB**: All current `scheduled_for` values are timezone-ambiguous bare strings. Session 2 makes new values correct but doesn't fix existing data. Data migration is Session 6.
- **Two compose paths**: Until Session 4 unifies them, the compose flow and Draft Studio flow will have different levels of timezone awareness after Session 2.
- **Legacy endpoints**: `/api/content/tweets` and `/api/content/threads` are not touched until Session 4.
