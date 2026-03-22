# Graph RAG Architecture: Backlink Synthesizer

This document specifies the concrete storage, extraction, retrieval, ranking, and provenance design for graph-aware Ghostwriter retrieval. There are no TBDs.

## 1. Link Extraction

### When

During `chunk_node()` in `crates/tuitbot-core/src/automation/watchtower/chunker.rs`, after fragment extraction completes. Link extraction runs on the same `body_text` input.

### What

Parse `body_text` for three link types:

1. **Wikilinks:** `[[Target Note]]`, `[[Target Note|Display Text]]`, `[[Target Note#Heading]]`, `[[Target Note#Heading|Display Text]]`
2. **Markdown links to local files:** `[text](./relative/path.md)`, `[text](relative/path.md)`, `[text](path.md)` — only links ending in `.md` or with no extension (assumed note reference). External URLs (starting with `http://` or `https://`) are skipped.
3. **Inline tags:** `#tag-name` — tags that appear in body text (not just frontmatter). Must be preceded by whitespace or start-of-line to avoid false positives on CSS colors, issue numbers, etc.

### Regex Patterns

```rust
// Wikilinks: [[target]] or [[target|display]] or [[target#heading]] or [[target#heading|display]]
static WIKILINK_RE: &str = r"\[\[([^\]\|#]+)(?:#([^\]\|]+))?(?:\|([^\]]+))?\]\]";

// Markdown links to local files: [text](path) where path doesn't start with http
static MD_LINK_RE: &str = r"\[([^\]]+)\]\((?!https?://)([^)]+)\)";

// Inline tags: #tag-name (preceded by whitespace or start-of-line)
static INLINE_TAG_RE: &str = r"(?:^|[\s])#([a-zA-Z][a-zA-Z0-9_/-]*)";
```

### Output Types

```rust
/// A raw link extracted from note body text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLink {
    /// The raw target reference: note title (wikilink), relative path (md link), or tag name.
    pub target_raw: String,
    /// The type of link.
    pub link_type: LinkType,
    /// Display text (wikilink alias or markdown link text).
    pub display_text: Option<String>,
    /// Heading anchor within the target note (wikilink #heading).
    pub heading_anchor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    Wikilink,
    MarkdownLink,
    InlineTag,
}
```

### Code Fence Awareness

Link extraction must skip content inside fenced code blocks (`` ``` ``), just as the heading parser already does. The existing `in_code_block` tracking in `extract_fragments()` will be reused or the link extractor will independently track fences.

### File Location

New function `extract_links(body: &str) -> Vec<RawLink>` in a new file `crates/tuitbot-core/src/automation/watchtower/link_extractor.rs` (keeping `chunker.rs` under the 500-line limit).

## 2. Edge Resolution and Storage

### New Table: `note_edges`

```sql
CREATE TABLE IF NOT EXISTS note_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    source_node_id INTEGER NOT NULL REFERENCES content_nodes(id) ON DELETE CASCADE,
    target_node_id INTEGER NOT NULL REFERENCES content_nodes(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,       -- 'wikilink', 'markdown_link', 'shared_tag', 'backlink'
    edge_label TEXT,               -- display text, tag name, or link alias
    source_chunk_id INTEGER REFERENCES content_chunks(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id, source_node_id, target_node_id, edge_type, edge_label)
);

CREATE INDEX IF NOT EXISTS idx_note_edges_source
    ON note_edges(account_id, source_node_id);
CREATE INDEX IF NOT EXISTS idx_note_edges_target
    ON note_edges(account_id, target_node_id);
```

### Resolution Rules

After `chunk_node()` completes and links are extracted, resolve each `RawLink` to a `content_nodes` row:

| Link Type | Resolution Strategy |
|-----------|-------------------|
| Wikilink | Match `target_raw` against `content_nodes.title` (case-insensitive exact match). If no title match, try `content_nodes.relative_path` ending with `{target_raw}.md`. |
| Markdown link | Normalize the path relative to the source note's directory. Match against `content_nodes.relative_path`. |
| Inline tag | Not resolved to a node directly. Instead, used for tag normalization (Section 3) and shared-tag edge creation. |

### Bidirectional Edges

When inserting a forward edge `A → B` with `edge_type = 'wikilink'`, also insert the reverse edge `B → A` with `edge_type = 'backlink'`. This makes `expand_graph_neighbors()` a single `WHERE source_node_id = ?` query instead of a UNION.

Storage trade-off: ~2x rows for wikilink/markdown edges. Negligible for SQLite (vault sizes are typically < 10K notes).

### Idempotency

Before inserting edges for a node, delete all existing edges where `source_node_id = ?` for that node. This makes re-chunking idempotent. The UNIQUE constraint prevents duplicate edges within a single extraction pass.

### Unresolvable Links

If a wikilink target doesn't match any indexed `content_nodes` row, the link is logged at `debug` level and skipped. No edge is created. This is fail-open: the pipeline continues, and when the target note is eventually indexed, the next re-chunk of the source note will resolve it.

### File Location

New module `crates/tuitbot-core/src/storage/watchtower/edges.rs` with functions:
- `delete_edges_for_source(pool, account_id, source_node_id) -> Result<u64>`
- `insert_edges(pool, account_id, edges: &[NewEdge]) -> Result<()>`
- `get_neighbors(pool, account_id, node_id, max_results) -> Result<Vec<GraphNeighbor>>`

## 3. Tag Normalization

### New Table: `note_tags`

```sql
CREATE TABLE IF NOT EXISTS note_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    node_id INTEGER NOT NULL REFERENCES content_nodes(id) ON DELETE CASCADE,
    tag_text TEXT NOT NULL,          -- lowercased, trimmed, no leading #
    source TEXT NOT NULL DEFAULT 'frontmatter',  -- 'frontmatter' or 'inline'
    UNIQUE(account_id, node_id, tag_text)
);

CREATE INDEX IF NOT EXISTS idx_note_tags_tag
    ON note_tags(account_id, tag_text);
CREATE INDEX IF NOT EXISTS idx_note_tags_node
    ON note_tags(account_id, node_id);
```

### Normalization Rules

1. Strip leading `#` if present
2. Lowercase
3. Trim whitespace
4. Collapse internal whitespace to single `-`
5. Examples: `#Rust` → `rust`, `# My Tag` → `my-tag`, `distributed-systems` → `distributed-systems`

### When

During ingestion, after frontmatter parsing (for frontmatter tags) and after link extraction (for inline `#tags`).

### Shared-Tag Edge Creation

After normalizing tags for a node, query other nodes in the same account that share any of its tags:

```sql
SELECT DISTINCT nt2.node_id
FROM note_tags nt1
JOIN note_tags nt2
  ON nt1.account_id = nt2.account_id
  AND nt1.tag_text = nt2.tag_text
  AND nt1.node_id != nt2.node_id
WHERE nt1.account_id = ? AND nt1.node_id = ?
```

For each shared-tag pair `(source_node, target_node, tag)`, insert a bidirectional `shared_tag` edge with `edge_label = tag_text`.

**Cap:** Maximum 10 shared-tag edges per node. If more than 10 tag-connected nodes exist, keep the 10 with the most shared tags (GROUP BY target_node, COUNT DESC). This prevents popular tags like `#ideas` from creating a fully connected subgraph.

### Idempotency

Before inserting tags for a node, delete existing `note_tags` rows for that `(account_id, node_id)`. Same pattern as edge idempotency.

### File Location

New module `crates/tuitbot-core/src/storage/watchtower/tags.rs` with functions:
- `delete_tags_for_node(pool, account_id, node_id) -> Result<u64>`
- `insert_tags(pool, account_id, node_id, tags: &[NormalizedTag]) -> Result<()>`
- `find_shared_tag_neighbors(pool, account_id, node_id, max_results) -> Result<Vec<(i64, String)>>`

## 4. Graph Expansion (1-hop)

### New Function

Added to `crates/tuitbot-core/src/context/retrieval.rs`:

```rust
/// A related note discovered via graph expansion.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNeighbor {
    /// The content_nodes.id of the neighbor.
    pub node_id: i64,
    /// Title of the neighbor note (may be None).
    pub node_title: Option<String>,
    /// Relative file path.
    pub relative_path: String,
    /// Primary edge type connecting to this neighbor.
    pub edge_type: String,
    /// Edge label (link display text, tag name, etc.).
    pub edge_label: Option<String>,
    /// Number of distinct edges connecting to this neighbor.
    pub edge_count: u32,
    /// Number of shared tags with the selected note.
    pub shared_tag_count: u32,
    /// Composite ranking score (see Section 5).
    pub score: f64,
    /// Best chunk snippet from this neighbor (120 chars).
    pub snippet: Option<String>,
}
```

### Query

Single SQL query joining `note_edges` with `content_nodes`:

```sql
SELECT
    cn.id AS node_id,
    cn.title AS node_title,
    cn.relative_path,
    ne.edge_type,
    ne.edge_label,
    COUNT(*) AS edge_count,
    COALESCE(shared_tags.tag_count, 0) AS shared_tag_count
FROM note_edges ne
JOIN content_nodes cn ON cn.id = ne.target_node_id
LEFT JOIN (
    SELECT nt2.node_id, COUNT(*) AS tag_count
    FROM note_tags nt1
    JOIN note_tags nt2 ON nt1.tag_text = nt2.tag_text
        AND nt1.account_id = nt2.account_id
        AND nt1.node_id != nt2.node_id
    WHERE nt1.account_id = ? AND nt1.node_id = ?
    GROUP BY nt2.node_id
) shared_tags ON shared_tags.node_id = cn.id
WHERE ne.account_id = ? AND ne.source_node_id = ?
GROUP BY cn.id
ORDER BY edge_count DESC, shared_tag_count DESC
LIMIT ?
```

### Parameters

- `max_neighbors = 8` (configurable via function argument, default 8)
- Account-scoped: always filtered by `account_id`

### Snippet Enrichment

After fetching neighbor nodes, load the best chunk (highest `retrieval_boost`, then lowest `chunk_index`) for each neighbor and truncate to 120 characters. This is a separate query to avoid a complex join.

### File Location

New function `expand_graph_neighbors(pool, account_id, node_id, max_neighbors) -> Result<Vec<GraphNeighbor>>` in `crates/tuitbot-core/src/context/retrieval.rs`.

## 5. Ranking

Related notes are ranked by a deterministic composite score with no LLM involvement:

```
score = (direct_link_weight × direct_link_count)
      + (backlink_weight × backlink_count)
      + (shared_tag_weight × shared_tag_count)
      + (chunk_boost × best_chunk_retrieval_boost)
```

### Weights

| Factor | Weight | Rationale |
|--------|--------|-----------|
| Direct link (wikilink or markdown link from selected note) | 3.0 | Strongest signal: user explicitly linked these notes |
| Backlink (target note links back to selected note) | 2.0 | Strong signal: bidirectional connection |
| Shared tag | 1.0 | Weaker signal: topical overlap but no explicit connection |
| Best chunk retrieval boost | 0.5 | Tie-breaker: notes with higher-boosted chunks rank higher |

### Tie-Breaking

When scores are equal, sort by `edge_count DESC`, then `node_id ASC` (deterministic).

### Implementation

Scoring is computed in Rust after the SQL query returns raw edge counts. This keeps the SQL simple and the ranking logic testable in isolation.

```rust
fn compute_neighbor_score(
    direct_links: u32,    // wikilink + markdown_link edges where source = selected
    backlinks: u32,       // backlink edges where source = selected
    shared_tags: u32,
    best_chunk_boost: f64,
) -> f64 {
    (3.0 * direct_links as f64)
        + (2.0 * backlinks as f64)
        + (1.0 * shared_tags as f64)
        + (0.5 * best_chunk_boost)
}
```

## 6. Provenance Extension

### ALTER TABLE

Add two nullable columns to `vault_provenance_links`:

```sql
ALTER TABLE vault_provenance_links ADD COLUMN edge_type TEXT;
ALTER TABLE vault_provenance_links ADD COLUMN edge_label TEXT;
```

These columns are nullable to maintain backward compatibility — existing provenance rows (from pre-graph retrieval) will have `NULL` for both.

### ProvenanceRef Extension

```rust
pub struct ProvenanceRef {
    pub node_id: Option<i64>,
    pub chunk_id: Option<i64>,
    pub seed_id: Option<i64>,
    pub source_path: Option<String>,
    pub heading_path: Option<String>,
    pub snippet: Option<String>,
    // New fields:
    pub edge_type: Option<String>,    // "wikilink", "backlink", "shared_tag", "markdown_link"
    pub edge_label: Option<String>,   // link display text or tag name
}
```

### How Provenance Flows

1. User accepts related note N with `edge_type = "wikilink"` and `edge_label = "CAP Theorem"`
2. Hooks/thread generation includes chunks from note N
3. When the generated content is saved (to `approval_queue` or `scheduled_content`), provenance links are created with:
   - `node_id` = N's node ID
   - `chunk_id` = the specific chunk used
   - `source_path` = N's `relative_path`
   - `heading_path` = the chunk's heading hierarchy
   - `snippet` = 120-char excerpt
   - `edge_type` = "wikilink"
   - `edge_label` = "CAP Theorem"

This preserves the "why was this note included?" reason through the entire content lifecycle.

## 7. New API Endpoint

### `GET /api/vault/notes/{node_id}/neighbors`

Returns ranked related notes for a given content node.

**Request:**
```
GET /api/vault/notes/42/neighbors?max=8
Authorization: Bearer {token}
```

**Response:**
```json
{
  "node_id": 42,
  "neighbors": [
    {
      "node_id": 55,
      "node_title": "CAP Theorem",
      "relative_path": "notes/cap-theorem.md",
      "edge_type": "wikilink",
      "edge_label": "CAP Theorem",
      "edge_count": 2,
      "shared_tag_count": 1,
      "score": 7.5,
      "snippet": "The CAP theorem states that a distributed system can only guarantee two of three..."
    }
  ],
  "total_edges": 12
}
```

**Privacy:** Response includes note titles and snippets (120 chars) — both are metadata/excerpts, consistent with existing privacy rules. No raw `body_text`.

## 8. Fallback Behavior

| Scenario | Behavior |
|----------|----------|
| Selected note has no edges | Standard retrieval (today's behavior). No suggestion panel. Info label shown. |
| Edges exist but target notes aren't indexed | Skip unresolvable targets. Log at `debug` level. Surface only indexed neighbors. |
| All neighbors are dismissed by user | Fall back to selected-note-only context. Proceed with hook generation. |
| Graph expansion query fails | Fail open. Log error. Proceed with standard retrieval. Warning label shown. |
| Node not found in `content_nodes` | Standard flow with `selected_text` as direct context. Info label: "This note hasn't been indexed yet." |
| `note_edges` table is empty (new install, no re-chunk yet) | Same as "no edges" — standard retrieval. Graph populates as notes are re-chunked. |

All existing `/api/assist/*` endpoints continue to work without the `graph_neighbors` parameter. The graph expansion is additive — it enriches the `selected_node_ids` list but never replaces it.

## 9. Data Flow Summary

```
Ingestion (write-time):
  note body → extract_fragments() → content_chunks
            → extract_links()     → RawLink[]
            → resolve to content_nodes → note_edges (wikilink, markdown_link, backlink)
            → normalize tags      → note_tags
            → find shared-tag neighbors → note_edges (shared_tag)

Retrieval (read-time):
  selected node_id → expand_graph_neighbors() → GraphNeighbor[] (ranked)
                   → user accepts/dismisses
                   → accepted node_ids added to selected_node_ids
                   → build_draft_context_with_selection() (existing)
                   → DraftContext with expanded vault_citations
                   → provenance with edge_type/edge_label
```
