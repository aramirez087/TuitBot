-- Transient storage for Ghostwriter selections sent from the Obsidian plugin.
-- Selections have a 30-minute TTL and are cleaned up hourly.

CREATE TABLE vault_selections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    vault_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    selected_text TEXT NOT NULL,
    heading_context TEXT,
    selection_start_line INTEGER NOT NULL DEFAULT 0,
    selection_end_line INTEGER NOT NULL DEFAULT 0,
    note_title TEXT,
    frontmatter_tags TEXT,
    resolved_node_id INTEGER REFERENCES content_nodes(id),
    resolved_chunk_id INTEGER REFERENCES content_chunks(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_vault_selections_session ON vault_selections(session_id);
CREATE INDEX idx_vault_selections_account ON vault_selections(account_id, created_at);
