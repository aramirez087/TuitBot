# Implementation Map: Scheduling System Overhaul

_Created 2026-03-10_

---

## Session Dependency Graph

```
Session 1 (this session)
    ├── Charter, audit, competitive research, this map
    │
Session 2: Timezone Foundation
    ├── Backend normalization, shared tz utility, ScheduleConfig propagation
    │
Session 3: Unified Scheduling Component
    ├── Merge TimePicker + DraftScheduleSection into SchedulePicker
    │   Depends on: Session 2 (tz utility)
    │
Session 4: API Unification
    ├── Explicit action field, atomic reschedule, deprecate legacy
    │   Depends on: Session 2 (tz normalization)
    │
Session 5: Calendar & Integration
    ├── Timezone-aware queries, UI polish, end-to-end integration
    │   Depends on: Sessions 2, 3, 4
    │
Session 6: Migration & Hardening
    ├── Data migration, E2E tests, monitoring
    │   Depends on: Sessions 2-5
```

---

## Session 2: Timezone Foundation

**Mission**: Establish account timezone as the canonical timezone for all scheduling operations.

**Inputs**: Audit findings G1 (timezone broken), G6 (calendar queries naive), G8 (mixed UTC/local)

**Outputs**:
- Shared `tz` utility module in dashboard (`$lib/utils/timezone.ts`)
- Backend timezone validation and normalization
- ScheduleConfig timezone propagated to all scheduling surfaces

**Files to create/modify**:
- `dashboard/src/lib/utils/timezone.ts` — new: `toAccountTz()`, `fromAccountTz()`, `formatInAccountTz()`, `buildScheduledFor()`
- `dashboard/src/lib/stores/calendar.ts` — modify: `formatDateISO()` and `getDateRange()` to use account tz
- `dashboard/src/lib/utils/composeHandlers.ts` — modify: `buildComposeRequest()` to use account tz
- `dashboard/src/lib/components/drafts/DraftScheduleSection.svelte` — modify: date/time construction
- `crates/tuitbot-server/src/routes/content/calendar.rs` — modify: accept timezone param in calendar query
- `crates/tuitbot-server/src/routes/content/compose.rs` — modify: validate and normalize `scheduled_for`

**Tests to add**:
- Unit tests for `timezone.ts` conversion functions
- Backend tests for `scheduled_for` validation with various timezone inputs
- Calendar query tests with timezone-aware boundaries

**Key decisions**:
- D1: Account timezone as canonical (from charter)
- Use `Intl.DateTimeFormat` for frontend timezone conversions (no external dependency)
- Server normalizes all incoming `scheduled_for` to UTC for storage, annotates with original timezone
- Calendar queries accept optional `timezone` param; server computes boundaries

---

## Session 3: Unified Scheduling Component

**Mission**: Replace `TimePicker.svelte` and `DraftScheduleSection.svelte` with a single `SchedulePicker.svelte` component.

**Inputs**: Audit finding G5 (two incompatible UIs), competitive research (Typefully's slot-based picker)

**Outputs**:
- New `SchedulePicker.svelte` component
- All scheduling surfaces use the shared component
- Consistent UX across home composer, modal composer, and Draft Studio

**Files to create/modify**:
- `dashboard/src/lib/components/SchedulePicker.svelte` — new: unified date+time+slot picker
- `dashboard/src/lib/components/TimePicker.svelte` — delete (replaced)
- `dashboard/src/lib/components/drafts/DraftScheduleSection.svelte` — delete (replaced)
- `dashboard/src/lib/components/composer/InspectorContent.svelte` — modify: use SchedulePicker
- `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte` — modify: use SchedulePicker

**Tests to add**:
- Component tests for SchedulePicker (date selection, time selection, slot selection, timezone display)

**Key decisions**:
- D4: Shared scheduling component (from charter)
- Component accepts `timezone` prop (from ScheduleConfig) and displays it
- Preferred time slots shown as quick-select buttons
- Date picker for future dates, custom time input for exact times
- Emits a timezone-aware ISO string (constructed via `timezone.ts`)

---

## Session 4: API Unification

**Mission**: Add explicit action field to compose API, implement atomic reschedule, clean up dual paths.

**Inputs**: Audit findings G2 (implicit intent), G3 (non-atomic reschedule), G4 (dual API paths), G7 (legacy endpoints)

**Outputs**:
- `ComposeRequest` includes `action: "schedule" | "publish" | "save_draft"` field
- `PATCH /api/drafts/:id/reschedule` endpoint (atomic)
- Legacy endpoints marked deprecated with warnings
- Frontend updated to use explicit action field

**Files to create/modify**:
- `crates/tuitbot-server/src/routes/content/compose.rs` — modify: add `action` field to `ComposeRequest`, refactor `persist_content()` to use it, remove silent fallback
- `crates/tuitbot-server/src/routes/content/draft_studio.rs` — modify: add `reschedule_studio_draft()` handler
- `crates/tuitbot-core/src/storage/scheduled_content/mod.rs` — modify: add `reschedule_draft_for()` function
- `dashboard/src/lib/utils/composeHandlers.ts` — modify: add `action` field to request
- `dashboard/src/lib/stores/draftStudio.svelte.ts` — modify: `rescheduleDraft()` to use single API call
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte` — modify: wire `onpublishnow` correctly
- `dashboard/src/lib/api/index.ts` — modify: add reschedule API method

**Tests to add**:
- Backend tests for `action` field handling (all combinations)
- Backend test for atomic reschedule
- Backend test that silent fallback no longer occurs
- Frontend integration test for publish vs schedule button behavior

**Key decisions**:
- D2: Separate publish intent from schedule intent (from charter)
- D3: Atomic reschedule endpoint (from charter)
- Legacy endpoints (`/api/content/tweets`, `/api/content/threads`) remain functional but log deprecation warnings
- `action` field is required; requests without it return 400

---

## Session 5: Calendar & Integration

**Mission**: Fix calendar timezone queries, polish scheduling UX, integrate all pieces end-to-end.

**Inputs**: All previous sessions, audit finding G6 (calendar queries)

**Outputs**:
- Calendar queries use account timezone for boundary computation
- Calendar view shows timezone indicator
- End-to-end scheduling flow works correctly across all surfaces
- Content tags visible on calendar items

**Files to create/modify**:
- `dashboard/src/lib/stores/calendar.ts` — modify: timezone-aware queries, pass tz to API
- `dashboard/src/routes/(app)/content/+page.svelte` — modify: show timezone indicator, tag dots
- `crates/tuitbot-server/src/routes/content/calendar.rs` — modify: accept and use timezone param
- Integration testing across compose -> calendar -> Draft Studio flows

**Tests to add**:
- Calendar boundary computation tests across timezone offsets
- End-to-end test: schedule from composer, verify on calendar, reschedule from Draft Studio
- Test that timezone indicator matches ScheduleConfig

**Key decisions**:
- D5: Timezone-aware date ranges (from charter)
- Calendar API backward compatible: `timezone` param is optional, defaults to UTC
- Calendar popover compose is deferred to a future enhancement

---

## Session 6: Migration & Hardening

**Mission**: Migrate existing data, add monitoring, final hardening.

**Inputs**: All previous sessions complete

**Outputs**:
- Existing `scheduled_for` values migrated with timezone annotation
- Monitoring for scheduling accuracy (scheduled_for vs actual post time)
- Posting engine updated to handle timezone-aware scheduled_for
- Documentation updated

**Files to create/modify**:
- `crates/tuitbot-core/src/storage/migrations/` — new: add `timezone` column to `scheduled_content`
- `crates/tuitbot-core/src/storage/scheduled_content/mod.rs` — modify: `get_due_items_for()` to use timezone-aware comparison
- `crates/tuitbot-core/src/automation/` — modify: posting engine timezone handling
- Comprehensive E2E test suite for scheduling

**Tests to add**:
- Migration test: verify existing data preserved
- Posting engine test: due items computed correctly across timezones
- Regression test: all audit gaps (G1-G9) verified fixed

**Key decisions**:
- Migration adds `timezone` column (nullable, default NULL = legacy)
- Backfill script infers timezone from account's ScheduleConfig
- Legacy items without timezone treated as UTC for backward compatibility
- Posting engine prefers `timezone`-annotated values, falls back to raw comparison

---

## Migration Strategy

### Phase 1: Additive (Sessions 2-4)
- New `timezone.ts` utility used by new code
- Backend accepts timezone-annotated `scheduled_for` values alongside bare strings
- No existing behavior changes; old code paths still work

### Phase 2: Switchover (Session 5)
- All frontend surfaces use timezone-aware construction
- Calendar queries include timezone parameter
- Old bare-string construction paths removed from frontend

### Phase 3: Data Migration (Session 6)
- Add `timezone` column to `scheduled_content` table
- Backfill existing rows with account timezone
- Posting engine uses timezone-aware comparison
- Monitor for one release cycle before removing legacy support

### Rollback Plan

Each session's changes are independently revertible:
- Session 2: `timezone.ts` is additive; removing it reverts to old behavior
- Session 3: Old components can be restored; new component is a drop-in replacement
- Session 4: `action` field can be made optional with fallback to old inference logic
- Session 5: Calendar timezone param is optional; removing it reverts to bare strings
- Session 6: `timezone` column is nullable; NULL means legacy behavior continues
