# Epic Charter: Backlink Synthesizer

## Problem Statement

Today, selecting a note for Ghostwriter retrieves chunks from that single note plus keyword-matched fragments from elsewhere in the vault. The user's vault is a knowledge graph — notes connect to each other via `[[wikilinks]]`, `[markdown links]`, and shared `#tags` — but retrieval ignores those connections entirely.

The result: drafts miss relevant context that the user's own notes already connect. A user who selects their "Distributed Systems" note expects their linked "CAP Theorem" and "Raft Consensus" notes to surface too. Instead, they get unrelated keyword hits or nothing at all.

The Backlink Synthesizer closes this gap by extracting the note graph at ingestion time, expanding retrieval to 1-hop neighbors, and giving the user visible control over which related notes contribute to their draft.

## Competitive Edge

No competing tool in the social content space — Typefully, Hypefury, Buffer, or Taplio — offers graph-aware retrieval from a personal knowledge base. Most offer generic AI generation with no vault integration at all.

TuitBot already differentiates with vault-backed Ghostwriter (the Obsidian → compose → draft pipeline). The Backlink Synthesizer extends this from flat file search to network-aware context retrieval, making the user's vault connections a first-class input to draft quality.

This turns the vault from a searchable archive into a live knowledge graph that actively improves every draft.

## User Value

> "When I select my note about distributed systems, I want my Rust implementation note and my CAP theorem note to surface too — because I linked them. I shouldn't have to manually select every related note."

**Power users** maintain heavily interlinked vaults. Their wikilinks encode domain expertise structure. Surfacing those connections during draft composition means:

- Drafts draw on broader, more interconnected knowledge
- The user sees *why* a note was suggested (reason labels: "linked from your note", "shares tag #rust")
- Dismissed suggestions don't come back in the same session
- Every surfaced fragment has full provenance (which note, which edge, which heading)

## Scope

### In-Scope

1. **Link extraction** — Parse `[[wikilinks]]`, `[markdown](links)`, and inline `#tags` from note bodies during chunking
2. **Edge storage** — New `note_edges` table with directed, typed edges between `content_nodes`
3. **Tag normalization** — New `note_tags` table for efficient shared-tag queries
4. **1-hop graph expansion** — Given a selected note, retrieve its direct neighbors via edges
5. **Deterministic ranking** — Composite score: direct link weight + shared tag weight + chunk relevance boost
6. **Related-note suggestion UX** — Suggestion cards with reason labels, accept/dismiss controls
7. **Thread-slot insertion** — Accepted related-note fragments can contribute to specific thread slots
8. **Provenance extension** — `edge_type` and `edge_label` fields on `vault_provenance_links`
9. **Fallback behavior** — Graceful degradation when graph data is sparse or absent

### Out-of-Scope (Non-Goals)

1. **Multi-hop traversal (> 1 hop)** — Friends-of-friends adds exponential complexity and diminishing relevance. Future v2 enhancement.
2. **Embedding-based similarity** — Operator constraint: prefer deterministic graph retrieval before LLM summarization. No vector DB, no embedding models.
3. **External link resolution** — Links to URLs, PDFs, or non-vault resources are not resolved. Only vault-internal note-to-note edges.
4. **Cross-vault graph merging** — Each vault is a separate graph. No merging across vaults or accounts.
5. **LLM-based link discovery** — No "this note seems related" inference. Only explicit user-authored links and tags.
6. **Collaborative graph** — Single-user scope. No shared graph editing or multi-user vaults.

## Success Criteria

1. **Related notes surface:** Given a selected note with 3+ wikilinks to indexed notes, at least 2 related notes appear in the suggestion panel.
2. **Provenance is complete:** Every accepted suggestion appears in `vault_provenance_links` with `edge_type` (e.g., "wikilink", "shared_tag") and `edge_label` (e.g., the link text or tag name).
3. **Dismiss works:** Dismissed suggestions are excluded from retrieval for the remainder of that session.
4. **Fallback is seamless:** When a selected note has no edges, the standard retrieval path (today's behavior) activates with no UX disruption. The user sees "No linked notes found — using this note only."
5. **Privacy invariants hold:** All 10 existing privacy invariants remain satisfied. No raw `body_text` or `chunk_text` in API responses. Cloud mode continues to omit `selected_text`.
6. **Backward compatible:** All existing `/api/assist/*` endpoints continue to work without the `graph_neighbors` parameter. No breaking changes.

## Principles

1. **User in control** — Related notes are suggested, never silently injected. The user accepts or dismisses each one.
2. **Visible reasons** — Every suggestion shows why it was surfaced (link type, tag name). No black-box recommendations.
3. **Deterministic first** — Ranking uses a transparent composite score. No LLM calls in the ranking path.
4. **Fail open** — Unresolvable links are logged and skipped. Missing graph data falls back to today's behavior.
5. **Additive schema** — New tables and columns only. No existing columns removed or renamed.
6. **Privacy by default** — Link targets and tag names are metadata. Snippet privacy rules (120 char) still apply.
