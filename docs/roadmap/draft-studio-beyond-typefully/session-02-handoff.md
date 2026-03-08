# Session 02 Handoff

## What Changed

### Migration

- `migrations/20260308000100_draft_studio_foundation.sql` — additive schema changes
- `crates/tuitbot-core/migrations/20260308000100_draft_studio_foundation.sql` — identical copy

Changes:
- 3 `ALTER TABLE` on `scheduled_content`: added `title`, `notes`, `archived_at` (all nullable TEXT)
- 4 `CREATE TABLE`: `content_revisions`, `content_tags`, `content_tag_assignments`, `content_activity`
- 2 `CREATE INDEX`: on `content_revisions(content_id)` and `content_activity(content_id)`

### Storage module refactor

Converted `scheduled_content.rs` (637 lines) into a module directory to stay within the 500-line limit:

```
crates/tuitbot-core/src/storage/scheduled_content/
├── mod.rs          — structs + existing CRUD functions (364 lines)
├── drafts.rs       — archive, restore, duplicate, meta update, list archived (106 lines)
├── revisions.rs    — insert/list revision snapshots (47 lines)
├── activity.rs     — insert/list activity log (46 lines)
├── tags.rs         — create, list, assign, unassign tags (67 lines)
└── tests.rs        — all tests: 11 existing + 11 new (370 lines)
```

All public items re-exported from `mod.rs` — no breaking changes to consumers.

### Struct changes

`ScheduledContent` gained 3 fields:
- `title: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`
- `notes: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`
- `archived_at: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`

New structs: `ContentRevision`, `ContentTag`, `ContentActivity`.

### Query changes

- `list_drafts_for` now filters `AND archived_at IS NULL`

### Supporting changes

- `crates/tuitbot-core/src/storage/reset.rs`: added 4 new tables to `TABLES_TO_CLEAR`, updated table count from 31 to 35
- `crates/tuitbot-core/src/storage/mod.rs`: added 4 new table assertions to `init_test_db_creates_all_tables`
- `crates/tuitbot-server/tests/factory_reset.rs`: updated `tables_cleared` assertion from 31 to 35

### Documentation

- `docs/roadmap/draft-studio-beyond-typefully/data-model.md` — full schema reference, lifecycle states, compatibility rules, storage function inventory

## Key Decisions

1. **`trigger_kind` instead of `trigger`**: SQLite reserves `TRIGGER` as a keyword. Used `trigger_kind` for the column name in `content_revisions`. Documented in data-model.md.

2. **`id DESC` ordering for revisions and activity**: `datetime('now')` has second-level granularity, so rows inserted in the same second get identical `created_at` values. Using `ORDER BY id DESC` ensures deterministic newest-first ordering in tests and production.

3. **`duplicate_draft_for` returns `Option<i64>`**: Returns `None` when the source draft doesn't exist instead of erroring. This simplifies API-layer handling in Session 03.

4. **`list_archived_drafts_for` added beyond plan**: The plan didn't explicitly call for this, but the archive tab in the UI (Session 04+) will need it. Added proactively — 8 lines of SQL.

5. **Archive is orthogonal to status**: `archive_draft_for` only sets `archived_at`, not `status`. A scheduled item can be archived (hidden but still scheduled). This matches the Session 01 decision.

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
| 03 | API and sync | 14 new endpoints, autosave PATCH with conflict detection, updated API client |
| 04 | Shell and selection | `DraftStudioShell`, `DraftRail`, `draftStudio` store, URL-based selection |
| 05 | Composer binding | `ComposeWorkspace` draftId prop, server hydration, debounced PATCH |
| 06 | Keyboard and bulk | Rail keyboard nav, focus management, command palette, bulk archive |
| 07 | Metadata and filters | Title editing, tags UI, search, content type filter, sort options |
| 08 | Schedule and calendar | Schedule/unschedule from studio, calendar integration |
| 09 | Revisions and activity | Revision timeline, preview/restore, activity log |
| 10 | Entrypoints and rollout | Home redirect, calendar redirect, feature flag, mobile |
| 11 | Validation and launch | QA, accessibility, mobile, performance, docs |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| API layer needs additional queries not yet written | Medium | Low | Session 03 can add queries as needed; the module structure makes this easy |
| `skip_serializing_if` on new fields may confuse frontend code that expects them | Low | Low | Frontend doesn't use these fields yet; Session 03+ will handle |
| Root `migrations/` is still missing migration 21 (`account_profile_fields`) | Pre-existing | Low | Not related to Draft Studio; out of scope |

## Exact Inputs for Session 03

1. Read `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md` for the API surface spec (14 new endpoints)
2. Read `docs/roadmap/draft-studio-beyond-typefully/data-model.md` for the storage function inventory
3. Read `crates/tuitbot-server/src/routes/content/drafts.rs` for existing draft API routes
4. Read `crates/tuitbot-server/src/routes/content/mod.rs` for router setup
5. Read `dashboard/src/lib/api/client.ts` for existing API client patterns
6. Create `crates/tuitbot-server/src/routes/drafts.rs` (or extend existing) with 14 new endpoints
7. Implement autosave PATCH with `updated_at` conflict detection (409 response)
8. Add `draftStudio` API namespace to `dashboard/src/lib/api/client.ts`
9. Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings && RUSTFLAGS="-D warnings" cargo test --workspace`
