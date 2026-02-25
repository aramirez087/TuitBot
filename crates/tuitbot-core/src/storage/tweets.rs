//! CRUD operations for discovered tweets.
//!
//! Provides functions to insert, query, and update tweets discovered
//! from X search results.

use super::DbPool;
use crate::error::StorageError;

/// A tweet discovered from X search matching configured keywords.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct DiscoveredTweet {
    /// X tweet ID (globally unique).
    pub id: String,
    /// X user ID of tweet author.
    pub author_id: String,
    /// @handle of tweet author.
    pub author_username: String,
    /// Full tweet text.
    pub content: String,
    /// Likes at discovery time.
    pub like_count: i64,
    /// Retweets at discovery time.
    pub retweet_count: i64,
    /// Replies at discovery time.
    pub reply_count: i64,
    /// Impressions if available.
    pub impression_count: Option<i64>,
    /// Computed relevance score (0-100).
    pub relevance_score: Option<f64>,
    /// Which keyword triggered discovery.
    pub matched_keyword: Option<String>,
    /// ISO-8601 UTC timestamp of discovery.
    pub discovered_at: String,
    /// Whether a reply has been sent (0 = no, 1 = yes).
    pub replied_to: i64,
}

/// Insert a discovered tweet. Uses `INSERT OR IGNORE` to handle duplicates gracefully.
pub async fn insert_discovered_tweet(
    pool: &DbPool,
    tweet: &DiscoveredTweet,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO discovered_tweets \
         (id, author_id, author_username, content, like_count, retweet_count, \
          reply_count, impression_count, relevance_score, matched_keyword, \
          discovered_at, replied_to) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&tweet.id)
    .bind(&tweet.author_id)
    .bind(&tweet.author_username)
    .bind(&tweet.content)
    .bind(tweet.like_count)
    .bind(tweet.retweet_count)
    .bind(tweet.reply_count)
    .bind(tweet.impression_count)
    .bind(tweet.relevance_score)
    .bind(&tweet.matched_keyword)
    .bind(&tweet.discovered_at)
    .bind(tweet.replied_to)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Fetch a single tweet by its X ID. Returns `None` if not found.
pub async fn get_tweet_by_id(
    pool: &DbPool,
    tweet_id: &str,
) -> Result<Option<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>("SELECT * FROM discovered_tweets WHERE id = ?")
        .bind(tweet_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Mark a tweet as having been replied to.
pub async fn mark_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError> {
    sqlx::query("UPDATE discovered_tweets SET replied_to = 1 WHERE id = ?")
        .bind(tweet_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Fetch unreplied tweets with relevance score at or above the threshold,
/// ordered by score descending.
pub async fn get_unreplied_tweets_above_score(
    pool: &DbPool,
    threshold: f64,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>(
        "SELECT * FROM discovered_tweets \
         WHERE replied_to = 0 AND relevance_score >= ? \
         ORDER BY relevance_score DESC",
    )
    .bind(threshold)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch discovered tweets above a score threshold, ordered by discovery time (newest first).
pub async fn get_discovery_feed(
    pool: &DbPool,
    min_score: f64,
    limit: u32,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>(
        "SELECT * FROM discovered_tweets \
         WHERE COALESCE(relevance_score, 0.0) >= ? \
         ORDER BY discovered_at DESC \
         LIMIT ?",
    )
    .bind(min_score)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Check if a tweet exists in the database.
pub async fn tweet_exists(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError> {
    let row: (i64,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM discovered_tweets WHERE id = ?)")
        .bind(tweet_id)
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0 == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn sample_tweet(id: &str, score: Option<f64>) -> DiscoveredTweet {
        DiscoveredTweet {
            id: id.to_string(),
            author_id: "user123".to_string(),
            author_username: "testuser".to_string(),
            content: "Test tweet content".to_string(),
            like_count: 10,
            retweet_count: 2,
            reply_count: 1,
            impression_count: Some(500),
            relevance_score: score,
            matched_keyword: Some("rust".to_string()),
            discovered_at: "2026-02-21T12:00:00Z".to_string(),
            replied_to: 0,
        }
    }

    #[tokio::test]
    async fn insert_and_retrieve_tweet() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("tweet_1", Some(85.0));

        insert_discovered_tweet(&pool, &tweet)
            .await
            .expect("insert");
        let fetched = get_tweet_by_id(&pool, "tweet_1")
            .await
            .expect("get")
            .expect("should exist");

        assert_eq!(fetched.id, "tweet_1");
        assert_eq!(fetched.author_username, "testuser");
        assert_eq!(fetched.relevance_score, Some(85.0));
    }

    #[tokio::test]
    async fn duplicate_insert_is_ignored() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("tweet_dup", Some(50.0));

        insert_discovered_tweet(&pool, &tweet).await.expect("first");
        insert_discovered_tweet(&pool, &tweet)
            .await
            .expect("duplicate should be ignored");
    }

    #[tokio::test]
    async fn get_nonexistent_tweet_returns_none() {
        let pool = init_test_db().await.expect("init db");
        let result = get_tweet_by_id(&pool, "nonexistent").await.expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mark_replied_and_query_unreplied() {
        let pool = init_test_db().await.expect("init db");

        let tweet1 = sample_tweet("t1", Some(90.0));
        let tweet2 = sample_tweet("t2", Some(80.0));
        let tweet3 = sample_tweet("t3", Some(60.0));

        insert_discovered_tweet(&pool, &tweet1).await.expect("ins1");
        insert_discovered_tweet(&pool, &tweet2).await.expect("ins2");
        insert_discovered_tweet(&pool, &tweet3).await.expect("ins3");

        mark_tweet_replied(&pool, "t1").await.expect("mark");

        let unreplied = get_unreplied_tweets_above_score(&pool, 70.0)
            .await
            .expect("query");
        assert_eq!(unreplied.len(), 1);
        assert_eq!(unreplied[0].id, "t2");
    }

    #[tokio::test]
    async fn tweet_exists_check() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("exists_test", Some(50.0));

        assert!(!tweet_exists(&pool, "exists_test").await.expect("check"));
        insert_discovered_tweet(&pool, &tweet).await.expect("ins");
        assert!(tweet_exists(&pool, "exists_test").await.expect("check"));
    }
}
