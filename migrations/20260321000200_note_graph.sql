-- Note graph: link edges and normalized tags for graph-aware retrieval.
-- Additive-only migration — no existing columns removed or renamed.

-- Note-to-note edges extracted from wikilinks, markdown links, and shared tags.
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

-- Normalized tags extracted from note frontmatter and inline #tags.
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

-- Provenance extension: track graph edge origin on provenance links.
ALTER TABLE vault_provenance_links ADD COLUMN edge_type TEXT;
ALTER TABLE vault_provenance_links ADD COLUMN edge_label TEXT;
