-- Watchtower ingestion tables for the Cold-Start Watchtower RAG epic.

-- Registered content sources
CREATE TABLE IF NOT EXISTS source_contexts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    source_type TEXT NOT NULL,
    config_json TEXT NOT NULL DEFAULT '{}',
    sync_cursor TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Ingested content chunks from sources
CREATE TABLE IF NOT EXISTS content_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    source_id INTEGER NOT NULL REFERENCES source_contexts(id),
    relative_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    title TEXT,
    body_text TEXT NOT NULL,
    front_matter_json TEXT,
    tags TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    ingested_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_id, relative_path)
);

-- Pre-computed draft seeds (hooks/angles from content nodes)
CREATE TABLE IF NOT EXISTS draft_seeds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    node_id INTEGER NOT NULL REFERENCES content_nodes(id),
    seed_text TEXT NOT NULL,
    archetype_suggestion TEXT,
    engagement_weight REAL NOT NULL DEFAULT 0.5,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    used_at TEXT
);

-- Indexes for new tables
CREATE INDEX IF NOT EXISTS idx_content_nodes_source ON content_nodes(source_id, status);
CREATE INDEX IF NOT EXISTS idx_content_nodes_hash ON content_nodes(content_hash);
CREATE INDEX IF NOT EXISTS idx_draft_seeds_status ON draft_seeds(status, engagement_weight DESC);
CREATE INDEX IF NOT EXISTS idx_draft_seeds_node ON draft_seeds(node_id);

-- Additive columns on existing tables
ALTER TABLE tweet_performance ADD COLUMN archetype_vibe TEXT;
ALTER TABLE reply_performance ADD COLUMN archetype_vibe TEXT;
ALTER TABLE tweet_performance ADD COLUMN engagement_score REAL;
ALTER TABLE reply_performance ADD COLUMN engagement_score REAL;
ALTER TABLE original_tweets ADD COLUMN source_node_id INTEGER REFERENCES content_nodes(id);
