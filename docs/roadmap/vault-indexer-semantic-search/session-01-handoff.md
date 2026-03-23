# Session 01 Handoff: Semantic Evidence Charter & UX

**Date:** 2026-03-22
**Branch:** `epic/vault-indexer-semantic-search`

---

## What Changed

Six documentation deliverables created under `docs/roadmap/vault-indexer-semantic-search/`:

1. **`current-state-audit.md`** — Complete audit of the live compose flow from Obsidian selection through published draft. Maps every component, route, storage table, and retrieval function. Identifies 7 specific seams where semantic evidence integrates: selection review, hook generation, fragment retrieval, slot refinement, vault search, auto-expand on selection, and provenance chain.

2. **`epic-charter.md`** — Problem statement, competitive edge, user value, scope (11 in-scope, 8 out-of-scope), 9 measurable success criteria, and 8 design principles. Modeled on the existing `backlink-synthesizer/epic-charter.md` format.

3. **`semantic-index-architecture.md`** — Full technical architecture: `EmbeddingProvider` trait design, `EmbeddingConfig` struct, `chunk_embeddings` SQLite schema, `usearch` HNSW index lifecycle, dirty-state tracking via hash comparison, freshness model with thresholds, deployment matrix (Desktop/Self-host/Cloud), migration plan (additive only), performance and memory budgets.

4. **`ghostwriter-evidence-ux.md`** — Complete UX specification: Evidence Rail placement in ComposerInspector, search-before-generation flow, search-during-editing flow (auto-query with 800ms debounce, opt-in), scope controls, index status affordances (5 states), empty states (3 variants), degraded states (3 variants), result card anatomy, 4 result actions (pin, dismiss, apply-to-slot, view source), keyboard shortcuts, responsive behavior, and 7 analytics events.

5. **`implementation-map.md`** — Sessions 2–6 breakdown with file-level anchors and measurable success criteria. Strictly sequential dependency graph. Risk register with 8 identified risks and mitigations.

6. **`session-01-handoff.md`** — This file.

**No source code was modified.** This session is documentation-only.

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Separate `EmbeddingProvider` trait (not overloading `LlmProvider`) | Different API shape: batch in → batch out, no streaming. Cleaner separation. |
| D2 | SQLite BLOBs + in-process `usearch` HNSW index | No new infrastructure dependency. Desktop/self-host users don't want a vector DB server. Rebuild from SQLite at startup. |
| D3 | Chunk-level embeddings (not note-level or paragraph-level) | Chunks are the existing retrieval unit. 1:1 mapping with `VaultCitation` and provenance. |
| D4 | Hash-based dirty tracking with generation counter | Extends existing content hashing in Watchtower. Generation counter enables freshness % without full scan. |
| D5 | `MatchReason` enum on all retrieval results | Operator requirement: "semantic as complementary, not replacement." Users see why each result surfaced. |
| D6 | Evidence Rail inside existing ComposerInspector | Operator requirement: "power tool, not parallel workflow." Slots into existing compose flow. |
| D7 | Auto-query opt-in with 800ms debounce | Operator requirement: "never injected without a visible action." Debounce prevents query spam. |
| D8 | Ollama default for Desktop, OpenAI for Cloud | Local-first: free embedding for Desktop users. Cloud uses server-side provider. |
| D9 | 50K vector cap with overflow eviction | Memory budget: ~124MB for 50K vectors at 1536d. Acceptable for Desktop. |
| D10 | Feature flag for gradual rollout | Session 6 adds `semantic_search_enabled` config flag. Evidence Rail hidden when disabled. |

---

## Residual Risks

| Risk | Severity | Status |
|------|----------|--------|
| `usearch` crate may have API differences from expected design | Medium | Needs validation in Session 2. Fallback: `hnsw_rs` or brute-force cosine. |
| Embedding API costs for large vaults using OpenAI | Medium | Mitigated by Ollama default + hash-based skip. Session 6 should add cost estimation UI. |
| HNSW memory on constrained machines (4GB RAM) | Low | Cap at 50K vectors. Could lower to 25K for memory-constrained mode. |
| Auto-query UX may feel intrusive despite opt-in | Low | Session 5 UX testing will validate. Can be removed if metrics show low adoption. |
| Cross-platform BLOB handling in CI | Low | SQLite + sqlx handle BLOBs natively. Session 2 tests will cover this. |

---

## Required Inputs for Session 2

Session 2 (Indexer & Storage Foundation) can begin immediately with the following inputs:

### Must Read Before Starting
- `docs/roadmap/vault-indexer-semantic-search/semantic-index-architecture.md` — the full architecture spec
- `crates/tuitbot-core/src/llm/mod.rs` — existing `LlmProvider` trait pattern to mirror
- `crates/tuitbot-core/src/llm/factory.rs` — existing factory pattern to extend
- `crates/tuitbot-core/src/llm/openai_compat.rs` — existing OpenAI client to reference for embedding implementation
- `crates/tuitbot-core/src/config/types.rs` — where to add `EmbeddingConfig`
- `crates/tuitbot-core/src/storage/watchtower/chunks.rs` — existing chunk CRUD to reference for embeddings CRUD
- `crates/tuitbot-core/src/automation/watchtower/mod.rs` — where to wire `EmbeddingWorker` into the Watchtower loop
- `crates/tuitbot-core/src/automation/watchtower/chunker.rs` — chunk_hash computation to extend for dirty tracking

### External Dependencies
- Add `usearch = "^2.0"` to `crates/tuitbot-core/Cargo.toml` — verify it compiles on macOS, Linux, Windows
- No other new external dependencies

### Key Constraints
- `tuitbot-core` file size limit: 500 lines per `.rs` file
- Dependency layering: Toolkit ← Workflow ← Autopilot — embedding code goes in `llm/` (Toolkit layer)
- Error handling: `thiserror` in tuitbot-core
- Cross-platform: use `std::env::temp_dir()` in tests, not `/tmp`
- All existing tests must continue to pass

### Expected Outputs
- All files listed in Session 2 of `implementation-map.md` created
- `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace` passes
- `get_dirty_chunks_for()` returns newly chunked content
- `EmbeddingWorker` processes dirty chunks (testable with mock provider)
- `SemanticIndex` insert/search/delete operations verified by unit tests
