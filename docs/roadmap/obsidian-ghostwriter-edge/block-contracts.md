# Block Identity Contracts

## Overview

This document describes the server-side contracts for receiving Ghostwriter selections from the Obsidian plugin, resolving them against indexed vault data, and preserving provenance through compose and draft workflows.

## Selection Ingress Contract

### Endpoint: `POST /api/vault/send-selection`

Receives an exact text selection from the Obsidian plugin. The contract follows the schema defined in `obsidian-plugin-contract.md`.

**Request**: JSON body with `vault_name`, `file_path`, `selected_text`, and optional `heading_context`, `note_title`, `frontmatter_tags`.

**Response**: `{ status: "received", session_id: "<uuid>", composer_url: "/compose?selection=<uuid>" }`

**Error codes**: 401 (invalid token), 413 (text > 10000 chars), 422 (validation), 429 (rate limit: 10/min/account).

### Endpoint: `GET /api/vault/selection/{session_id}`

Retrieves a stored selection by `session_id`, scoped to the authenticated account.

**Privacy gate**: In Cloud deployment mode, `selected_text` is omitted from the response (returns metadata only).

## Block Identity Resolution Algorithm

When a selection is received, the server attempts best-effort resolution to the nearest indexed block:

1. **Node lookup**: `find_node_by_path_for(account_id, file_path)` — searches `content_nodes` by `relative_path`, returns the most recently updated match across all sources.

2. **Chunk matching**: If a node is found and `heading_context` is provided, `find_best_chunk_by_heading_for(account_id, node_id, heading_context)` finds the chunk whose `heading_path` has the longest overlap with the provided heading context.

3. **Fallback**: If no node exists (note not yet indexed), both `resolved_node_id` and `resolved_chunk_id` are `NULL`. The `selected_text` itself is always the authoritative payload.

**Why best-effort**: Users may send selections from newly created notes or un-indexed vaults. Requiring prior indexing would make the feature fragile. Resolution enriches provenance when available but never gates functionality.

## Provenance Flow

```
Selection (Obsidian) → vault_selections table (transient, 30-min TTL)
    ↓
Dashboard retrieves via GET /api/vault/selection/{session_id}
    ↓
User generates content from selection (assist/tweet, assist/thread)
    ↓
Compose/Draft submission includes ProvenanceRef:
  { node_id, chunk_id, source_path, heading_path, snippet }
    ↓
vault_provenance_links table (persistent, polymorphic)
    ↓
Approval queue → posted content (provenance copied via copy_links_for)
    ↓
Citation rendering in dashboard (heading-anchor deep-links)
```

The `ProvenanceRef` struct already has all required fields (`node_id`, `chunk_id`, `seed_id`, `source_path`, `heading_path`, `snippet`). No new provenance types were needed.

## Storage Design

### `vault_selections` Table

- **Purpose**: Transient storage for selections received from Obsidian.
- **Key columns**: `session_id` (UUID, unique indexed), `account_id`, `file_path`, `selected_text`, `resolved_node_id`, `resolved_chunk_id`.
- **TTL**: `expires_at` = `created_at` + 30 minutes.
- **Cleanup**: `cleanup_expired()` deletes rows where `expires_at <= datetime('now')`. Called by hourly cleanup task.
- **Rate limiting**: `count_recent_for(account_id, 60)` counts selections in the last 60 seconds. Limit: 10 per minute per account.

### Privacy Invariants

- `GET /api/vault/selection/{session_id}` omits `selected_text` when `deployment_mode` is Cloud.
- All queries are account-scoped — no cross-account data leakage.
- The `vault_selections` table uses the same account isolation pattern as `vault_provenance_links`.

## Decisions Made

### Decision 1: Selection storage as transient table with TTL

**Chose**: New `vault_selections` table with 30-minute TTL, hourly cleanup.
**Rationale**: Selections are ephemeral — the user sends from Obsidian, the dashboard consumes within minutes. No long-lived state needed.

### Decision 2: Best-effort block identity resolution

**Chose**: Attempt resolution at insert time. Store `NULL` if note isn't indexed.
**Rejected**: Requiring prior indexing (too fragile).
**Rationale**: `selected_text` is always the authoritative payload.

### Decision 3: Provenance flows through existing ProvenanceRef pipeline

**Chose**: Construct `ProvenanceRef` from resolved IDs. No new provenance types.
**Rationale**: `ProvenanceRef` already has all fields. Existing compose/draft/approval pipelines handle it unchanged.

### Decision 4: Cloud mode privacy gate on GET endpoint

**Chose**: Omit `selected_text` in Cloud mode responses.
**Rationale**: Privacy invariant from architecture doc. Raw note content must not be retrievable via API in Cloud deployments.

### Decision 5: Vault route module directory split

**Chose**: `vault.rs` → `vault/mod.rs` + `vault/selections.rs`.
**Rationale**: Adding two handlers + DTOs would push `vault.rs` past the 500-line file limit. Module directory follows existing patterns.

### Decision 6: WebSocket event for selection receipt

**Chose**: Added `SelectionReceived { session_id }` variant to `WsEvent`.
**Rationale**: Dashboard needs real-time notification to auto-open composer when selection arrives. Additive change — existing variants unchanged.
