# Session 05 Handoff

_Completed 2026-03-10_

---

## What Was Done

1. **Schema migration** — Added `scheduled_for TEXT DEFAULT NULL` column to `approval_queue` table (`20260311000100_scheduled_approval_intent.sql`). Backward-compatible; existing rows have NULL (immediate post intent).

2. **Core storage layer** — Added `scheduled_for` to `ApprovalRow`, `ApprovalItem`, and `From` impl. Updated `SELECT_COLS` to include the new column. Extended `enqueue_with_context_for` and `enqueue_with_provenance_for` with `scheduled_for` parameter. Added `scheduled` count to `ApprovalStats` and its query.

3. **Compose endpoints** — All four approval branches in `compose.rs` now pass `scheduled_for` (or `normalized_schedule`) through to the enqueue call: `compose_tweet`, `compose_thread`, `compose_thread_blocks_flow`, and `persist_content`. Response JSON includes `scheduled_for` when present.

4. **Approval endpoints** — `approve_item` now checks `scheduled_for` on the item. If the timestamp is in the future, it bridges to `scheduled_content` and sets the approval item status to `"scheduled"`. If past or absent, existing immediate-post behavior is preserved. `approve_all` / batch approve uses a shared `approve_single_item` helper that handles scheduling per-item.

5. **External callers updated** — `drafts.rs` (publish endpoint) and `discovery.rs` (queue reply) both pass `None` for `scheduled_for` to `enqueue_with_provenance_for`.

6. **Frontend** — Added `scheduled_for` to `ApprovalItem` type and `scheduled` to `ApprovalStats`. `ApprovalCard` shows a blue scheduling badge with timezone-aware formatting. `ApprovalStats` component shows "N scheduled" count. Approval page loads timezone from calendar store and threads it to cards. Settings copy updated to explain scheduling + approval interaction.

7. **Tests** — 5 new storage tests (scheduled_for preservation, null default, scheduled status in stats, exclusion from posting engine, provenance+schedule combo). 4 new workflow tests (autopilot+approval on/off, scheduled_for preservation through enqueue, schedule preserved across status changes).

8. **Documentation** — Created `mode-and-approval-matrix.md` with full mode matrix, status lifecycle, API response changes, and UI behavior.

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Nullable `scheduled_for` column on `approval_queue` | Simpler than a separate table. NULL = immediate post intent, which is correct for all existing rows. |
| D2 | `"scheduled"` status distinct from `"approved"` | Posting engine queries `status = 'approved'`. A separate status prevents double-posting without modifying the posting engine query. |
| D3 | Expired `scheduled_for` falls back to immediate post | User waited too long to approve. Posting immediately is better UX than erroring. |
| D4 | `approve_single_item` helper for batch approve | Each item may have different scheduling intent. Per-item handling is necessary; the helper avoids duplicating the bridge logic. |
| D5 | Timezone loaded from calendar store (not a new API call) | `loadSchedule()` from calendar store already fetches timezone. Reusing it avoids a duplicate API call. |

## Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/storage/approval_queue/mod.rs` | Added `scheduled_for` to row/item structs, `scheduled` to stats |
| `crates/tuitbot-core/src/storage/approval_queue/queries.rs` | `scheduled_for` in SELECT_COLS, enqueue functions, stats query |
| `crates/tuitbot-core/src/storage/approval_queue/tests.rs` | 5 new tests for scheduling intent |
| `crates/tuitbot-core/src/workflow/tests.rs` | 4 new approval+scheduling mode tests |
| `crates/tuitbot-server/src/routes/content/compose.rs` | Pass scheduled_for through all 4 approval branches |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Extra None param for scheduled_for |
| `crates/tuitbot-server/src/routes/discovery.rs` | Extra None param for scheduled_for |
| `crates/tuitbot-server/src/routes/approval.rs` | Schedule bridge on approve, approve_single_item helper, batch handling |
| `dashboard/src/lib/api/types.ts` | `scheduled_for` on ApprovalItem, `scheduled` on ApprovalStats |
| `dashboard/src/lib/components/ApprovalCard.svelte` | Scheduling badge, timezone prop, scheduled status styling |
| `dashboard/src/lib/components/ApprovalStats.svelte` | Scheduled count display |
| `dashboard/src/lib/stores/approval.ts` | Comment noting "scheduled" in WS handler |
| `dashboard/src/routes/(app)/approval/+page.svelte` | Load timezone, thread to cards, updated copy |
| `dashboard/src/routes/(app)/settings/SafetyLimitsSection.svelte` | Updated mode and approval hint text |

## Files Created

| File | Purpose |
|------|---------|
| `migrations/20260311000100_scheduled_approval_intent.sql` | Add `scheduled_for` column |
| `crates/tuitbot-core/migrations/20260311000100_scheduled_approval_intent.sql` | Same migration in crate dir for test DB |
| `docs/roadmap/scheduling-system-overhaul/mode-and-approval-matrix.md` | Full mode matrix documentation |
| `docs/roadmap/scheduling-system-overhaul/session-05-handoff.md` | This handoff |

## Quality Gates

| Gate | Result |
|------|--------|
| `cargo fmt --all --check` | Pass |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | 2152 passed, 0 failed |
| `cargo clippy --workspace -- -D warnings` | Pass |
| `npm run check` (svelte-check) | 0 errors, 0 warnings |
| `npm run build` | Success |

## Risks Carried Forward

- **`enqueue_with_provenance_for` now has 14 parameters.** Builder pattern refactor should be a future session.
- **ComposeWorkspace at ~1310 lines.** Over the 500-line limit. Schedule state extraction is a future refactor.
- **CalendarMonthView not timezone-aware for item grouping.** Lower priority summary view.
- **Legacy bare timestamps in DB.** Existing `scheduled_for` values in `scheduled_content` remain timezone-ambiguous. Migration is a later session.
- **No inline reschedule from approval card.** Users must approve first, then reschedule from Draft Studio or Calendar. Acceptable for MVP.
- **Batch approve with mixed scheduled/immediate items.** Each item is processed individually. A partial failure leaves some approved and some not — matches existing batch approve behavior.

## Session 06 Suggestions

1. **Scheduling validation hardening** — Timezone round-trip tests for `timezone.ts` functions with DST edge cases. Calendar store boundary tests for different account timezones.
2. **Server reschedule endpoint tests** — Integration tests for atomic reschedule flow (revision creation, activity logging, status preservation).
3. **E2E scheduling flow validation** — Full compose → schedule → reschedule → unschedule lifecycle.
4. **Legacy timestamp migration planning** — Document migration strategy for existing `scheduled_for` values lacking timezone context.
5. **ComposeWorkspace refactor** — Extract schedule state into a dedicated sub-store to bring the file under 500 lines.
