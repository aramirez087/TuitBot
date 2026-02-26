-- Media upload tracking for idempotent re-uploads and agent observability.
-- Tracks every media upload from file hash through X API media_id.
CREATE TABLE IF NOT EXISTS media_uploads (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_hash       TEXT NOT NULL,                    -- SHA-256 of file content
    file_name       TEXT NOT NULL DEFAULT '',         -- Original filename
    file_size_bytes INTEGER NOT NULL,                 -- Size in bytes
    media_type      TEXT NOT NULL,                    -- MIME type (image/jpeg, etc.)
    upload_strategy TEXT NOT NULL DEFAULT 'simple',   -- 'simple' or 'chunked'
    segment_count   INTEGER NOT NULL DEFAULT 1,       -- Number of chunks uploaded
    x_media_id      TEXT DEFAULT NULL,                -- X API media_id_string (null until finalized)
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'uploading', 'processing', 'ready', 'failed', 'expired'
    error_message   TEXT DEFAULT NULL,                -- Error detail on failure
    alt_text        TEXT DEFAULT NULL,                -- Accessibility alt text
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    finalized_at    TEXT DEFAULT NULL,                -- When FINALIZE completed
    expires_at      TEXT DEFAULT NULL                 -- X media IDs expire after 24h
);

-- Index for idempotent lookups by file hash.
CREATE INDEX IF NOT EXISTS idx_media_uploads_hash ON media_uploads(file_hash);

-- Index for finding ready media by X media ID.
CREATE INDEX IF NOT EXISTS idx_media_uploads_x_media_id ON media_uploads(x_media_id);
