# Graph API Contract

## GET /api/vault/notes/{node_id}/neighbors

Returns graph-aware neighbor suggestions for a content node.

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `max` | u32 | 8 | Maximum neighbors to return (capped at 100) |

### Response Shape

```json
{
  "node_id": 42,
  "neighbors": [
    {
      "node_id": 55,
      "node_title": "Async Patterns",
      "reason": "linked_note",
      "reason_label": "linked note",
      "intent": "pro_tip",
      "matched_tags": [],
      "score": 3.5,
      "snippet": "Async patterns in Rust use tokio...",
      "best_chunk_id": 120,
      "heading_path": "# Async > ## Tokio",
      "relative_path": "notes/async-patterns.md"
    }
  ],
  "total_edges": 3,
  "graph_state": "available"
}
```

### GraphState Values

| Value | When | UI Guidance |
|-------|------|-------------|
| `available` | Neighbors found and returned | Show neighbor cards |
| `no_related_notes` | Node exists, no graph edges | Show "no related notes" empty state |
| `node_not_indexed` | Node ID not in content_nodes | Show "note not indexed yet" |
| `fallback_active` | Graph query failed (logged) | Fall back to standard retrieval, show nothing |
| `unresolved_links` | Reserved for future use | Edges exist but targets not resolved |

### Reason Values

| Value | Human Label | Meaning |
|-------|-------------|---------|
| `linked_note` | "linked note" | Selected note links to this note |
| `backlink` | "backlink" | This note links back to selected note |
| `mutual_link` | "mutual link" | Bidirectional link |
| `shared_tag` | "shared tag: #rust, #async" | Notes share tags (tags listed in label) |

### Intent Values

| Value | Meaning |
|-------|---------|
| `pro_tip` | Related background knowledge or how-to |
| `counterpoint` | Contrasting or alternative viewpoint |
| `evidence` | Supporting data or research |
| `related` | General related context (default) |

### Privacy Envelope

- **Desktop/Self-host mode**: `relative_path` included in response
- **Cloud mode**: `relative_path` omitted (set to null, skipped in serialization)
- **All modes**: No raw note body text exposed; snippets capped at 120 characters

### Error Handling

All states return `200 OK` with a `graph_state` field. No 404/500 for graph issues — the endpoint is supplementary.

## GET /api/vault/selection/{session_id} — Additive Fields

When `resolved_node_id` is present in a selection, the response auto-expands graph neighbors:

```json
{
  "session_id": "abc-123",
  "vault_name": "marketing",
  "file_path": "notes/test.md",
  "selected_text": "...",
  "resolved_node_id": 42,
  "resolved_chunk_id": 100,
  "privacy_envelope": "local_first",
  "graph_neighbors": [
    {
      "node_id": 55,
      "reason": "linked_note",
      "reason_label": "linked note",
      "intent": "related",
      "matched_tags": [],
      "score": 3.5,
      "snippet": "...",
      "best_chunk_id": 120,
      "heading_path": null,
      "relative_path": "notes/async.md"
    }
  ],
  "graph_state": "available"
}
```

Both `graph_neighbors` and `graph_state` are optional fields (`skip_serializing_if`). They are omitted when:
- `resolved_node_id` is null (no node to expand from)
- This maintains backward compatibility with existing clients

## Provenance Extension Fields

`vault_provenance_links` table has two additive nullable columns:

| Column | Type | Description |
|--------|------|-------------|
| `edge_type` | TEXT (nullable) | Graph edge type: "wikilink", "backlink", "shared_tag" |
| `edge_label` | TEXT (nullable) | Edge label for display (tag name, link text) |

These flow through `ProvenanceRef` → `ProvenanceLink` → API responses. Existing rows have NULL for both fields (backward compatible).

## Example Request/Response Pairs

### Available Graph

```
GET /api/vault/notes/42/neighbors?max=3
```

```json
{
  "node_id": 42,
  "neighbors": [
    {"node_id": 55, "reason": "mutual_link", "reason_label": "mutual link", "intent": "related", "score": 5.5, ...},
    {"node_id": 78, "reason": "linked_note", "reason_label": "linked note", "intent": "pro_tip", "score": 3.5, ...},
    {"node_id": 91, "reason": "shared_tag", "reason_label": "shared tag: #rust", "intent": "related", "score": 1.5, ...}
  ],
  "total_edges": 6,
  "graph_state": "available"
}
```

### No Related Notes

```
GET /api/vault/notes/42/neighbors
```

```json
{
  "node_id": 42,
  "neighbors": [],
  "total_edges": 0,
  "graph_state": "no_related_notes"
}
```

### Node Not Indexed

```
GET /api/vault/notes/99999/neighbors
```

```json
{
  "node_id": 99999,
  "neighbors": [],
  "total_edges": 0,
  "graph_state": "node_not_indexed"
}
```
