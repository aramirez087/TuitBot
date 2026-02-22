-- Cursor key-value table for persisting loop state (e.g., since_id values).
CREATE TABLE IF NOT EXISTS cursors (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
