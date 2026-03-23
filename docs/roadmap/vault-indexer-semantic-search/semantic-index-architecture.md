# Semantic Index Architecture

## Overview

This document specifies the full technical architecture for the background semantic vault indexer: embedding provider abstraction, storage schema, HNSW index lifecycle, dirty-state tracking, freshness model, deployment matrix, and migration plan.

---

## 1. Embedding Provider Abstraction

### Trait Design

A new `EmbeddingProvider` trait lives alongside the existing `LlmProvider` trait in `crates/tuitbot-core/src/llm/`. Embeddings have a different API shape (batch input → batch output, no streaming) so a separate trait is cleaner than overloading `LlmProvider`.

```rust
// crates/tuitbot-core/src/llm/embedding.rs

/// A batch of text inputs to embed.
pub type EmbeddingInput = Vec<String>;

/// A single embedding vector.
pub type EmbeddingVector = Vec<f32>;

/// Result of a batch embedding request.
#[derive(Debug)]
pub struct EmbeddingResponse {
    /// One vector per input, in the same order as the input batch.
    pub embeddings: Vec<EmbeddingVector>,
    /// The model that produced these embeddings.
    pub model: String,
    /// Dimensionality of each embedding vector.
    pub dimension: usize,
    /// Token usage for billing/metering.
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Default)]
pub struct EmbeddingUsage {
    pub total_tokens: u64,
}

/// Object-safe trait for embedding providers.
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Provider display name (e.g., "openai", "ollama").
    fn name(&self) -> &str;

    /// Dimensionality of the vectors this provider produces.
    fn dimension(&self) -> usize;

    /// Model identifier (e.g., "text-embedding-3-small").
    fn model_id(&self) -> &str;

    /// Embed a batch of text inputs. Returns one vector per input.
    async fn embed(&self, inputs: EmbeddingInput) -> Result<EmbeddingResponse, EmbeddingError>;

    /// Health check — verifies the provider is reachable and configured.
    async fn health_check(&self) -> Result<(), EmbeddingError>;
}
```

### Error Type

```rust
// In the same file

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("provider not configured: {0}")]
    NotConfigured(String),

    #[error("API error: {status} — {message}")]
    Api { status: u16, message: String },

    #[error("network error: {0}")]
    Network(String),

    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("batch too large: {size} inputs, max {max}")]
    BatchTooLarge { size: usize, max: usize },

    #[error("rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
}
```

### Factory

```rust
// crates/tuitbot-core/src/llm/embedding_factory.rs

pub fn create_embedding_provider(
    config: &EmbeddingConfig,
) -> Result<Box<dyn EmbeddingProvider>, EmbeddingError> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(OpenAiEmbeddingProvider::new(config)?)),
        "ollama" => Ok(Box::new(OllamaEmbeddingProvider::new(config)?)),
        _ => Err(EmbeddingError::NotConfigured(
            format!("unknown embedding provider: {}", config.provider),
        )),
    }
}
```

### Concrete Implementations

**OpenAI (`text-embedding-3-small`):**
- Endpoint: `POST /v1/embeddings`
- Model: `text-embedding-3-small` (1536 dimensions)
- Max batch size: 2048 inputs per request
- Rate limiting: respect `Retry-After` header, exponential backoff

**Ollama (`nomic-embed-text`):**
- Endpoint: `POST /api/embed` (Ollama's embedding endpoint)
- Model: `nomic-embed-text` (768 dimensions)
- No rate limiting (local)
- Max batch size: 100 inputs per request (Ollama memory constraint)

---

## 2. Configuration

### EmbeddingConfig

Added to `crates/tuitbot-core/src/config/types.rs` alongside existing `LlmConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider: "openai" or "ollama".
    #[serde(default = "default_embedding_provider")]
    pub provider: String,

    /// API key (required for OpenAI, ignored for Ollama).
    #[serde(default)]
    pub api_key: Option<String>,

    /// Model identifier. Defaults: openai→"text-embedding-3-small", ollama→"nomic-embed-text".
    #[serde(default)]
    pub model: Option<String>,

    /// Base URL override. Defaults: openai→"https://api.openai.com/v1", ollama→"http://localhost:11434".
    #[serde(default)]
    pub base_url: Option<String>,

    /// Maximum batch size per embedding request. Default: 100.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Whether background indexing is enabled. Default: true.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_embedding_provider() -> String { "ollama".to_string() }
fn default_batch_size() -> usize { 100 }
fn default_true() -> bool { true }
```

### config.toml Section

```toml
[embedding]
provider = "ollama"                    # "openai" or "ollama"
# api_key = "sk-..."                  # Required for OpenAI
# model = "text-embedding-3-small"    # Override default model
# base_url = "http://localhost:11434" # Override default base URL
batch_size = 100                       # Chunks per embedding request
enabled = true                         # Enable/disable background indexing
```

---

## 3. Storage Schema

### chunk_embeddings Table

```sql
CREATE TABLE IF NOT EXISTS chunk_embeddings (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id        INTEGER NOT NULL REFERENCES content_chunks(id) ON DELETE CASCADE,
    account_id      TEXT    NOT NULL DEFAULT 'default',
    embedding       BLOB    NOT NULL,       -- f32 vector as little-endian bytes
    model_id        TEXT    NOT NULL,        -- e.g., "text-embedding-3-small"
    dimension       INTEGER NOT NULL,        -- e.g., 1536 or 768
    embedding_hash  TEXT    NOT NULL,        -- SHA256 of chunk_text at embed time
    generation      INTEGER NOT NULL DEFAULT 1, -- monotonic counter for freshness
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    UNIQUE(chunk_id, account_id)
);

CREATE INDEX idx_chunk_embeddings_account ON chunk_embeddings(account_id);
CREATE INDEX idx_chunk_embeddings_generation ON chunk_embeddings(account_id, generation);
CREATE INDEX idx_chunk_embeddings_model ON chunk_embeddings(model_id);
```

### Schema Conventions

- **`embedding` BLOB format:** Raw `Vec<f32>` serialized as little-endian bytes. For 1536-dim: 1536 × 4 bytes = 6,144 bytes per row. For 768-dim: 3,072 bytes per row.
- **`embedding_hash`:** SHA256 of the `chunk_text` at the time the embedding was computed. Used for dirty detection: if `content_chunks.chunk_hash != chunk_embeddings.embedding_hash`, the embedding is stale.
- **`generation`:** Monotonic counter incremented each time the embedding is recomputed. Enables "index is N% fresh" queries without scanning all rows.
- **`model_id`:** Tracks which model produced the embedding. When the user switches models, all embeddings with the old `model_id` are marked for re-embedding.
- **Cascade delete:** When a `content_chunk` is deleted, its embedding is automatically removed.
- **Account scoping:** All queries include `account_id` to prevent cross-account leakage.

### Storage Functions

```rust
// crates/tuitbot-core/src/storage/watchtower/embeddings.rs

/// Insert or update an embedding for a chunk.
pub async fn upsert_chunk_embedding(
    pool: &DbPool,
    chunk_id: i64,
    account_id: &str,
    embedding: &[f32],
    model_id: &str,
    dimension: usize,
    embedding_hash: &str,
    generation: i64,
) -> Result<(), StorageError>;

/// Get all embeddings for an account (used to build the HNSW index at startup).
pub async fn get_all_embeddings_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<ChunkEmbeddingRow>, StorageError>;

/// Get chunks that need (re-)embedding: either no embedding exists, or
/// the chunk_hash has changed since the embedding was computed.
pub async fn get_dirty_chunks_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<DirtyChunk>, StorageError>;

/// Get index freshness statistics.
pub async fn get_index_stats_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<IndexStats, StorageError>;

/// Delete embeddings for a specific model (used when switching models).
pub async fn delete_embeddings_by_model(
    pool: &DbPool,
    account_id: &str,
    model_id: &str,
) -> Result<u64, StorageError>;
```

---

## 4. HNSW Index Lifecycle (usearch)

### Why usearch

- **Rust-native:** Single crate, no C++ dependency compilation issues
- **Mutable index:** Supports incremental insert and delete without full rebuild
- **In-process:** No server process to manage — critical for Desktop/Self-host
- **Performance:** < 10ms search latency for 100K vectors at 1536 dimensions
- **Memory:** ~100MB for 50K vectors at 1536d (6KB per vector + HNSW overhead)

### Index Manager

```rust
// crates/tuitbot-core/src/context/semantic_index.rs

pub struct SemanticIndex {
    /// The in-memory HNSW index.
    index: usearch::Index,
    /// Mapping from usearch internal keys to chunk_ids.
    key_to_chunk_id: HashMap<u64, i64>,
    /// Reverse mapping for deletion.
    chunk_id_to_key: HashMap<i64, u64>,
    /// Next available key for insertion.
    next_key: AtomicU64,
    /// Dimension of vectors in this index.
    dimension: usize,
    /// Model ID this index was built for.
    model_id: String,
}
```

### Lifecycle

1. **Startup:** Load all embeddings from `chunk_embeddings` via `get_all_embeddings_for()`. Build HNSW index from persisted vectors. This is a cold start — ~2 seconds for 50K vectors.
2. **Incremental insert:** When the background indexer embeds a new chunk, insert the vector into the live HNSW index and persist to SQLite. No full rebuild needed.
3. **Incremental delete:** When a chunk is deleted or goes stale, remove from the HNSW index by key and delete the SQLite row.
4. **Model switch:** When the user changes the embedding model in config, delete all embeddings for the old model, rebuild the index from empty, and re-embed all chunks with the new model.
5. **No periodic persist:** The HNSW index is ephemeral (rebuilt from SQLite at startup). SQLite is the source of truth. This simplifies crash recovery — if the process dies, the index is rebuilt cleanly on restart.

### Search Interface

```rust
impl SemanticIndex {
    /// Search for the K nearest neighbors of a query vector.
    /// Returns (chunk_id, distance) pairs sorted by ascending distance.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(i64, f32)>, SemanticSearchError>;

    /// Number of vectors in the index.
    pub fn len(&self) -> usize;

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool;
}
```

### Capacity Management

- **Cap at 50,000 vectors:** For vaults larger than 50K chunks, only the most recently updated chunks are indexed. Older chunks fall back to keyword-only retrieval.
- **Memory budget:** ~100MB for 50K vectors at 1536d. Acceptable for Desktop (typical 8-16GB RAM).
- **Overflow strategy:** When at capacity, evict the oldest-generation vectors (lowest `generation` value) to make room for new inserts.

---

## 5. Dirty-State Tracking

### How It Works

The Watchtower ingest pipeline already computes content hashes for each note (`content_hash` in `content_nodes`). The chunker computes `chunk_hash` for each chunk.

When `ingest_content()` re-chunks a note:
1. `upsert_chunks_for_node()` inserts or updates chunks, setting `chunk_hash`
2. Any chunk whose `chunk_hash` differs from the corresponding `embedding_hash` in `chunk_embeddings` is "dirty"
3. Any chunk with no row in `chunk_embeddings` is "unembedded"
4. Both categories are returned by `get_dirty_chunks_for()`

### Background Indexer Worker

```rust
// crates/tuitbot-core/src/automation/watchtower/embedding_worker.rs

pub struct EmbeddingWorker {
    pool: DbPool,
    provider: Box<dyn EmbeddingProvider>,
    index: Arc<RwLock<SemanticIndex>>,
    batch_size: usize,
    poll_interval: Duration,  // 5 seconds
}

impl EmbeddingWorker {
    pub async fn run(&self, cancel: CancellationToken) {
        loop {
            tokio::select! {
                () = cancel.cancelled() => break,
                _ = tokio::time::sleep(self.poll_interval) => {
                    self.process_dirty_batch().await;
                }
            }
        }
    }

    async fn process_dirty_batch(&self) {
        // 1. Fetch dirty chunks (limit: batch_size)
        // 2. Extract chunk_text for each
        // 3. Call provider.embed(texts)
        // 4. For each (chunk, embedding):
        //    a. Compute embedding_hash = SHA256(chunk_text)
        //    b. Upsert into chunk_embeddings
        //    c. Insert into live HNSW index
        // 5. Log batch stats (embedded count, error count, duration)
    }
}
```

### Error Handling

- **Provider unavailable:** Log warning, skip batch, retry on next poll interval. Do not block Watchtower ingest.
- **Partial batch failure:** Some inputs may fail (e.g., too long). Embed what we can, log failures, retry failed inputs on next batch.
- **Rate limiting:** Respect `RateLimited` error, back off for the specified duration.
- **Dimension mismatch:** If the provider returns vectors with unexpected dimensions, reject the batch, log an error, and halt indexing until the config is fixed.

---

## 6. Freshness Model

### Metrics

| Metric | Source | Computation |
|--------|--------|-------------|
| `total_chunks` | `content_chunks` | `COUNT(*) WHERE status='active' AND account_id=?` |
| `embedded_chunks` | `chunk_embeddings` | `COUNT(*) WHERE account_id=?` |
| `dirty_chunks` | Join | `COUNT(*) WHERE cc.chunk_hash != ce.embedding_hash OR ce.id IS NULL` |
| `freshness_pct` | Derived | `(embedded_chunks - dirty_chunks) / total_chunks × 100` |
| `latest_generation` | `chunk_embeddings` | `MAX(generation) WHERE account_id=?` |
| `last_indexed_at` | `chunk_embeddings` | `MAX(updated_at) WHERE account_id=?` |

### Freshness Thresholds

| State | Condition | UX Indicator |
|-------|-----------|--------------|
| **Fresh** | `freshness_pct >= 95` | Green dot, "Index up to date" |
| **Indexing** | `0 < dirty_chunks AND freshness_pct >= 50` | Animated progress bar, "Indexing N chunks..." |
| **Stale** | `freshness_pct < 50` | Yellow warning, "Index is stale — N chunks need re-indexing" |
| **Empty** | `embedded_chunks == 0` | Gray badge, "No index — semantic search unavailable" |
| **Error** | Provider health check fails | Red badge, "Embedding provider unavailable" |

### API Endpoint

```
GET /api/vault/index-status
```

Response:
```json
{
  "total_chunks": 1234,
  "embedded_chunks": 1200,
  "dirty_chunks": 34,
  "freshness_pct": 97.2,
  "last_indexed_at": "2026-03-22T10:15:00Z",
  "model_id": "text-embedding-3-small",
  "dimension": 1536,
  "provider_status": "healthy"
}
```

---

## 7. Deployment Matrix

| Aspect | Desktop | Self-Host | Cloud |
|--------|---------|-----------|-------|
| **Embedding provider** | Ollama (default) or OpenAI | Configurable in config.toml | Server-side provider |
| **Index location** | In-process (same process as tuitbot-server) | In-process | In-process (future: pgvector) |
| **Privacy envelope** | local_first — full chunk text available | user_controlled — full chunk text available | provider_controlled — snippet only |
| **Embedding storage** | SQLite BLOB in local DB | SQLite BLOB in local DB | SQLite BLOB (future: Postgres) |
| **API responses** | Full snippet (120 chars) + heading_path + match_reason | Full snippet (120 chars) + heading_path + match_reason | Snippet (120 chars) + heading_path + match_reason. No raw embedding vectors. |
| **Config location** | `config.toml` [embedding] section | `config.toml` [embedding] section | Server environment variables |
| **Default model** | nomic-embed-text (768d) | nomic-embed-text (768d) | text-embedding-3-small (1536d) |
| **Cost** | Free (Ollama is local) | Free or API cost | API cost per token |

### Cloud Mode Constraints

- Raw embedding vectors are never returned in API responses (defense in depth)
- `chunk_text` is never included in search responses — only `snippet` (120 chars)
- Embedding computation uses the server-side provider; user's browser never sees vectors
- Account scoping enforced on all embedding queries

---

## 8. Migration Plan

### Migration: Add chunk_embeddings Table

**Type:** Additive — new table, no changes to existing tables.

**File:** `crates/tuitbot-core/src/storage/migrations/NNNN_add_chunk_embeddings.sql`

```sql
-- Add chunk_embeddings table for semantic vector storage.
-- This is a pure addition — no existing tables are modified.

CREATE TABLE IF NOT EXISTS chunk_embeddings (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id        INTEGER NOT NULL REFERENCES content_chunks(id) ON DELETE CASCADE,
    account_id      TEXT    NOT NULL DEFAULT 'default',
    embedding       BLOB    NOT NULL,
    model_id        TEXT    NOT NULL,
    dimension       INTEGER NOT NULL,
    embedding_hash  TEXT    NOT NULL,
    generation      INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),

    UNIQUE(chunk_id, account_id)
);

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_account
    ON chunk_embeddings(account_id);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_generation
    ON chunk_embeddings(account_id, generation);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_model
    ON chunk_embeddings(model_id);
```

### Migration Safety

- No data loss risk — purely additive
- No existing queries affected — no columns added to existing tables
- No downtime required — SQLite supports concurrent reads during migration
- Rollback: `DROP TABLE IF EXISTS chunk_embeddings;`

### EmbeddingConfig in AppConfig

Add `embedding: Option<EmbeddingConfig>` to the top-level `AppConfig` struct. When `None`, semantic indexing is disabled and the evidence rail shows keyword+graph results only.

---

## 9. Dependency Additions

| Crate | Version | Purpose | Size Impact |
|-------|---------|---------|-------------|
| `usearch` | `^2.0` | In-process HNSW index | ~500KB binary size |
| `async-trait` | Already in workspace | For `EmbeddingProvider` trait | None |
| `sha2` | Already in workspace | For embedding_hash computation | None |

No new binary dependencies. No new system libraries. `usearch` is a pure Rust crate with no C/C++ FFI.

---

## 10. Performance Budget

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Single vector search (50K index) | < 10ms | `usearch` benchmark at 1536d |
| Batch embed (100 chunks, Ollama) | < 5s | nomic-embed-text local benchmark |
| Batch embed (100 chunks, OpenAI) | < 2s | API latency + network |
| Index startup (50K vectors from SQLite) | < 3s | Sequential BLOB read + HNSW build |
| Dirty chunk detection query | < 50ms | Single JOIN query |
| Index status query | < 10ms | Aggregate query |
| Embedding storage (single upsert) | < 5ms | SQLite BLOB write |

### Memory Budget

| Component | 50K vectors (1536d) | 50K vectors (768d) |
|-----------|--------------------|--------------------|
| HNSW index (usearch) | ~120MB | ~60MB |
| SQLite BLOB cache | OS-managed | OS-managed |
| Key mappings (HashMaps) | ~4MB | ~4MB |
| **Total process overhead** | **~124MB** | **~64MB** |
