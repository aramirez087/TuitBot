# Search API Contract

## Endpoints

### GET /api/vault/evidence

Unified semantic evidence endpoint for all Ghostwriter surfaces (composer, hook picker, thread editor, selection review).

#### Query Parameters

| Parameter | Type   | Required | Default  | Description |
|-----------|--------|----------|----------|-------------|
| `q`       | string | yes      | —        | Search query text (non-empty) |
| `limit`   | u32    | no       | 8        | Max results (clamped to 1–20) |
| `mode`    | string | no       | "hybrid" | "hybrid", "semantic", or "keyword" |
| `scope`   | string | no       | —        | Optional scope, e.g. `selection:{session_id}` |

#### Response (200)

```json
{
  "results": [
    {
      "chunk_id": 42,
      "node_id": 10,
      "heading_path": "# Guide > ## Setup",
      "snippet": "Install the CLI with cargo install tuitbot...",
      "relative_path": "notes/guide.md",
      "match_reason": "hybrid",
      "score": 0.032,
      "node_title": "Installation Guide"
    }
  ],
  "query": "install tuitbot",
  "mode": "hybrid",
  "index_status": {
    "total_chunks": 1200,
    "embedded_chunks": 1180,
    "freshness_pct": 98.3
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `results[].chunk_id` | i64 | Content chunk ID |
| `results[].node_id` | i64 | Parent content node ID |
| `results[].heading_path` | string | Heading hierarchy (e.g. "# Title > ## Section") |
| `results[].snippet` | string | Truncated excerpt (max 120 chars) |
| `results[].relative_path` | string? | File path (omitted in Cloud mode) |
| `results[].match_reason` | string | "semantic", "keyword", "graph", or "hybrid" |
| `results[].score` | f64 | RRF-fused score (relative, not absolute) |
| `results[].node_title` | string? | Source note title |
| `query` | string | Echo of the query parameter |
| `mode` | string | Effective retrieval mode |
| `index_status.total_chunks` | i64 | Total active chunks for this account |
| `index_status.embedded_chunks` | i64 | Chunks with embeddings |
| `index_status.freshness_pct` | f64 | Percentage of chunks with current embeddings |

#### Error Responses

| Status | Condition | Body |
|--------|-----------|------|
| 400 | `q` is missing or empty | `{"error": "bad_request", "message": "query parameter 'q' is required..."}` |

#### Mode Behavior

| Mode | Semantic Available | Behavior |
|------|-------------------|----------|
| `hybrid` | yes | Blends semantic + keyword via RRF |
| `hybrid` | no | Falls back to keyword-only (silent) |
| `semantic` | yes | Semantic results only |
| `semantic` | no | Returns empty results |
| `keyword` | n/a | Keyword search only, never uses embeddings |

#### Privacy (Deployment Mode)

| Mode | `relative_path` | Behavior |
|------|-----------------|----------|
| Desktop | included | Full path visible |
| Self-Host | included | Full path visible |
| Cloud | omitted (null) | Path stripped from response |

---

### GET /api/vault/index-status

Returns semantic index health and statistics for the requesting account.

#### Response (200)

```json
{
  "total_chunks": 1200,
  "embedded_chunks": 1180,
  "dirty_chunks": 20,
  "freshness_pct": 98.3,
  "last_indexed_at": "2026-03-23T14:30:00Z",
  "model_id": "nomic-embed-text",
  "provider_configured": true,
  "index_loaded": true,
  "index_size": 1180
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_chunks` | i64 | Total active chunks for this account |
| `embedded_chunks` | i64 | Chunks with stored embeddings |
| `dirty_chunks` | i64 | Chunks needing re-embedding (hash mismatch) |
| `freshness_pct` | f64 | `embedded_chunks / total_chunks * 100` |
| `last_indexed_at` | string? | Timestamp of most recent embedding |
| `model_id` | string? | Embedding model identifier |
| `provider_configured` | bool | Whether an embedding provider is configured |
| `index_loaded` | bool | Whether the in-memory index is loaded |
| `index_size` | usize | Number of vectors in the in-memory index |

---

## Backward Compatibility

- `GET /api/vault/search` — unchanged, continues to work as before
- `POST /api/vault/resolve-refs` — unchanged
- `VaultCitation` — new optional fields (`match_reason`, `score`) are `skip_serializing_if = None`, so existing serialization is identical when these fields are unset

## Rate Limiting

Both endpoints inherit the existing API middleware rate limiting. No additional per-endpoint limits.

## Account Scoping

All results are scoped to the requesting account via `AccountContext` middleware. No cross-account data leakage is possible.
