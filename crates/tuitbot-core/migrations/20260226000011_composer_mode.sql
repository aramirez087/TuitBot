-- Composer mode support: posted_tweet_id on approval_queue, source/draft support on scheduled_content.

-- Track which X tweet ID was posted for approved items.
ALTER TABLE approval_queue ADD COLUMN posted_tweet_id TEXT DEFAULT NULL;

-- Track how scheduled content was created (manual, assist, discovery).
ALTER TABLE scheduled_content ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
