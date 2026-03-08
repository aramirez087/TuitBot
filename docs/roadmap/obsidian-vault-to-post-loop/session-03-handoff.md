# Session 03 Handoff — Content Source Runtime & Lifecycle

## What Changed

### Config Types (`types.rs`)
- **New field:** `ContentSourceEntry::enabled` (`Option<bool>`, default `None`) —
  explicit enabled/disabled toggle. Falls back to legacy `watch` when `None`.
- **New field:** `ContentSourceEntry::change_detection` (`String`, default `"auto"`) —
  controls how changes are detected: `"auto"`, `"poll"`, or `"none"`.
- **New methods:** `is_enabled()`, `effective_change_detection()`, `is_poll_only()`,
  `is_scan_only()` — semantic accessors for source lifecycle state.
- **Constants:** `CHANGE_DETECTION_AUTO`, `CHANGE_DETECTION_POLL`,
  `CHANGE_DETECTION_NONE`, `MIN_POLL_INTERVAL_SECONDS`.

### Config Validation (`validation.rs`)
- Validates `change_detection` value (must be auto/poll/none).
- Validates `poll_interval_seconds >= 30` when set.
- Validates enabled sources have required fields (path for local_fs,
  folder_id for google_drive).
- Disabled sources skip field completeness checks.

### Watchtower Automation (`automation/watchtower/mod.rs`)
- Replaced `s.watch` filter with `s.is_enabled()` for source selection.
- Sources partitioned by `change_detection` mode:
  - `"auto"` sources register with notify watcher + fallback polling
  - `"poll"` sources skip notify, participate in fallback polling only
  - `"none"` sources do initial scan only, exit the event loop
- Added status transitions: `"syncing"` before scan/poll, `"active"` on
  success, `"error"` on failure — for both local and remote sources.
- Added `reindex_local_source()` public method for one-shot full rescan.

### Storage (`storage/watchtower/sources.rs`)
- Added `get_all_source_contexts()` — returns all sources regardless of
  status (for the status API, unlike `get_source_contexts` which filters
  to `status = 'active'`).

### AppState (`state.rs`)
- `watchtower_cancel` changed from `Option<CancellationToken>` to
  `RwLock<Option<CancellationToken>>`.
- `content_sources` changed from `ContentSourcesConfig` to
  `RwLock<ContentSourcesConfig>`.
- Added `restart_watchtower()` method: cancels current watcher, reloads
  config from disk, spawns new WatchtowerLoop with updated sources.

### Server Startup (`main.rs`)
- Updated source filter to use `is_enabled()` instead of `s.watch`.
- AppState construction uses `RwLock::new()` for watchtower_cancel and
  content_sources.
- Shutdown path reads watchtower_cancel through RwLock.

### Settings Routes (`routes/settings.rs`)
- `PATCH /api/settings`: calls `restart_watchtower()` when patch touches
  `content_sources` or `deployment_mode` (default account only).
- Factory reset: acquires write lock on watchtower_cancel before cancel.

### Ingest Route (`routes/ingest.rs`)
- File hints path reads `content_sources` through RwLock with short-lived
  guard (clones needed data before await points).

### New: Source Status Routes (`routes/sources.rs`)
- `GET /api/sources/status` — returns all source_contexts with runtime status.
- `POST /api/sources/{id}/reindex` — triggers background rescan of a local_fs
  source. Returns immediately with `{ "status": "reindex_started" }`.

## Files Created
- `crates/tuitbot-server/src/routes/sources.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/source-lifecycle.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-03-handoff.md`

## Files Modified
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-core/src/config/tests.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/automation/watchtower/tests.rs`
- `crates/tuitbot-core/src/storage/watchtower/sources.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/main.rs`
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/routes/ingest.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/tests/api_tests.rs`
- `crates/tuitbot-server/tests/factory_reset.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`
- `crates/tuitbot-server/tests/compose_contract_tests.rs`
- `crates/tuitbot-server/tests/fresh_install_auth.rs`

## Test Results
- All tests pass (see CI checklist run)
- New tests added for:
  - `is_enabled()` fallback to `watch`
  - `enabled` overriding `watch`
  - `change_detection` defaults and modes
  - Validation of invalid `change_detection` values
  - Validation of `poll_interval_seconds` minimum
  - Validation of enabled sources without required fields
  - Disabled sources skip field checks
  - Legacy `watch=false` parsing

## What Remains

| Session | Scope | Status |
|---------|-------|--------|
| Chunker | Fragment chunking pipeline | Next |
| Seed Worker | Query chunked nodes, pass chunk_id | Future |
| Chunk RAG | search_chunks_by_keywords integration | Future |
| Provenance Wiring | Populate source_node_id, source_seed_id | Future |
| Loop-Back | update_chunk_retrieval_boost from analytics | Future |
| Account Isolation Audit | Ensure all callers use _for variants | Future |
| Dashboard: Vault Health | Source status UI, sync indicators | Future |
| Dashboard: Source Config | Enable/disable toggle, change_detection picker | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| RwLock on content_sources held across await points | Low | High | All reads use short-lived guards; data cloned before drops |
| Watchtower restart drops in-flight ingests | Low | Medium | CancellationToken is cooperative; current scan completes |
| `enabled: None` + `watch: true` backward compat | Low | Low | `is_enabled()` tested; serde defaults verified |
| Reindex on large vaults blocks source_id updates | Low | Low | Runs in spawned task; returns immediately |

## Decisions Made

1. **Cancel-and-respawn over internal reload** — Simpler than adding config
   channel inside WatchtowerLoop. The cooperative cancellation ensures clean
   shutdown of the old loop.
2. **`enabled: Option<bool>` not `bool`** — Allows detecting "user hasn't set
   this yet" vs "user explicitly disabled". When `None`, falls back to `watch`
   for perfect backward compatibility.
3. **`change_detection` as String not enum** — Matches existing TOML/JSON serde
   patterns in the codebase. Validated at config validation time.
4. **Reindex returns immediately** — Long-running rescans should not block the
   HTTP response. Status is observable via `GET /api/sources/status`.
5. **`get_all_source_contexts` includes all statuses** — Unlike the existing
   `get_source_contexts` which filters to `active`, the status API needs to
   show error and syncing sources too.

## Inputs for Next Session

- `source-lifecycle.md` — lifecycle contract reference
- `data-model.md` — schema reference (from session 02)
- Key files to create/modify:
  - `crates/tuitbot-core/src/automation/watchtower/chunker.rs` (new)
  - `crates/tuitbot-core/src/automation/watchtower/mod.rs` (integrate chunker)
- Storage functions ready for use:
  - `upsert_chunks_for_node`, `mark_chunks_stale`, `mark_node_chunked`
  - `NewChunk` struct for batch insert input
- Source lifecycle now provides:
  - Status transitions that downstream UIs can observe
  - Reindex API for manual re-processing after chunker is integrated
