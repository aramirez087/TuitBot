CREATE TABLE IF NOT EXISTS strategy_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    week_start TEXT NOT NULL,
    week_end TEXT NOT NULL,
    -- Output volume
    replies_sent INTEGER NOT NULL DEFAULT 0,
    tweets_posted INTEGER NOT NULL DEFAULT 0,
    threads_posted INTEGER NOT NULL DEFAULT 0,
    target_replies INTEGER NOT NULL DEFAULT 0,
    -- Follower metrics
    follower_start INTEGER NOT NULL DEFAULT 0,
    follower_end INTEGER NOT NULL DEFAULT 0,
    follower_delta INTEGER NOT NULL DEFAULT 0,
    -- Engagement metrics
    avg_reply_score REAL NOT NULL DEFAULT 0.0,
    avg_tweet_score REAL NOT NULL DEFAULT 0.0,
    reply_acceptance_rate REAL NOT NULL DEFAULT 0.0,
    estimated_follow_conversion REAL NOT NULL DEFAULT 0.0,
    -- Structured JSON columns
    top_topics_json TEXT NOT NULL DEFAULT '[]',
    bottom_topics_json TEXT NOT NULL DEFAULT '[]',
    top_content_json TEXT NOT NULL DEFAULT '[]',
    recommendations_json TEXT NOT NULL DEFAULT '[]',
    -- Metadata
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(week_start)
);
