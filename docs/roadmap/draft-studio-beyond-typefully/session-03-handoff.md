# Session 03 Handoff

## What Changed

### New handler module

`crates/tuitbot-server/src/routes/content/draft_studio.rs` — 13 handlers implementing the Draft Studio API under `/api/drafts`:

| Endpoint | Handler |
|----------|---------|
| `GET /api/drafts` | `list_studio_drafts` — collection query with status/search/archived filters |
| `GET /api/drafts/:id` | `get_studio_draft` — full content fetch |
| `POST /api/drafts` | `create_studio_draft` — blank or seeded draft creation |
| `PATCH /api/drafts/:id` | `autosave_draft` — content update with `updated_at` conflict detection |
| `PATCH /api/drafts/:id/meta` | `patch_draft_meta` — title and notes update |
| `POST /api/drafts/:id/schedule` | `schedule_studio_draft` — draft → scheduled with revision + activity |
| `POST /api/drafts/:id/unschedule` | `unschedule_studio_draft` — scheduled → draft with revision + activity |
| `POST /api/drafts/:id/archive` | `archive_studio_draft` — soft-delete |
| `POST /api/drafts/:id/restore` | `restore_studio_draft` — restore from archive |
| `POST /api/drafts/:id/duplicate` | `duplicate_studio_draft` — clone draft |
| `GET /api/drafts/:id/revisions` | `list_draft_revisions` — revision history |
| `POST /api/drafts/:id/revisions` | `create_draft_revision` — manual revision snapshot |
| `GET /api/drafts/:id/activity` | `list_draft_activity` — activity log |

### Storage layer additions

`crates/tuitbot-core/src/storage/scheduled_content/mod.rs`:
- `unschedule_draft_for(pool, account_id, id)` — sets status back to `draft`, clears `scheduled_for`
- `autosave_draft_for(pool, account_id, id, content, content_type, expected_updated_at)` — atomic update with optimistic concurrency via `updated_at` comparison

### Route registration

`crates/tuitbot-server/src/lib.rs` — 12 new route entries under `/api/drafts`. Legacy `/api/content/drafts` routes remain unchanged.

`crates/tuitbot-server/src/routes/content/mod.rs` — added `mod draft_studio` and re-exports for all 13 handlers.

### Frontend API client

`dashboard/src/lib/api/client.ts`:
- New `api.draftStudio` namespace with 13 typed methods
- Existing `api.drafts` namespace unchanged

`dashboard/src/lib/api/types.ts`:
- `DraftSummary`, `AutosaveResponse`, `StaleWriteError`, `ContentRevision`, `ContentActivity`
- `ScheduledContentItem` gained optional `title`, `notes`, `archived_at` fields

### Integration tests

`crates/tuitbot-server/tests/draft_studio_api_tests.rs` — 24 tests covering:
- Collection & CRUD (7 tests)
- Autosave with conflict detection (4 tests)
- Metadata patching (1 test)
- Workflow transitions (3 tests)
- Archive/restore (3 tests)
- Duplicate (1 test)
- Revisions & activity (3 tests)
- Backward compatibility (2 tests)

### Documentation

- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md` — full API contract reference

## Key Decisions

1. **Autosave conflict detection via `updated_at` string comparison.** The client sends its last known `updated_at` in the PATCH body. The server checks this against the DB row and returns 409 if mismatched. Simpler than `If-Unmodified-Since` headers and works with SQLite text timestamps.

2. **`autosave_draft_for` as atomic storage function.** Rather than a read-then-write in the handler, the storage function uses `WHERE updated_at = ?` in the UPDATE to make the check-and-write atomic. The handler does a pre-check for better error messages on non-existent drafts.

3. **409 Conflict body is a JSON string inside `error`.** The `ApiError::Conflict` variant serializes as `{ "error": "<message>" }`. For stale writes, the message is a JSON string containing `{ "error": "stale_write", "server_updated_at": "..." }`. The client should parse `body.error` as JSON to extract `server_updated_at`.

4. **Blank draft uses space placeholder.** `POST /api/drafts` with no content inserts `" "` (single space) since the storage layer expects non-empty content. The autosave PATCH will overwrite immediately.

5. **Search/status filtering is in-application.** List query loads all drafts, then filters in Rust. With typical draft counts (<200), this is fast. Can optimize to SQL if needed.

6. **Tag filtering reserved but not wired.** The `tag` query parameter is accepted but has no effect yet (no tag assignment endpoints exposed in this session).

## What Remains

| Session | Focus | Key Deliverables |
|---------|-------|-----------------|
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
| `updated_at` second-level granularity false negatives | Low | Medium | In practice, two autosave writes in the same second from different tabs is unlikely. Could add a `revision_counter` column later. |
| 409 body format requires client JSON double-parse | Low | Low | Document in contract. Session 05 will implement the client-side handling. |
| Tag filtering not yet wired to SQL | Certain | None | No UI consumes it yet. Session 07 will implement tag endpoints and filtering. |

## Exact Inputs for Session 04

1. Read `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md` for the complete API contract
2. Read `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md` for the frontend component plan
3. Read `dashboard/src/lib/api/client.ts` — the `draftStudio` namespace is ready
4. Read `dashboard/src/lib/api/types.ts` — `DraftSummary` and related types are defined
5. Read `dashboard/src/routes/(app)/drafts/+page.svelte` for the current drafts page to replace
6. Create `dashboard/src/lib/stores/draftStudio.ts` — Svelte 5 rune-based store
7. Create `dashboard/src/lib/components/drafts/DraftStudioShell.svelte` — three-zone layout
8. Create `dashboard/src/lib/components/drafts/DraftRail.svelte` — tab bar + draft list
9. Create `dashboard/src/lib/components/drafts/DraftRailEntry.svelte` — single draft item
10. Create `dashboard/src/lib/components/drafts/DraftEmptyState.svelte` — empty states
11. Replace `/drafts/+page.svelte` content with `DraftStudioShell`
12. URL-based selection: `/drafts?id=42`
13. Run: `npm --prefix dashboard run check`
