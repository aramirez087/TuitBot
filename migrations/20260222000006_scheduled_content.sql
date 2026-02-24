-- Manually composed content with optional scheduling.
CREATE TABLE IF NOT EXISTS scheduled_content (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,          -- 'tweet' or 'thread'
    content TEXT NOT NULL,              -- text for tweet, JSON array for thread
    scheduled_for TEXT,                 -- ISO8601 timestamp, NULL = next available slot
    status TEXT NOT NULL DEFAULT 'scheduled',  -- scheduled, posted, cancelled
    posted_tweet_id TEXT,              -- filled after posting
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
