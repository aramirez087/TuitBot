# Vault Domain Foundation — Data Model

## Tables

### content_chunks (NEW)

Heading-delimited fragments of content nodes. Created by the chunker
(future session), consumed by RAG retrieval and loop-back.

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| `id` | INTEGER PK | AUTOINCREMENT | Primary key |
| `account_id` | TEXT NOT NULL | `00000000-...` | Owner account; enforces isolation |
| `node_id` | INTEGER NOT NULL FK | — | References `content_nodes(id)`, CASCADE on delete |
| `heading_path` | TEXT NOT NULL | `''` | Slash-delimited heading hierarchy (e.g. `## Intro/### Background`) |
| `chunk_text` | TEXT NOT NULL | — | Section body text |
| `chunk_hash` | TEXT NOT NULL | — | SHA-256 of chunk_text; dedup key on re-chunk |
| `chunk_index` | INTEGER NOT NULL | `0` | Ordering within the note (0-based) |
| `retrieval_boost` | REAL NOT NULL | `1.0` | Multiplicative relevance factor. Range [0.1, 5.0] enforced in app code. |
| `status` | TEXT NOT NULL | `'active'` | `active` or `stale` |
| `created_at` | TEXT NOT NULL | `datetime('now')` | Row creation timestamp |
| `updated_at` | TEXT NOT NULL | `datetime('now')` | Last modification timestamp |

**Indexes:**
- `idx_content_chunks_node` — `(account_id, node_id)` — list chunks for a node
- `idx_content_chunks_status` — `(account_id, status)` — filter active/stale
- `idx_content_chunks_hash` — `(node_id, chunk_hash)` — dedup during re-chunking

### draft_seeds (MODIFIED)

| New Column | Type | Default | Description |
|------------|------|---------|-------------|
| `chunk_id` | INTEGER FK (nullable) | `NULL` | References `content_chunks(id)`. Set when a seed is extracted from a specific fragment. |

### approval_queue (MODIFIED)

| New Column | Type | Default | Description |
|------------|------|---------|-------------|
| `source_node_id` | INTEGER FK (nullable) | `NULL` | References `content_nodes(id)`. Primary note that influenced the draft. |
| `source_seed_id` | INTEGER FK (nullable) | `NULL` | References `draft_seeds(id)`. Specific seed used (if applicable). |
| `source_chunks_json` | TEXT | `'[]'` | JSON array of chunk references used during generation. |

## Status Lifecycles

### Content Node Status

```
pending → chunked → processed
```

- `pending` — ingested, awaiting chunking
- `chunked` — fragments created, awaiting seed extraction
- `processed` — seeds extracted, fully processed

### Chunk Status

```
active → stale
```

- `active` — current fragment, available for retrieval
- `stale` — parent note was re-chunked; old fragment preserved for FK integrity

## Account Isolation Rules

Every storage query filters by `account_id`:

- Source contexts: `get_source_contexts_for(pool, account_id)`
- Content nodes: `get_pending_content_nodes_for(pool, account_id, limit)`
- Content chunks: all chunk functions require `account_id` as a parameter
- Draft seeds: `get_pending_seeds_for(pool, account_id, limit)`
- Keyword search: `search_chunks_by_keywords(pool, account_id, keywords, limit)`

The `DEFAULT_ACCOUNT_ID` (`00000000-0000-0000-0000-000000000000`) is used by
legacy wrapper functions that maintain backward compatibility.

## Backward Compatibility

### Wrapper Pattern

Every existing function retains its original signature. New `_for` variants
accept `account_id` as a parameter:

```
insert_source_context(pool, source_type, config_json)          // legacy
insert_source_context_for(pool, account_id, source_type, ...)  // account-scoped
```

Legacy functions delegate to `_for` variants with `DEFAULT_ACCOUNT_ID`.

### New Columns Are Nullable

- `draft_seeds.chunk_id` — `NULL` for all existing seeds
- `approval_queue.source_node_id` — `NULL` for all existing items
- `approval_queue.source_seed_id` — `NULL` for all existing items
- `approval_queue.source_chunks_json` — defaults to `'[]'` (empty array)

No backfill is required. Existing rows are unaffected.

### ON DELETE CASCADE

`content_chunks.node_id` has `ON DELETE CASCADE`. When a content node is
deleted, all its chunks are automatically removed. This only triggers via
manual DB intervention or factory reset — there is no application-level
content node deletion path.

## Module Structure

The watchtower storage module was split to respect the 500-line file limit:

```
storage/watchtower/
├── mod.rs          — Re-exports, struct definitions, row types (~230 lines)
├── sources.rs      — Source context CRUD + _for variants (~240 lines)
├── nodes.rs        — Content node CRUD + _for variants (~200 lines)
├── chunks.rs       — Content chunk CRUD (all account-scoped) (~210 lines)
├── seeds.rs        — Draft seed CRUD + _for variants (~170 lines)
├── connections.rs  — Connection CRUD (unchanged)
└── tests.rs        — Comprehensive tests including new functionality
```

All public types and functions are re-exported from `mod.rs`, so external
callers (`use crate::storage::watchtower::*`) are unaffected.

## Migration

File: `20260308010000_vault_domain_foundation.sql` (mirrored in both
`migrations/` and `crates/tuitbot-core/migrations/`).

Operations:
1. `CREATE TABLE content_chunks` with indexes
2. `ALTER TABLE draft_seeds ADD COLUMN chunk_id`
3. `ALTER TABLE approval_queue ADD COLUMN source_node_id`
4. `ALTER TABLE approval_queue ADD COLUMN source_seed_id`
5. `ALTER TABLE approval_queue ADD COLUMN source_chunks_json`

All additive. No destructive changes. Safe for existing installs.
