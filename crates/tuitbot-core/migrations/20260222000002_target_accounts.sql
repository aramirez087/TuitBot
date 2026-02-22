-- WP12: Target Account Monitoring tables.

-- Tracks target accounts configured for relationship-based engagement.
CREATE TABLE IF NOT EXISTS target_accounts (
    account_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    followed_at TEXT,
    first_engagement_at TEXT,
    total_replies_sent INTEGER NOT NULL DEFAULT 0,
    last_reply_at TEXT,
    status TEXT NOT NULL DEFAULT 'active'
);

-- Stores tweets discovered from target accounts.
CREATE TABLE IF NOT EXISTS target_tweets (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES target_accounts(account_id),
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT '',
    conversation_id TEXT,
    reply_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    discovered_at TEXT NOT NULL DEFAULT (datetime('now')),
    replied_to INTEGER NOT NULL DEFAULT 0,
    relevance_score REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_target_tweets_account ON target_tweets(account_id);
CREATE INDEX IF NOT EXISTS idx_target_tweets_replied ON target_tweets(replied_to);
