-- Vault Domain Foundation: content_chunks, provenance links, account-scoped chunk_id

-- Fragment table: heading-delimited sections of content_nodes
CREATE TABLE IF NOT EXISTS content_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    node_id INTEGER NOT NULL REFERENCES content_nodes(id) ON DELETE CASCADE,
    heading_path TEXT NOT NULL DEFAULT '',
    chunk_text TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    chunk_index INTEGER NOT NULL DEFAULT 0,
    retrieval_boost REAL NOT NULL DEFAULT 1.0,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_chunks_node
    ON content_chunks(account_id, node_id);
CREATE INDEX IF NOT EXISTS idx_content_chunks_status
    ON content_chunks(account_id, status);
CREATE INDEX IF NOT EXISTS idx_content_chunks_hash
    ON content_chunks(node_id, chunk_hash);

-- Provenance: link draft_seeds to specific chunks
ALTER TABLE draft_seeds ADD COLUMN chunk_id INTEGER REFERENCES content_chunks(id);

-- Provenance: link approval_queue items back to source material
ALTER TABLE approval_queue ADD COLUMN source_node_id INTEGER REFERENCES content_nodes(id);
ALTER TABLE approval_queue ADD COLUMN source_seed_id INTEGER REFERENCES draft_seeds(id);
ALTER TABLE approval_queue ADD COLUMN source_chunks_json TEXT DEFAULT '[]';
