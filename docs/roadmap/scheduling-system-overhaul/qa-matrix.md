# QA Matrix — Scheduling System Overhaul

_Session 06 — 2026-03-10_

This document defines scenario-based QA validation for the scheduling system overhaul shipped across Sessions 01-06. Each scenario should be traced through the code to confirm the described path exists.

---

## A. Compose -> Schedule (6 scenarios)

| # | Scenario | Conditions | Expected Result |
|---|----------|-----------|-----------------|
| A1 | Tweet + schedule + no approval | Composer mode, `approval_mode = false`, future `scheduled_for` | `scheduled_content` entry created, calendar shows item, response has `scheduled_for` and `status: "scheduled"` |
| A2 | Thread + schedule + no approval | Composer mode, `approval_mode = false`, future `scheduled_for`, 2+ blocks | `scheduled_content` with thread JSON, response has `block_ids` and `scheduled_for` |
| A3 | Tweet + schedule + approval on | Any mode, `approval_mode = true`, future `scheduled_for` | `approval_queue` entry with `scheduled_for`, response `status: "queued_for_approval"` with `scheduled_for` |
| A4 | Publish now + no approval | Composer mode, `approval_mode = false`, no `scheduled_for`, `can_post = true` | Direct post via X API, `status: "posted"` |
| A5 | Publish now + approval on | Any mode, `approval_mode = true`, no `scheduled_for` | `approval_queue` entry with `scheduled_for = NULL`, response `status: "queued_for_approval"` |
| A6 | Past time rejected | Any config, `scheduled_for` in past (>5 min) | HTTP 400, error message contains "past" |

**Code paths:**
- `crates/tuitbot-server/src/routes/content/compose.rs` — `compose_tweet`, `compose_thread`, `compose_thread_blocks_flow`, `persist_content`
- Frontend: `ComposeWorkspace.svelte` `handleSubmit` -> `buildComposeRequest` -> `onsubmit`

---

## B. Draft Studio Scheduling (5 scenarios)

| # | Scenario | Expected Result |
|---|----------|-----------------|
| B1 | Schedule a draft | `POST /api/drafts/:id/schedule` -> status = "scheduled", revision created with `trigger_kind: "schedule"`, activity logged as "scheduled" |
| B2 | Reschedule a scheduled draft | `PATCH /api/drafts/:id/reschedule` -> atomic update, new revision with `trigger_kind: "reschedule"` and `detail` containing `from`/`to` timestamps |
| B3 | Unschedule a scheduled draft | `POST /api/drafts/:id/unschedule` -> status = "draft", `scheduled_for` cleared |
| B4 | Schedule non-draft status fails | `POST /api/drafts/:id/schedule` on already-scheduled item -> HTTP 400 |
| B5 | Duplicate a scheduled draft | `POST /api/drafts/:id/duplicate` -> new draft created, `scheduled_for` cleared, title has "(copy)" suffix |

**Code paths:**
- `crates/tuitbot-server/src/routes/content/drafts.rs` — schedule, unschedule, reschedule, duplicate endpoints
- `crates/tuitbot-core/src/storage/scheduled_content/` — `schedule_draft_for`, `reschedule_draft_for`, `unschedule_draft_for`
- Frontend: `DraftScheduleSection.svelte` -> `SchedulePicker.svelte`

---

## C. Approval + Scheduling (5 scenarios)

| # | Scenario | Expected Result |
|---|----------|-----------------|
| C1 | Approve item with future schedule | `scheduled_for` is in the future -> bridges to `scheduled_content`, approval item status set to `"scheduled"` |
| C2 | Approve item with past schedule | `scheduled_for` is in the past -> immediate post (fallback), status = "approved" |
| C3 | Approve item with no schedule | `scheduled_for` is NULL -> immediate post, status = "approved" |
| C4 | Batch approve mixed items | Each item processed individually via `approve_single_item` helper; scheduled items bridge, immediate items post directly |
| C5 | Reject scheduled item | Status = "rejected", no `scheduled_content` created |

**Code paths:**
- `crates/tuitbot-server/src/routes/approval.rs` — `approve_item`, `approve_single_item`, `approve_all`
- Frontend: `ApprovalCard.svelte` — approve button fires `schedule:approval-bridge` telemetry when `scheduled_for` present

---

## D. Calendar & ContentItem (4 scenarios)

| # | Scenario | Expected Result |
|---|----------|-----------------|
| D1 | Reschedule from calendar item | ContentItem reschedule button fires `schedule:reschedule` telemetry, navigates to Draft Studio with SchedulePicker prefilled |
| D2 | Unschedule from calendar item | ContentItem unschedule button fires `schedule:unschedule` telemetry, API call with fallback (draftStudio -> scheduledContent), item removed from calendar |
| D3 | Cancel from calendar item | Status set to "cancelled", removed from active calendar view |
| D4 | Edit from calendar (manual only) | Edit button shown only when `source === 'manual'`, navigates to Draft Studio with draft loaded |

**Code paths:**
- `dashboard/src/lib/components/ContentItem.svelte` — action buttons with telemetry
- `dashboard/src/routes/(app)/content/+page.svelte` — calendar page event handlers

---

## E. Timezone Edge Cases (4 scenarios)

| # | Scenario | Expected Result |
|---|----------|-----------------|
| E1 | Account tz != browser tz | All displayed times use account timezone via `formatInAccountTz()`, not browser local time |
| E2 | DST transition (spring forward) | 2:30 AM in spring-forward zone -> valid UTC conversion via `buildScheduledFor()` (no "missing hour" error) |
| E3 | DST transition (fall back) | Ambiguous 1:30 AM -> deterministic UTC conversion (Intl API picks one) |
| E4 | UTC account timezone | No offset applied; times display as UTC with "UTC" abbreviation |

**Code paths:**
- `dashboard/src/lib/utils/timezone.ts` — `buildScheduledFor`, `formatInAccountTz`, `toAccountTzParts`, `nowInAccountTz`
- `dashboard/src/lib/components/SchedulePicker.svelte` — timezone display via `tzLabel()` and `tzFull()`

---

## F. Autopilot Interactions (3 scenarios)

| # | Scenario | Expected Result |
|---|----------|-----------------|
| F1 | Autopilot on + manual schedule | Manual compose works normally alongside autopilot. User can schedule via composer or Draft Studio. |
| F2 | Autopilot on + approval on + schedule | Manual compose -> `approval_queue` with `scheduled_for` preserved. Autopilot items have no `scheduled_for`. |
| F3 | Autopilot off + no `can_post` + compose | Falls back to schedule/save-to-calendar flow (no publish-now button shown when `canPublish = false`) |

**Code paths:**
- `crates/tuitbot-server/src/routes/content/compose.rs` — `read_approval_mode`, `can_post` check
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` — `canPublish` prop controls submit button behavior

---

## G. Telemetry Verification (9 events)

| Event | Trigger Location | Expected Properties |
|-------|-----------------|---------------------|
| `schedule:created` | ComposeWorkspace `handleSubmit` (scheduled path) | `mode`, `has_approval`, `timezone` |
| `compose:publish-now` | ComposeWorkspace `handleSubmit` (publish path) | `mode`, `has_approval` |
| `compose:save-draft` | ComposeWorkspace `handleSubmit` (save path) | `mode` |
| `compose:submit-error` | ComposeWorkspace error catch | `error_type`, `mode` |
| `schedule:time-selected` | SchedulePicker `selectPreferredTime` / `selectCustom` | `context`, `source`, `timezone` |
| `schedule:reschedule` | DraftScheduleSection `handleSchedule` (reschedule) / ContentItem button | `source`, `timezone` (draft-studio) or `source` (calendar) |
| `schedule:unschedule` | DraftScheduleSection `handleUnschedule` / ContentItem button | `source` |
| `schedule:created` (draft-studio) | DraftScheduleSection `handleSchedule` (new schedule) | `mode: 'draft-studio'`, `timezone` |
| `schedule:approval-bridge` | ApprovalCard approve button (when `scheduled_for` present) | `has_scheduled_for: true` |

**Verification method:** Open browser console, filter for `[tuitbot:funnel]`, perform each action, confirm JSON output matches expected properties.

---

## Coverage Cross-Reference

Each row in `mode-and-approval-matrix.md` is covered:

| Matrix Row | QA Scenarios |
|------------|-------------|
| Autopilot Off + Approval Off + No Schedule | A4 |
| Autopilot Off + Approval Off + Schedule | A1, A2 |
| Autopilot Off + Approval On + No Schedule | A5, C3 |
| Autopilot Off + Approval On + Schedule | A3, C1, C2 |
| Autopilot On + Approval Off + No Schedule | F1 |
| Autopilot On + Approval Off + Schedule | F1, A1 |
| Autopilot On + Approval On + No Schedule | F2, A5 |
| Autopilot On + Approval On + Schedule | F2, A3 |
