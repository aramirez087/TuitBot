# Obsidian Vault to Post Loop — Charter

## Problem Statement

Tuitbot already indexes vault notes, extracts draft seeds via LLM, and injects
those seeds into generation prompts as cold-start RAG context. Two prior epics
shipped the foundation:

- **Deployment-Aware Content Source Setup** — OAuth-linked Google Drive,
  `connections` table, mode-specific onboarding defaults.
- **Composer Auto-Vault Context** — `resolve_composer_rag_context()` helper,
  `_with_context` generator variants, automatic vault context in all four
  composer assist endpoints.

What's missing is the closed loop: a user writes a note in Obsidian, Tuitbot
turns it into a post, the post's performance feeds back into retrieval weights,
and the user can see the full chain. Today:

| Gap | Description |
|-----|-------------|
| No chunk-level retrieval | Ingestion stores whole note bodies; retrieval is seed-based (LLM-extracted hooks). Cannot cite specific paragraphs. |
| No explicit provenance | `approval_queue` has no FK to `content_nodes` or `draft_seeds`. Cannot trace a post back to its source note. |
| No seed usage tracking | `draft_seeds.used_at` is never set when a seed feeds a draft. |
| No "From Vault" composer | Users cannot intentionally ground a post in a specific note or section. |
| No vault health UI | No dashboard view of sync status, note counts, seed counts, or errors. |
| No citation metadata | Assist endpoints return plain text; users don't know which notes influenced output. |
| Reply assist lacks RAG | `compose_reply` in `discovery.rs` calls `generate_reply` without vault context. |
| No loop-back visibility | `loopback.rs` writes YAML metadata but the dashboard doesn't surface it. |
| No Obsidian-specific affordances | No vault structure awareness, no `obsidian://` URI support. |
| No per-account isolation enforcement | Storage queries use `DEFAULT_ACCOUNT_ID` constant; multi-account data could leak. |

## Non-Negotiable Defaults

These are locked for the epic and cannot be deferred:

1. **Per-account vault isolation** — Every watchtower storage query binds an
   `account_id` parameter. No cross-account leakage of vault data, retrieval
   results, or winning patterns.

2. **Chunk-level retrieval with citations** — Notes are split into
   heading-delimited fragments stored in `content_chunks`. Retrieval operates
   on fragments, not whole notes. Responses include structured citation
   metadata referencing the source note and heading.

3. **Explicit provenance** — The full chain is tracked:
   Note → Fragment → Seed → Draft → Post → Performance → Note.
   `approval_queue` gains `source_node_id`, `source_seed_id`, and
   `source_chunks_json` columns. `draft_seeds` gains `chunk_id`.

4. **Real loop-back** — When analytics records a post's performance, the
   provenance chain is traversed to update `content_chunks.retrieval_boost`.
   High-performing fragments get retrieved more often. `loopback.rs` writes
   performance scores to source file front-matter.

5. **Consistent vault context across composer and reply** — Both composer
   assist (`routes/assist.rs`) and discovery reply (`routes/discovery.rs`)
   inject vault RAG context. All responses include optional citation metadata.

## Design Decisions

### D1: Heading-Based Chunking

Split notes on H1–H3 headings (`^#{1,3} ` regex). Obsidian users organize
content under headings; this preserves semantic coherence and maps to citation
paths ("Note Title > Heading"). Notes without headings fall back to
paragraph splitting (double-newline) with 500-char max per chunk.

- Minimum chunk size: 50 characters (skip trivially small sections).
- Maximum chunk size: 2000 characters (split at paragraph breaks if exceeded).
- `heading_path`: slash-delimited hierarchy (e.g. `## Intro/### Background`).

### D2: content_chunks Table

New `content_chunks` table keeps `content_nodes` as the file-level record.
Chunks are the retrieval unit.

```sql
CREATE TABLE content_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    node_id INTEGER NOT NULL REFERENCES content_nodes(id),
    heading_path TEXT NOT NULL DEFAULT '',
    chunk_text TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    chunk_index INTEGER NOT NULL DEFAULT 0,
    retrieval_boost REAL NOT NULL DEFAULT 1.0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### D3: Provenance as Optional FKs

Add nullable columns to `approval_queue`:
- `source_node_id INTEGER REFERENCES content_nodes(id)`
- `source_seed_id INTEGER REFERENCES draft_seeds(id)`
- `source_chunks_json TEXT` — JSON array of `{chunk_id, heading_path, relevance_score}`

Add `chunk_id INTEGER REFERENCES content_chunks(id)` to `draft_seeds`.

All nullable for backward compatibility.

### D4: Keyword-Based Retrieval (No Embeddings)

LIKE-based keyword search on `chunk_text`, ranked by
`keyword_match_count * retrieval_boost`. Avoids embedding model dependency.
The existing `product_keywords`, `competitor_keywords`, and `industry_topics`
from `BusinessProfile` provide retrieval signals.

- Max chunks per request: 3 (keeps prompt under `RAG_MAX_CHARS`).
- Cold-start seeds remain as fallback when no chunks match.
- SQLite FTS5 can be added later without API changes.

### D5: Retrieval Boost

Multiplicative factor (default 1.0, range 0.1–5.0) applied to keyword-match
relevance. Updated by analytics loop-back:

```
new_boost = current_boost * (1 + 0.1 * normalized_engagement)
```

Capped at 5.0 max, 0.1 min. Recency decay in `winning_dna.rs` handles
staleness independently.

### D6: Citation Format

Structured JSON in assist responses:

```json
{
  "citations": [
    {
      "note_title": "Marketing Playbook",
      "heading": "## Key Insight",
      "chunk_preview": "First 120 characters of chunk text...",
      "obsidian_uri": "obsidian://open?vault=notes&file=Marketing%20Playbook"
    }
  ]
}
```

`obsidian_uri` populated only when `deployment_mode === Desktop` and source is
`local_fs`. All citation fields are optional (backward compatible).

### D7: Reply RAG Parity

Apply `resolve_composer_rag_context()` to `compose_reply` in `discovery.rs`.
Same fail-open behavior as composer endpoints. Response includes optional
`citations` array.

### D8: Obsidian URI Links

Desktop mode citations include `obsidian://open?vault={vault_name}&file={path}`
links. Vault name extracted from the last directory component of the vault path.
Only shown when `deployment_mode === Desktop` and source type is `local_fs`.

## Scope

### In Scope

- Fragment chunking pipeline (heading-based splitter, storage, indexing).
- Provenance schema and wiring through draft → approval → post pipeline.
- Chunk-level RAG retrieval replacing seed-only retrieval.
- Citation metadata in all assist and discovery reply responses.
- Per-account isolation audit and enforcement.
- Loop-back learning (performance → chunk retrieval_boost).
- Vault health API and dashboard page.
- "From Vault" composer surface (note/fragment picker).
- Citation display component and Obsidian URI links.
- Integration testing and documentation.

### Out of Scope

- Embedding-based semantic search (future epic).
- Obsidian plugin / bidirectional sync protocol.
- Multi-vault support (one vault per source entry is sufficient).
- Real-time collaborative editing awareness.
- Vault encryption / E2E encrypted notes.

## Relationship to Existing Charters

This epic **extends** both prior charters:

- **Deployment-Aware Content Source Setup**: We use the `connections` table,
  `ContentSourceEntry.connection_id`, and mode-specific onboarding defaults
  without modification. Vault health UX adds to settings; it does not replace
  the connector flow.

- **Composer Auto-Vault Context**: We extend the automatic RAG from seed-only
  to chunk-level retrieval. The `resolve_composer_rag_context()` helper gains
  chunk retrieval alongside seeds. Response shapes gain optional citation
  metadata. No breaking changes.

## Success Criteria

1. A note saved in an Obsidian vault is automatically chunked into fragments
   within one watchtower polling cycle.
2. Composing a tweet with "From Vault" shows searchable notes, selectable
   fragments, and returns generated content with citations.
3. Discovery reply assist returns vault-grounded content with citations.
4. A posted tweet's performance updates `retrieval_boost` on the fragments
   that influenced it.
5. The vault health dashboard accurately reflects sync state, note counts,
   fragment counts, and seed counts.
6. No cross-account data leakage in watchtower queries.
7. All existing tests pass; no warnings; no regressions in existing flows.
