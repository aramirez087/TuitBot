-- WP11: Safety enhancements - per-author interaction tracking

-- Track per-author daily reply counts to prevent spam behavior
CREATE TABLE IF NOT EXISTS author_interactions (
    author_id TEXT NOT NULL,
    author_username TEXT NOT NULL DEFAULT '',
    interaction_date TEXT NOT NULL,
    reply_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (author_id, interaction_date)
);

CREATE INDEX IF NOT EXISTS idx_author_interactions_date ON author_interactions(interaction_date);

-- Add author tracking columns to replies_sent
ALTER TABLE replies_sent ADD COLUMN author_id TEXT DEFAULT '';
ALTER TABLE replies_sent ADD COLUMN author_username TEXT DEFAULT '';
