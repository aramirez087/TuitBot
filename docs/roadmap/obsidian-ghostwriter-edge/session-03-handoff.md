# Session 03 Handoff: Backend Block Contracts

## What Changed

Server-side contracts for receiving Ghostwriter selections from the Obsidian plugin, resolving them against indexed vault data, and preserving provenance through compose and draft workflows.

### Files Created

| File | Purpose |
|---|---|
| `crates/tuitbot-core/migrations/20260321000100_vault_selections.sql` | Migration for transient `vault_selections` table with session_id unique index |
| `crates/tuitbot-core/src/storage/vault_selections.rs` | CRUD module: insert, get-by-session, cleanup-expired, count-recent-for |
| `crates/tuitbot-server/src/routes/vault/selections.rs` | POST send-selection and GET selection/{session_id} handlers with DTOs |
| `docs/roadmap/obsidian-ghostwriter-edge/block-contracts.md` | Contract documentation: ingress, resolution algorithm, provenance flow |
| `docs/roadmap/obsidian-ghostwriter-edge/session-03-handoff.md` | This file |

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-core/src/storage/mod.rs` | Added `pub mod vault_selections` and table assertion in test |
| `crates/tuitbot-core/src/storage/watchtower/nodes.rs` | Added `find_node_by_path_for()` for path-based node lookup |
| `crates/tuitbot-core/src/storage/watchtower/chunks.rs` | Added `find_best_chunk_by_heading_for()` for heading prefix matching |
| `crates/tuitbot-core/src/context/retrieval.rs` | Added `resolve_selection_identity()` orchestrator |
| `crates/tuitbot-core/src/storage/reset.rs` | Added `vault_selections` to TABLES_TO_CLEAR, updated table counts |
| `crates/tuitbot-server/src/ws.rs` | Added `SelectionReceived { session_id }` WsEvent variant |
| `crates/tuitbot-server/src/routes/vault.rs` → `vault/mod.rs` | Converted to module directory, added `pub mod selections` |
| `crates/tuitbot-server/src/lib.rs` | Registered `/vault/send-selection` and `/vault/selection/{session_id}` routes |
| `crates/tuitbot-server/tests/assist_rag_tests.rs` | Added 10 integration tests for selection endpoints |
| `crates/tuitbot-server/tests/compose_contract_tests.rs` | Added 3 integration tests for provenance flow |
| `crates/tuitbot-server/tests/factory_reset.rs` | Updated table count assertion (37 → 38) |

## Decisions Made

See `block-contracts.md` for full decision log (6 decisions).

Key decisions:
1. **Transient storage with 30-minute TTL** — selections are ephemeral, consumed within minutes
2. **Best-effort block identity resolution** — `selected_text` is always authoritative; resolution enriches provenance when available
3. **Cloud mode privacy gate** — `selected_text` omitted from GET response in Cloud deployment mode
4. **Module directory split** — `vault.rs` → `vault/mod.rs` + `vault/selections.rs` to stay within 500-line limit

## Exit Criteria Met

- [x] `POST /api/vault/send-selection` receives selection, validates, rate-limits, persists to `vault_selections`
- [x] `GET /api/vault/selection/{session_id}` retrieves selection with Cloud privacy gate
- [x] Block identity resolution: file_path → content_node → best-matching content_chunk
- [x] `SelectionReceived` WebSocket event emitted on selection receipt
- [x] ProvenanceRef pipeline unchanged — compose/draft can construct from resolved IDs
- [x] All tests pass: `cargo fmt`, `cargo clippy`, `cargo test` (3325 tests, 0 failures)

## What Session 4 Needs

1. **Dashboard composer integration**: Retrieve selection via GET endpoint when `?selection=<session_id>` query param is present, populate composer with selection context
2. **ProvenanceRef construction**: When user generates content from a selection, build `ProvenanceRef` from `resolved_node_id`/`resolved_chunk_id` and attach to compose/draft submission
3. **Cleanup task registration**: Wire `vault_selections::cleanup_expired()` into the existing hourly cleanup loop
4. **Citation rendering**: Display provenance links with heading-anchor deep-links in the dashboard
