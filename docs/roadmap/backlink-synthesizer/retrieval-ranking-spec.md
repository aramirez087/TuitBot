# Retrieval Ranking Specification

## Ranking Formula

Each graph neighbor receives a composite score:

```
score = 3.0 * direct_links + 2.0 * backlinks + 1.0 * shared_tags + 0.5 * best_chunk_boost
```

### Weight Rationale

| Factor | Weight | Rationale |
|--------|--------|-----------|
| Direct links (wikilink, markdown_link) | 3.0 | Strongest signal: author explicitly linked to this note |
| Backlinks | 2.0 | Strong signal: another note references this one |
| Shared tags | 1.0 | Moderate signal: topical overlap via tags |
| Best chunk boost | 0.5 | Mild boost for notes with high-quality indexed content |

### Worked Examples

**Example 1: Direct wikilink only**
- Node B has 1 wikilink from selected node A, chunk boost 1.0
- Score = 3.0*1 + 2.0*0 + 1.0*0 + 0.5*1.0 = **3.5**

**Example 2: Mutual link**
- Node B has 1 wikilink from A AND links back to A, chunk boost 1.5
- Score = 3.0*1 + 2.0*1 + 1.0*0 + 0.5*1.5 = **5.75**

**Example 3: Shared tags only**
- Node B shares 3 tags with A, chunk boost 1.0
- Score = 3.0*0 + 2.0*0 + 1.0*3 + 0.5*1.0 = **3.5**

**Example 4: Mixed signals**
- Node B has 1 wikilink, 1 backlink, 2 shared tags, chunk boost 2.0
- Score = 3.0*1 + 2.0*1 + 1.0*2 + 0.5*2.0 = **8.0**

## Sorting and Tie-Breaking

1. **Primary**: Score descending
2. **Secondary**: Edge count descending (total edges to this neighbor)
3. **Tertiary**: Node ID ascending (deterministic tie-break)

## Diversity Cap

- **MAX_GRAPH_FRAGMENTS_PER_NOTE = 3**: When graph neighbors feed into `retrieve_vault_fragments`, each neighbor contributes at most 3 chunks to the LLM prompt.
- **MAX_FRAGMENTS = 5**: Total fragments across all sources still capped at 5.
- This prevents one verbose neighbor from dominating the context window.

## Neighbor Cap

- Default: **8 neighbors** per expansion
- Configurable via `?max=N` query parameter (capped at 100)
- 1-hop only: no multi-hop traversal

## Fallback Behavior

| Condition | GraphState | Neighbors | Behavior |
|-----------|-----------|-----------|----------|
| Node has edges, neighbors resolved | `available` | Populated | Normal operation |
| Node exists, no edges | `no_related_notes` | Empty | UI shows "no related notes" |
| Node not in content_nodes | `node_not_indexed` | Empty | UI shows "note not indexed yet" |
| Graph expansion query fails | `fallback_active` | Empty | Fall back to current note-centric retrieval |
| Edges exist but all targets missing | `no_related_notes` | Empty | Targets not yet indexed |

## Edge Type Classification

Outgoing edges (`get_edges_for_source`) are classified by `edge_type`:
- `wikilink`, `markdown_link` -> counted as **direct links**
- `backlink` -> counted as **backlinks**
- `shared_tag` -> counted as **shared tags** (label is the tag name)

Incoming edges (`get_edges_for_target`) are classified as:
- `wikilink`, `markdown_link` -> counted as **backlinks** (reverse direction)
- `shared_tag` -> counted as **shared tags**

## Suggestion Reason Classification

| Direct Links | Backlinks | Shared Tags | Reason |
|-------------|-----------|-------------|--------|
| > 0 | > 0 | any | `mutual_link` |
| > 0 | 0 | any | `linked_note` |
| 0 | > 0 | any | `backlink` |
| 0 | 0 | > 0 | `shared_tag` |

## Intent Classification (Heuristic)

Edge labels are scanned for keywords to classify intent:
- `counterpoint`, `vs`, `alternative`, `contrast` -> `counterpoint`
- `tip`, `how-to`, `how to`, `guide` -> `pro_tip`
- `data`, `evidence`, `study`, `stat` -> `evidence`
- All other labels -> `related` (safe default)
