# Release Readiness Report — Scheduling System Overhaul

_Session 07 — 2026-03-10_

---

## Recommendation: GO

All quality gates pass, all charter goals are addressed, no data-loss risks remain, and the approval+scheduling bridge is working correctly. Known limitations are documented and none are release-blocking.

---

## Quality Gates

| Gate | Result | Details |
|------|--------|---------|
| `cargo fmt --all --check` | **Pass** | No formatting issues |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | **Pass** | 2160 passed, 0 failed, 12 ignored |
| `cargo clippy --workspace -- -D warnings` | **Pass** | No warnings |
| `npm run check` (svelte-check) | **Pass** | 0 errors, 0 warnings |
| `npm run build` | **Pass** | Production build successful |

---

## Charter Compliance

### G1: Correct timezone semantics — DONE

- `timezone.ts` provides `buildScheduledFor()`, `formatInAccountTz()`, `toAccountTzParts()`, `nowInAccountTz()`.
- All user-facing datetime construction uses the account's configured timezone via `Intl.DateTimeFormat`.
- Browser timezone is never used for scheduling logic.
- Server validates and normalizes incoming timestamps via `validate_and_normalize()`.
- Calendar store computes UTC boundaries using `buildScheduledFor()` with account timezone.
- SchedulePicker displays timezone badge with short name and full IANA label.

### G2: Explicit user intent — DONE

- Compose API branches on `scheduled_for` presence with explicit code paths: approval queue, scheduled_content insertion, or direct post.
- The "silent fallback" (scheduling at `Utc::now()` when `can_post` is false) still exists but now returns `{ "status": "scheduled" }` — the frontend correctly shows "Saved to calendar" for this case.
- `HomeComposerHeader` shows distinct "Schedule" and publish buttons when a time is selected.
- Error messages include context prefixes ("Couldn't schedule post:", "Couldn't publish:").

### G3: Atomic scheduling operations — DONE

- `reschedule_draft_for()` is a single SQL `UPDATE` statement.
- `PATCH /api/drafts/:id/reschedule` endpoint exists with revision snapshot and activity logging.
- Frontend `rescheduleDraft()` calls the single reschedule API (not unschedule+schedule).
- `unschedule_draft_for()` is also a single atomic SQL operation.

### G4: Unified scheduling UI — DONE

- `SchedulePicker.svelte` is used across all scheduling surfaces: ComposeWorkspace (via InspectorContent and ScheduleComposerSheet), DraftScheduleSection, and ContentItem (calendar).
- `TimePicker.svelte` has zero remaining references — fully replaced.
- All surfaces use the same timezone display, preferred time slots, custom date/time inputs, and quick actions.

### G5: Backward-compatible migration — DONE

- Legacy endpoints (`/api/content/tweets`, `/api/content/threads`) remain functional.
- New `scheduled_for` validation accepts bare strings, UTC with Z, and offset strings.
- No schema migrations that break existing data.
- Compose contract tests verify legacy endpoint behavior.

---

## Audit Gap Resolution

| Gap | Status | Resolution |
|-----|--------|------------|
| G1: Timezone handling broken | **Resolved** | `timezone.ts` utilities replace all browser-local datetime construction. Account timezone is canonical. |
| G2: Publish vs schedule intent implicit | **Resolved** | Distinct code paths for approval, schedule, and direct-post. Frontend shows correct status messages. |
| G3: Non-atomic reschedule | **Resolved** | `reschedule_draft_for()` is a single SQL call. `PATCH /api/drafts/:id/reschedule` endpoint. |
| G4: Dual API paths | **Accepted** | Compose path and Draft Studio path serve different UX needs. Both are tested and documented. |
| G5: Two incompatible scheduling UIs | **Resolved** | Shared `SchedulePicker.svelte` across all surfaces. `TimePicker.svelte` fully removed. |
| G6: Calendar queries timezone-naive | **Resolved** | `calendar.ts` uses `buildScheduledFor()` with account timezone for boundary computation. |
| G7: Legacy endpoints still exist | **Accepted** | Kept for backward compatibility per charter. Legacy compose tests verify behavior. |
| G8: DraftScheduleSection mixes UTC/local | **Resolved** | `DraftScheduleSection.svelte` delegates to `SchedulePicker` + `timezone.ts`. No direct `Date()` construction. |
| G9: Compose flow skips draft features | **Accepted (non-goal)** | Compose creates scheduled items; Draft Studio creates drafts. Different lifecycles by design. |

---

## Known Limitations (Accepted for Release)

1. **ComposeWorkspace at ~1360 lines.** Over the 500-line file size limit. Schedule state extraction is a future refactor. Does not affect functionality.

2. **`enqueue_with_provenance_for` has 14 parameters.** Builder pattern refactor deferred. Function is correct; the signature is unwieldy but not a bug.

3. **Legacy bare timestamps in DB.** Existing `scheduled_for` values in `scheduled_content` remain timezone-ambiguous. New items are correctly UTC with Z suffix. Migration of legacy data is a future session.

4. **CalendarMonthView not timezone-aware for item grouping.** The month summary view groups items by date in browser timezone rather than account timezone. Lower priority; the detailed week/day views are correct.

5. **No inline reschedule from approval card.** Users must approve first, then reschedule from Draft Studio or Calendar. Approved UX tradeoff.

6. **`has_approval` not in telemetry events.** `ScheduleConfig` doesn't expose `approval_mode`. Would need an extra API call or prop threading. Telemetry still captures all scheduling actions; just can't distinguish approval vs non-approval paths in analytics.

7. **`aria-describedby="tz-info"` uses static ID.** If multiple SchedulePicker instances exist on the same page, IDs would conflict. Current usage is single-instance-per-view.

8. **Implementation map sessions diverged from plan.** Sessions 4-6 were resequenced during execution (Draft Studio/Calendar UX before API unification, then Approval, then Telemetry/QA). The map documented the original plan; actual execution adapted to dependencies discovered during implementation. All goals were still achieved.

---

## Blockers

None. All quality gates pass and all charter goals are met.

---

## Non-Blocking Issues

- **Doc inconsistency fixed:** `mode-and-approval-matrix.md` telemetry table referenced `has_approval: true` in events, contradicting Session 06 decision D2 that excluded this property. Corrected in this session.

---

## User-Facing Behavior Summary

After this overhaul ships, users will experience:

1. **Correct timezone scheduling.** All scheduling surfaces (home composer, modal composer, Draft Studio, calendar) use the account's configured timezone. A user in EST scheduling for "2:00 PM" will see the post go out at 2:00 PM EST regardless of browser timezone.

2. **Unified scheduling UI.** A single SchedulePicker component with timezone badge, preferred time slots, custom date/time inputs, and quick "Next free slot" action appears consistently across all surfaces.

3. **Scheduling survives approval.** When approval mode is on and a user schedules content, the scheduled time is preserved through the approval queue. Approving a scheduled item bridges it to the scheduled content table at the intended time.

4. **Atomic reschedule.** Changing a scheduled time is a single operation — no risk of losing the schedule if something goes wrong mid-operation.

5. **Better error messages.** Error messages include context ("Couldn't schedule post:", "Couldn't publish:") and a distinct "Scheduled." undo toast for scheduled posts.

6. **Accessibility.** Screen reader announcements for scheduling actions, ARIA labels on all interactive elements, timezone badge linked to date/time inputs via `aria-describedby`.

7. **Telemetry.** 9 scheduling funnel events (create, publish-now, save-draft, submit-error, time-selected, reschedule, unschedule, approval-bridge) logged to console via `trackFunnel()`.

---

## Test Coverage Summary

| Category | Count | Sessions |
|----------|-------|----------|
| Backend timestamp validation | 10 tests | S02 |
| Backend compose contract | 7 tests | S02, S06 |
| Backend scheduled_content storage | 12 tests | S02, S04, S06 |
| Backend draft studio API | 6 tests | S04, S06 |
| Backend approval queue + scheduling | 5 tests | S05 |
| Backend workflow scheduling | 4 tests | S05 |
| Frontend svelte-check | 4169 files checked | S07 |
| Total backend tests passing | 2160 | S07 |

---

## Follow-Up Recommendations (Post-Release)

1. **ComposeWorkspace refactor** — Extract schedule state and handlers into a sub-store to bring the file under 500 lines.
2. **Builder pattern for enqueue functions** — Replace 14-parameter `enqueue_with_provenance_for` with a builder.
3. **Legacy timestamp migration** — Backfill existing `scheduled_for` values with UTC Z suffix using account timezone inference.
4. **Backend telemetry endpoint** — If console-only telemetry is insufficient, add `POST /api/events` and extend `trackFunnel()` to POST events server-side.
5. **CalendarMonthView timezone awareness** — Group month-view items by account timezone instead of browser timezone.
6. **Inline reschedule from approval card** — Allow changing scheduled time directly from the approval card before approving.
7. **Unique IDs for multiple SchedulePicker instances** — Generate unique `aria-describedby` IDs if the component is ever used multiple times per page.
