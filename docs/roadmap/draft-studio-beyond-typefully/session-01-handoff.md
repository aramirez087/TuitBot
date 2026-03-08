# Session 01 Handoff

## What Changed

Four planning documents created under `docs/roadmap/draft-studio-beyond-typefully/`:

| Document | Purpose |
|----------|---------|
| `charter.md` | Product charter: goals, non-goals, locked defaults, success metrics, relationship to existing composer redesign |
| `ux-blueprint.md` | UX specification: 12 screen states, keyboard nav spec, mobile behavior, Typefully differentiators, component inventory |
| `technical-architecture.md` | Data model (4 new tables, 3 additive columns), API surface (14 new endpoints), frontend architecture (store + 6 new components), phased session plan (02-11), migration safety analysis |
| `session-01-handoff.md` | This file |

No code changes. No migrations. No frontend modifications.

## Key Decisions Made

1. **Extend `scheduled_content`, don't create parallel table.** The existing status lifecycle (`draft -> scheduled -> posted -> cancelled`) already covers the draft workflow. Adding `title`, `notes`, `archived_at` columns keeps everything in one place.

2. **Archive replaces hard delete.** Current "delete draft" sets `status = 'cancelled'`. Draft Studio introduces `archived_at` for soft delete with restore capability. Existing `cancelled` status is preserved for scheduled items that are cancelled.

3. **Server autosave with 1.5s debounce.** Client sends `PATCH /api/drafts/:id` with content on a 1.5-second debounce timer. localStorage fallback (500ms debounce) acts as crash recovery only. Conflict detection via `updated_at` comparison.

4. **Revision snapshots on meaningful events, not every keystroke.** Storing a revision on every autosave would create excessive data. Instead, revisions are created on: AI rewrite (before replacement), schedule, unschedule, and manual "save revision" action.

5. **Home composer redirects to Draft Studio.** When `homeSurface === 'composer'`, the `/` route redirects to `/drafts`. This eliminates the parallel composer state. Analytics mode is unaffected.

6. **Calendar compose creates a draft and navigates.** Instead of opening a ComposeModal with ephemeral state, the calendar's "Compose" action creates a server draft (with optional prefilled schedule time) and navigates to `/drafts?id={newId}`.

7. **Deprecate old draft endpoints, don't remove.** `/api/content/drafts` paths are kept as aliases during transition. New code uses `/api/drafts`. Old paths are removed in a later cleanup session.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 02 | Schema and domain | Migration file, updated Rust structs, storage functions for archive/restore/duplicate/revision/activity/tags |
| 03 | API and sync | 14 new endpoints, autosave PATCH with conflict detection, updated API client |
| 04 | Shell and selection | `DraftStudioShell`, `DraftRail`, `draftStudio` store, URL-based selection |
| 05 | Composer binding | `ComposeWorkspace` draftId prop, server hydration, debounced PATCH, sync badge |
| 06 | Keyboard and bulk | Rail keyboard nav, focus management, command palette extensions, bulk archive |
| 07 | Metadata and filters | Title editing, tags UI, search, content type filter, sort options |
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration, scheduled tab |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile layout |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs update |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Schema migration breaks content poster | Medium | High | All new columns nullable with defaults. Session 02 must test all existing storage functions against migrated schema before merging. |
| `ComposeWorkspace` refactor (893 lines) is larger than one session | High | Medium | Session 05 is dedicated to this. Use prop-based hydration (`draftId` prop) rather than store injection to minimize changes. If needed, spill to Session 06. |
| Autosave race conditions during draft switching | Medium | Medium | Server-side `updated_at` comparison on PATCH. Client flushes pending save before switching. 409 Conflict response triggers re-fetch. |
| Multi-tab editing of the same draft | Low | Medium | `updated_at` conflict detection covers this. The "conflict" sync badge state alerts the user. No auto-merge — user chooses which version to keep. |
| Draft list performance with 500+ items | Low | Low | Server-side pagination (`?limit=50&offset=0`). Rail virtualizes long lists with intersection observer or virtual scroll. |
| Existing composer redesign charter overlap | Low | Low | Draft Studio subsumes the redesign. The charter documents the relationship explicitly. Visual polish goals apply within Draft Studio. |

## Exact Inputs for Session 02

1. Read `docs/roadmap/draft-studio-beyond-typefully/charter.md` and `technical-architecture.md` for schema and storage function specs.
2. Read `crates/tuitbot-core/src/storage/scheduled_content.rs` (638 lines) — all existing queries must continue to work.
3. Read `crates/tuitbot-core/migrations/` — 21 existing migration files. New migration is `20260308000022_draft_studio_schema.sql`.
4. Read `crates/tuitbot-core/src/storage/mod.rs` to understand module structure for new storage submodules.
5. Create the migration file with: 3 ALTER TABLE statements on `scheduled_content`, 4 CREATE TABLE statements, 2 CREATE INDEX statements.
6. Update `ScheduledContent` struct with `title: Option<String>`, `notes: Option<String>`, `archived_at: Option<String>`.
7. Update `list_drafts_for` to filter `AND archived_at IS NULL`.
8. Add storage functions: `archive_draft_for`, `restore_draft_for`, `duplicate_draft_for`, `update_draft_meta_for`, `insert_revision_for`, `list_revisions_for`, `insert_activity_for`, `list_activity_for`, `create_tag_for`, `list_tags_for`, `assign_tag_for`, `unassign_tag_for`.
9. Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings && RUSTFLAGS="-D warnings" cargo test --workspace`.
