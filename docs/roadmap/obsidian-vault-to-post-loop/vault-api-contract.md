# Vault API Contract

Stable REST API surface for vault search, note preview, fragment retrieval, and selected-reference resolution from the dashboard.

All endpoints are **account-scoped** via the `X-Account-Id` header (defaults to the sentinel account). Responses are **privacy-safe**: no raw note bodies are returned — only titles, relative paths, tags, heading paths, and truncated snippets (≤120 chars).

---

## Endpoints

### `GET /api/vault/sources`

Returns all source contexts for the account with per-source node counts.

**Response:**
```json
{
  "sources": [
    {
      "id": 1,
      "source_type": "local_fs",
      "status": "active",
      "error_message": null,
      "node_count": 42,
      "updated_at": "2026-03-07T10:00:00Z"
    }
  ]
}
```

**Error codes:** 500 (storage error).

---

### `GET /api/vault/notes`

Search or list content nodes. No `body_text` in response.

**Query parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `q` | string | — | LIKE search on title + relative_path |
| `source_id` | i64 | — | Filter by source |
| `limit` | u32 | 20 | Max results (clamped to 100) |

When neither `q` nor `source_id` is provided, returns the most recent nodes.

**Response:**
```json
{
  "notes": [
    {
      "node_id": 1,
      "source_id": 1,
      "title": "Rust Tips",
      "relative_path": "notes/rust-tips.md",
      "tags": "rust,programming",
      "status": "chunked",
      "chunk_count": 5,
      "updated_at": "2026-03-07T10:00:00Z"
    }
  ]
}
```

**Error codes:** 500 (storage error).

---

### `GET /api/vault/notes/{id}`

Note detail with chunk heading summaries. No raw body text — only heading paths and 120-char snippets.

**Response:**
```json
{
  "node_id": 1,
  "source_id": 1,
  "title": "Rust Tips",
  "relative_path": "notes/rust-tips.md",
  "tags": "rust,programming",
  "status": "chunked",
  "ingested_at": "2026-03-06T09:00:00Z",
  "updated_at": "2026-03-07T10:00:00Z",
  "chunks": [
    {
      "chunk_id": 10,
      "heading_path": "# Rust Tips > ## Ownership",
      "snippet": "Ownership makes concurrency safe in Rust...",
      "retrieval_boost": 1.0
    }
  ]
}
```

**Error codes:** 404 (note not found), 500 (storage error).

---

### `GET /api/vault/search`

Full-text fragment search across chunks. Returns `VaultCitation` records (same shape as assist responses).

**Query parameters:**
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `q` | string | **required** | Space-separated keywords |
| `limit` | u32 | 20 | Max results (clamped to 100) |

**Response:**
```json
{
  "fragments": [
    {
      "chunk_id": 10,
      "node_id": 1,
      "heading_path": "# Rust Tips > ## Ownership",
      "source_path": "notes/rust-tips.md",
      "source_title": "Rust Tips",
      "snippet": "Ownership makes concurrency safe...",
      "retrieval_boost": 1.0
    }
  ]
}
```

**Error codes:** 500 (storage error).

---

### `POST /api/vault/resolve-refs`

Resolve selected note IDs to their chunk citations.

**Request body:**
```json
{ "node_ids": [1, 5, 12] }
```

**Response:**
```json
{
  "citations": [
    {
      "chunk_id": 10,
      "node_id": 1,
      "heading_path": "# Rust Tips > ## Ownership",
      "source_path": "notes/rust-tips.md",
      "source_title": "Rust Tips",
      "snippet": "Ownership makes concurrency safe...",
      "retrieval_boost": 1.0
    }
  ]
}
```

Empty `node_ids` returns empty `citations`. Max 100 citations returned.

**Error codes:** 500 (storage error).

---

## Assist Request Extensions

The following request structs now accept an optional `selected_node_ids` field:

- `POST /api/assist/tweet` — `{ topic, selected_node_ids? }`
- `POST /api/assist/thread` — `{ topic, selected_node_ids? }`
- `POST /api/assist/improve` — `{ draft, context?, selected_node_ids? }`

When provided, chunks from the selected notes are retrieved first during RAG context building, with remaining slots filled by keyword search. The field uses `#[serde(default)]` so existing clients are unaffected.

All three response types include `vault_citations: VaultCitation[]` (empty array omitted via `skip_serializing_if`).

---

## Privacy Guarantees

1. **No raw body text** — Note search returns only `node_id`, `title`, `relative_path`, `tags`, `status`, `chunk_count`.
2. **Snippets truncated** — All snippets capped at 120 characters with ellipsis.
3. **Account isolation** — Every query is scoped to the authenticated account via `AccountContext`.
4. **No config_json** — Source responses omit `config_json` (which may contain filesystem paths).

---

## TypeScript Types

Dashboard types defined in `dashboard/src/lib/api/types.ts`:

- `VaultCitation` — chunk citation with heading_path, source_path, snippet
- `ProvenanceRef` — lightweight provenance reference for draft/compose requests
- `VaultSourceStatus` — source status with node_count
- `VaultNoteItem` — note search result
- `VaultChunkSummary` — chunk heading + snippet for note detail
- `VaultNoteDetail` — full note detail with chunks array

Client methods in `dashboard/src/lib/api/client.ts`:

- `api.vault.sources()` — GET /api/vault/sources
- `api.vault.searchNotes({ q?, source_id?, limit? })` — GET /api/vault/notes
- `api.vault.noteDetail(id)` — GET /api/vault/notes/{id}
- `api.vault.searchFragments({ q, limit? })` — GET /api/vault/search
- `api.vault.resolveRefs(nodeIds)` — POST /api/vault/resolve-refs
