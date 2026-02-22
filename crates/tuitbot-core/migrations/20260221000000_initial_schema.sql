-- Initial schema for Tuitbot storage layer
-- All tables match data-model.md exactly

-- Discovered tweets from X search
CREATE TABLE IF NOT EXISTS discovered_tweets (
    id TEXT PRIMARY KEY,
    author_id TEXT NOT NULL,
    author_username TEXT NOT NULL,
    content TEXT NOT NULL,
    like_count INTEGER NOT NULL DEFAULT 0,
    retweet_count INTEGER NOT NULL DEFAULT 0,
    reply_count INTEGER NOT NULL DEFAULT 0,
    impression_count INTEGER DEFAULT 0,
    relevance_score REAL,
    matched_keyword TEXT,
    discovered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    replied_to INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_discovered_tweets_discovered_at ON discovered_tweets(discovered_at);
CREATE INDEX IF NOT EXISTS idx_discovered_tweets_matched_keyword ON discovered_tweets(matched_keyword);
CREATE INDEX IF NOT EXISTS idx_discovered_tweets_replied_score ON discovered_tweets(replied_to, relevance_score DESC);

-- Replies sent by the agent
CREATE TABLE IF NOT EXISTS replies_sent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_tweet_id TEXT NOT NULL,
    reply_tweet_id TEXT,
    reply_content TEXT NOT NULL,
    llm_provider TEXT,
    llm_model TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    status TEXT NOT NULL DEFAULT 'sent',
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_replies_sent_created_at ON replies_sent(created_at);
CREATE INDEX IF NOT EXISTS idx_replies_sent_target_tweet_id ON replies_sent(target_tweet_id);

-- Original educational tweets
CREATE TABLE IF NOT EXISTS original_tweets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tweet_id TEXT,
    content TEXT NOT NULL,
    topic TEXT,
    llm_provider TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    status TEXT NOT NULL DEFAULT 'sent',
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_original_tweets_created_at ON original_tweets(created_at);

-- Educational threads
CREATE TABLE IF NOT EXISTS threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    tweet_count INTEGER NOT NULL DEFAULT 0,
    root_tweet_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    status TEXT NOT NULL DEFAULT 'sent'
);

-- Individual tweets within a thread
CREATE TABLE IF NOT EXISTS thread_tweets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    thread_id INTEGER NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    tweet_id TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(thread_id, position)
);

-- Rate limit tracking per action type
CREATE TABLE IF NOT EXISTS rate_limits (
    action_type TEXT PRIMARY KEY,
    request_count INTEGER NOT NULL DEFAULT 0,
    period_start TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    max_requests INTEGER NOT NULL,
    period_seconds INTEGER NOT NULL
);

-- Append-only audit trail
CREATE TABLE IF NOT EXISTS action_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'success',
    message TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_action_log_created_at ON action_log(created_at);
CREATE INDEX IF NOT EXISTS idx_action_log_type_created ON action_log(action_type, created_at);
