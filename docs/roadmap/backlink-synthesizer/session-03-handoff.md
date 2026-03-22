# Session 03 Handoff: Graph-Aware Retrieval & Privacy-Safe APIs

**Date:** 2026-03-21
**Branch:** `epic/backlink-synthesizer`

## What Changed

Implemented graph-aware neighbor expansion, deterministic ranking, and privacy-safe API endpoints that surface related-note suggestions with human-readable reasons.

### Files Created

| File | Purpose | Lines |
|------|---------|-------|
| `crates/tuitbot-core/src/context/graph_expansion.rs` | Graph neighbor expansion, ranking, classification + 30 unit tests | ~480 |
| `crates/tuitbot-server/tests/graph_neighbor_tests.rs` | 8 server integration tests for neighbors API | ~440 |
| `docs/roadmap/backlink-synthesizer/retrieval-ranking-spec.md` | Ranking formula, weights, fallback behavior documentation | ~110 |
| `docs/roadmap/backlink-synthesizer/graph-api-contract.md` | API endpoint spec, response shapes, privacy rules | ~150 |
| `docs/roadmap/backlink-synthesizer/session-03-handoff.md` | This file | ~100 |

### Files Modified

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/context/mod.rs` | Added `pub mod graph_expansion;` |
| `crates/tuitbot-core/src/context/retrieval.rs` | Added `edge_type`/`edge_label` to `VaultCitation`, updated `citations_to_provenance_refs()` and test fixtures |
| `crates/tuitbot-core/src/storage/provenance.rs` | Added `edge_type`/`edge_label` to `ProvenanceRef` and `ProvenanceLink`, updated insert/copy SQL, added 2 new tests |
| `crates/tuitbot-core/src/storage/watchtower/nodes.rs` | Added `get_nodes_by_ids()` batch query |
| `crates/tuitbot-core/src/storage/watchtower/chunks.rs` | Added `get_best_chunks_for_nodes()` batch query |
| `crates/tuitbot-server/src/routes/vault/mod.rs` | Added `note_neighbors` handler, `NoteNeighborsResponse`, `NeighborItem` types, test router update |
| `crates/tuitbot-server/src/routes/vault/selections.rs` | Added `graph_neighbors`/`graph_state` to `GetSelectionResponse`, auto-expand on `get_selection` |
| `crates/tuitbot-server/src/routes/rag_helpers.rs` | Added `resolve_graph_suggestions()` and `GraphSuggestionResult` |
| `crates/tuitbot-server/src/lib.rs` | Registered `/vault/notes/{id}/neighbors` route (before `{id}`) |
| `crates/tuitbot-server/src/routes/content/drafts.rs` | Updated `ProvenanceRef` construction to include `edge_type`/`edge_label` |
| `crates/tuitbot-server/src/routes/content/compose/tests/routing.rs` | Updated test `ProvenanceRef` fixtures |
| `crates/tuitbot-server/src/routes/content/compose/tests/types.rs` | Updated test `ProvenanceRef` fixtures |
| `crates/tuitbot-core/src/storage/scheduled_content/tests/scheduling.rs` | Updated test `ProvenanceRef` fixture |
| `crates/tuitbot-core/src/storage/scheduled_content/tests/provenance.rs` | Updated test `ProvenanceRef` fixtures |

### Files Removed

| File | Reason |
|------|--------|
| `crates/tuitbot-core/migrations/20260321000300_provenance_graph_fields.sql` | Duplicate — Session 2 already added `edge_type`/`edge_label` in migration 20260321000200 |
| `migrations/20260321000300_provenance_graph_fields.sql` | Same (workspace root copy) |

## Decisions Made

1. **No new migration needed.** Session 2's `20260321000200_note_graph.sql` already added `edge_type` and `edge_label` columns to `vault_provenance_links`. Removed the duplicate migration that caused "duplicate column name" errors.

2. **Edge type classification in expansion loop.** Outgoing edges from `get_edges_for_source()` are classified by `edge_type` field (wikilink/markdown_link -> direct, backlink -> backlink, shared_tag -> shared tag). Previously the plan counted all outgoing edges as direct links, which would misrank shared-tag edges.

3. **Reason/intent serialized as strings, not enum objects.** The `NeighborItem` API type uses `String` for `reason` and `intent` (via `serde_json::to_value` on the enum). This avoids coupling the frontend to Rust enum representations while keeping snake_case values like `linked_note`, `pro_tip`.

4. **Graph expansion is fail-open.** `resolve_graph_suggestions()` catches errors and returns `FallbackActive` with empty neighbors. The endpoint always returns 200 OK — graph is supplementary, never blocking.

5. **Cloud mode omits `relative_path`.** Consistent with existing privacy envelope: Cloud mode doesn't expose local filesystem paths. The field is `skip_serializing_if = "Option::is_none"`.

6. **Selection auto-expands neighbors.** `GET /vault/selection/{session_id}` auto-expands graph neighbors when `resolved_node_id` is present, avoiding a second roundtrip. The fields are optional for backward compatibility.

7. **`get_best_chunks_for_nodes()` uses correlated subquery.** Returns at most 1 chunk per node (highest boost, lowest index). This is more efficient than fetching all chunks and filtering in Rust.

## Verification

```
cargo fmt --all --check          ✅ clean
cargo clippy --workspace -- -D warnings  ✅ no warnings
RUSTFLAGS="-D warnings" cargo test --workspace  ✅ 6,062 passed, 0 failed, 1 ignored
```

Test breakdown for new code:
- `graph_expansion` unit tests: 30 passed (scoring, classification, reason labels, serde)
- `graph_neighbor_tests` server integration: 8 passed (API, isolation, cloud mode, ordering)
- `provenance` edge field tests: 2 passed (roundtrip, insert)
- Existing tests: all green (including provenance, assist, compose, selection)

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `find_shared_tag_neighbors()` returns `(node_id, tag_text)` pairs but GROUP BY collapses to one tag per neighbor | Low | The expansion loop also collects tags from outgoing edges. The SQL query's single-tag-per-row is supplementary. |
| Intent classification heuristic may misclassify edge labels | Low | Defaults to `Related`. Intent is a UI hint, not a decision gate. Can be refined with LLM classification in v2. |
| `get_best_chunks_for_nodes()` correlated subquery may be slow on very large chunk tables | Low | Limited to 8 neighbor node_ids max. Indexed by `(account_id, node_id, status)`. |
| `NeighborItem` duplicates some fields from `GraphNeighbor` | Low | Intentional API/domain separation. Server types don't leak core types. |

## Required Inputs for Session 4

Session 4 ("Frontend Integration") needs:

1. **This session's output:**
   - `graph_expansion.rs` types: `GraphNeighbor`, `GraphState`, `SuggestionReason`, `SuggestionIntent`
   - API endpoint: `GET /api/vault/notes/{id}/neighbors`
   - Selection response with `graph_neighbors` and `graph_state`

2. **API contract document:** `docs/roadmap/backlink-synthesizer/graph-api-contract.md`

3. **Key types for frontend TypeScript:**
   - `NoteNeighborsResponse` — top-level response
   - `NeighborItem` — individual neighbor card
   - `GraphState` enum — UI state branching
   - `reason`/`intent` as snake_case strings

4. **Frontend integration points:**
   - Compose page: when `selection` query param is present, fetch selection → display `graph_neighbors` cards
   - Note detail page: fetch `/vault/notes/{id}/neighbors` → display related notes sidebar
   - User can accept/reject neighbor suggestions before compose

5. **Decisions to carry forward:**
   - Accepted neighbor IDs should be merged into `selected_node_ids` for `resolve_composer_rag_context()`
   - MAX_GRAPH_FRAGMENTS_PER_NOTE = 3 per neighbor in the prompt
   - Graph UI should show `reason_label` and `intent` visually
   - User opt-out: suggestions are visible but not auto-included
