# Session 02 Handoff — Vault Domain Foundation

## What Changed

### Schema
- **New table:** `content_chunks` — heading-delimited fragments of content nodes,
  with `retrieval_boost` for loop-back learning and `status` for stale tracking.
- **New column:** `draft_seeds.chunk_id` (nullable FK) — links seeds to their
  source fragment.
- **New columns on `approval_queue`:** `source_node_id`, `source_seed_id`,
  `source_chunks_json` — provenance links from drafts to source material.
- Migration: `20260308010000_vault_domain_foundation.sql` mirrored in both dirs.

### Storage Module Restructure
- Split `watchtower/mod.rs` (702 lines) into a module directory:
  `mod.rs`, `sources.rs`, `nodes.rs`, `chunks.rs`, `seeds.rs`.
- All public types and functions re-exported — zero impact on callers.

### Account-Scoped APIs
- Added `_for` variants for all source, node, and seed functions.
- All chunk functions are account-scoped from the start (no legacy wrappers).
- Legacy functions preserved as thin wrappers passing `DEFAULT_ACCOUNT_ID`.

### New Chunk CRUD
- `insert_chunk`, `upsert_chunks_for_node` (hash-based dedup)
- `get_chunks_for_node`, `get_chunk_by_id`, `get_chunks_by_ids`
- `mark_chunks_stale`, `update_chunk_retrieval_boost` (clamped [0.1, 5.0])
- `search_chunks_by_keywords` — LIKE-based keyword search, ordered by boost
- `mark_node_chunked` — sets content node status to `chunked`

### Approval Queue Provenance
- `ApprovalRow` and `ApprovalItem` now include `source_node_id`,
  `source_seed_id`, and `source_chunks_json`.
- `SELECT_COLS` updated to include provenance columns.

### Supporting Changes
- `storage/reset.rs`: Added `content_chunks` to `TABLES_TO_CLEAR` (before
  `content_nodes` for FK ordering). Updated table count assertions (31 → 32).
- `storage/mod.rs`: Test now asserts `content_chunks` table exists.

## Files Created
- `migrations/20260308010000_vault_domain_foundation.sql`
- `crates/tuitbot-core/migrations/20260308010000_vault_domain_foundation.sql`
- `crates/tuitbot-core/src/storage/watchtower/sources.rs`
- `crates/tuitbot-core/src/storage/watchtower/nodes.rs`
- `crates/tuitbot-core/src/storage/watchtower/seeds.rs`
- `crates/tuitbot-core/src/storage/watchtower/chunks.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/data-model.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-02-handoff.md`

## Files Modified
- `crates/tuitbot-core/src/storage/watchtower/mod.rs` — rewritten as re-export hub
- `crates/tuitbot-core/src/storage/watchtower/tests.rs` — extended with chunk/account tests
- `crates/tuitbot-core/src/storage/approval_queue/mod.rs` — provenance fields
- `crates/tuitbot-core/src/storage/approval_queue/queries.rs` — SELECT_COLS update
- `crates/tuitbot-core/src/storage/mod.rs` — content_chunks table assertion
- `crates/tuitbot-core/src/storage/reset.rs` — content_chunks in TABLES_TO_CLEAR
- `crates/tuitbot-server/tests/factory_reset.rs` — table count assertion update

## Test Results
- **1967 tests passed**, 0 failed, 1 ignored (network-dependent)
- **cargo fmt** — clean
- **cargo clippy** — zero warnings

## What Remains

| Session | Scope | Status |
|---------|-------|--------|
| Chunker | Fragment chunking pipeline (`automation/watchtower/chunker.rs`) | Next |
| Seed Worker | Update seed worker to query `chunked` nodes, pass `chunk_id` | Future |
| Chunk RAG | `search_chunks_by_keywords` integration into assist context | Future |
| Provenance Wiring | Populate `source_node_id`, `source_seed_id`, `source_chunks_json` | Future |
| Loop-Back | `update_chunk_retrieval_boost` from analytics feedback | Future |
| Account Isolation Audit | Ensure all callers use `_for` variants | Future |
| Vault Health API | Stats endpoint for vault status | Future |
| Dashboard UI | Vault health + From Vault composer | Future |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Module split breaks downstream in a branch we haven't tested | Low | Medium | All `pub use` re-exports verified; grep confirmed all callers use module path |
| `ON DELETE CASCADE` on chunks causes unexpected data loss | Low | Medium | Only triggers on content_node row deletion (no app path does this) |
| LIKE-based keyword search too slow on large chunk sets | Medium | Low | Bounded by `LIMIT`; can add FTS5 later without schema changes |
| `retrieval_boost` clamp in app code not enforced in DB | Low | Low | Intentional — avoids SQLite CHECK constraint portability issues |

## Decisions Made

1. **Boost clamping in app code, not schema** — `update_chunk_retrieval_boost`
   clamps to [0.1, 5.0]. CHECK constraints in SQLite vary across versions.
2. **No backfill of chunk_id on existing seeds** — existing seeds have
   `chunk_id = NULL`, which correctly represents "pre-chunking era" data.
3. **`get_source_contexts` (legacy) queries all active sources regardless of
   account** — matches the old behavior exactly. The `_for` variant filters
   by account.
4. **Module split keeps `chunks` as `pub mod`** — other submodules are private
   with `pub use` re-exports. Chunks is public because `NewChunk` is a struct
   callers need to construct directly.

## Inputs for Next Session

- `data-model.md` — full schema reference
- `product-model.md` — chunking algorithm specification (heading-based split)
- Key files to create/modify:
  - `crates/tuitbot-core/src/automation/watchtower/chunker.rs` (new)
  - `crates/tuitbot-core/src/automation/watchtower/mod.rs` (integrate chunker)
- Storage functions ready for use:
  - `upsert_chunks_for_node`, `mark_chunks_stale`, `mark_node_chunked`
  - `NewChunk` struct for batch insert input
