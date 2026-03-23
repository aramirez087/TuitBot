# Epic Charter: Vault Indexer + Semantic Search

## Problem Statement

Today, Ghostwriter retrieves vault fragments using two mechanisms: keyword matching (LIKE-based chunk search) and structural graph expansion (1-hop wikilinks, backlinks, shared tags). Both are valuable, but both miss an entire class of relevant content: notes that are **conceptually related but not explicitly linked or keyword-matched**.

A user writing about "burnout in remote teams" should see their notes on "async communication culture" and "calendar fragmentation" — even if those notes don't share a wikilink or the word "burnout." The current keyword search returns nothing because the vocabulary doesn't overlap. The graph expansion returns nothing because no explicit link exists.

The Backlink Synthesizer (previous epic) proved that surfacing connected notes dramatically improves draft quality. But it only works when the user has already built those connections. Semantic search extends this to the rest of the vault — the 80% of notes that aren't linked but are still relevant.

## Competitive Edge

No competing tool in the social content space — Typefully, Hypefury, Buffer, Taplio — offers semantic retrieval from a personal knowledge base. Most offer generic AI generation with no vault integration at all.

TuitBot already differentiates with:
1. Vault-backed Ghostwriter (Obsidian → compose → draft pipeline)
2. Graph-aware retrieval via Backlink Synthesizer (1-hop neighbor suggestions)

The Vault Indexer + Semantic Search adds a third layer: **dense retrieval from the user's entire vault**, making conceptual connections visible even when structural connections don't exist. This turns every note into potential evidence for every draft, without requiring the user to manually interlink everything.

## User Value

> "I have 500 notes in my vault. I've linked maybe 50 of them to each other. When I'm drafting about a topic, I know I've written about related things before, but I can't remember which notes. I want my vault to tell me."

**Power users** benefit from:
- Drafts grounded in broader vault knowledge, not just the notes they remembered to link
- Discovering forgotten-but-relevant notes during composition
- Understanding *why* a result was suggested (semantic similarity vs. graph edge vs. keyword match)
- Confidence that the index is current — stale chunks are visibly flagged

**Casual users** benefit from:
- Vault search that works even with imprecise queries
- A compose flow that just works without needing to build a heavily interlinked vault

## Scope

### In-Scope

1. **Chunk-level embedding** — Embed each `content_chunk` using a configurable embedding provider (OpenAI `text-embedding-3-small`, Ollama `nomic-embed-text`)
2. **Background indexer** — Incremental, fail-open background worker that embeds dirty chunks after Watchtower ingest
3. **SQLite + in-process HNSW index** — Store embeddings as BLOBs in SQLite, load into `usearch` HNSW for fast ANN queries
4. **Hybrid retrieval** — Blend semantic, graph, and keyword results with explainable `match_reason` per result
5. **Evidence rail UX** — Collapsible panel inside ComposerInspector for browsing, searching, and pinning semantic results
6. **Auto-query (opt-in)** — Debounced semantic search from draft text during editing, surfacing suggested evidence
7. **Pin/dismiss actions** — Lock evidence across edits or remove it, matching existing accept/dismiss pattern
8. **Index status affordances** — Progress indicators, freshness reporting, stale warnings
9. **Provenance extension** — `match_reason: "semantic"` and `similarity_score` in provenance chain
10. **Embedding provider abstraction** — Trait-based provider with factory pattern, matching existing `LlmProvider` architecture
11. **Degraded-state fallback** — When embedding provider is unavailable or index is empty, silently fall back to keyword+graph retrieval

### Out-of-Scope (Non-Goals)

1. **Multi-modal embeddings** — Images, PDFs, audio transcripts are not embedded. Text chunks only.
2. **Cross-vault search** — Each vault is a separate index. No merging across vaults or accounts.
3. **Collaborative/shared indexes** — Single-user scope. No multi-user semantic indexes.
4. **Custom fine-tuned embedding models** — Use off-the-shelf models. No training or fine-tuning.
5. **Real-time streaming embeddings** — Embeddings are computed in background batches, not on the hot path.
6. **Multi-hop semantic traversal** — No "friends-of-friends" via embedding similarity. Direct similarity only.
7. **Embedding-based graph edge creation** — Semantic similarity does not create new `note_edges`. Graph remains author-defined.
8. **External knowledge base integration** — No Wikipedia, no web search. Vault content only.

## Success Criteria

1. **Semantic results surface:** Given a query text and a vault with 50+ indexed notes, semantic search returns at least 3 relevant results within 100ms on Desktop.
2. **Hybrid blending works:** Results from semantic, graph, and keyword retrieval are merged, deduplicated, and ranked with visible `match_reason` badges per result.
3. **Index freshness visible:** The user can see "N of M chunks indexed" and "Last indexed: X minutes ago" in the evidence rail.
4. **Degraded fallback seamless:** When the embedding provider is unavailable (Ollama not running, API key missing), the evidence rail shows keyword+graph results with no error state — just a subtle "Semantic search unavailable" indicator.
5. **Pin/dismiss persistent per session:** Pinned evidence stays across draft edits within the same compose session. Dismissed evidence doesn't return.
6. **Privacy invariants hold:** All 10 existing privacy invariants remain satisfied. No raw embedding vectors in API responses. No raw chunk text beyond the 120-char snippet limit. Cloud mode restrictions preserved.
7. **Provenance complete:** Every evidence fragment that contributes to a draft has full provenance: match_reason, similarity_score (if semantic), source_node, heading_path.
8. **Indexer fail-open:** If the background indexer crashes or stalls, Watchtower ingest continues normally. Index status shows "Indexing paused" but compose flow is unaffected.
9. **Backward compatible:** All existing `/api/vault/*` and `/api/assist/*` endpoints continue to work without semantic parameters. No breaking changes.

## Design Principles

1. **User in control** — Semantic evidence is suggested, never silently injected. The user pins, dismisses, or applies each result with a visible action. Auto-query is opt-in.
2. **Visible reasons** — Every result shows why it was surfaced: "Similar content," "Linked note," "Keyword match." No black-box recommendations.
3. **Complementary, not replacement** — Semantic retrieval augments graph and keyword retrieval. It never overrides structural connections. When graph neighbors exist, they take precedence in the UI.
4. **Fail open** — Embedding provider down? Index stale? Missing config? The compose flow works exactly as it does today. Semantic search is additive.
5. **Local-first by default** — Desktop mode uses Ollama for embeddings and in-process HNSW for search. No cloud dependency required. Self-host and Cloud modes use configured providers.
6. **Additive schema** — New tables and columns only. No existing columns removed or renamed. No migration that alters existing table structures.
7. **Incremental indexing** — Only re-embed chunks whose content has changed. Hash-based dirty tracking ensures minimal API calls and compute.
8. **Explainable ranking** — The blended score is transparent: each signal (semantic similarity, graph score, keyword relevance) is reported separately, not hidden in an opaque combined score.
