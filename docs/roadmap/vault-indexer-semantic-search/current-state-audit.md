# Current State Audit: Compose Flow & Semantic Integration Seams

## Overview

This audit maps the live Ghostwriter compose flow from Obsidian selection through published draft, identifying every component, route, storage table, and retrieval function involved. It then names the exact seams where semantic evidence can improve outcomes without adding clutter.

---

## 1. Compose Flow: End-to-End

### Phase 1: Selection Ingress

| Step | Actor | Component / Route | Storage | Notes |
|------|-------|-------------------|---------|-------|
| User highlights text in Obsidian | Obsidian plugin | — | — | Captures vault_name, file_path, selected_text, heading_context, frontmatter_tags |
| Plugin sends selection | HTTP POST | `POST /api/vault/send-selection` (`selections.rs:65`) | `vault_selections` table | 30-min TTL session, rate-limited 10/min |
| Server resolves block identity | Server | `retrieval::resolve_selection_identity()` (`retrieval.rs`) | `content_nodes`, `content_chunks` | Best-effort match by file_path + heading_context |
| Server auto-expands graph neighbors | Server | `rag_helpers::resolve_graph_suggestions()` → `graph_expansion::expand_graph_neighbors()` (`graph_expansion.rs`) | `note_edges`, `note_tags`, `content_chunks` | 1-hop wikilinks + backlinks + shared tags, scored deterministically |
| WebSocket notification | Server | `event_tx.send(SelectionReceived)` (`selections.rs:197`) | — | Dashboard receives session_id in real-time |

**Cloud mode gate:** `send_selection` returns 403 in Cloud mode — raw text never leaves the user's device (`selections.rs:72`).

### Phase 2: Composer Hydration

| Step | Component | File | Details |
|------|-----------|------|---------|
| Composer opens with selection session | `ComposerInspector` | `ComposerInspector.svelte` | Receives `selectionSessionId` prop |
| FromVaultPanel detects active session | `FromVaultPanel` | `FromVaultPanel.svelte:178-194` | Sets `selectionActive = true`, skips manual search |
| VaultSelectionReview loads session | `VaultSelectionReview` | `VaultSelectionReview.svelte:91-100` | Calls `api.vault.getSelection(sessionId)` |
| Selection card renders | `VaultSelectionReview` | `VaultSelectionReview.svelte:382-401` | Shows note_title, heading_context, selected_text (or cloud privacy notice), frontmatter_tags |
| Graph neighbor cards render | `GraphSuggestionCards` | `VaultSelectionReview.svelte:419-426` | Shows `visibleNeighbors` with accept/dismiss controls |
| User accepts/dismisses neighbors | `VaultSelectionReview` | `VaultSelectionReview.svelte:102-131` | `acceptedNeighbors` Map, `dismissedNodeIds` Set, analytics tracked |

### Phase 3: Hook / Angle Generation

| Step | Component | File | Details |
|------|-----------|------|---------|
| User clicks "Generate" | `VaultSelectionReview` | `VaultSelectionReview.svelte:134-178` | Branches on `synthesisEnabled && acceptedNeighbors.size > 0` |
| **With neighbors:** Angle mining | `VaultSelectionReview` → `api.assist.angles()` | `VaultSelectionReview.svelte:140-163` | Hook Miner: extracts cross-note angles from primary + neighbor notes |
| **Without neighbors:** Hook generation | `VaultSelectionReview` → `api.assist.hooks()` | `VaultSelectionReview.svelte:165-177` | Standard hook generation from primary note only |
| Angle cards / Hook picker renders | `AngleCards` / `HookPicker` | `VaultSelectionReview.svelte:352-380` | User selects one angle or hook |
| Fallback: angle mining fails | `AngleFallback` | `VaultSelectionReview.svelte:346-351` | Falls back to generic hooks via `handleFallbackToGenericHooks()` |

### Phase 4: Draft Creation

| Step | Component | File | Details |
|------|-----------|------|---------|
| User selects hook/angle + format | `VaultSelectionReview` | `handleHookSelected()` / `handleAngleSelected()` | Builds `neighborProv[]` with edge_type, edge_label, angle_kind, signal_kind, signal_text |
| Calls `ongenerate()` → parent | `ComposerInspector` | `ComposerInspector.svelte:273-326` | `handleGenerateFromVault()` — captures full provenance chain |
| Server retrieves vault fragments | Server | `retrieval::retrieve_vault_fragments()` (`retrieval.rs`) | Selected node IDs → chunks → keyword fill, capped at 5 fragments / 1000 chars |
| Server formats LLM prompt | Server | `retrieval::format_fragments_prompt()` (`retrieval.rs`) | Inline citations: `[1] heading_path: snippet` |
| LLM generates draft | Server | `api.assist.tweet()` / `api.assist.thread()` | Via `LlmProvider` trait (`llm/mod.rs`) |
| Draft renders in composer | `ComposerInspector` | `ComposerInspector.svelte:294-318` | Sets `tweetText` or `threadBlocks` |

### Phase 5: Slot Refinement

| Step | Component | File | Details |
|------|-----------|------|---------|
| SlotTargetPanel maps neighbors → slots | `SlotTargetPanel` | `VaultSelectionReview.svelte:469-478` | Only shown when `hasExistingContent && acceptedNeighbors.size > 0 && synthesisEnabled` |
| User applies neighbor to specific slot | `ComposerInspector` | `handleSlotInsert()` (`ComposerInspector.svelte:98-143`) | Calls `api.assist.improve()` with neighbor context |
| Undo stack tracks insert | `draftInsertStore` | `ComposerInspector.svelte:118-128` | `pushInsert()` → `DraftInsertState` with previous/inserted text, source metadata |
| Insert badges show on thread cards | `ThreadFlowCard` | `ThreadFlowLane.svelte:288` | `getInsertsForBlock(insertState, block.id)` |
| Undo available for 10 seconds | `ComposerInspector` | `ComposerInspector.svelte:192-196` | Timer-based undo banner |

### Phase 6: Provenance

| Step | Component | File | Details |
|------|-----------|------|---------|
| Provenance captured during generation | `ComposerInspector` | `ComposerInspector.svelte:278-292` | `vaultProvenance: ProvenanceRef[]` — node_id, edge_type, edge_label, angle_kind, signal_kind, signal_text, source_role |
| Hook style recorded | `ComposerInspector` | `ComposerInspector.svelte:293` | `vaultHookStyle` — the selected hook/angle style |
| Provenance extended on slot insert | `ComposerInspector` | `ComposerInspector.svelte:131-134` | Appends `accepted_neighbor` provenance ref |
| Parent reads provenance at publish | `ComposerInspector` | `getVaultProvenance()` / `getVaultHookStyle()` (`ComposerInspector.svelte:70-77`) | Exported methods called by parent compose page |

---

## 2. Current Retrieval Stack

| Layer | Mechanism | File | Key Functions |
|-------|-----------|------|---------------|
| **Keyword search** | SQLite FTS5 on `content_nodes` (title, relative_path) | `storage/watchtower/nodes.rs` | `search_nodes_for()` — LIKE-based title/path matching |
| **Chunk search** | LIKE-based keyword search on `content_chunks` | `storage/watchtower/chunks.rs` | `search_chunks_by_keywords()`, `search_chunks_with_context()` |
| **Graph expansion** | 1-hop wikilinks + backlinks + shared tags | `context/graph_expansion.rs` | `expand_graph_neighbors()` — deterministic composite score |
| **Fragment retrieval** | Selected node IDs → chunks → keyword fill | `context/retrieval.rs` | `retrieve_vault_fragments()` — 5 fragments, 1000 chars max |
| **LLM context** | Formatted prompt with inline citations | `context/retrieval.rs` | `format_fragments_prompt()` |

### Scoring Model (Graph Expansion)

```
score = 3.0 × direct_links + 2.0 × backlinks + 1.0 × shared_tags + 0.5 × chunk_boost
```

Defined in `graph_expansion.rs` constants: `WEIGHT_DIRECT_LINK=3.0`, `WEIGHT_BACKLINK=2.0`, `WEIGHT_SHARED_TAG=1.0`, `WEIGHT_CHUNK_BOOST=0.5`.

Classification is deterministic: `classify_suggestion_reason()` picks the highest-weight edge type. `classify_suggestion_intent()` uses keyword heuristics (counterpoint, pro_tip, evidence, related).

### What's Missing

- No vector embeddings anywhere in the stack
- No embedding storage (no `chunk_embeddings` table, no BLOB columns)
- No dense similarity search (no ANN index, no cosine distance)
- No semantic re-ranking (BM25 or learned-to-rank)
- No embedding provider config (`LlmConfig` covers completion only, no `EmbeddingConfig`)
- Graph expansion is purely structural — no semantic signal in scoring
- `retrieval_boost` is a static DB column on `content_chunks`, never dynamically tuned
- Fragment retrieval is keyword-only: `retrieve_vault_fragments()` splits query into whitespace tokens and does LIKE matching
- No query understanding: user's draft text is never used to find relevant vault content

---

## 3. Semantic Evidence Integration Seams

These are the exact points where semantic retrieval adds value without disrupting the existing flow.

### Seam 1: Selection Review — Semantic Neighbors

**Where:** `VaultSelectionReview.svelte` → `GraphSuggestionCards` rendering
**Current behavior:** Graph neighbors come exclusively from structural edges (wikilinks, backlinks, shared tags). When a note has few or no edges, the graph is sparse and `GraphState::NoRelatedNotes` or `GraphState::FallbackActive` is shown.
**Semantic upgrade:** When graph neighbors are sparse (< 3 results), supplement with semantically similar chunks from the vault. These appear as a new `SuggestionReason::SemanticMatch` variant with a reason label like "Similar content in vault."
**Why it helps:** Users who don't heavily interlink their notes still get relevant suggestions. The semantic layer acts as a safety net for sparse graphs.

### Seam 2: Hook / Angle Generation — Richer Context

**Where:** `VaultSelectionReview.svelte:134-178` → `api.assist.angles()` / `api.assist.hooks()`
**Current behavior:** Hook and angle mining use only the primary selection text and accepted graph neighbors. The LLM prompt includes fragments from `retrieve_vault_fragments()` which does keyword matching only.
**Semantic upgrade:** Before calling `assist.hooks()` or `assist.angles()`, run a semantic search using the selection text as query. Include top-K semantically similar chunks as additional context in the LLM prompt. This gives the hook miner access to conceptually related material the user didn't explicitly link.
**Why it helps:** Hooks and angles become more creative and grounded when the LLM sees conceptually adjacent notes, not just structurally connected ones.

### Seam 3: Fragment Retrieval — Semantic Fill

**Where:** `context/retrieval.rs` → `retrieve_vault_fragments()`
**Current behavior:** Retrieves fragments by (a) selected node IDs first, then (b) keyword fill from vault-wide chunk search. The keyword fill uses whitespace-split LIKE matching, which misses semantic synonyms and conceptual matches.
**Semantic upgrade:** Add a third retrieval path: (c) semantic fill — when keyword fill returns fewer than `MAX_FRAGMENTS`, backfill with top-K semantically similar chunks. Fragments carry a `match_reason` field (`Keyword`, `Selected`, `Semantic`) for transparency.
**Why it helps:** Fragment retrieval becomes robust against vocabulary mismatch. A note about "distributed consensus" matches a chunk about "leader election" even though they share no keywords.

### Seam 4: Slot Refinement — Semantic Suggestions for Slots

**Where:** `SlotTargetPanel` → `ComposerInspector.handleSlotInsert()`
**Current behavior:** Only accepted graph neighbors can be applied to thread slots. The user must have accepted a neighbor during the review phase.
**Semantic upgrade:** Allow semantically matched chunks to be applied to slots, not just graph neighbors. When the user has a draft in a slot, semantic search using that slot's text can suggest relevant vault fragments to enrich it. These suggestions use the same `handleSlotInsert()` → `api.assist.improve()` pattern.
**Why it helps:** Slot refinement becomes available even when no graph neighbors exist, and suggestions are contextually relevant to each specific slot's content.

### Seam 5: Vault Search Endpoint — Semantic Upgrade

**Where:** `GET /api/vault/search` (`vault/mod.rs:364-386`)
**Current behavior:** Splits query into whitespace tokens, calls `retrieve_vault_fragments()` with keyword matching only. Returns `VaultCitation[]`.
**Semantic upgrade:** Accept an optional `mode=semantic|keyword|hybrid` parameter. When `mode=semantic` or `mode=hybrid`, run vector similarity search against the chunk embeddings index. Return results with `match_reason` indicating which retrieval path produced each result.
**Why it helps:** The manual search in `FromVaultPanel` (the non-selection browse mode) becomes dramatically more useful — users can search by concept, not just by exact title keywords.

### Seam 6: Auto-Expand on Selection — Semantic Pre-Population

**Where:** `selections.rs:290-308` → `resolve_graph_suggestions()`
**Current behavior:** When a selection is received, the server automatically expands 1-hop graph neighbors and includes them in the `GetSelectionResponse`. The dashboard renders these immediately in `GraphSuggestionCards`.
**Semantic upgrade:** Alongside graph neighbors, include top-K semantically similar chunks in the response. These appear as a separate `semantic_neighbors` field (or merged into `graph_neighbors` with a `SemanticMatch` reason). The dashboard renders them with a distinct badge.
**Why it helps:** The initial selection review immediately shows the user what their vault knows about this topic — not just what's structurally linked, but what's conceptually related.

### Seam 7: Provenance Chain — Semantic Match Metadata

**Where:** `ComposerInspector.svelte:278-292` → `ProvenanceRef` construction
**Current behavior:** Provenance tracks `edge_type` (wikilink, backlink, shared_tag), `edge_label`, `source_role` (primary_selection, accepted_neighbor), and hook miner fields (angle_kind, signal_kind, signal_text).
**Semantic upgrade:** Add `match_reason: "semantic"` and `similarity_score: f64` fields to `ProvenanceRef` when a fragment was found via semantic search. This preserves full explainability: the user (and analytics) can see that a fragment contributed because of semantic similarity, not structural links.
**Why it helps:** Provenance remains complete and auditable. Analytics can compare engagement on drafts that used semantic evidence vs. purely structural evidence.

---

## 4. Storage Schema (Current)

### Tables Relevant to Semantic Indexing

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `content_nodes` | Ingested vault notes | id, account_id, source_id, relative_path, content_hash, title, body_text, tags, status |
| `content_chunks` | Heading-delimited sections of notes | id, account_id, node_id, heading_path, chunk_text, chunk_hash, chunk_index, retrieval_boost, status |
| `note_edges` | Directed edges between notes | source_node_id, target_node_id, edge_type, edge_label |
| `note_tags` | Normalized tags per note | node_id, tag |
| `vault_selections` | Obsidian selections (30-min TTL) | session_id, account_id, vault_name, file_path, selected_text, resolved_node_id, resolved_chunk_id |
| `source_contexts` | Configured vault sources | id, source_type, config_json, status, sync_cursor |

### Missing Tables for Semantic Indexing

| Table | Purpose | Needed Columns |
|-------|---------|----------------|
| `chunk_embeddings` | Vector embeddings per chunk | chunk_id FK, embedding BLOB, model_id, dimension, embedding_hash, generation, created_at, updated_at |

---

## 5. Privacy Invariants (Verified)

All existing privacy rules must be preserved by the semantic indexer:

1. **Cloud mode blocks `send-selection`** — `selections.rs:72` returns 403
2. **Cloud mode omits `selected_text`** — `selections.rs:277-280` returns `None`
3. **Snippet truncation** — `SNIPPET_MAX_LEN = 120` in `vault/mod.rs:25`
4. **No raw `body_text` in API responses** — chunk text only via `snippet` field, truncated
5. **No raw `chunk_text` in search results** — `build_citations()` uses snippet extraction
6. **Account scoping** — every query includes `account_id` via `AccountContext`
7. **Cloud mode omits `relative_path`** — `NeighborItem::from_graph_neighbor()` at `vault/mod.rs:294`
8. **Fragment prompt capped** — `MAX_FRAGMENT_CHARS = 1000`, `MAX_FRAGMENTS = 5` in `retrieval.rs`
9. **Selection rate-limited** — 10 per minute per account at `selections.rs:125`
10. **Local-first file paths** — vault source `path` only exposed in non-Cloud modes at `vault/mod.rs:91`

**Semantic search must not:**
- Return raw embedding vectors in any API response
- Expose full chunk_text beyond the 120-char snippet limit
- Allow cross-account vector similarity queries
- Store embeddings for cloud-mode content that hasn't been locally indexed
