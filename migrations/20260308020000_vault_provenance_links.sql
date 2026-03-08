-- Provenance link table: maps any content record to vault source material.
-- entity_type + entity_id form a polymorphic FK (e.g., 'approval_queue' + 42).
CREATE TABLE IF NOT EXISTS vault_provenance_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    entity_type TEXT NOT NULL,          -- 'approval_queue', 'scheduled_content', 'original_tweet', 'thread'
    entity_id INTEGER NOT NULL,
    node_id INTEGER REFERENCES content_nodes(id),
    chunk_id INTEGER REFERENCES content_chunks(id),
    seed_id INTEGER REFERENCES draft_seeds(id),
    source_path TEXT,                   -- snapshot of relative_path at creation
    heading_path TEXT,                  -- snapshot of heading hierarchy
    snippet TEXT,                       -- snapshot of chunk excerpt
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_provenance_entity
    ON vault_provenance_links(account_id, entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_provenance_node
    ON vault_provenance_links(account_id, node_id);
