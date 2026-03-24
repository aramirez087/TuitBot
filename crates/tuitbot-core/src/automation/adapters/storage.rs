//! Storage adapter implementations.

use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

use super::super::analytics_loop::{AnalyticsError, AnalyticsStorage};
use super::super::loop_helpers::{
    ContentLoopError, ContentStorage, LoopError, LoopStorage, LoopTweet, TopicScorer,
};
use super::super::posting_queue::PostAction;
use super::super::target_loop::TargetStorage;
use super::helpers::{parse_datetime, sqlx_to_content_error, storage_to_loop_error};
use crate::storage::{self, DbPool};

/// Adapts `DbPool` to the `LoopStorage` port trait.
///
/// Provides cursor persistence (via the `cursors` table), tweet dedup,
/// discovered tweet recording, and action logging.
pub struct StorageAdapter {
    pool: DbPool,
}

impl StorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LoopStorage for StorageAdapter {
    async fn get_cursor(&self, key: &str) -> Result<Option<String>, LoopError> {
        storage::cursors::get_cursor(&self.pool, key)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn set_cursor(&self, key: &str, value: &str) -> Result<(), LoopError> {
        storage::cursors::set_cursor(&self.pool, key, value)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
        storage::tweets::tweet_exists(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn store_discovered_tweet(
        &self,
        tweet: &LoopTweet,
        score: f32,
        keyword: &str,
    ) -> Result<(), LoopError> {
        let discovered = storage::tweets::DiscoveredTweet {
            id: tweet.id.clone(),
            author_id: tweet.author_id.clone(),
            author_username: tweet.author_username.clone(),
            content: tweet.text.clone(),
            like_count: tweet.likes as i64,
            retweet_count: tweet.retweets as i64,
            reply_count: tweet.replies as i64,
            impression_count: None,
            relevance_score: Some(score as f64),
            matched_keyword: Some(keyword.to_string()),
            discovered_at: Utc::now().to_rfc3339(),
            replied_to: 0,
        };
        storage::tweets::insert_discovered_tweet(&self.pool, &discovered)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(storage_to_loop_error)
    }
}

/// Adapts `DbPool` + posting queue to the `ContentStorage` port trait.
pub struct ContentStorageAdapter {
    pool: DbPool,
    post_tx: mpsc::Sender<PostAction>,
}

impl ContentStorageAdapter {
    pub fn new(pool: DbPool, post_tx: mpsc::Sender<PostAction>) -> Self {
        Self { pool, post_tx }
    }
}

#[async_trait::async_trait]
impl ContentStorage for ContentStorageAdapter {
    async fn last_tweet_time(&self) -> Result<Option<DateTime<Utc>>, ContentLoopError> {
        let time_str = storage::threads::get_last_original_tweet_time(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_str.and_then(|s| parse_datetime(&s)))
    }

    async fn todays_tweet_times(&self) -> Result<Vec<DateTime<Utc>>, ContentLoopError> {
        let time_strs = storage::threads::get_todays_tweet_times(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_strs.iter().filter_map(|s| parse_datetime(s)).collect())
    }

    async fn last_thread_time(&self) -> Result<Option<DateTime<Utc>>, ContentLoopError> {
        let time_str = storage::threads::get_last_thread_time(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(time_str.and_then(|s| parse_datetime(&s)))
    }

    async fn post_tweet(&self, topic: &str, content: &str) -> Result<(), ContentLoopError> {
        // Send to the posting queue and await result.
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        self.post_tx
            .send(PostAction::Tweet {
                content: content.to_string(),
                media_ids: vec![],
                result_tx: Some(result_tx),
            })
            .await
            .map_err(|e| ContentLoopError::PostFailed(e.to_string()))?;

        let tweet_id = result_rx
            .await
            .map_err(|e| ContentLoopError::PostFailed(e.to_string()))?
            .map_err(ContentLoopError::PostFailed)?;

        // Record in the database.
        let original = storage::threads::OriginalTweet {
            id: 0,
            tweet_id: Some(tweet_id),
            content: content.to_string(),
            topic: Some(topic.to_string()),
            llm_provider: None,
            created_at: Utc::now().to_rfc3339(),
            status: "sent".to_string(),
            error_message: None,
        };
        storage::threads::insert_original_tweet(&self.pool, &original)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        // Increment rate limit.
        storage::rate_limits::increment_rate_limit(&self.pool, "tweet")
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn create_thread(
        &self,
        topic: &str,
        tweet_count: usize,
    ) -> Result<String, ContentLoopError> {
        let thread = storage::threads::Thread {
            id: 0,
            topic: topic.to_string(),
            tweet_count: tweet_count as i64,
            root_tweet_id: None,
            created_at: Utc::now().to_rfc3339(),
            status: "pending".to_string(),
        };
        let id = storage::threads::insert_thread(&self.pool, &thread)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(id.to_string())
    }

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: &str,
        tweet_count: usize,
        root_tweet_id: Option<&str>,
    ) -> Result<(), ContentLoopError> {
        let id: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        sqlx::query(
            "UPDATE threads SET status = ?1, tweet_count = ?2, root_tweet_id = ?3 WHERE id = ?4",
        )
        .bind(status)
        .bind(tweet_count as i64)
        .bind(root_tweet_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_content_error)?;

        // If the thread was fully posted, increment the rate limit.
        if status == "sent" {
            storage::rate_limits::increment_rate_limit(&self.pool, "thread")
                .await
                .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        }

        Ok(())
    }

    async fn store_thread_tweet(
        &self,
        thread_id: &str,
        position: usize,
        tweet_id: &str,
        content: &str,
    ) -> Result<(), ContentLoopError> {
        let tid: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        sqlx::query(
            "INSERT INTO thread_tweets (thread_id, position, tweet_id, content, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        )
        .bind(tid)
        .bind(position as i64)
        .bind(tweet_id)
        .bind(content)
        .execute(&self.pool)
        .await
        .map_err(sqlx_to_content_error)?;

        Ok(())
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), ContentLoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))
    }

    async fn next_scheduled_item(&self) -> Result<Option<(i64, String, String)>, ContentLoopError> {
        let items = storage::scheduled_content::get_due_items(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(items
            .into_iter()
            .next()
            .map(|item| (item.id, item.content_type, item.content)))
    }

    async fn mark_scheduled_posted(
        &self,
        id: i64,
        tweet_id: Option<&str>,
    ) -> Result<(), ContentLoopError> {
        storage::scheduled_content::update_status(&self.pool, id, "posted", tweet_id)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))
    }

    async fn mark_failed_permanent(
        &self,
        thread_id: &str,
        error: &str,
    ) -> Result<(), ContentLoopError> {
        let id: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        // Update thread status to failed
        sqlx::query(
            "UPDATE threads SET status = ?1, failure_kind = ?2, last_error = ?3, failed_at = datetime('now') WHERE id = ?4",
        )
        .bind("failed")
        .bind("permanent")
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        // Fetch thread details for approval queue entry
        // Concatenate all thread tweets into a single content string for the approval queue
        let row: (String, u32) =
            sqlx::query_as("SELECT topic, retry_count FROM threads WHERE id = ?1")
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        let (topic, retry_count) = row;

        // Fetch all tweets in the thread and concatenate them
        let tweets: Vec<(String,)> = sqlx::query_as(
            "SELECT content FROM thread_tweets WHERE thread_id = ?1 ORDER BY position",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        let content = if tweets.is_empty() {
            format!("Failed thread id={}", id)
        } else {
            tweets
                .iter()
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>()
                .join("\n---\n")
        };

        // Build metadata JSON for the approval queue entry
        let metadata = format!(
            "Failed thread id={}, retries={}, error: {}",
            id, retry_count, error
        );

        // Insert into approval_queue with status="pending" for human review
        sqlx::query(
            "INSERT INTO approval_queue (action_type, generated_content, topic, status, reason) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind("failed_post_recovery")
        .bind(content)
        .bind(topic)
        .bind("pending")
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn increment_retry(&self, thread_id: &str, error: &str) -> Result<u32, ContentLoopError> {
        let id: i64 = thread_id
            .parse()
            .map_err(|_| ContentLoopError::StorageError("invalid thread_id".to_string()))?;

        // Increment retry_count and update failure metadata
        sqlx::query(
            "UPDATE threads SET retry_count = retry_count + 1, failure_kind = ?1, last_error = ?2, failed_at = datetime('now') WHERE id = ?3",
        )
        .bind("transient")
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        // Fetch updated retry_count
        let row: (i64,) = sqlx::query_as("SELECT retry_count FROM threads WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;

        Ok(row.0 as u32)
    }
}

/// Adapts `DbPool` to the `TargetStorage` port trait.
pub struct TargetStorageAdapter {
    pool: DbPool,
}

impl TargetStorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TargetStorage for TargetStorageAdapter {
    async fn upsert_target_account(
        &self,
        account_id: &str,
        username: &str,
    ) -> Result<(), LoopError> {
        storage::target_accounts::upsert_target_account(&self.pool, account_id, username)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn target_tweet_exists(&self, tweet_id: &str) -> Result<bool, LoopError> {
        storage::target_accounts::target_tweet_exists(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn store_target_tweet(
        &self,
        tweet_id: &str,
        account_id: &str,
        content: &str,
        created_at: &str,
        reply_count: i64,
        like_count: i64,
        relevance_score: f64,
    ) -> Result<(), LoopError> {
        storage::target_accounts::store_target_tweet(
            &self.pool,
            tweet_id,
            account_id,
            content,
            created_at,
            reply_count,
            like_count,
            relevance_score,
        )
        .await
        .map_err(storage_to_loop_error)
    }

    async fn mark_target_tweet_replied(&self, tweet_id: &str) -> Result<(), LoopError> {
        storage::target_accounts::mark_target_tweet_replied(&self.pool, tweet_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn record_target_reply(&self, account_id: &str) -> Result<(), LoopError> {
        storage::target_accounts::record_target_reply(&self.pool, account_id)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn count_target_replies_today(&self) -> Result<i64, LoopError> {
        storage::target_accounts::count_target_replies_today(&self.pool)
            .await
            .map_err(storage_to_loop_error)
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), LoopError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(storage_to_loop_error)
    }
}

/// Adapts `DbPool` to the `AnalyticsStorage` port trait.
pub struct AnalyticsStorageAdapter {
    pool: DbPool,
}

impl AnalyticsStorageAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AnalyticsStorage for AnalyticsStorageAdapter {
    async fn store_follower_snapshot(
        &self,
        followers: i64,
        following: i64,
        tweets: i64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_follower_snapshot(&self.pool, followers, following, tweets)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn get_yesterday_followers(&self) -> Result<Option<i64>, AnalyticsError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT follower_count FROM follower_snapshots
             WHERE snapshot_date < date('now')
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(row.map(|(c,)| c))
    }

    async fn get_replies_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT rs.reply_tweet_id FROM replies_sent rs
             WHERE rs.status = 'sent'
               AND rs.reply_tweet_id IS NOT NULL
               AND rs.created_at >= datetime('now', '-25 hours')
               AND rs.created_at <= datetime('now', '-23 hours')
               AND NOT EXISTS (
                   SELECT 1 FROM reply_performance rp WHERE rp.reply_id = rs.reply_tweet_id
               )",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn get_tweets_needing_measurement(&self) -> Result<Vec<String>, AnalyticsError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT ot.tweet_id FROM original_tweets ot
             WHERE ot.status = 'sent'
               AND ot.tweet_id IS NOT NULL
               AND ot.created_at >= datetime('now', '-25 hours')
               AND ot.created_at <= datetime('now', '-23 hours')
               AND NOT EXISTS (
                   SELECT 1 FROM tweet_performance tp WHERE tp.tweet_id = ot.tweet_id
               )",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    async fn store_reply_performance(
        &self,
        reply_id: &str,
        likes: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_reply_performance(
            &self.pool,
            reply_id,
            likes,
            replies,
            impressions,
            score,
        )
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn store_tweet_performance(
        &self,
        tweet_id: &str,
        likes: i64,
        retweets: i64,
        replies: i64,
        impressions: i64,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::upsert_tweet_performance(
            &self.pool,
            tweet_id,
            likes,
            retweets,
            replies,
            impressions,
            score,
        )
        .await
        .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn update_content_score(
        &self,
        topic: &str,
        format: &str,
        score: f64,
    ) -> Result<(), AnalyticsError> {
        storage::analytics::update_content_score(&self.pool, topic, format, score)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn log_action(
        &self,
        action_type: &str,
        status: &str,
        message: &str,
    ) -> Result<(), AnalyticsError> {
        storage::action_log::log_action(&self.pool, action_type, status, Some(message), None)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))
    }

    async fn run_aggregations(&self) -> Result<(), AnalyticsError> {
        let account_id = storage::accounts::DEFAULT_ACCOUNT_ID;
        storage::analytics::aggregate_best_times_for(&self.pool, account_id)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        storage::analytics::aggregate_reach_for(&self.pool, account_id)
            .await
            .map_err(|e| AnalyticsError::StorageError(e.to_string()))?;
        Ok(())
    }
}

/// Adapts `DbPool` to the `TopicScorer` port trait.
pub struct TopicScorerAdapter {
    pool: DbPool,
}

impl TopicScorerAdapter {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TopicScorer for TopicScorerAdapter {
    async fn get_top_topics(&self, limit: u32) -> Result<Vec<String>, ContentLoopError> {
        let scores = storage::analytics::get_top_topics(&self.pool, limit)
            .await
            .map_err(|e| ContentLoopError::StorageError(e.to_string()))?;
        Ok(scores.into_iter().map(|cs| cs.topic).collect())
    }
}
