-- Semantic embedding storage for content chunks.
-- Stores vector embeddings alongside their source chunk for incremental indexing.

CREATE TABLE IF NOT EXISTS chunk_embeddings (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id        INTEGER NOT NULL REFERENCES content_chunks(id) ON DELETE CASCADE,
    account_id      TEXT    NOT NULL DEFAULT 'default',
    embedding       BLOB    NOT NULL,
    model_id        TEXT    NOT NULL,
    dimension       INTEGER NOT NULL,
    embedding_hash  TEXT    NOT NULL,
    generation      INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(chunk_id, account_id)
);

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_account
    ON chunk_embeddings(account_id);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_generation
    ON chunk_embeddings(account_id, generation);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_model
    ON chunk_embeddings(model_id);
