-- Add retry tracking columns to threads table for dead-letter queue support
-- Enables exponential backoff and failure classification (transient vs permanent)

-- Add retry_count, last_error, failed_at, and failure_kind columns to threads
ALTER TABLE threads ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE threads ADD COLUMN last_error TEXT;
ALTER TABLE threads ADD COLUMN failed_at TEXT;
ALTER TABLE threads ADD COLUMN failure_kind TEXT; -- 'transient' or 'permanent'

-- Index for dead-letter queue queries (find failed threads to retry)
CREATE INDEX IF NOT EXISTS idx_threads_failure_kind_retry_count 
ON threads(failure_kind, retry_count) WHERE failure_kind = 'transient' AND retry_count < 3;

-- Also add similar columns to original_tweets for single-tweet failures
ALTER TABLE original_tweets ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE original_tweets ADD COLUMN last_error TEXT;
ALTER TABLE original_tweets ADD COLUMN failed_at TEXT;
ALTER TABLE original_tweets ADD COLUMN failure_kind TEXT;

-- Index for dead-letter query on original_tweets
CREATE INDEX IF NOT EXISTS idx_original_tweets_failure_kind_retry_count
ON original_tweets(failure_kind, retry_count) WHERE failure_kind = 'transient' AND retry_count < 3;
