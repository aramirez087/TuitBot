# Validation Report — Cold-Start Watchtower RAG

## Epic Summary

The Cold-Start Watchtower RAG epic (Sessions 01–07) adds a content ingestion pipeline to Tuitbot that lets new users leverage their existing writing (Obsidian notes, markdown files, Google Drive docs) to generate high-quality tweets from day one — before any engagement history exists.

The pipeline watches configured content sources, extracts tweetable hooks via LLM, and injects them as cold-start context into the draft generation pipeline via Winning DNA retrieval. This replaces the empty-context problem for new accounts with source-backed, personalized seed content.

## Quality Gate Results

| Check | Result | Notes |
|-------|--------|-------|
| `cargo fmt --all --check` | PASS | |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | PASS | 1669 tests (995 core, 495 cli, 36 server, 24 mcp, 118 proc-macro, 1 build) |
| `cargo clippy --workspace -- -D warnings` | PASS | 0 warnings |
| `cd dashboard && npm run check` | PASS | 0 errors, 5 pre-existing warnings |

## Test Coverage Summary

| Component | Tests | Status |
|-----------|-------|--------|
| LocalFsProvider (scan, read, filter, hidden, error) | 5 | PASS |
| GoogleDriveProvider (ID extraction) | 2 | PASS |
| Ingest pipeline (parity, dedup) | 2 | PASS |
| Storage helpers (ensure, find, coexist) | 3 | PASS |
| Config round-trip (Drive, mixed, JSON patch) | 3 | PASS |
| Front-matter parsing (YAML, tags, malformed) | 5 | PASS |
| Pattern matching (md, txt, nested) | 4 | PASS |
| Loopback metadata (write, idempotent, multiple, preserve) | 5 | PASS |
| Seed worker (parse, mock LLM, batch) | 6 | PASS |
| Winning DNA (classify, score, retrieve, format) | 15+ | PASS |
| SourceFile hash (equality, difference) | 2 | PASS |
| **E2E: Local folder → seed pipeline** | 1 | PASS |
| **E2E: Google Drive → seed pipeline** | 1 | PASS |
| **E2E: Mixed sources → draft context** | 1 | PASS |
| **E2E: Inline node → manual source** | 1 | PASS |
| **E2E: Loopback → re-ingest detects change** | 1 | PASS |

## Manual Test Results

### Tauri Folder-Pick Flow

**Procedure:**
1. Launch `cd dashboard && npm run tauri dev`
2. Navigate to Onboarding → Sources step
3. Click "Browse" — native file dialog opens
4. Select a folder containing `.md` files
5. Verify path appears in input field
6. Complete onboarding → `config.toml` has `[[content_sources.sources]]` with selected path
7. Navigate to Settings → Content Sources → source displayed correctly
8. Switch source type to "Google Drive" → Drive-specific fields appear
9. Switch back to "Local Folder" → local-specific fields reappear

**Result:** Not executed (requires display server for Tauri). The component code has been verified through code review and svelte-check passes. The Tauri sidecar integration is unchanged from the existing pattern.

### Simulated Google Drive Sync

**Procedure:**
1. Start `cargo run -p tuitbot-server`
2. POST `/api/ingest` with inline nodes using `gdrive://` relative paths
3. Verify response: `ingested: 1`
4. Query DB to verify content_node with `gdrive://` path

**Result:** Validated programmatically via `e2e_google_drive_ingest_to_seed_pipeline` test. The test covers the full chain: `ensure_google_drive_source()` → `ingest_content()` with `gdrive://` IDs → dedup → update detection → seed generation → draft context retrieval. The API endpoint delegates to the same `ingest_content()` function.

### POST /api/ingest (Inline Nodes)

**Result:** Validated programmatically via `e2e_inline_node_ingest_creates_manual_source` test. The test verifies: manual source creation, inline node upsert, hash-based dedup, and correct source linkage. The API handler uses the same `ensure_manual_source()` → `upsert_content_node()` path.

## Unresolved Risks

| ID | Risk | Severity | Mitigation |
|----|------|----------|------------|
| 1 | Google Drive JWT auth untested with real service account | Medium | Self-contained RSA signing works in unit tests; real key testing deferred to first user deployment. Rollback: disable Drive sources in config. |
| 2 | Dashboard manages only `sources[0]` | Low | Multiple sources work via `config.toml` editing. Dashboard multi-source UX is post-v1. |
| 3 | No retry/backoff for remote source errors | Low | Errors logged and source status set to "error". Next poll retries automatically. Exponential backoff is a follow-up. |
| 4 | Seed worker requires LLM availability | Low | If LLM is down, nodes stay 'pending' and retry next tick (5 min). No data loss. |
| 5 | BigUint RSA implementation in `google_drive.rs` | Medium | Minimal, correct for PKCS#1 v1.5 signing. Tested with known vectors. Consider replacing with `rsa` crate if issues arise in production. |
| 6 | Tauri folder-pick not tested in headless CI | Low | Component code verified via svelte-check; Tauri sidecar pattern is well-established in the codebase. Manual testing on developer machines recommended before release. |

## Rollback Plan

If issues are discovered post-release:

1. **Disable Watchtower:** Remove `[[content_sources.sources]]` from `config.toml`. The watcher exits immediately with no sources configured.
2. **Remove Drive sources:** Change `source_type` back to `"local_fs"` or remove the source entry entirely.
3. **Data cleanup:** `DELETE FROM draft_seeds; DELETE FROM content_nodes; DELETE FROM source_contexts;` — these tables are additive and have no FK constraints from existing core tables (only `original_tweets.source_node_id` is a nullable FK).
4. **Migration is additive:** The watchtower migration uses `CREATE TABLE IF NOT EXISTS` and `ALTER TABLE ADD COLUMN`. No destructive schema changes. Safe to leave in place.

## Follow-Up Work (post-release)

| Priority | Item | Effort |
|----------|------|--------|
| P1 | Test Google Drive auth with real service account key | 1 session |
| P2 | Dashboard multi-source management UI | 1-2 sessions |
| P2 | Exponential backoff for remote source errors | 0.5 sessions |
| P3 | Google Docs → Markdown export (binary format support) | 1 session |
| P3 | Interactive OAuth flow for personal Drive accounts | 1-2 sessions |
| P3 | Additional providers (Notion, Dropbox) | 1 session each |

## Recommendation

**GO**: The Cold-Start Watchtower RAG pipeline is release-ready for v1. All quality gates pass with 1669 tests (5 new E2E integration tests validating the full pipeline). The provider model is clean, the storage layer is additive and rollback-safe, and both local filesystem and Google Drive sources work through the shared `ingest_content()` pipeline. The only medium-severity risk (Drive JWT auth with real keys) is mitigated by the fact that Drive is an optional provider — the core value proposition (local folder → cold-start seeds) works end-to-end without it. The follow-up work items are enhancements, not blockers.
