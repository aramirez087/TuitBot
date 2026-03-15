//! CRUD operations for discovered tweets.
//!
//! Provides functions to insert, query, and update tweets discovered
//! from X search results.

use super::accounts::DEFAULT_ACCOUNT_ID;
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

/// Insert a discovered tweet for a specific account. Uses `INSERT OR IGNORE` to handle duplicates gracefully.
pub async fn insert_discovered_tweet_for(
    pool: &DbPool,
    account_id: &str,
    tweet: &DiscoveredTweet,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO discovered_tweets \
         (account_id, id, author_id, author_username, content, like_count, retweet_count, \
          reply_count, impression_count, relevance_score, matched_keyword, \
          discovered_at, replied_to) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(account_id)
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

/// Insert a discovered tweet. Uses `INSERT OR IGNORE` to handle duplicates gracefully.
pub async fn insert_discovered_tweet(
    pool: &DbPool,
    tweet: &DiscoveredTweet,
) -> Result<(), StorageError> {
    insert_discovered_tweet_for(pool, DEFAULT_ACCOUNT_ID, tweet).await
}

/// Fetch a single tweet by its X ID for a specific account. Returns `None` if not found.
pub async fn get_tweet_by_id_for(
    pool: &DbPool,
    account_id: &str,
    tweet_id: &str,
) -> Result<Option<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>(
        "SELECT * FROM discovered_tweets WHERE account_id = ? AND id = ?",
    )
    .bind(account_id)
    .bind(tweet_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch a single tweet by its X ID. Returns `None` if not found.
pub async fn get_tweet_by_id(
    pool: &DbPool,
    tweet_id: &str,
) -> Result<Option<DiscoveredTweet>, StorageError> {
    get_tweet_by_id_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
}

/// Mark a tweet as having been replied to for a specific account.
pub async fn mark_tweet_replied_for(
    pool: &DbPool,
    account_id: &str,
    tweet_id: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE discovered_tweets SET replied_to = 1 WHERE account_id = ? AND id = ?")
        .bind(account_id)
        .bind(tweet_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Mark a tweet as having been replied to.
pub async fn mark_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError> {
    mark_tweet_replied_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
}

/// Fetch unreplied tweets with relevance score at or above the threshold for a specific account,
/// ordered by score descending.
pub async fn get_unreplied_tweets_above_score_for(
    pool: &DbPool,
    account_id: &str,
    threshold: f64,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>(
        "SELECT * FROM discovered_tweets \
         WHERE account_id = ? AND replied_to = 0 AND relevance_score >= ? \
         ORDER BY relevance_score DESC",
    )
    .bind(account_id)
    .bind(threshold)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Fetch unreplied tweets with relevance score at or above the threshold,
/// ordered by score descending.
pub async fn get_unreplied_tweets_above_score(
    pool: &DbPool,
    threshold: f64,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    get_unreplied_tweets_above_score_for(pool, DEFAULT_ACCOUNT_ID, threshold).await
}

/// Fetch discovered tweets above a score threshold for a specific account, ordered by discovery time (newest first).
pub async fn get_discovery_feed_for(
    pool: &DbPool,
    account_id: &str,
    min_score: f64,
    limit: u32,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    sqlx::query_as::<_, DiscoveredTweet>(
        "SELECT * FROM discovered_tweets \
         WHERE account_id = ? AND COALESCE(relevance_score, 0.0) >= ? \
         ORDER BY discovered_at DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(min_score)
    .bind(limit)
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
    get_discovery_feed_for(pool, DEFAULT_ACCOUNT_ID, min_score, limit).await
}

/// Fetch discovered tweets with advanced filters for a specific account: score range, keyword, and limit.
pub async fn get_discovery_feed_filtered_for(
    pool: &DbPool,
    account_id: &str,
    min_score: f64,
    max_score: Option<f64>,
    keyword: Option<&str>,
    limit: u32,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    let mut sql = String::from(
        "SELECT * FROM discovered_tweets WHERE account_id = ? AND COALESCE(relevance_score, 0.0) >= ?",
    );
    if max_score.is_some() {
        sql.push_str(" AND COALESCE(relevance_score, 0.0) <= ?");
    }
    if keyword.is_some() {
        sql.push_str(" AND matched_keyword = ?");
    }
    sql.push_str(" ORDER BY discovered_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, DiscoveredTweet>(&sql)
        .bind(account_id)
        .bind(min_score);
    if let Some(max) = max_score {
        query = query.bind(max);
    }
    if let Some(kw) = keyword {
        query = query.bind(kw);
    }
    query = query.bind(limit);

    query
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Fetch discovered tweets with advanced filters: score range, keyword, and limit.
pub async fn get_discovery_feed_filtered(
    pool: &DbPool,
    min_score: f64,
    max_score: Option<f64>,
    keyword: Option<&str>,
    limit: u32,
) -> Result<Vec<DiscoveredTweet>, StorageError> {
    get_discovery_feed_filtered_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        min_score,
        max_score,
        keyword,
        limit,
    )
    .await
}

/// Get distinct matched keywords from discovered tweets for a specific account.
pub async fn get_distinct_keywords_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<String>, StorageError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT matched_keyword FROM discovered_tweets \
         WHERE account_id = ? AND matched_keyword IS NOT NULL AND matched_keyword != '' \
         ORDER BY matched_keyword",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(|(kw,)| kw).collect())
}

/// Get distinct matched keywords from discovered tweets.
pub async fn get_distinct_keywords(pool: &DbPool) -> Result<Vec<String>, StorageError> {
    get_distinct_keywords_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Check if a tweet exists in the database for a specific account.
pub async fn tweet_exists_for(
    pool: &DbPool,
    account_id: &str,
    tweet_id: &str,
) -> Result<bool, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM discovered_tweets WHERE account_id = ? AND id = ?)",
    )
    .bind(account_id)
    .bind(tweet_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0 == 1)
}

/// Check if a tweet exists in the database.
pub async fn tweet_exists(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError> {
    tweet_exists_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
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

    #[tokio::test]
    async fn discovery_feed_returns_ordered_by_discovered_at() {
        let pool = init_test_db().await.expect("init db");

        let mut t1 = sample_tweet("feed_1", Some(80.0));
        t1.discovered_at = "2026-02-21T12:00:00Z".to_string();
        let mut t2 = sample_tweet("feed_2", Some(90.0));
        t2.discovered_at = "2026-02-21T13:00:00Z".to_string();

        insert_discovered_tweet(&pool, &t1).await.expect("ins1");
        insert_discovered_tweet(&pool, &t2).await.expect("ins2");

        let feed = get_discovery_feed(&pool, 70.0, 10).await.expect("feed");
        assert_eq!(feed.len(), 2);
        assert_eq!(feed[0].id, "feed_2", "newest should be first");
    }

    #[tokio::test]
    async fn discovery_feed_respects_min_score() {
        let pool = init_test_db().await.expect("init db");

        insert_discovered_tweet(&pool, &sample_tweet("low_1", Some(30.0)))
            .await
            .expect("ins");
        insert_discovered_tweet(&pool, &sample_tweet("high_1", Some(80.0)))
            .await
            .expect("ins");

        let feed = get_discovery_feed(&pool, 50.0, 10).await.expect("feed");
        assert_eq!(feed.len(), 1);
        assert_eq!(feed[0].id, "high_1");
    }

    #[tokio::test]
    async fn discovery_feed_respects_limit() {
        let pool = init_test_db().await.expect("init db");

        for i in 0..5 {
            insert_discovered_tweet(&pool, &sample_tweet(&format!("lim_{i}"), Some(80.0)))
                .await
                .expect("ins");
        }

        let feed = get_discovery_feed(&pool, 0.0, 3).await.expect("feed");
        assert_eq!(feed.len(), 3);
    }

    #[tokio::test]
    async fn discovery_feed_filtered_with_max_score() {
        let pool = init_test_db().await.expect("init db");

        insert_discovered_tweet(&pool, &sample_tweet("f_low", Some(30.0)))
            .await
            .expect("ins");
        insert_discovered_tweet(&pool, &sample_tweet("f_mid", Some(60.0)))
            .await
            .expect("ins");
        insert_discovered_tweet(&pool, &sample_tweet("f_high", Some(90.0)))
            .await
            .expect("ins");

        let feed = get_discovery_feed_filtered(&pool, 40.0, Some(70.0), None, 10)
            .await
            .expect("feed");
        assert_eq!(feed.len(), 1);
        assert_eq!(feed[0].id, "f_mid");
    }

    #[tokio::test]
    async fn discovery_feed_filtered_with_keyword() {
        let pool = init_test_db().await.expect("init db");

        let mut t1 = sample_tweet("kw_1", Some(80.0));
        t1.matched_keyword = Some("rust".to_string());
        let mut t2 = sample_tweet("kw_2", Some(80.0));
        t2.matched_keyword = Some("python".to_string());

        insert_discovered_tweet(&pool, &t1).await.expect("ins");
        insert_discovered_tweet(&pool, &t2).await.expect("ins");

        let feed = get_discovery_feed_filtered(&pool, 0.0, None, Some("rust"), 10)
            .await
            .expect("feed");
        assert_eq!(feed.len(), 1);
        assert_eq!(feed[0].id, "kw_1");
    }

    #[tokio::test]
    async fn get_distinct_keywords_returns_unique_sorted() {
        let pool = init_test_db().await.expect("init db");

        let mut t1 = sample_tweet("dk_1", Some(80.0));
        t1.matched_keyword = Some("rust".to_string());
        let mut t2 = sample_tweet("dk_2", Some(80.0));
        t2.matched_keyword = Some("python".to_string());
        let mut t3 = sample_tweet("dk_3", Some(80.0));
        t3.matched_keyword = Some("rust".to_string());

        insert_discovered_tweet(&pool, &t1).await.expect("ins");
        insert_discovered_tweet(&pool, &t2).await.expect("ins");
        insert_discovered_tweet(&pool, &t3).await.expect("ins");

        let keywords = get_distinct_keywords(&pool).await.expect("keywords");
        assert_eq!(keywords.len(), 2);
        assert_eq!(keywords[0], "python");
        assert_eq!(keywords[1], "rust");
    }

    #[tokio::test]
    async fn get_distinct_keywords_excludes_null_and_empty() {
        let pool = init_test_db().await.expect("init db");

        let mut t1 = sample_tweet("dk_null", Some(80.0));
        t1.matched_keyword = None;
        let mut t2 = sample_tweet("dk_empty", Some(80.0));
        t2.matched_keyword = Some(String::new());
        let mut t3 = sample_tweet("dk_valid", Some(80.0));
        t3.matched_keyword = Some("valid".to_string());

        insert_discovered_tweet(&pool, &t1).await.expect("ins");
        insert_discovered_tweet(&pool, &t2).await.expect("ins");
        insert_discovered_tweet(&pool, &t3).await.expect("ins");

        let keywords = get_distinct_keywords(&pool).await.expect("keywords");
        assert_eq!(keywords.len(), 1);
        assert_eq!(keywords[0], "valid");
    }

    #[tokio::test]
    async fn insert_and_retrieve_tweet_for_account() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("acct_t1", Some(75.0));

        insert_discovered_tweet_for(&pool, "acct_a", &tweet)
            .await
            .expect("ins");

        // Should not be found under default account
        let result = get_tweet_by_id(&pool, "acct_t1").await.expect("get");
        assert!(result.is_none());

        // Should be found under acct_a
        let result = get_tweet_by_id_for(&pool, "acct_a", "acct_t1")
            .await
            .expect("get");
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn tweet_exists_for_account_isolation() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("iso_t1", Some(50.0));

        insert_discovered_tweet_for(&pool, "acct_x", &tweet)
            .await
            .expect("ins");

        assert!(tweet_exists_for(&pool, "acct_x", "iso_t1")
            .await
            .expect("check"));
        assert!(!tweet_exists_for(&pool, "acct_y", "iso_t1")
            .await
            .expect("check"));
    }

    #[tokio::test]
    async fn mark_tweet_replied_for_account_isolation() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("mr_t1", Some(80.0));

        insert_discovered_tweet_for(&pool, "acct_r", &tweet)
            .await
            .expect("ins");
        mark_tweet_replied_for(&pool, "acct_r", "mr_t1")
            .await
            .expect("mark");

        let fetched = get_tweet_by_id_for(&pool, "acct_r", "mr_t1")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.replied_to, 1);
    }

    #[tokio::test]
    async fn sample_tweet_with_none_score() {
        let pool = init_test_db().await.expect("init db");
        let tweet = sample_tweet("none_score", None);

        insert_discovered_tweet(&pool, &tweet).await.expect("ins");
        let fetched = get_tweet_by_id(&pool, "none_score")
            .await
            .expect("get")
            .expect("exists");
        assert!(fetched.relevance_score.is_none());
    }
}
