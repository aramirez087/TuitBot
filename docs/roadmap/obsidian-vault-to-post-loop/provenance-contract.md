# Provenance Contract

## Overview

Every generated draft, approval queue item, or posted tweet can be traced back to the vault notes and chunks that influenced its creation. Provenance is optional — manual content and legacy flows work unchanged.

## Data Model

### `vault_provenance_links` Table

Polymorphic link table mapping content entities to vault source material.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment row ID |
| `account_id` | TEXT | Account isolation key |
| `entity_type` | TEXT | `'approval_queue'`, `'scheduled_content'`, `'original_tweet'`, `'thread'` |
| `entity_id` | INTEGER | ID in the referenced table |
| `node_id` | INTEGER? | FK → `content_nodes.id` (nullable) |
| `chunk_id` | INTEGER? | FK → `content_chunks.id` (nullable) |
| `seed_id` | INTEGER? | FK → `draft_seeds.id` (nullable) |
| `source_path` | TEXT? | Snapshot of `relative_path` at creation |
| `heading_path` | TEXT? | Snapshot of heading hierarchy |
| `snippet` | TEXT? | Snapshot of chunk excerpt |
| `created_at` | TEXT | ISO-8601 timestamp |

**Indexes:**
- `idx_provenance_entity(account_id, entity_type, entity_id)` — lookup by entity
- `idx_provenance_node(account_id, node_id)` — lookup by source note

### Inline Columns (Legacy)

The `approval_queue` table retains inline provenance columns for backward compatibility:
- `source_node_id` — first node ID from provenance refs
- `source_seed_id` — first seed ID from provenance refs
- `source_chunks_json` — JSON array of chunk references

These are populated alongside the link table when provenance is provided.

## API Payload

### `ProvenanceRef` (JSON)

```json
{
  "node_id": 42,
  "chunk_id": 100,
  "seed_id": null,
  "source_path": "notes/async-patterns.md",
  "heading_path": "# Async > ## Patterns",
  "snippet": "When using async/await in Rust..."
}
```

All fields are optional. The `source_path`, `heading_path`, and `snippet` are snapshot values that survive source deletion.

### Endpoints Accepting Provenance

| Endpoint | Field | Description |
|----------|-------|-------------|
| `POST /api/content/drafts` | `provenance: ProvenanceRef[]?` | Stored in link table for `scheduled_content` |
| `POST /api/content/compose` | `provenance: ProvenanceRef[]?` | Stored in link table for `approval_queue` or `scheduled_content` |
| `POST /api/content/tweets` | `provenance: ProvenanceRef[]?` | Stored in link table for `approval_queue` |

### Endpoints Returning Citations

| Endpoint | Field | Description |
|----------|-------|-------------|
| `POST /api/assist/tweet` | `vault_citations: VaultCitation[]` | Citations from RAG context |
| `POST /api/assist/thread` | `vault_citations: VaultCitation[]` | Citations from RAG context |
| `POST /api/assist/improve` | `vault_citations: VaultCitation[]` | Citations from RAG context |

Citations are omitted from the response when no vault context was used (`skip_serializing_if = "Vec::is_empty"`).

## Lifecycle

### Draft Creation
1. Frontend calls `/api/assist/tweet` → receives `vault_citations`
2. Frontend calls `POST /api/content/drafts` with `provenance` derived from citations
3. Server inserts `scheduled_content` row + `vault_provenance_links` rows

### Draft Publish
1. Frontend calls `POST /api/content/drafts/{id}/publish`
2. Server loads provenance links from `scheduled_content` entity
3. Server creates `approval_queue` entry with provenance (both inline columns and link table)
4. Item is auto-approved for posting

### Approval Poster
1. Approval poster picks up approved item
2. Posts to X API
3. If item has `source_node_id`, creates `original_tweets` record with `source_node_id`
4. Copies provenance links from `approval_queue` to `original_tweet`

### Compose Flow
1. Frontend calls `POST /api/content/compose` with `provenance`
2. In approval mode: provenance stored on `approval_queue` entry
3. In schedule mode: provenance stored on `scheduled_content` entry (future work)

## Compatibility

- All provenance fields are optional with `#[serde(default)]`
- Legacy callers that omit `provenance` work exactly as before
- Manual content creation continues to produce NULL provenance
- Existing `enqueue_for()` and `insert_draft_for()` still work unchanged

## Storage Functions

| Function | Module | Description |
|----------|--------|-------------|
| `insert_links_for()` | `storage::provenance` | Batch insert provenance link rows |
| `get_links_for()` | `storage::provenance` | Query links by entity |
| `copy_links_for()` | `storage::provenance` | Copy links between entities |
| `delete_links_for()` | `storage::provenance` | Remove links for an entity |
| `enqueue_with_provenance_for()` | `storage::approval_queue` | Enqueue with inline + link provenance |
| `insert_draft_with_provenance_for()` | `storage::scheduled_content` | Draft with link provenance |
| `insert_original_tweet_with_provenance_for()` | `storage::threads` | Tweet with link provenance |
| `set_original_tweet_source_node_for()` | `storage::threads` | Set source_node_id on existing tweet |
| `citations_to_provenance_refs()` | `context::retrieval` | Convert VaultCitation → ProvenanceRef |
| `citations_to_chunks_json()` | `context::retrieval` | Convert VaultCitation → legacy JSON |
