-- Schema: Analytics depth signals (engagement rate, reach, follower growth, best-time-to-post)
-- Supports charting with time-series data for Dashboard

-- Table: Engagement metrics per post (populated by background X API fetch job)
CREATE TABLE IF NOT EXISTS engagement_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    post_id TEXT NOT NULL,
    tweet_id TEXT,  -- X API tweet ID
    impressions INTEGER DEFAULT 0,
    likes INTEGER DEFAULT 0,
    retweets INTEGER DEFAULT 0,
    replies INTEGER DEFAULT 0,
    bookmarks INTEGER DEFAULT 0,
    engagement_rate REAL DEFAULT 0.0,  -- (likes + retweets + replies + bookmarks) / impressions
    posted_at TEXT,  -- ISO-8601 UTC timestamp when tweet was posted
    fetched_at TEXT NOT NULL,  -- When metrics were last fetched from X API
    created_at TEXT NOT NULL,
    UNIQUE(account_id, post_id),
    FOREIGN KEY(account_id) REFERENCES accounts(id)
);

CREATE INDEX IF NOT EXISTS idx_engagement_metrics_account_posted 
    ON engagement_metrics(account_id, posted_at DESC);

CREATE INDEX IF NOT EXISTS idx_engagement_metrics_account_engagement 
    ON engagement_metrics(account_id, engagement_rate DESC);

-- Table: Best-time-to-post rankings (populated by background aggregation job)
CREATE TABLE IF NOT EXISTS best_times (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    hour_of_day INTEGER NOT NULL,  -- 0-23 (UTC)
    day_of_week INTEGER NOT NULL,  -- 0=Sunday, 6=Saturday
    avg_engagement REAL DEFAULT 0.0,  -- Average engagement_rate for posts at this time slot
    confidence_score REAL DEFAULT 0.0,  -- 0-100 (higher = more historical data)
    sample_size INTEGER DEFAULT 0,  -- Number of posts in this time slot
    last_updated TEXT NOT NULL,
    UNIQUE(account_id, hour_of_day, day_of_week),
    FOREIGN KEY(account_id) REFERENCES accounts(id)
);

CREATE INDEX IF NOT EXISTS idx_best_times_account_engagement 
    ON best_times(account_id, avg_engagement DESC);

-- Table: Reach aggregations (daily snapshots for time-series charting)
-- Derived from engagement_metrics, grouped by date and account
CREATE TABLE IF NOT EXISTS reach_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    snapshot_date TEXT NOT NULL,  -- YYYY-MM-DD
    total_reach INTEGER DEFAULT 0,  -- Sum of impressions for posts on this date
    avg_reach_per_post REAL DEFAULT 0.0,  -- Average impressions per post
    post_count INTEGER DEFAULT 0,  -- Number of posts on this date
    created_at TEXT NOT NULL,
    UNIQUE(account_id, snapshot_date),
    FOREIGN KEY(account_id) REFERENCES accounts(id)
);

CREATE INDEX IF NOT EXISTS idx_reach_snapshots_account_date 
    ON reach_snapshots(account_id, snapshot_date DESC);
