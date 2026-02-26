-- Mutation audit trail for idempotency enforcement and incident review.
--
-- Every mutation-capable MCP tool records an entry here before executing.
-- The table serves dual purposes:
--   1. Idempotency — recent identical mutations are detected and short-circuited.
--   2. Audit — every mutation attempt is traceable via correlation_id.

CREATE TABLE IF NOT EXISTS mutation_audit (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    correlation_id    TEXT    NOT NULL,                      -- UUID v4, unique per attempt
    idempotency_key   TEXT,                                  -- caller-provided key (optional)
    tool_name         TEXT    NOT NULL,                      -- e.g. 'x_post_tweet'
    params_hash       TEXT    NOT NULL,                      -- SHA-256 of canonical params JSON
    params_summary    TEXT    NOT NULL,                      -- truncated JSON for display
    status            TEXT    NOT NULL DEFAULT 'pending',    -- pending | success | failure | duplicate
    result_summary    TEXT,                                  -- truncated JSON of result
    rollback_action   TEXT,                                  -- JSON: {"tool": "...", "params": {...}}
    error_message     TEXT,                                  -- on failure
    elapsed_ms        INTEGER,                               -- wall-clock duration
    account_id        TEXT    NOT NULL DEFAULT 'default',
    created_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at      TEXT
);

-- Fast lookup by caller-provided idempotency key.
CREATE INDEX IF NOT EXISTS idx_mutation_audit_key
    ON mutation_audit(idempotency_key) WHERE idempotency_key IS NOT NULL;

-- Recent mutations by tool name (for "recent writes" queries).
CREATE INDEX IF NOT EXISTS idx_mutation_audit_tool
    ON mutation_audit(tool_name, created_at);

-- Trace a single mutation across systems.
CREATE INDEX IF NOT EXISTS idx_mutation_audit_correlation
    ON mutation_audit(correlation_id);

-- Time-based queries and cleanup.
CREATE INDEX IF NOT EXISTS idx_mutation_audit_created
    ON mutation_audit(created_at);

-- Params-hash lookup for fingerprint-based idempotency.
CREATE INDEX IF NOT EXISTS idx_mutation_audit_hash
    ON mutation_audit(tool_name, params_hash, status, created_at);
