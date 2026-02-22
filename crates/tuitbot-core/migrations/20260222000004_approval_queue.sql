-- Approval queue for human-in-the-loop review before posting.
CREATE TABLE IF NOT EXISTS approval_queue (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    action_type     TEXT NOT NULL,           -- 'reply', 'tweet', 'thread_tweet'
    target_tweet_id TEXT DEFAULT '',         -- tweet being replied to (for replies)
    target_author   TEXT DEFAULT '',         -- author of the target tweet
    generated_content TEXT NOT NULL,         -- the generated text to post
    topic           TEXT DEFAULT '',         -- topic used for generation
    archetype       TEXT DEFAULT '',         -- reply archetype or tweet format used
    score           REAL DEFAULT 0.0,       -- relevance score (for replies)
    status          TEXT NOT NULL DEFAULT 'pending', -- pending, approved, rejected, expired
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    reviewed_at     TEXT DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_approval_queue_status ON approval_queue(status);
CREATE INDEX IF NOT EXISTS idx_approval_queue_created ON approval_queue(created_at);
