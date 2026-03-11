# Epic Charter: Scheduling System Overhaul

_Created 2026-03-10_

---

## Problem Statement

ReplyGuy users cannot reliably schedule content for future posting. The scheduling system has timezone bugs that cause posts to publish at wrong times, implicit intent handling that confuses "publish now" with "schedule for later," non-atomic operations that silently lose scheduled times, and two incompatible scheduling UIs that behave differently.

**Why scheduling must remain available even when direct publish is possible**: Users who can post directly still need scheduling because (1) they batch-create content during creative sessions and spread it across days, (2) they want to hit optimal engagement windows even when composing outside those windows, and (3) the approval queue workflow requires scheduling as a staging step before posts go live. Scheduling is not a fallback for users without X API credentials — it is a core workflow for all users.

## User Stories

**As a content creator**, I want to schedule a tweet for 2pm EST tomorrow and trust it will post at exactly that time, regardless of what timezone my browser is in.

**As a user composing from Draft Studio**, I want to reschedule a post to a different time without risk of losing my schedule if something goes wrong.

**As a user on the home page**, I want a clear distinction between "Publish this right now" and "Schedule this for later," with no silent fallbacks.

**As a user clicking a calendar time slot**, I want to compose content pre-filled with that exact time, using my account timezone, not my browser's.

**As a user reviewing my calendar**, I want to see all times in my account's configured timezone, with a clear indicator of which timezone is being used.

## Goals

1. **Correct timezone semantics**: All user-facing datetime construction uses the account's configured timezone. The server validates and normalizes incoming timestamps. Stored datetimes include timezone context.

2. **Explicit user intent**: The compose API distinguishes between "publish now," "schedule for later," and "save as draft" via an explicit action field. No silent fallbacks.

3. **Atomic scheduling operations**: Reschedule is a single API call. Schedule/unschedule operations are idempotent and safe.

4. **Unified scheduling UI**: One shared `SchedulePicker` component used across all surfaces — home composer, modal composer, and Draft Studio — with consistent behavior.

5. **Backward-compatible migration**: Existing scheduled content continues to work. New timezone-aware columns are additive. No data loss during rollout.

## Non-Goals

- Multi-platform scheduling (ReplyGuy is X-only)
- Queue-based auto-scheduling with "next free slot" (future enhancement, not this epic)
- Drag-to-reschedule on calendar (future enhancement)
- Bulk scheduling / CSV import
- Team collaboration features
- Changes to the autonomous/discovery content scheduling paths (this epic focuses on manual content)

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Posts scheduled at correct time | Unknown (no monitoring) | 100% within 1 minute of target | Compare `scheduled_for` to `posted_at` in action log |
| Timezone-related support issues | Occasional user reports | Zero | User feedback channels |
| Reschedule data loss incidents | Possible (non-atomic) | Zero | Monitor for drafts losing `scheduled_for` without user action |
| Scheduling surfaces using shared component | 0 (two separate UIs) | All surfaces | Code audit |
| Compose API using explicit action field | 0% | 100% | API usage logs |

## Rollout Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Existing bare datetimes shift when timezone normalization is added | High | Scheduled posts fire at wrong time | Additive migration: new `scheduled_for_tz` column alongside existing, backfill assuming account timezone |
| Two compose paths sharing one table means schema changes affect both | Medium | Regression in Draft Studio or compose flow | Test both paths exhaustively; session 4 unifies them |
| Browser timezone != account timezone for existing users | Medium | Displayed times change after fix | Show "Times shown in [tz]" indicator; one-time migration notification |
| Legacy endpoints called by automation | Low | Automation breaks | Audit all callers before deprecating; keep legacy endpoints working with deprecation warnings |
| Non-atomic reschedule in flight during rollout | Low | In-progress reschedule could lose data | Deploy atomic reschedule endpoint first, update frontend second |

## Dependencies and Constraints

- `ScheduleConfig.timezone` must already be populated for all accounts. If not, the migration must set a default (e.g., UTC or inferred from browser on next login).
- The posting engine that picks up due items must be updated to compare `scheduled_for` in the correct timezone.
- Approval queue flow is out of scope for scheduling changes, but the compose API changes must not break approval mode.

## Scope Boundaries

**In scope**: Manual content scheduling through all UI surfaces, API changes for explicit intent, timezone handling, shared components, data migration.

**Out of scope**: Autonomous loop scheduling, discovery content scheduling, approval queue workflow changes (except ensuring compatibility), calendar visual redesign, mobile-specific optimizations.
