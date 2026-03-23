# Implementation Map: Vault Indexer + Semantic Search

## Overview

This map breaks the epic into 5 follow-on sessions (Sessions 2–6), each with file-level anchors, measurable success criteria, and explicit dependencies on prior sessions.

---

## Session 2: Indexer & Storage Foundation

### Goal
Build the embedding pipeline: provider abstraction, storage schema, dirty-state tracking, and background indexer worker. No frontend changes. No search API yet.

### Files to Create

| File | Crate | Purpose |
|------|-------|---------|
| `src/llm/embedding.rs` | tuitbot-core | `EmbeddingProvider` trait, `EmbeddingError`, `EmbeddingResponse`, `EmbeddingUsage` types |
| `src/llm/embedding_factory.rs` | tuitbot-core | `create_embedding_provider()` factory, returns `Box<dyn EmbeddingProvider>` |
| `src/llm/openai_embedding.rs` | tuitbot-core | OpenAI `text-embedding-3-small` implementation of `EmbeddingProvider` |
| `src/llm/ollama_embedding.rs` | tuitbot-core | Ollama `nomic-embed-text` implementation of `EmbeddingProvider` |
| `src/storage/watchtower/embeddings.rs` | tuitbot-core | CRUD for `chunk_embeddings` table: upsert, get_all, get_dirty, get_stats, delete_by_model |
| `src/context/semantic_index.rs` | tuitbot-core | `SemanticIndex` struct wrapping `usearch::Index` with insert/delete/search/len |
| `src/automation/watchtower/embedding_worker.rs` | tuitbot-core | `EmbeddingWorker` background task: polls dirty chunks, embeds in batches, inserts into index |
| Migration SQL | tuitbot-core | `CREATE TABLE chunk_embeddings` with indexes |

### Files to Modify

| File | Change |
|------|--------|
| `src/config/types.rs` | Add `EmbeddingConfig` struct, add `embedding: Option<EmbeddingConfig>` to `AppConfig` |
| `src/llm/mod.rs` | Re-export `embedding` module |
| `src/storage/watchtower/mod.rs` | Re-export `embeddings` module |
| `src/automation/watchtower/mod.rs` | Wire `EmbeddingWorker` into `WatchtowerLoop::run()` — start after initial scan + chunking |
| `Cargo.toml` (tuitbot-core) | Add `usearch` dependency |

### Success Criteria

1. `cargo test -p tuitbot-core embedding` — all embedding provider tests pass (mock provider)
2. `cargo test -p tuitbot-core embeddings` — all storage CRUD tests pass
3. `cargo test -p tuitbot-core semantic_index` — usearch insert/search/delete tests pass
4. After `ingest_content()` + `chunk_pending()`, dirty chunks are detected by `get_dirty_chunks_for()`
5. `EmbeddingWorker` processes dirty chunks and inserts embeddings into both SQLite and the live HNSW index
6. `get_index_stats_for()` returns correct total/embedded/dirty counts
7. Full CI checklist passes: `cargo fmt`, `cargo clippy`, `cargo test`

### Dependencies
- None (first implementation session)

### Risk
- `usearch` crate API may differ from expected. Mitigation: review crate docs before coding; fall back to `hnsw_rs` if `usearch` has blocking issues.

---

## Session 3: Hybrid Retrieval & API Contract

### Goal
Build the semantic search function, hybrid retrieval blending, and the new `/api/vault/evidence` endpoint. No frontend changes yet.

### Files to Create

| File | Crate | Purpose |
|------|-------|---------|
| `src/context/semantic_search.rs` | tuitbot-core | `semantic_search()` function: query → embed → HNSW search → chunk metadata lookup |
| `src/context/hybrid_retrieval.rs` | tuitbot-core | `hybrid_search()` function: parallel semantic + keyword + graph → deduplicate → blend scores → return with `match_reason` |
| `src/routes/vault/evidence.rs` | tuitbot-server | `GET /api/vault/evidence?q=&scope=&limit=&mode=` endpoint |
| `src/routes/vault/index_status.rs` | tuitbot-server | `GET /api/vault/index-status` endpoint |

### Files to Modify

| File | Change |
|------|--------|
| `src/context/retrieval.rs` | Add `MatchReason` enum (`Semantic`, `Graph`, `Keyword`, `Hybrid`), extend `VaultCitation` with `match_reason` field |
| `src/context/mod.rs` | Re-export new modules |
| `src/routes/vault/mod.rs` | Register new routes |
| `src/state.rs` (tuitbot-server) | Add `semantic_index: Option<Arc<RwLock<SemanticIndex>>>` and `embedding_provider: Option<Arc<dyn EmbeddingProvider>>` to `AppState` |

### API Contract

**`GET /api/vault/evidence`**

Query parameters:
- `q` (required): Search query text
- `limit` (optional, default 8, max 20): Maximum results
- `mode` (optional, default "hybrid"): `semantic`, `keyword`, or `hybrid`

Response:
```json
{
  "results": [
    {
      "chunk_id": 42,
      "node_id": 7,
      "heading_path": "Distributed Systems > CAP Theorem",
      "snippet": "The CAP theorem states that...",
      "relative_path": "notes/distributed-systems.md",
      "match_reason": "semantic",
      "score": 0.87,
      "node_title": "Distributed Systems"
    }
  ],
  "query": "consensus algorithms",
  "mode": "hybrid",
  "index_status": {
    "total_chunks": 1234,
    "embedded_chunks": 1200,
    "freshness_pct": 97.2
  }
}
```

**`GET /api/vault/index-status`**

Response: see `semantic-index-architecture.md` Section 6.

### Success Criteria

1. `semantic_search("consensus algorithms")` returns chunks about consensus from a test vault with embedded chunks
2. `hybrid_search()` merges semantic + keyword results, deduplicates by chunk_id, and returns sorted by blended score
3. Each result carries a correct `match_reason` (`Semantic`, `Keyword`, or `Hybrid` when both matched)
4. When `SemanticIndex` is empty or `EmbeddingProvider` is `None`, `hybrid_search()` falls back to keyword-only with no error
5. `/api/vault/evidence?q=test&mode=keyword` returns keyword results even without embeddings
6. `/api/vault/index-status` returns correct stats
7. Cloud mode: `relative_path` is omitted from evidence responses
8. Full CI checklist passes

### Dependencies
- Session 2 (embedding storage, SemanticIndex, EmbeddingProvider)

---

## Session 4: Ghostwriter Evidence Panel

### Goal
Build the frontend Evidence Rail: search UI, result cards, pin/dismiss, index status, and integration with ComposerInspector. No auto-query yet.

### Files to Create

| File | Location | Purpose |
|------|----------|---------|
| `EvidenceRail.svelte` | `dashboard/src/lib/components/composer/` | Main evidence panel: search bar, result list, pinned section, index status |
| `EvidenceCard.svelte` | `dashboard/src/lib/components/composer/` | Individual result card with match_reason badge, snippet, actions |
| `IndexStatusBadge.svelte` | `dashboard/src/lib/components/composer/` | 8px dot + tooltip showing index freshness |
| `src/lib/api/evidence.ts` | `dashboard/src/lib/api/` | API client functions: `searchEvidence()`, `getIndexStatus()` |
| `tests/unit/EvidenceRail.test.ts` | `dashboard/tests/unit/` | Unit tests for EvidenceRail rendering, pin/dismiss, empty states |

### Files to Modify

| File | Change |
|------|--------|
| `InspectorContent.svelte` | Add `EvidenceRail` between VoiceContextPanel and FromVaultPanel |
| `ComposerInspector.svelte` | Add `pinnedEvidence` state, pass to InspectorContent, include in `handleGenerateFromVault()` context |
| `src/lib/api/types.ts` | Add `EvidenceResult`, `IndexStatus`, `MatchReason` types |
| `src/lib/api/index.ts` | Export new evidence API functions |

### Success Criteria

1. Evidence Rail renders in the inspector when embedding config is present
2. Typing in the search bar fires `GET /api/vault/evidence` after 300ms debounce
3. Results render as EvidenceCards with correct match_reason badges
4. Pin action moves result to Pinned section (max 5)
5. Dismiss action removes result and excludes its chunk_id from future queries in the session
6. IndexStatusBadge shows correct color and tooltip
7. Empty states render correctly for: no index, no results, provider unavailable
8. Pinned evidence is included in LLM context when generating drafts
9. `npm run check` passes, `npm run build` succeeds, `npx vitest run` passes
10. Keyboard shortcuts work: `Cmd+Shift+E` opens/closes rail

### Dependencies
- Session 3 (evidence API endpoint, index-status endpoint)

---

## Session 5: Auto-Query & Slot Actions

### Goal
Add auto-query during editing (opt-in, debounced), apply evidence to thread slots, and extend provenance with semantic metadata.

### Files to Modify

| File | Change |
|------|--------|
| `EvidenceRail.svelte` | Add auto-query toggle, debounced query from focused block text, "Suggested" badge on auto-results, AbortController for cancellation |
| `ComposerInspector.svelte` | Wire `handleSlotInsert()` to accept evidence results (not just `NeighborItem`), add `similarity_score` and `match_reason` to `ProvenanceRef` |
| `SlotTargetPanel.svelte` | Accept evidence items alongside neighbor items for slot targeting |
| `EvidenceCard.svelte` | Add "Apply to slot" dropdown action |
| `src/lib/api/types.ts` | Extend `ProvenanceRef` with optional `similarity_score` and `match_reason` fields |
| `src/lib/analytics/backlinkFunnel.ts` | Add evidence-specific analytics events |

### New Analytics Events

- `evidence_rail_opened`
- `evidence_search_executed`
- `evidence_pinned`
- `evidence_dismissed`
- `evidence_applied_to_slot`
- `evidence_auto_query_toggled`
- `evidence_contributed_to_draft`

### Success Criteria

1. Auto-query toggle works: off by default, fires semantic search 800ms after typing stops
2. Auto-query results show "Suggested" badge, clear when user switches blocks
3. In-flight auto-queries are cancelled when new text is typed
4. "Apply to slot" on an evidence card opens a slot picker matching existing SlotTargetPanel UX
5. Applying evidence to a slot calls `api.assist.improve()` with evidence context
6. Provenance includes `match_reason: "semantic"` and `similarity_score` for evidence-sourced fragments
7. Analytics events fire correctly
8. Auto-query hidden on mobile (< 640px)
9. `npm run check` passes, `npx vitest run` passes

### Dependencies
- Session 4 (Evidence Rail, EvidenceCard, pin/dismiss)

---

## Session 6: Observability & Release Readiness

### Goal
Add index health metrics, feature flag for gradual rollout, QA across Desktop/Self-host/Cloud, and documentation.

### Files to Create / Modify

| File | Change |
|------|--------|
| `src/config/types.rs` | Add `semantic_search_enabled: bool` feature flag to `AppConfig` (default: `true`) |
| `src/routes/vault/evidence.rs` | Check feature flag, return 404 when disabled |
| `EvidenceRail.svelte` | Check feature flag from runtime config |
| `src/routes/vault/index_status.rs` | Add embedding rate, error rate, provider latency metrics |
| Tests | Integration tests covering: provider unavailable, index empty, stale index, model switch, cloud mode privacy |

### QA Matrix

| Scenario | Desktop | Self-Host | Cloud |
|----------|---------|-----------|-------|
| Fresh install, no embedding config | Evidence Rail hidden | Evidence Rail hidden | Evidence Rail hidden |
| Ollama running, vault indexed | Full semantic search | Full semantic search | N/A |
| OpenAI configured, vault indexed | Full semantic search | Full semantic search | Full semantic search |
| Provider unavailable | Keyword fallback, warning shown | Keyword fallback, warning shown | Keyword fallback, warning shown |
| Cloud mode privacy | N/A | N/A | No relative_path, no raw vectors, snippet-only |
| Large vault (10K+ notes) | Index capped at 50K chunks | Index capped at 50K chunks | Index capped at 50K chunks |
| Model switch | Re-embedding triggered | Re-embedding triggered | Re-embedding triggered |
| Feature flag disabled | Evidence Rail hidden | Evidence Rail hidden | Evidence Rail hidden |

### Success Criteria

1. Feature flag `semantic_search_enabled = false` hides Evidence Rail and returns 404 on evidence endpoints
2. All QA matrix scenarios pass manually
3. Index health metrics exposed in `/api/vault/index-status` response
4. Full CI checklist passes on macOS, Linux, Windows
5. Coverage thresholds maintained (75% core, 60% MCP, 70% frontend)
6. No privacy invariant violations in any deployment mode

### Dependencies
- Sessions 2–5 (all implementation complete)

---

## Session Dependency Graph

```
Session 2: Indexer & Storage Foundation
    │
    ▼
Session 3: Hybrid Retrieval & API Contract
    │
    ▼
Session 4: Ghostwriter Evidence Panel
    │
    ▼
Session 5: Auto-Query & Slot Actions
    │
    ▼
Session 6: Observability & Release Readiness
```

All sessions are strictly sequential. Each session's success criteria must be met before the next session begins.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `usearch` crate API breaks or has bugs | Low | High (blocks Session 2) | Review crate docs and examples first. Fallback: `hnsw_rs` crate or manual brute-force cosine for small indexes. |
| Embedding API cost for large vaults | Medium | Medium (user surprise) | Default to Ollama (free, local). Batch size limits. Skip unchanged chunks via hash. Show cost estimate in UI before reindex. |
| HNSW memory for large vaults | Low | Medium (OOM on small machines) | Cap at 50K vectors. Graceful overflow to SQLite-only cosine scan. Log warning when approaching cap. |
| Auto-query latency makes typing feel laggy | Medium | High (UX degradation) | 800ms debounce. Cancel in-flight. Loading skeleton. Opt-in only. |
| Cross-platform SQLite BLOB handling | Low | Medium (CI failures) | Use sqlx's native BLOB binding. Test in CI on all 3 platforms. |
| Privacy leakage in Cloud mode | Low | Critical | Same `SNIPPET_MAX_LEN = 120` truncation. No raw embeddings in API. Account-scoped queries. Existing privacy test suite covers this. |
| Dimension mismatch when switching models | Medium | Low (recoverable) | Detect at indexer startup. Delete old-model embeddings. Rebuild index from empty. Show "Re-indexing for new model" status. |
| Ollama not installed on Desktop | High | Low (graceful fallback) | Evidence Rail shows keyword results only. Status badge: "Embedding provider unavailable." No error modal. |
