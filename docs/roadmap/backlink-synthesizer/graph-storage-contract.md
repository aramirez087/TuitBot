# Graph Storage Contract

## Tables

### `note_edges`

Stores directed edges between content nodes extracted from wikilinks, markdown links, shared tags, and auto-generated backlinks.

```sql
CREATE TABLE note_edges (
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
```

**Indexes:** `(account_id, source_node_id)`, `(account_id, target_node_id)`

### `note_tags`

Normalized tags extracted from frontmatter and inline `#tags`.

```sql
CREATE TABLE note_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    node_id INTEGER NOT NULL REFERENCES content_nodes(id) ON DELETE CASCADE,
    tag_text TEXT NOT NULL,          -- lowercased, trimmed, no leading #
    source TEXT NOT NULL DEFAULT 'frontmatter',  -- 'frontmatter' or 'inline'
    UNIQUE(account_id, node_id, tag_text)
);
```

**Indexes:** `(account_id, tag_text)`, `(account_id, node_id)`

### `vault_provenance_links` (extended)

Two nullable columns added: `edge_type TEXT`, `edge_label TEXT`. Backward compatible.

## Idempotency Contract

When a node is re-chunked (`chunk_node()`), the graph ingest pipeline:

1. **Deletes** all edges where `source_node_id = node_id` (covers forward links, shared-tag edges, and backlink edges this node created).
2. **Deletes** all tags where `node_id = node_id`.
3. **Re-extracts** links and tags from the note body.
4. **Re-inserts** tags, then resolves links to edges, then creates shared-tag edges.

This delete-before-insert pattern ensures stale links are removed when a note changes. Backlink edges auto-created by OTHER nodes pointing to this node are also deleted by step 1 (since those backlinks have `source_node_id = other_node`... wait, actually backlinks from A→B have `source_node_id = B`). The simplification: only edges with `source_node_id = this_node` are deleted, which covers this node's own forward edges and the reverse backlink edges it created. Backlinks that other nodes created pointing to this node (where `source_node_id = other_node`) are NOT deleted — they remain until those other nodes are re-chunked.

**Temporary inconsistency:** After re-chunking node A, backlink edges from other nodes to A may briefly be stale. These are restored when those other nodes are re-chunked. For full re-index, all nodes are re-chunked, restoring full consistency.

## Resolution Rules

### Wikilinks (`[[Target]]`)

1. Case-insensitive exact title match: `LOWER(title) = LOWER(target)`
2. Path ending match: `relative_path = 'target.md'` OR `relative_path LIKE '%/target.md'`
3. If no match: skip (fail-open, logged at debug level)

### Markdown links (`[text](path.md)`)

1. Get source node's `relative_path` to determine its directory
2. Strip leading `./`, join with source directory, resolve `..` components
3. Match normalized path against `content_nodes.relative_path`
4. If no match: skip (fail-open)

### Inline tags (`#tag-name`)

- Tag regex requires first char after `#` to be `[a-zA-Z]` (skips numeric hex like `#123456`)
- Tags glued to a word without preceding whitespace are not extracted
- Known limitation: alphabetic hex colors like `#ff0000` may be false positives

## Account Scoping

All queries are filtered by `account_id`. Edges and tags from account A are never visible to account B.

## Cascade Behavior

- `ON DELETE CASCADE` on `source_node_id` and `target_node_id`: deleting a content node automatically removes all its edges.
- `ON DELETE CASCADE` on `node_id` in `note_tags`: deleting a node removes its tags.

## Backward Compatibility

- Existing `content_nodes.tags` column is unchanged. `note_tags` is additive.
- Notes without links or tags produce zero edges and zero tag rows — the pipeline path is identical to before.
- The `extract_and_persist_graph()` call in `chunk_node()` is fail-open: errors are logged but do not prevent chunking from completing.
