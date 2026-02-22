-- WP15: Analytics tables for performance tracking.

-- Daily follower count snapshots.
CREATE TABLE IF NOT EXISTS follower_snapshots (
    snapshot_date TEXT PRIMARY KEY,
    follower_count INTEGER NOT NULL DEFAULT 0,
    following_count INTEGER NOT NULL DEFAULT 0,
    tweet_count INTEGER NOT NULL DEFAULT 0
);

-- Performance metrics for sent replies.
CREATE TABLE IF NOT EXISTS reply_performance (
    reply_id TEXT PRIMARY KEY,
    likes_received INTEGER NOT NULL DEFAULT 0,
    replies_received INTEGER NOT NULL DEFAULT 0,
    impressions INTEGER NOT NULL DEFAULT 0,
    performance_score REAL NOT NULL DEFAULT 0.0,
    measured_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Performance metrics for original tweets.
CREATE TABLE IF NOT EXISTS tweet_performance (
    tweet_id TEXT PRIMARY KEY,
    likes_received INTEGER NOT NULL DEFAULT 0,
    retweets_received INTEGER NOT NULL DEFAULT 0,
    replies_received INTEGER NOT NULL DEFAULT 0,
    impressions INTEGER NOT NULL DEFAULT 0,
    performance_score REAL NOT NULL DEFAULT 0.0,
    measured_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Running averages of content performance by topic and format.
CREATE TABLE IF NOT EXISTS content_scores (
    topic TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT '',
    total_posts INTEGER NOT NULL DEFAULT 0,
    avg_performance REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (topic, format)
);
