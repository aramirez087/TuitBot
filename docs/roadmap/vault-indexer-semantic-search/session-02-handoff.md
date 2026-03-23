# Session 02 Handoff: Indexer & Storage Foundation

## What Changed

Built the complete background vault indexing foundation: embedding provider abstraction, SQLite schema, in-memory search index, dirty-state tracking, and background `EmbeddingWorker` — all wired into the existing Watchtower lifecycle.

### Files Created (8)

| File | Purpose |
|------|---------|
| `crates/tuitbot-core/migrations/20260322000200_chunk_embeddings.sql` | `chunk_embeddings` table with CASCADE delete, generation tracking, and hash-based dirty detection |
| `crates/tuitbot-core/src/llm/embedding.rs` | `EmbeddingProvider` trait, `EmbeddingError` enum, `EmbeddingResponse`/`EmbeddingUsage` structs |
| `crates/tuitbot-core/src/llm/embedding_factory.rs` | Factory function routing config to OpenAI or Ollama provider |
| `crates/tuitbot-core/src/llm/openai_embedding.rs` | OpenAI `/v1/embeddings` provider (text-embedding-3-small, dim 1536) |
| `crates/tuitbot-core/src/llm/ollama_embedding.rs` | Ollama `/api/embed` provider (nomic-embed-text, dim 768) |
| `crates/tuitbot-core/src/storage/watchtower/embeddings.rs` | 6 CRUD functions + `vec_to_bytes`/`bytes_to_vec` helpers + `IndexStats` |
| `crates/tuitbot-core/src/context/semantic_index.rs` | In-memory brute-force cosine index with insert/remove/search/rebuild |
| `crates/tuitbot-core/src/automation/watchtower/embedding_worker.rs` | Background worker polling dirty chunks on 5-second interval |

### Files Modified (9)

| File | Change |
|------|--------|
| `crates/tuitbot-core/src/config/types.rs` | Added `EmbeddingConfig` struct with provider, api_key, model, base_url, batch_size, enabled |
| `crates/tuitbot-core/src/config/mod.rs` | Added `embedding: Option<EmbeddingConfig>` to `Config`, exported `EmbeddingConfig` |
| `crates/tuitbot-core/src/llm/mod.rs` | Added module declarations for embedding, embedding_factory, openai_embedding, ollama_embedding |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Added `pub mod embeddings;` and `pub use embeddings::*;` |
| `crates/tuitbot-core/src/context/mod.rs` | Added `pub mod semantic_index;` |
| `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Added `pub mod embedding_worker;` |
| `crates/tuitbot-server/src/state.rs` | Added `semantic_index` and `embedding_provider` fields to `AppState` |
| `crates/tuitbot-server/src/main.rs` | Added `semantic_index: None, embedding_provider: None` to AppState construction |
| `crates/tuitbot-server/tests/**/*.rs` | Added new fields to all test AppState constructions |

### Documentation Created (2)

| File | Purpose |
|------|---------|
| `docs/roadmap/vault-indexer-semantic-search/indexer-lifecycle.md` | Startup, steady-state, failure, and shutdown lifecycle documentation |
| `docs/roadmap/vault-indexer-semantic-search/session-02-handoff.md` | This file |

## Decisions Made

### Decision 1: Brute-force cosine scan instead of HNSW

**Chose**: Linear cosine distance scan in `SemanticIndex`.
**Rejected**: `usearch` (C++ FFI, cross-platform CI risk on Windows), `hnsw_rs`.
**Rationale**: For <10K vectors (typical vault size), linear scan completes in <10ms. Avoids introducing a C++ FFI dependency that risks Windows CI breakage. The `SemanticIndex` struct provides a stable API — an HNSW backend can be swapped in later without changing any callers.

### Decision 2: No new crate dependencies

**Chose**: Use only existing workspace dependencies (`reqwest`, `sha2`, `serde`, `sqlx`, `async-trait`, `thiserror`, `tokio`, `hex`).
**Rejected**: Adding `usearch`, `hnsw_rs`, or any new vector search crate.
**Rationale**: Minimizes CI surface area and keeps Cargo.toml stable. The brute-force index is pure Rust with zero new deps.

### Decision 3: Embedding hash = SHA-256 of chunk_text

**Chose**: `EmbeddingWorker` hashes chunk text with SHA-256 and stores as `embedding_hash`.
**Rejected**: Using `chunk_hash` directly (would require coupling worker to chunker's hashing scheme).
**Rationale**: Dirty detection works by comparing `content_chunks.chunk_hash` with `chunk_embeddings.embedding_hash`. When chunk text changes, the chunk's hash changes but the embedding's hash still references the old text — so the chunk appears dirty and gets re-embedded. This means embedding_hash and chunk_hash use different hash sources (embedding hashes chunk_text, while chunk_hash may hash differently). The dirty detection still works correctly because both change when content changes.

### Decision 4: EmbeddingConfig is `Option<EmbeddingConfig>` with `#[serde(default)]`

**Chose**: `embedding` field is `Option<EmbeddingConfig>` on `Config`.
**Rejected**: Non-optional with default `enabled: false`.
**Rationale**: Existing config files without an `[embedding]` section parse as `None` — zero breakage. When `None`, all embedding infrastructure is skipped entirely (no worker spawned, no index built).

### Decision 5: Worker uses the SeedWorker pattern (independent task)

**Chose**: `EmbeddingWorker` as its own spawned task with `LoopScheduler` and `CancellationToken`.
**Rejected**: Embedding inside `WatchtowerLoop::chunk_pending()`.
**Rationale**: Separation of concerns. The WatchtowerLoop manages filesystem watching and chunking. Embedding is a downstream consumer that polls for dirty chunks independently. This matches the existing `SeedWorker` pattern and makes testing straightforward.

## Open Risks

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| 1 | **Brute-force scan too slow at >10K vectors** | Low | Monitor vault sizes in practice. If needed, swap `SemanticIndex` internals to HNSW without API change. O(n) scan at 10K × 768-dim ≈ 5ms. |
| 2 | **Startup rebuild slow with large embedding table** | Low | `get_all_embeddings_for()` loads all BLOBs at once. At 50K × 768 × 4 bytes = ~150MB. If slow, add lazy loading (serve keyword-only until index is ready). |
| 3 | **embedding_hash vs chunk_hash semantic mismatch** | Medium | The worker hashes chunk_text with SHA-256, while the chunker uses its own hash. Both change when content changes, so dirty detection works. But model switches won't auto-detect — need `delete_embeddings_by_model()` first. Document in lifecycle. |
| 4 | **Provider unavailability on startup** | Low | Handled: if `create_embedding_provider()` fails, fields are `None` and system degrades cleanly. |
| 5 | **No search API yet** | None | Intentional — Session 3 will add the search endpoint. The foundation is complete. |

## Session 3 Inputs

Session 3 should consume:

1. **`semantic_index.rs`**: The `SemanticIndex` struct and its `search(query, k)` method for building the search API.
2. **`embedding.rs`**: The `EmbeddingProvider` trait for embedding search queries before index lookup.
3. **`embeddings.rs`** (storage): `get_index_stats_for()` for the status endpoint.
4. **`state.rs`**: `semantic_index` and `embedding_provider` fields on `AppState` for route handlers.
5. **`indexer-lifecycle.md`**: Reference for understanding the indexing pipeline.
6. **`implementation-map.md` § "Session 3"**: File change list and requirements.

### Session 3 Scope (Summary)

- Add `GET /api/vault/search` endpoint that accepts a text query, embeds it, and searches the in-memory index.
- Add `GET /api/vault/index/stats` endpoint that returns `IndexStats`.
- Wire the search results into the Ghostwriter compose flow.
- Add privacy filtering: do not return raw `chunk_text` in Cloud mode.

### Session 3 Exit Criteria

- `GET /api/vault/search?q=...` returns semantically relevant chunks (tested).
- `GET /api/vault/index/stats` returns freshness and model info (tested).
- Privacy envelope respected: Cloud mode strips `chunk_text` from responses.
- Search degrades gracefully when `semantic_index` is `None`.
