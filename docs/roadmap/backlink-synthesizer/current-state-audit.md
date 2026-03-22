# Current State Audit: Ghostwriter Pipeline vs. Backlink Synthesizer

> Audited 2026-03-21 against `main` @ `84da0d62`.

## Existing Pipeline Overview

```
Obsidian Plugin                    Dashboard + Server                     Core
─────────────────                  ──────────────────                     ────
[User selects text]
       │
       ▼
GhostwriterPayload ──POST──▶ /api/vault/send-selection
  (selected_text,              │  ├─ validate payload
   heading_context,            │  ├─ resolve_selection_identity()
   frontmatter_tags,           │  │    → (node_id, chunk_id)
   file_path)                  │  ├─ insert vault_selections (30-min TTL)
                               │  └─ emit WsEvent::SelectionReceived
                               │
                               ▼
                          /compose?selection={session_id}
                               │
                               ▼
                     resolve_selection_rag_context()
                               │
                               ├─ fetch vault_selections row
                               ├─ resolve_composer_rag_context()
                               │    ├─ load business keywords
                               │    └─ build_draft_context_with_selection()
                               │         ├─ Tier 1: retrieve_ancestors()
                               │         ├─ Tier 2: retrieve_vault_fragments()
                               │         │    ├─ selected-node chunks first
                               │         │    └─ keyword search to fill slots
                               │         └─ Tier 3: cold-start seeds (fallback)
                               │
                               ▼
                     DraftContext { prompt_block, vault_citations, ... }
                               │
                               ▼
                     /api/assist/hooks → /api/assist/thread
                               │
                               ▼
                     vault_provenance_links (polymorphic)
```

## What Exists Today

| Layer | Component | File | Summary |
|-------|-----------|------|---------|
| **Obsidian Plugin** | Ghostwriter payload sender | `plugins/obsidian-tuitbot/src/main.ts` | Sends `selected_text`, `heading_context`, `frontmatter_tags`, `file_path` to server |
| **Selection Ingress** | `send_selection` / `get_selection` | `crates/tuitbot-server/src/routes/vault/selections.rs` | Stores in `vault_selections` table with 30-min TTL; resolves `node_id`/`chunk_id` via `resolve_selection_identity()` |
| **Watchtower Ingestion** | File watcher + ingest pipeline | `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Watches local/remote dirs, parses frontmatter, upserts `content_nodes`, triggers chunking |
| **Chunker** | Fragment extraction | `crates/tuitbot-core/src/automation/watchtower/chunker.rs` | Splits markdown by headings into `content_chunks`. Handles code fences, heading hierarchy, hash dedup |
| **Retrieval** | Two-phase fragment retrieval | `crates/tuitbot-core/src/context/retrieval.rs` | Phase 1: chunks from selected nodes. Phase 2: keyword search to fill remaining slots (max 5 fragments, 1000 char budget) |
| **Winning DNA** | Three-tier context builder | `crates/tuitbot-core/src/context/winning_dna/analysis.rs` | Ancestors (engagement-scored) + vault fragments + cold-start seeds. Combined prompt capped at `RAG_MAX_CHARS` (2000) |
| **RAG Helpers** | Session → context resolver | `crates/tuitbot-server/src/routes/rag_helpers.rs` | Glues session_id → selection → node IDs → `build_draft_context_with_selection()` |
| **Provenance** | Polymorphic source tracking | `crates/tuitbot-core/src/storage/provenance.rs` | Maps `(entity_type, entity_id)` → `(node_id, chunk_id, seed_id, source_path, heading_path, snippet)` |
| **Privacy** | Deployment-aware gates | `crates/tuitbot-server/src/routes/vault/selections.rs:261` | Cloud mode omits `selected_text`; 120-char snippets universal; no raw `body_text`/`chunk_text` in read APIs |
| **Storage** | Watchtower CRUD modules | `crates/tuitbot-core/src/storage/watchtower/` | `sources`, `nodes`, `chunks`, `seeds`, `connections` — all account-scoped |

## What's Missing (Implementation Gaps)

### Gap 1: No Link Extraction from Note Bodies

**Affected file:** `crates/tuitbot-core/src/automation/watchtower/chunker.rs`

The chunker's `extract_fragments()` function (line 61) parses headings via `^(#{1,6})\s+(.+)$` but has zero awareness of `[[wikilinks]]`, `[markdown](links)`, or inline `#tags` in body text. Links embedded in note content are treated as opaque text.

**Why it matters:** Without extracting links, we cannot build edges between notes. The user's explicit connections (wikilinks) are invisible to retrieval.

### Gap 2: No Note-to-Note Edge Table

**Affected location:** `crates/tuitbot-core/src/storage/watchtower/` (no `edges.rs` module exists)

There is no `note_edges` table in any migration. Notes are isolated rows in `content_nodes` with no relational graph structure. The only "relationship" between notes is coincidental keyword overlap at query time.

**Why it matters:** Graph expansion requires persisted edges. Without them, 1-hop neighbor queries are impossible.

### Gap 3: No Tag Normalization

**Affected location:** `content_nodes.tags` column stores raw comma-separated strings from frontmatter (e.g., `"rust,async,tokio"`). There is no normalized `note_tags` table for many-to-many queries.

**Why it matters:** Shared-tag edges require efficient `WHERE tag_text = ?` lookups across nodes. Scanning raw CSV strings is fragile (casing, whitespace, aliases) and slow at scale.

### Gap 4: No Graph Expansion in Retrieval

**Affected file:** `crates/tuitbot-core/src/context/retrieval.rs`

`retrieve_vault_fragments()` (line 62) accepts `selected_node_ids` and `keywords` — it fetches chunks from those explicit nodes, then fills remaining slots via keyword search. It has no concept of walking edges to neighbor notes.

**Why it matters:** The user selects one note. If that note links to 5 others via `[[wikilinks]]`, none of those neighbors are considered unless the user manually selects them too.

### Gap 5: No Related-Note Suggestions in the UX

**Affected location:** Dashboard Ghostwriter flow (compose page)

The current flow shows the selected note's text and generates hooks/threads from it. There is no UI component that says "This note links to X, Y, Z — want to include them?" No suggestion cards, no accept/dismiss controls.

**Why it matters:** Even if the backend had graph data, there's no UX surface to let the user choose which related notes to include.

### Gap 6: No Graph-Aware Ranking

**Affected file:** `crates/tuitbot-core/src/context/retrieval.rs`

Retrieval ranking is per-chunk via the `retrieval_boost` column on `content_chunks`. There is no edge-distance weighting, no shared-tag affinity scoring, no composite rank that considers graph proximity.

**Why it matters:** When graph expansion surfaces 8 candidate neighbors, we need a deterministic ranking to show the most relevant ones first.

### Gap 7: No Suggestion Cards or Reason Labels

**Affected location:** Dashboard frontend (no component exists)

There is no `RelatedNoteSuggestions.svelte` or equivalent. The compose flow has no mechanism for displaying "Related note X (linked via [[wikilink]])" with reason badges and accept/dismiss buttons.

**Why it matters:** The user needs to understand *why* a note was suggested and control *whether* it's included. Without this, graph-aware retrieval is invisible and uncontrollable.

### Gap 8: No Thread-Slot Insertion from Related Notes

**Affected files:** `crates/tuitbot-core/src/context/winning_dna/analysis.rs`, assist route handlers

Thread generation uses a single `DraftContext` built from the selected note + keyword search. There is no mechanism to inject specific related-note fragments into individual thread slots, and no way to attribute which slot drew from which neighbor.

**Why it matters:** A thread about distributed systems should be able to draw slot 1 from the user's "CAP Theorem" note and slot 3 from their "Raft Consensus" note, with per-slot provenance.

## Schema Gap Analysis

| Table | Exists? | Change Needed |
|-------|---------|---------------|
| `content_nodes` | Yes | No changes — already has `tags`, `title`, `relative_path`, `body_text` |
| `content_chunks` | Yes | No changes — already has `heading_path`, `retrieval_boost`, `chunk_text` |
| `vault_selections` | Yes | No changes — already resolves `node_id` |
| `vault_provenance_links` | Yes | Additive: add `edge_type` and `edge_label` columns for graph provenance |
| `note_edges` | **No** | **Create** — directed edges between content_nodes with edge_type |
| `note_tags` | **No** | **Create** — normalized many-to-many tag index |

## Privacy Implications

The Backlink Synthesizer introduces two new data categories:

1. **Link targets** — Note titles referenced in `[[wikilinks]]` and `[markdown](links)`. These are metadata about the note graph structure, not note content. They are comparable to `relative_path` and `title` already stored in `content_nodes`.

2. **Normalized tag names** — Lowercased tag strings from frontmatter and inline `#tags`. These are already exposed in the `content_nodes.tags` column and `vault_selections.frontmatter_tags` field.

**Privacy verdict:** Neither category introduces new raw note body exposure. The existing privacy invariants are preserved:
- Cloud mode continues to omit `selected_text` from read APIs
- Snippets remain capped at 120 characters
- No raw `body_text` or `chunk_text` in API responses
- Link targets and tag names are metadata, not content

## Key Design Decisions (Recorded)

1. **Link extraction at chunk-time, not query-time.** Links are stable metadata embedded in note content. Extracting during `chunk_node()` avoids re-parsing on every retrieval query. Only changed nodes get re-extracted (incremental).

2. **Tag normalization is a write-time index, not a read-time parse.** The `content_nodes.tags` column stores raw CSV that may contain casing differences and aliases. A normalized `note_tags` table enables efficient graph queries.
