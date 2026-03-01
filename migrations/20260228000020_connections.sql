-- Remote sync connections for account-linked content sources.
--
-- Stores connector metadata and encrypted credentials for user-account
-- OAuth flows (Google Drive, future: OneDrive, Dropbox, Notion).
-- The encrypted_credentials column holds AES-256-GCM encrypted refresh
-- tokens; it is never exposed in API responses.

CREATE TABLE IF NOT EXISTS connections (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id            TEXT    NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    connector_type        TEXT    NOT NULL,          -- "google_drive", future: "onedrive", etc.
    account_email         TEXT,                      -- Display only (user's linked account email)
    display_name          TEXT,                      -- e.g. "My Google Drive"
    encrypted_credentials BLOB,                      -- Refresh token, encrypted at rest
    status                TEXT    NOT NULL DEFAULT 'active', -- "active", "expired", "revoked"
    metadata_json         TEXT    NOT NULL DEFAULT '{}',     -- Non-secret connector metadata
    created_at            TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_connections_type_status
    ON connections(connector_type, status);
