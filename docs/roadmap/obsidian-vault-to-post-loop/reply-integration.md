# Reply Surface Vault Integration

Documents how vault context flows through every reply surface in the system.

## Reply Surface Behavior Matrix

| Surface | Vault Context | Product Mention | Provenance Stored | Notes |
|---------|--------------|-----------------|-------------------|-------|
| `POST /api/assist/reply` | Yes, when keywords configured | User-controlled | No (stateless) | Uses `resolve_composer_rag_context` with optional `selected_node_ids` |
| `POST /api/assist/tweet` | Yes | N/A | No (stateless) | Same RAG helper |
| `POST /api/assist/thread` | Yes | N/A | No (stateless) | Same RAG helper |
| `POST /api/assist/improve` | Yes | N/A | No (stateless) | Same RAG helper |
| `POST /api/discovery/{id}/compose-reply` | Yes, when keywords configured | User-controlled | No (stateless) | Uses shared `resolve_composer_rag_context` |
| `POST /api/discovery/{id}/queue-reply` | N/A (user provides text) | N/A | Yes, when `provenance` field present | Uses `enqueue_with_provenance_for` |
| Workflow batch draft (`draft::execute`) | Yes, automatic from keywords | Config-controlled | Carried in `DraftResult::Success.vault_citations` | Citations flow to approval queue via orchestrate step |
| Discovery loop | Yes, via `VaultAwareLlmReplyAdapter` | Always `true` | Future (when approval mode wired) | RAG prompt built once at adapter construction |
| Target loop | Yes, via `VaultAwareLlmReplyAdapter` | Always `false` (genuine) | Future (when approval mode wired) | Vault grounds domain knowledge without product push |
| Mentions loop | Yes, via `VaultAwareLlmReplyAdapter` | Always `true` (direct ask) | Future (when approval mode wired) | Same adapter pattern |

## Architecture

### Shared RAG Helper

`routes/rag_helpers.rs` contains `resolve_composer_rag_context()`, used by both `assist.rs` and `discovery.rs`. It:

1. Loads the account's effective config
2. Extracts `draft_context_keywords()` from the business profile
3. Calls `build_draft_context_with_selection()` with optional node ID bias
4. Returns `Option<DraftContext>` (fail-open on any error)

### ReplyGenerator Trait Extension

The `ReplyGenerator` trait now has two methods:

- `generate_reply()` — original signature, returns `String`
- `generate_reply_with_rag()` — new default method, returns `ReplyOutput { text, vault_citations }`

The default implementation of `generate_reply_with_rag` delegates to `generate_reply` and returns empty citations. This ensures backward compatibility with all existing mock implementations.

### VaultAwareLlmReplyAdapter

A new adapter in `automation/adapters/llm.rs` that:

- Accepts a pre-built `rag_prompt: Option<String>` and `vault_citations: Vec<VaultCitation>` at construction
- Injects the RAG prompt into every reply via `generate_reply_with_context()`
- Returns the cached citations from `generate_reply_with_rag()`
- Avoids per-tweet DB queries — the RAG context is built once by the server/CLI wiring layer

### Provenance Flow

```
Generation → ReplyOutput.vault_citations
                    ↓
            DraftResult::Success.vault_citations
                    ↓
            Approval queue (via enqueue_with_provenance_for)
                    ↓
            vault_provenance_links table
```

For manual flows (compose-reply → queue-reply), the frontend converts `VaultCitation` to `ProvenanceRef` and passes it in the `provenance` field of the queue request.

## Intentional Differences

### Target Loop: No Product Mention + Yes Vault Context

Target loop engagement is genuine relationship-building. Product mentions are always `false`. However, vault context still applies — it grounds replies in domain knowledge (e.g., technical expertise from notes) without inserting product pitches.

### Automation vs Manual: RAG Timing

- **Manual routes**: RAG context is built per-request using `resolve_composer_rag_context()`. Users can select specific notes via `selected_node_ids`.
- **Automation loops**: RAG context is built once at adapter construction time. All replies in a loop iteration share the same context. This avoids per-tweet DB queries in hot paths.

### Stateless Assist Endpoints

`/api/assist/*` endpoints do not store provenance because they return content without posting. The frontend stores citations locally and includes them when the user chooses to queue/post.

## API Changes

### `POST /api/assist/reply`

**Request** — new optional field:
```json
{
  "tweet_text": "...",
  "tweet_author": "...",
  "mention_product": false,
  "selected_node_ids": [1, 2, 3]
}
```

**Response** — new field:
```json
{
  "content": "...",
  "vault_citations": [...]
}
```

### `POST /api/discovery/{id}/compose-reply`

**Request** — new optional field:
```json
{
  "mention_product": true,
  "selected_node_ids": [1, 2]
}
```

**Response** — new field:
```json
{
  "content": "...",
  "tweet_id": "...",
  "vault_citations": [...]
}
```

### `POST /api/discovery/{id}/queue-reply`

**Request** — new optional field:
```json
{
  "content": "...",
  "provenance": [
    {
      "node_id": 1,
      "chunk_id": 2,
      "source_path": "notes/topic.md",
      "heading_path": "# Topic > ## Section",
      "snippet": "Key insight..."
    }
  ]
}
```

### `DraftResult::Success`

New field in the serialized output:
```json
{
  "status": "success",
  "candidate_id": "...",
  "draft_text": "...",
  "vault_citations": [...]
}
```

Omitted from JSON when empty (via `skip_serializing_if`).
