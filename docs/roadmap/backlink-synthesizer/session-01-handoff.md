# Session 01 Handoff: Charter + Architecture

**Date:** 2026-03-21
**Branch:** `epic/backlink-synthesizer`

## What Changed

Six documentation artifacts created under `docs/roadmap/backlink-synthesizer/`. No source code was modified.

| File | Purpose |
|------|---------|
| `current-state-audit.md` | Maps the existing Ghostwriter pipeline and names 8 implementation gaps with file-level specificity |
| `epic-charter.md` | Defines why graph-aware retrieval is product-critical, scope, non-goals, and success criteria |
| `end-to-end-ux-journey.md` | Full state machine from Obsidian selection through draft review, with concrete UX copy for every state |
| `graph-rag-architecture.md` | Concrete storage and retrieval design: schemas, regex patterns, ranking formula, API spec, fallback behavior |
| `implementation-map.md` | 8-session breakdown with file-level anchors and dependency graph |
| `session-01-handoff.md` | This file |

## Decisions Made

1. **Link extraction at chunk-time, not query-time.** Links are stable metadata; re-parsing on every query is wasteful. Only changed nodes get re-extracted (incremental via `chunk_node()`).

2. **Bidirectional edge storage.** Storing both `A → B (wikilink)` and `B → A (backlink)` makes `expand_graph_neighbors()` a single `WHERE source_node_id = ?` query. ~2x edge rows, negligible for SQLite at vault scale.

3. **1-hop only in v1.** Multi-hop traversal (friends-of-friends) adds exponential complexity with diminishing relevance. The strongest signal is in direct links. Multi-hop is a future v2 enhancement.

4. **Deterministic ranking (no LLM).** Composite score: direct links (3.0) + backlinks (2.0) + shared tags (1.0) + chunk boost (0.5). Transparent, debuggable, fast. Matches operator constraint: "Prefer deterministic graph retrieval and ranking before LLM summarization."

5. **Max 8 neighbors surfaced per selection.** Balances context breadth with prompt budget (RAG_MAX_CHARS = 2000). Users typically accept 2-4 suggestions.

6. **Shared-tag edges capped at 10 per node.** Prevents popular tags like `#ideas` from creating fully connected subgraphs and drowning out explicit link signals.

7. **Additive schema only.** New tables (`note_edges`, `note_tags`) and one `ALTER TABLE` on `vault_provenance_links` for `edge_type`/`edge_label`. No existing columns removed or renamed. Full backward compatibility.

8. **Fail-open on unresolvable links.** If a `[[wikilink]]` target doesn't match any indexed node, the link is logged at debug level and skipped. The current note-centric retrieval path continues to work.

## Verification

All gap claims were verified by codebase search:
- `note_edges` → 0 matches in source code (only in the planning doc)
- `extract_links` → 0 matches in source code
- `note_tags` → 0 matches in source code
- `[[wikilink]]` parsing in `chunker.rs` → 0 matches (no link extraction exists)

## Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Tag normalization may surface too many low-value shared-tag edges | Medium | Capped at 10 per node. Ranking prioritizes direct links (weight 3.0) over shared tags (weight 1.0). |
| Wikilink resolution is case-insensitive title match — may produce false positives on similar titles | Low | Exact match first. Wikilink display text preserved in `edge_label` for user disambiguation. Fuzzy matching deferred to v2. |
| Large vaults (1000+ notes) could make edge rebuilds slow during full re-index | Medium | Edges are rebuilt per-node during `chunk_node()`, not as a full-vault operation. Incremental by design. |
| Shared-tag edge creation requires scanning `note_tags` for matching tags — could be slow with many tags | Low | Indexed by `(account_id, tag_text)`. SQLite handles this well for < 100K rows. |
| Frontend suggestion panel adds new components — needs careful Svelte 5 runes usage | Low | Session 5 will follow existing Svelte 5 patterns (`$props()`, `$state()`, `$derived()`). Component tests via Vitest. |

## Required Inputs for Session 2

Session 2 ("Link Extraction + Edge Storage") needs:

1. **This session's output** — all 6 docs, especially `graph-rag-architecture.md` Sections 1-2 for extraction patterns and edge schema.

2. **Repository anchors to read:**
   - `crates/tuitbot-core/src/automation/watchtower/chunker.rs` — integration point for calling `extract_links()` from `chunk_node()`
   - `crates/tuitbot-core/src/storage/watchtower/mod.rs` — add `pub mod edges;`
   - `crates/tuitbot-core/src/storage/watchtower/nodes.rs` — understand `find_node_by_path_for()` for link resolution
   - `crates/tuitbot-core/migrations/` — naming convention for new migration file

3. **Decisions to carry forward:**
   - Regex patterns from `graph-rag-architecture.md` Section 1
   - Edge table schema from Section 2
   - Bidirectional edge insertion (forward + backlink)
   - Idempotency pattern: delete existing edges for source node before re-inserting
   - Code fence awareness: skip links inside `` ``` `` blocks
