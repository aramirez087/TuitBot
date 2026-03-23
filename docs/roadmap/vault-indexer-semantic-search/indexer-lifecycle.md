# Indexer Lifecycle

Documents the startup, steady-state, and failure lifecycle of the semantic embedding indexer.

## Startup Sequence

1. **Migration**: SQLite migration `20260322000200_chunk_embeddings.sql` runs via sqlx auto-migrate, creating the `chunk_embeddings` table if it doesn't exist.
2. **Config check**: `config.embedding` is checked. If `None` or `enabled == false`, all embedding infrastructure is skipped — `AppState.semantic_index` and `embedding_provider` remain `None`.
3. **Provider init**: `create_embedding_provider()` creates an `OpenAiEmbeddingProvider` or `OllamaEmbeddingProvider` based on config. If creation fails (e.g., missing API key for OpenAI), the error is logged and embedding is disabled (fail-open).
4. **Index rebuild**: `get_all_embeddings_for(pool, account_id)` loads all stored embeddings from SQLite. Each row's BLOB is deserialized from little-endian f32 bytes via `bytes_to_vec()`. A `SemanticIndex` is constructed and populated via `rebuild_from()`.
5. **Worker spawn**: An `EmbeddingWorker` is spawned as a Tokio task with the shared `CancellationToken`, polling every 5 seconds for dirty chunks.

## Steady-State Operation

```
Watchtower ingests file → upsert_content_node() → chunker writes content_chunks
    ↓
EmbeddingWorker polls get_dirty_chunks_for() every 5s
    ↓
Dirty chunks found → provider.embed(texts) → upsert_chunk_embedding()
    ↓
In-memory SemanticIndex updated via index.write().insert()
```

The worker processes dirty chunks in configurable batches (default 100). Each batch is all-or-nothing at the provider level.

## Note Edit Flow

1. User edits a markdown file in their vault.
2. Watchtower detects the file change via filesystem notify.
3. `upsert_content_node()` detects `content_hash` changed → node is re-chunked.
4. New chunks get new `chunk_hash` values.
5. Old embeddings have `embedding_hash` that no longer matches `chunk_hash` → they become dirty.
6. `EmbeddingWorker` picks them up on next tick, re-embeds, and updates the embedding row with new `embedding_hash`.

## Note Delete Flow

1. User deletes a file.
2. Watchtower marks the content_node as deleted.
3. Chunk status set to 'stale' or chunks are deleted.
4. `ON DELETE CASCADE` on `chunk_embeddings.chunk_id` automatically removes the embedding row.
5. The in-memory `SemanticIndex` still has the old vector until next rebuild (harmless — search results pointing to deleted chunks are filtered at query time).

## Model Switch Flow

1. User changes `config.embedding.provider` or `config.embedding.model`.
2. On next server restart:
   - Old embeddings are detected by model mismatch (stored `model_id` vs provider's `model_id()`).
   - `delete_embeddings_by_model(pool, account_id, old_model_id)` purges old embeddings.
   - All chunks become dirty (no embedding row matches).
   - `EmbeddingWorker` re-embeds everything with the new model.
   - In-memory index is rebuilt from scratch.

## Failure Modes

| Failure | Behavior | Recovery |
|---------|----------|----------|
| Embedding provider unavailable (Ollama not running) | Worker logs warning, skips batch, retries on next tick | Start Ollama; worker auto-recovers |
| Provider rate limited | Worker logs warning with retry-after, skips batch | Automatic retry on next tick |
| Provider returns dimension mismatch | Worker logs error and **halts** (returns from run loop) | Fix config (wrong model selected) and restart |
| SQLite write fails | Worker logs error, skips batch | Check disk space / permissions |
| Startup: provider creation fails | `semantic_index` and `embedding_provider` set to `None` | Fix config and restart; system degrades to keyword+graph retrieval |
| Stale index (embeddings exist but are old) | `get_dirty_chunks_for()` detects hash mismatch, worker re-embeds | Automatic |

All failure modes degrade gracefully to existing keyword and graph retrieval. Semantic search is additive.

## Shutdown Sequence

1. `CancellationToken` is cancelled (shared with Watchtower and other workers).
2. `EmbeddingWorker::run()` detects cancellation in `tokio::select!` and returns.
3. In-memory `SemanticIndex` is dropped (no persistence needed — SQLite has all data).
4. On next startup, index is rebuilt from SQLite.

## Observability

- `get_index_stats_for(pool, account_id)` returns an `IndexStats` struct with:
  - `total_chunks`: active chunks in the vault
  - `embedded_chunks`: chunks with fresh embeddings (hash matches)
  - `dirty_chunks`: chunks needing re-embedding
  - `freshness_pct`: percentage of chunks that are freshly embedded (0-100)
  - `last_indexed_at`: timestamp of most recent embedding update
  - `model_id`: current embedding model in use
- Worker logs `tracing::info!` on each batch with count of chunks embedded.
- Worker logs `tracing::warn!` on provider errors.

## Storage Format

Embeddings are stored as BLOBs of little-endian f32 values:
- `vec_to_bytes(&[f32]) -> Vec<u8>`: serializes for storage
- `bytes_to_vec(&[u8]) -> Vec<f32>`: deserializes for search

The `embedding_hash` field stores a SHA-256 hash of the chunk's text content, enabling dirty detection by comparing against `content_chunks.chunk_hash`.

## Design Decisions

### D1: Brute-force cosine scan instead of HNSW

**Chose**: Linear cosine distance scan in `SemanticIndex`.
**Rejected**: `usearch` (C++ FFI, cross-platform CI risk), `hnsw_rs`.
**Rationale**: For <10K vectors, linear scan is fast enough (<10ms). Avoids C++ FFI dependency that risks Windows CI breakage. The `SemanticIndex` API is stable — an HNSW backend can be swapped in later without changing callers.

### D2: SQLite is source of truth; HNSW rebuilt on startup

No periodic HNSW persistence. SQLite stores all embeddings; the in-memory index is rebuilt from `get_all_embeddings_for()` at startup. Simplifies crash recovery and eliminates file format versioning.

### D3: EmbeddingWorker is independent from WatchtowerLoop

Separate task with its own polling interval (5s vs Watchtower's filesystem watch + remote poll). Shares the same `CancellationToken` for coordinated shutdown. Follows the `SeedWorker` pattern.
