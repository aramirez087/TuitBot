# Implementation Map: Backlink Synthesizer

8-session breakdown with file-level anchors and dependency chains.

## Session Dependency Graph

```
S1 (Charter + Audit)
 │
 ▼
S2 (Link Extraction + Edge Storage)
 │
 ├──▶ S3 (Tag Normalization + Shared-Tag Edges)
 │     │
 │     ▼
 ├──▶ S4 (Graph Expansion + Ranking API)
 │     │
 │     ▼
 │    S5 (Suggestion Cards UX)
 │     │
 │     ▼
 └──▶ S6 (Thread-Slot Insertion + Accept/Dismiss)
       │
       ▼
      S7 (Provenance Extension + Polish)
       │
       ▼
      S8 (Validation + Release Readiness)
```

## Sessions

### Session 1: Charter + Architecture (this session)

**Goal:** Define product charter, architecture, UX journey, and implementation roadmap.

**Files created:**
- `docs/roadmap/backlink-synthesizer/current-state-audit.md`
- `docs/roadmap/backlink-synthesizer/epic-charter.md`
- `docs/roadmap/backlink-synthesizer/end-to-end-ux-journey.md`
- `docs/roadmap/backlink-synthesizer/graph-rag-architecture.md`
- `docs/roadmap/backlink-synthesizer/implementation-map.md`
- `docs/roadmap/backlink-synthesizer/session-01-handoff.md`

**Entry criteria:** None (first session).
**Exit criteria:** All 6 docs complete with no TBDs. Audit names real gaps verified by codebase search.

---

### Session 2: Link Extraction + Edge Storage

**Goal:** Extract wikilinks and markdown links from note bodies during chunking and persist as edges.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/src/automation/watchtower/link_extractor.rs` | Create | `extract_links(body) -> Vec<RawLink>` with regex for wikilinks, md links, inline tags |
| `crates/tuitbot-core/src/automation/watchtower/mod.rs` | Modify | Add `pub mod link_extractor;` |
| `crates/tuitbot-core/src/automation/watchtower/chunker.rs` | Modify | Call `extract_links()` after `extract_fragments()` in `chunk_node()` |
| `crates/tuitbot-core/src/storage/watchtower/edges.rs` | Create | CRUD for `note_edges`: `insert_edges()`, `delete_edges_for_source()`, `get_neighbors()` |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Modify | Add `pub mod edges;` and re-export |
| `crates/tuitbot-core/migrations/NNNN_note_edges.sql` | Create | `note_edges` table DDL + indexes |

**Entry criteria:** Session 1 docs complete (this session's output).
**Exit criteria:**
- `extract_links()` correctly parses wikilinks, md links, inline tags (unit tests)
- `chunk_node()` calls link extraction and creates edges (integration test)
- Unresolvable links are logged and skipped (fail-open verified)
- All CI checks pass

---

### Session 3: Tag Normalization + Shared-Tag Edges

**Goal:** Normalize frontmatter and inline tags into a queryable index, then create shared-tag edges.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/src/storage/watchtower/tags.rs` | Create | CRUD for `note_tags`: `insert_tags()`, `delete_tags_for_node()`, `find_shared_tag_neighbors()` |
| `crates/tuitbot-core/src/storage/watchtower/mod.rs` | Modify | Add `pub mod tags;` and re-export |
| `crates/tuitbot-core/src/automation/watchtower/chunker.rs` | Modify | After chunk + link extraction, normalize tags and insert into `note_tags`, create shared-tag edges |
| `crates/tuitbot-core/migrations/NNNN_note_tags.sql` | Create | `note_tags` table DDL + indexes |

**Entry criteria:** Session 2 complete (link extraction + edge storage working).
**Exit criteria:**
- Frontmatter tags normalized and stored in `note_tags` (unit test)
- Inline tags extracted and stored (unit test)
- Shared-tag edges created with 10-per-node cap (integration test)
- Re-chunking is idempotent (tags and edges are replaced, not duplicated)
- All CI checks pass

---

### Session 4: Graph Expansion + Ranking API

**Goal:** Add `expand_graph_neighbors()` to retrieval and expose via API endpoint.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/src/context/retrieval.rs` | Modify | Add `GraphNeighbor` struct, `expand_graph_neighbors()`, `compute_neighbor_score()` |
| `crates/tuitbot-server/src/routes/vault/mod.rs` | Modify | Add `GET /api/vault/notes/{id}/neighbors` route |
| `crates/tuitbot-server/src/routes/vault/neighbors.rs` | Create | Handler for the neighbors endpoint |

**Entry criteria:** Sessions 2 and 3 complete (edges and tags populated).
**Exit criteria:**
- `expand_graph_neighbors()` returns ranked neighbors with correct scores (unit test with mocked data)
- API endpoint returns JSON matching the spec in `graph-rag-architecture.md` Section 7
- Empty graph returns empty neighbors array (not an error)
- Account scoping enforced (integration test)
- All CI checks pass

---

### Session 5: Suggestion Cards UX

**Goal:** Build the frontend component for displaying and interacting with related-note suggestions.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `dashboard/src/lib/components/composer/RelatedNoteSuggestions.svelte` | Create | Suggestion card panel with accept/dismiss controls |
| `dashboard/src/lib/components/composer/SuggestionCard.svelte` | Create | Individual card: title, reason badge, snippet, include/skip buttons |
| `dashboard/src/lib/components/composer/IncludedNoteChip.svelte` | Create | Compact chip for accepted notes |
| `dashboard/src/lib/stores/suggestions.ts` | Create | Svelte 5 store for suggestion state (accepted, dismissed, pending) |
| `dashboard/src/lib/api/client.ts` | Modify | Add `getNeighbors(nodeId)` API call |
| `dashboard/src/lib/api/types.ts` | Modify | Add `GraphNeighbor` type |
| `dashboard/src/routes/(app)/compose/+page.svelte` | Modify | Integrate suggestion panel into compose flow |

**Entry criteria:** Session 4 complete (API endpoint working).
**Exit criteria:**
- Suggestion cards render with correct reason badges (Vitest component tests)
- Accept/dismiss state management works correctly (store tests)
- Empty graph shows fallback label, not an error
- Cards ordered by score
- Frontend tests pass (`npx vitest run`)
- `npm run check` passes

---

### Session 6: Thread-Slot Insertion + Accept/Dismiss Backend

**Goal:** Pass accepted neighbor node_ids through the hook/thread generation pipeline.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-server/src/routes/rag_helpers.rs` | Modify | Accept `neighbor_node_ids` in RAG context resolution, merge with selected node IDs |
| `crates/tuitbot-core/src/context/winning_dna/analysis.rs` | Modify | `build_draft_context_with_selection()` receives expanded node ID list |
| `crates/tuitbot-server/src/routes/content/compose/mod.rs` | Modify | Pass accepted neighbor IDs from request to RAG helpers |
| `dashboard/src/lib/utils/composeHandlers.ts` | Modify | Include accepted neighbor node_ids in compose API calls |

**Entry criteria:** Sessions 4 and 5 complete (API + UX working).
**Exit criteria:**
- Hook generation includes chunks from accepted neighbor notes (integration test)
- Dismissed notes are excluded from `selected_node_ids` (unit test)
- Thread generation draws from multiple notes with per-slot attribution
- Existing compose endpoints continue to work without `neighbor_node_ids` (backward compat test)
- All CI checks pass

---

### Session 7: Provenance Extension + Polish

**Goal:** Add graph provenance fields, finalize citation display, and polish UX copy.

**Files created/modified:**
| File | Action | Purpose |
|------|--------|---------|
| `crates/tuitbot-core/migrations/NNNN_provenance_edge_fields.sql` | Create | `ALTER TABLE vault_provenance_links ADD COLUMN edge_type TEXT` + `edge_label` |
| `crates/tuitbot-core/src/storage/provenance.rs` | Modify | Add `edge_type` and `edge_label` to `ProvenanceRef` and `ProvenanceLink`, update INSERT/SELECT |
| `crates/tuitbot-core/src/context/retrieval.rs` | Modify | `citations_to_provenance_refs()` includes `edge_type`/`edge_label` |
| `dashboard/src/lib/components/composer/ComposerInspector.svelte` | Modify | Display `edge_type` reason label in provenance panel |
| `dashboard/src/lib/api/types.ts` | Modify | Add `edge_type`/`edge_label` to provenance types |

**Entry criteria:** Session 6 complete (end-to-end flow working).
**Exit criteria:**
- Provenance rows include `edge_type` and `edge_label` for graph-sourced content
- Existing provenance rows have NULL for new fields (backward compat)
- Citation chips in UI show reason labels
- Return-to-source deep links work on Desktop
- All UX copy matches the `end-to-end-ux-journey.md` spec
- All CI checks pass

---

### Session 8: Validation + Release Readiness

**Goal:** Ensure test coverage, CI stability, privacy audit, and release documentation.

**Tasks:**
| Task | Target |
|------|--------|
| Rust coverage (core crates) | 75%+ lines |
| Rust coverage (tuitbot-mcp) | 60%+ lines |
| Frontend coverage (global) | 70%+ lines |
| Frontend coverage (stores) | 75%+ lines |
| Privacy audit | All 10 invariants verified |
| Manual smoke test | Full flow: Obsidian → suggestion cards → thread with provenance |
| Cross-platform CI | macOS, Linux, Windows all green |
| Release notes draft | Changelog entry for the Backlink Synthesizer feature |

**Entry criteria:** Session 7 complete (all features implemented and polished).
**Exit criteria:**
- All coverage thresholds met
- CI green on all platforms
- Privacy audit documented
- Release-plz packaging validates cleanly
- Manual smoke test passes end-to-end
