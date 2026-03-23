# Session 03 Handoff: Hybrid Retrieval & API Contract

## What Changed

Implemented the hybrid retrieval layer and two new API endpoints that turn indexed vault content into ranked, explainable semantic evidence results. Extended core types with `MatchReason` classification and optional scoring fields while maintaining full backward compatibility.

### Files Created

| File | Purpose |
|---|---|
| `crates/tuitbot-core/src/context/semantic_search.rs` | Pure semantic search function over `SemanticIndex` — converts raw (chunk_id, distance) pairs into scored `SemanticHit` results |
| `crates/tuitbot-core/src/context/hybrid_retrieval.rs` | RRF-based blending of semantic, keyword, and graph signals with per-result `MatchReason` classification |
| `crates/tuitbot-server/src/routes/vault/evidence.rs` | `GET /api/vault/evidence` — unified evidence endpoint for all Ghostwriter surfaces |
| `crates/tuitbot-server/src/routes/vault/index_status.rs` | `GET /api/vault/index-status` — semantic index health and stats endpoint |
| `docs/roadmap/vault-indexer-semantic-search/search-api-contract.md` | OpenAPI-style documentation for both endpoints |
| `docs/roadmap/vault-indexer-semantic-search/retrieval-ranking-spec.md` | RRF algorithm spec with worked example, fallback matrix, performance budget |
| `docs/roadmap/vault-indexer-semantic-search/session-03-handoff.md` | This file |

### Files Modified

| File | Change |
|---|---|
| `crates/tuitbot-core/src/context/retrieval.rs` | Added `MatchReason` enum; extended `VaultCitation` with optional `match_reason` and `score` fields |
| `crates/tuitbot-core/src/context/mod.rs` | Added `hybrid_retrieval` and `semantic_search` module declarations |
| `crates/tuitbot-core/src/storage/watchtower/chunks.rs` | Added `get_chunks_with_context_by_ids()` for enriching semantic-only hits |
| `crates/tuitbot-server/src/routes/vault/mod.rs` | Added `evidence` and `index_status` module declarations |
| `crates/tuitbot-server/src/routes/assist/angles.rs` | Updated VaultCitation constructions with new optional fields |
| `crates/tuitbot-server/src/lib.rs` | Registered `/vault/evidence` and `/vault/index-status` routes (before `/vault/search` per literal-before-parameterized rule) |
| `dashboard/src/lib/api/types.ts` | Added `MatchReason`, `EvidenceResult`, `EvidenceResponse`, `IndexStatusResponse`, `IndexStatusSummary` types; extended `VaultCitation` and `ProvenanceRef` |
| `dashboard/src/lib/api/client.ts` | Added `vault.searchEvidence()` and `vault.indexStatus()` methods |

## Decisions Made

### Decision 1: MatchReason as a serde-tagged enum in tuitbot-core

**Chose:** `#[serde(rename_all = "snake_case")]` enum with variants `Semantic`, `Keyword`, `Graph`, `Hybrid`.
**Rationale:** Lives in `retrieval.rs` alongside `VaultCitation` for tight coupling. Snake_case serialization matches the API contract. Both the server and frontend consume the same string values.

### Decision 2: Semantic search as a pure function (no async, no DB)

**Chose:** `semantic_search()` takes `&SemanticIndex` and `&[f32]` query embedding, returns `Vec<SemanticHit>`. Caller embeds the query first.
**Rationale:** Keeps the function testable without mocking HTTP. The server handler orchestrates "embed query → search index" and catches embedding errors for fallback.

### Decision 3: RRF with k=60 for fusion

**Chose:** Reciprocal Rank Fusion with k=60 constant per the original paper (Cormack et al. 2009).
**Rationale:** Standard approach for merging heterogeneous ranking signals. No score normalization needed. Results that appear in multiple lists naturally rank higher. The constant k=60 prevents rank-1 items from dominating.

### Decision 4: Single unified evidence endpoint

**Chose:** `GET /api/vault/evidence?q=&limit=&mode=&scope=` serves all consumers.
**Rationale:** Per operator rules: "extend request and response types so selection review, hook generation, tweet editing, and slot-targeted thread actions can issue evidence queries without bespoke APIs for each surface."

### Decision 5: Fail-open when semantic is unavailable

**Chose:** `embed_and_search()` returns `None` on any failure (no provider, empty index, API error, dimension mismatch). `hybrid_search()` treats `None` as keyword-only. Response includes `index_status.freshness_pct: 0.0` so frontend can show degraded state.
**Rationale:** Operator constraint: "If semantic indexing is stale or unavailable, fall back cleanly to the current vault behavior."

### Decision 6: Cloud mode omits relative_path

**Chose:** Evidence results in Cloud mode set `relative_path: None` (serialized as absent due to `skip_serializing_if`).
**Rationale:** Matches existing pattern in `note_neighbors` and `vault_sources` endpoints. Operator constraint: "Do not expose raw note bodies from read APIs beyond existing privacy rules."

### Decision 7: Added get_chunks_with_context_by_ids to watchtower

**Chose:** New storage function to look up chunks by chunk IDs (with node metadata JOIN).
**Rationale:** Semantic search returns chunk IDs only. To build `EvidenceResult` with heading_path, source_path, snippet, etc., we need to enrich these hits from the DB. Existing functions only look up by node_id or keyword.

## Quality Gates Passed

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | Clean |
| `cargo clippy --workspace -- -D warnings` | Clean |
| `RUSTFLAGS="-D warnings" cargo test --workspace` | 567 tests pass, 0 failures |
| `npm --prefix dashboard run check` | 0 errors, 0 warnings |

## Open Risks

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| 1 | **Embedding latency not yet measured in production** | Medium | Logged in `embed_and_search()`. Frontend can show loading skeleton. Target <100ms total. |
| 2 | **RRF scores not intuitive to users** | Low | Documented in ranking spec. Frontend should show High/Medium/Low badges, not raw scores. |
| 3 | **No integration test with real embeddings** | Medium | Unit tests cover RRF logic, fallback paths, and account isolation. E2E test requires a running embedding provider. |
| 4 | **SemanticIndex read lock contention** | Low | Read lock held only during search (<10ms for 50K vectors). Write lock only during indexer batch. |
| 5 | **get_chunks_with_context_by_ids not tested with populated DB** | Low | Function follows same pattern as existing `get_chunks_for_nodes_with_context` (tested). SQL is straightforward JOIN. |

## Session 4 Inputs

Session 4 (Ghostwriter Evidence UX) consumes:
1. `search-api-contract.md` — endpoint schemas for `GET /api/vault/evidence` and `GET /api/vault/index-status`
2. `retrieval-ranking-spec.md` — MatchReason values and score interpretation for UI display
3. `ghostwriter-evidence-ux.md` — UX flow for evidence panel in the compose experience
4. Frontend types: `EvidenceResult`, `EvidenceResponse`, `IndexStatusResponse`, `MatchReason` in `types.ts`
5. Client methods: `api.vault.searchEvidence()` and `api.vault.indexStatus()` in `client.ts`

### Session 4 Exit Criteria

- Evidence panel renders in the Ghostwriter compose flow
- Results display `match_reason` badges (Semantic/Keyword/Graph/Hybrid)
- Index status indicator shows freshness state
- Degraded state UI when semantic index is unavailable
- Evidence results can be pinned, dismissed, or applied to compose
- All existing compose flow tests continue to pass
