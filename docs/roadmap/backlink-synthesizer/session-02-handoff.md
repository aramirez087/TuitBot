# Session 02 Handoff: Graph Ingestion & Storage

**Date:** 2026-03-21
**Branch:** `epic/backlink-synthesizer`

## What Changed

Implemented the additive storage and ingestion layer that turns vault notes into a resolvable note graph.

### Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `crates/tuitbot-core/migrations/20260321000200_note_graph.sql` | Migration: `note_edges`, `note_tags` tables + `vault_provenance_links` extension | ~35 |
| `migrations/20260321000200_note_graph.sql` | Workspace-root copy of migration | ~35 |
| `crates/tuitbot-core/src/automation/watchtower/link_extractor.rs` | Pure link/tag extraction with regex + 17 unit tests | ~300 |
| `crates/tuitbot-core/src/automation/watchtower/graph_ingest.rs` | Integration: extract → resolve → persist edges/tags (fail-open) | ~200 |
| `crates/tuitbot-core/src/storage/watchtower/edges.rs` | CRUD for `note_edges`: delete, insert, query | ~150 |
| `crates/tuitbot-core/src/storage/watchtower/tags.rs` | CRUD for `note_tags`: delete, insert, query, shared-tag neighbors | ~150 |
| `crates/tuitbot-core/src/storage/watchtower/tests_graph.rs` | 17 integration tests: edges, tags, isolation, idempotency, e2e | ~700 |
| `docs/roadmap/backlink-synthesizer/graph-storage-contract.md` | Storage contract documentation | ~100 |
| `docs/roadmap/backlink-synthesizer/session-02-handoff.md` | This file | ~80 |

### Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Added `pub mod link_extractor;` and `pub mod graph_ingest;` |
| `crates/tuitbot-core/src/automation/watchtower/chunker.rs` | Added `extract_and_persist_graph()` call after chunk upsert |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Added `pub mod edges;`, `pub mod tags;`, re-exports, test module |
| `crates/tuitbot-core/src/storage/reset.rs` | Added `note_edges`, `note_tags` to `TABLES_TO_CLEAR` + updated count assertions |
| `crates/tuitbot-server/tests/factory_reset.rs` | Updated `tables_cleared` assertion from 38 to 40 |

## Decisions Made

1. **Regex look-ahead not available in Rust `regex` crate.** Changed markdown link extraction to match all `[text](path)` patterns, then filter out `http://`/`https://` URLs in the extraction loop.

2. **CSS hex colors with alphabetic prefix (e.g. `#ff0000`) match as tags.** Accepted as known limitation — inline CSS in markdown notes is rare. The tag regex requires first char after `#` to be `[a-zA-Z]`, which correctly skips pure numeric hex like `#123456`.

3. **Migration placed in crate-level `migrations/` directory** (alongside existing migrations), not just workspace root. `sqlx::migrate!("./migrations")` in `init_test_db()` reads from crate root.

4. **Backlink edge query uses `get_edges_for_target()`** instead of `get_edges_for_source()`. Backlinks are stored with `source_node_id = target_node` (the linked-to node), so querying "who links to me" requires querying by `target_node_id`.

5. **`graph_ingest.rs` separated from `chunker.rs`** as planned. `chunker.rs` (now 185 lines) calls `graph_ingest::extract_and_persist_graph()` as a single integration point. Graph ingest (~200 lines) handles all extraction, resolution, and persistence.

6. **Reset table count updated from 38 to 40.** Both `note_edges` and `note_tags` added before `vault_provenance_links` in delete order (FK-constrained children first).

## Verification

```
cargo fmt --all --check          ✅ clean
cargo clippy --workspace -- -D warnings  ✅ no warnings
RUSTFLAGS="-D warnings" cargo test --workspace  ✅ 6,023 passed, 0 failed
```

Test breakdown for new code:
- `link_extractor` unit tests: 17 passed
- `tests_graph` integration tests: 17 passed
- Existing tests: all green (including reset tests with updated counts)

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Backlink inconsistency after single-node re-chunk | Low | Temporary — restored when other nodes re-chunk. Full re-index restores all. |
| Alphabetic hex colors (`#ff0000`) extracted as tags | Low | Rare in markdown notes. Could add hex pattern exclusion in v2. |
| `find_shared_tag_neighbors` SQL JOIN may slow on high-tag vaults | Low | Indexed by `(account_id, tag_text)`, capped at 10 results. |
| Wikilink resolution false positives on similar titles | Low | Case-insensitive exact match only. No fuzzy matching in v1. |

## Required Inputs for Session 3

Session 3 ("Graph-Aware Retrieval") needs:

1. **This session's output** — storage modules (`edges.rs`, `tags.rs`), graph types (`NoteEdge`, `NoteTag`, `NewEdge`), and query functions.

2. **Repository anchors to read:**
   - `crates/tuitbot-core/src/storage/watchtower/edges.rs` — `get_edges_for_source()`, `get_edges_for_target()` for neighbor expansion
   - `crates/tuitbot-core/src/storage/watchtower/tags.rs` — `find_shared_tag_neighbors()` for shared-tag ranking
   - `crates/tuitbot-core/src/storage/watchtower/mod.rs` — public re-exports
   - Existing retrieval/selection code for the Ghostwriter pipeline (wherever chunk selection and assist routes live)

3. **Decisions to carry forward:**
   - Composite ranking formula from `graph-rag-architecture.md`: direct links (3.0) + backlinks (2.0) + shared tags (1.0) + chunk boost (0.5)
   - Max 8 neighbors per selection
   - 1-hop only (no multi-hop traversal)
   - Deterministic ranking (no LLM)
   - Fail-open: if graph retrieval fails, fall back to current note-centric retrieval

4. **Key types available:**
   - `store::NoteEdge` — edge row with `source_node_id`, `target_node_id`, `edge_type`, `edge_label`
   - `store::NoteTag` — tag row with `node_id`, `tag_text`, `source`
   - `store::get_edges_for_source(pool, account_id, node_id)` — forward edges
   - `store::get_edges_for_target(pool, account_id, node_id)` — incoming edges (backlinks)
   - `store::find_shared_tag_neighbors(pool, account_id, node_id, max)` — shared-tag neighbors ranked by overlap count
