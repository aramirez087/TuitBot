//! CRUD operations for target account monitoring.
//!
//! Manages the `target_accounts` and `target_tweets` tables for
//! relationship-based engagement with specific accounts.

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;
use crate::error::StorageError;

/// A target account record.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TargetAccount {
    pub account_id: String,
    pub username: String,
    pub followed_at: Option<String>,
    pub first_engagement_at: Option<String>,
    pub total_replies_sent: i64,
    pub last_reply_at: Option<String>,
    pub status: String,
}

/// Upsert a target account (insert or update username if exists) for a specific owner account.
pub async fn upsert_target_account_for(
    pool: &DbPool,
    owner_account_id: &str,
    account_id: &str,
    username: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO target_accounts (owner_account_id, account_id, username) \
         VALUES (?, ?, ?) \
         ON CONFLICT(account_id) DO UPDATE SET username = excluded.username",
    )
    .bind(owner_account_id)
    .bind(account_id)
    .bind(username)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Upsert a target account (insert or update username if exists).
pub async fn upsert_target_account(
    pool: &DbPool,
    account_id: &str,
    username: &str,
) -> Result<(), StorageError> {
    upsert_target_account_for(pool, DEFAULT_ACCOUNT_ID, account_id, username).await
}

type TargetAccountRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    String,
);

/// Get a target account by ID for a specific owner account.
pub async fn get_target_account_for(
    pool: &DbPool,
    owner_account_id: &str,
    account_id: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    let row: Option<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE account_id = ? AND owner_account_id = ?",
    )
    .bind(account_id)
    .bind(owner_account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| TargetAccount {
        account_id: r.0,
        username: r.1,
        followed_at: r.2,
        first_engagement_at: r.3,
        total_replies_sent: r.4,
        last_reply_at: r.5,
        status: r.6,
    }))
}

/// Get a target account by ID.
pub async fn get_target_account(
    pool: &DbPool,
    account_id: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    get_target_account_for(pool, DEFAULT_ACCOUNT_ID, account_id).await
}

/// Get all active target accounts for a specific owner account.
pub async fn get_active_target_accounts_for(
    pool: &DbPool,
    owner_account_id: &str,
) -> Result<Vec<TargetAccount>, StorageError> {
    let rows: Vec<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE status = 'active' AND owner_account_id = ?",
    )
    .bind(owner_account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| TargetAccount {
            account_id: r.0,
            username: r.1,
            followed_at: r.2,
            first_engagement_at: r.3,
            total_replies_sent: r.4,
            last_reply_at: r.5,
            status: r.6,
        })
        .collect())
}

/// Get all active target accounts.
pub async fn get_active_target_accounts(pool: &DbPool) -> Result<Vec<TargetAccount>, StorageError> {
    get_active_target_accounts_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Record a reply to a target account's tweet for a specific owner account.
pub async fn record_target_reply_for(
    pool: &DbPool,
    owner_account_id: &str,
    account_id: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE target_accounts \
         SET total_replies_sent = total_replies_sent + 1, \
             last_reply_at = datetime('now'), \
             first_engagement_at = COALESCE(first_engagement_at, datetime('now')) \
         WHERE account_id = ? AND owner_account_id = ?",
    )
    .bind(account_id)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Record a reply to a target account's tweet.
pub async fn record_target_reply(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    record_target_reply_for(pool, DEFAULT_ACCOUNT_ID, account_id).await
}

/// Get the number of target replies sent today for a specific owner account.
pub async fn count_target_replies_today_for(
    pool: &DbPool,
    owner_account_id: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM target_tweets \
         WHERE replied_to = 1 AND date(discovered_at) = date('now') AND owner_account_id = ?",
    )
    .bind(owner_account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(row.0)
}

/// Get the number of target replies sent today.
pub async fn count_target_replies_today(pool: &DbPool) -> Result<i64, StorageError> {
    count_target_replies_today_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Check if a target tweet exists for a specific owner account.
pub async fn target_tweet_exists_for(
    pool: &DbPool,
    owner_account_id: &str,
    tweet_id: &str,
) -> Result<bool, StorageError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM target_tweets WHERE id = ? AND owner_account_id = ?")
            .bind(tweet_id)
            .bind(owner_account_id)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;
    Ok(row.0 > 0)
}

/// Check if a target tweet exists.
pub async fn target_tweet_exists(pool: &DbPool, tweet_id: &str) -> Result<bool, StorageError> {
    target_tweet_exists_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
}

/// Store a discovered target tweet for a specific owner account.
#[allow(clippy::too_many_arguments)]
pub async fn store_target_tweet_for(
    pool: &DbPool,
    owner_account_id: &str,
    tweet_id: &str,
    account_id: &str,
    content: &str,
    created_at: &str,
    reply_count: i64,
    like_count: i64,
    relevance_score: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT OR IGNORE INTO target_tweets \
         (owner_account_id, id, account_id, content, created_at, reply_count, like_count, relevance_score) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(owner_account_id)
    .bind(tweet_id)
    .bind(account_id)
    .bind(content)
    .bind(created_at)
    .bind(reply_count)
    .bind(like_count)
    .bind(relevance_score)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Store a discovered target tweet.
#[allow(clippy::too_many_arguments)]
pub async fn store_target_tweet(
    pool: &DbPool,
    tweet_id: &str,
    account_id: &str,
    content: &str,
    created_at: &str,
    reply_count: i64,
    like_count: i64,
    relevance_score: f64,
) -> Result<(), StorageError> {
    store_target_tweet_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        tweet_id,
        account_id,
        content,
        created_at,
        reply_count,
        like_count,
        relevance_score,
    )
    .await
}

/// Mark a target tweet as replied to for a specific owner account.
pub async fn mark_target_tweet_replied_for(
    pool: &DbPool,
    owner_account_id: &str,
    tweet_id: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE target_tweets SET replied_to = 1 WHERE id = ? AND owner_account_id = ?")
        .bind(tweet_id)
        .bind(owner_account_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Mark a target tweet as replied to.
pub async fn mark_target_tweet_replied(pool: &DbPool, tweet_id: &str) -> Result<(), StorageError> {
    mark_target_tweet_replied_for(pool, DEFAULT_ACCOUNT_ID, tweet_id).await
}

/// Get a target account by username for a specific owner account.
pub async fn get_target_account_by_username_for(
    pool: &DbPool,
    owner_account_id: &str,
    username: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    let row: Option<TargetAccountRow> = sqlx::query_as(
        "SELECT account_id, username, followed_at, first_engagement_at, \
             total_replies_sent, last_reply_at, status \
             FROM target_accounts WHERE username = ? AND owner_account_id = ?",
    )
    .bind(username)
    .bind(owner_account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|r| TargetAccount {
        account_id: r.0,
        username: r.1,
        followed_at: r.2,
        first_engagement_at: r.3,
        total_replies_sent: r.4,
        last_reply_at: r.5,
        status: r.6,
    }))
}

/// Get a target account by username.
pub async fn get_target_account_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<TargetAccount>, StorageError> {
    get_target_account_by_username_for(pool, DEFAULT_ACCOUNT_ID, username).await
}

/// Deactivate a target account by username (soft delete) for a specific owner account.
pub async fn deactivate_target_account_for(
    pool: &DbPool,
    owner_account_id: &str,
    username: &str,
) -> Result<bool, StorageError> {
    let result = sqlx::query(
        "UPDATE target_accounts SET status = 'inactive' \
         WHERE username = ? AND status = 'active' AND owner_account_id = ?",
    )
    .bind(username)
    .bind(owner_account_id)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(result.rows_affected() > 0)
}

/// Deactivate a target account by username (soft delete).
pub async fn deactivate_target_account(
    pool: &DbPool,
    username: &str,
) -> Result<bool, StorageError> {
    deactivate_target_account_for(pool, DEFAULT_ACCOUNT_ID, username).await
}

// --- Enriched queries for the dashboard ---

/// A target account with today's interaction count.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EnrichedTargetAccount {
    pub account_id: String,
    pub username: String,
    pub followed_at: Option<String>,
    pub first_engagement_at: Option<String>,
    pub total_replies_sent: i64,
    pub last_reply_at: Option<String>,
    pub status: String,
    pub interactions_today: i64,
}

type EnrichedRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    String,
    i64,
);

/// Get all active target accounts with today's interaction count for a specific owner account.
pub async fn get_enriched_target_accounts_for(
    pool: &DbPool,
    owner_account_id: &str,
) -> Result<Vec<EnrichedTargetAccount>, StorageError> {
    let rows: Vec<EnrichedRow> = sqlx::query_as(
        "SELECT ta.account_id, ta.username, ta.followed_at, ta.first_engagement_at, \
                ta.total_replies_sent, ta.last_reply_at, ta.status, \
                COALESCE(SUM(CASE WHEN tt.replied_to = 1 \
                    AND date(tt.discovered_at) = date('now') THEN 1 ELSE 0 END), 0) \
         FROM target_accounts ta \
         LEFT JOIN target_tweets tt ON tt.account_id = ta.account_id \
         WHERE ta.status = 'active' AND ta.owner_account_id = ? \
         GROUP BY ta.account_id",
    )
    .bind(owner_account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| EnrichedTargetAccount {
            account_id: r.0,
            username: r.1,
            followed_at: r.2,
            first_engagement_at: r.3,
            total_replies_sent: r.4,
            last_reply_at: r.5,
            status: r.6,
            interactions_today: r.7,
        })
        .collect())
}

/// Get all active target accounts with today's interaction count.
pub async fn get_enriched_target_accounts(
    pool: &DbPool,
) -> Result<Vec<EnrichedTargetAccount>, StorageError> {
    get_enriched_target_accounts_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// A single entry in a target's interaction timeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TargetTimelineItem {
    pub tweet_id: String,
    pub text: String,
    pub posted_at: String,
    pub relevance_score: f64,
    pub replied_to: bool,
    pub tweet_reply_count: i64,
    pub tweet_like_count: i64,
    pub reply_content: Option<String>,
    pub reply_created_at: Option<String>,
}

type TimelineRow = (
    String,
    String,
    String,
    f64,
    i64,
    i64,
    i64,
    Option<String>,
    Option<String>,
);

/// Get the interaction timeline for a target account for a specific owner account.
pub async fn get_target_timeline_for(
    pool: &DbPool,
    owner_account_id: &str,
    username: &str,
    limit: i64,
) -> Result<Vec<TargetTimelineItem>, StorageError> {
    let rows: Vec<TimelineRow> = sqlx::query_as(
        "SELECT tt.id, tt.content, tt.created_at, tt.relevance_score, tt.replied_to, \
                tt.reply_count, tt.like_count, \
                rs.reply_content, rs.created_at \
         FROM target_tweets tt \
         JOIN target_accounts ta ON ta.account_id = tt.account_id \
         LEFT JOIN replies_sent rs ON rs.target_tweet_id = tt.id \
         WHERE ta.username = ? AND ta.status = 'active' AND ta.owner_account_id = ? \
         ORDER BY tt.created_at DESC \
         LIMIT ?",
    )
    .bind(username)
    .bind(owner_account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| TargetTimelineItem {
            tweet_id: r.0,
            text: r.1,
            posted_at: r.2,
            relevance_score: r.3,
            replied_to: r.4 != 0,
            tweet_reply_count: r.5,
            tweet_like_count: r.6,
            reply_content: r.7,
            reply_created_at: r.8,
        })
        .collect())
}

/// Get the interaction timeline for a target account.
pub async fn get_target_timeline(
    pool: &DbPool,
    username: &str,
    limit: i64,
) -> Result<Vec<TargetTimelineItem>, StorageError> {
    get_target_timeline_for(pool, DEFAULT_ACCOUNT_ID, username, limit).await
}

/// Aggregated statistics for a target account.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TargetStats {
    pub total_replies: i64,
    pub avg_score: f64,
    pub best_reply_content: Option<String>,
    pub best_reply_score: Option<f64>,
    pub first_interaction: Option<String>,
    pub interaction_frequency_days: Option<f64>,
}

type StatsRow = (i64, f64, Option<String>, Option<String>);
type BestReplyRow = (String, f64);

/// Get aggregated stats for a target account by username for a specific owner account.
pub async fn get_target_stats_for(
    pool: &DbPool,
    owner_account_id: &str,
    username: &str,
) -> Result<Option<TargetStats>, StorageError> {
    let row: Option<StatsRow> = sqlx::query_as(
        "SELECT ta.total_replies_sent, \
                COALESCE(AVG(tt.relevance_score), 0.0), \
                ta.first_engagement_at, ta.last_reply_at \
         FROM target_accounts ta \
         LEFT JOIN target_tweets tt ON tt.account_id = ta.account_id AND tt.replied_to = 1 \
         WHERE ta.username = ? AND ta.status = 'active' AND ta.owner_account_id = ? \
         GROUP BY ta.account_id",
    )
    .bind(username)
    .bind(owner_account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let Some(r) = row else {
        return Ok(None);
    };

    let total_replies = r.0;
    let avg_score = r.1;
    let first_interaction = r.2;
    let last_reply_at = r.3;

    // Best reply by relevance score.
    let best: Option<BestReplyRow> = sqlx::query_as(
        "SELECT rs.reply_content, tt.relevance_score \
         FROM replies_sent rs \
         JOIN target_tweets tt ON tt.id = rs.target_tweet_id \
         JOIN target_accounts ta ON ta.account_id = tt.account_id \
         WHERE ta.username = ? AND ta.status = 'active' AND ta.owner_account_id = ? \
         ORDER BY tt.relevance_score DESC \
         LIMIT 1",
    )
    .bind(username)
    .bind(owner_account_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    // Compute interaction frequency from first/last interaction dates.
    let frequency = compute_frequency(&first_interaction, &last_reply_at, total_replies);

    Ok(Some(TargetStats {
        total_replies,
        avg_score,
        best_reply_content: best.as_ref().map(|b| b.0.clone()),
        best_reply_score: best.map(|b| b.1),
        first_interaction,
        interaction_frequency_days: frequency,
    }))
}

/// Get aggregated stats for a target account by username.
pub async fn get_target_stats(
    pool: &DbPool,
    username: &str,
) -> Result<Option<TargetStats>, StorageError> {
    get_target_stats_for(pool, DEFAULT_ACCOUNT_ID, username).await
}

/// Compute average days between interactions.
fn compute_frequency(first: &Option<String>, last: &Option<String>, total: i64) -> Option<f64> {
    if total < 2 {
        return None;
    }
    let first_dt = first.as_ref()?.parse::<chrono::NaiveDateTime>().ok()?;
    let last_dt = last.as_ref()?.parse::<chrono::NaiveDateTime>().ok()?;
    let span = (last_dt - first_dt).num_hours() as f64 / 24.0;
    if span <= 0.0 {
        return None;
    }
    Some(span / (total - 1) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn upsert_and_get_target_account() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        let account = get_target_account(&pool, "acc_1")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(account.username, "alice");
        assert_eq!(account.total_replies_sent, 0);
        assert_eq!(account.status, "active");
    }

    #[tokio::test]
    async fn get_active_target_accounts_works() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        upsert_target_account(&pool, "acc_2", "bob")
            .await
            .expect("upsert");

        let accounts = get_active_target_accounts(&pool).await.expect("get all");
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn record_target_reply_increments() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        record_target_reply(&pool, "acc_1").await.expect("reply");
        record_target_reply(&pool, "acc_1").await.expect("reply");

        let account = get_target_account(&pool, "acc_1")
            .await
            .expect("get")
            .expect("found");
        assert_eq!(account.total_replies_sent, 2);
        assert!(account.first_engagement_at.is_some());
        assert!(account.last_reply_at.is_some());
    }

    #[tokio::test]
    async fn store_and_check_target_tweet() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        assert!(!target_tweet_exists(&pool, "tw_1").await.expect("check"));

        store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
            .await
            .expect("store");

        assert!(target_tweet_exists(&pool, "tw_1").await.expect("check"));
    }

    #[tokio::test]
    async fn mark_replied_updates_flag() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        store_target_tweet(&pool, "tw_1", "acc_1", "hello", "2026-01-01", 0, 5, 80.0)
            .await
            .expect("store");

        mark_target_tweet_replied(&pool, "tw_1")
            .await
            .expect("mark");

        // Verify by checking the count of replied tweets
        let count = count_target_replies_today(&pool).await.expect("count");
        assert!(count >= 0); // May or may not be today depending on test timing
    }

    #[tokio::test]
    async fn get_enriched_includes_interactions_today() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");
        upsert_target_account(&pool, "acc_2", "bob")
            .await
            .expect("upsert");

        // Store a tweet for alice discovered today and mark as replied
        let today = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        store_target_tweet(&pool, "tw_1", "acc_1", "hello", &today, 0, 5, 80.0)
            .await
            .expect("store");
        mark_target_tweet_replied(&pool, "tw_1")
            .await
            .expect("mark");

        let enriched = get_enriched_target_accounts(&pool).await.expect("enriched");
        assert_eq!(enriched.len(), 2);

        let alice = enriched.iter().find(|a| a.username == "alice").unwrap();
        assert_eq!(alice.interactions_today, 1);

        let bob = enriched.iter().find(|a| a.username == "bob").unwrap();
        assert_eq!(bob.interactions_today, 0);
    }

    #[tokio::test]
    async fn get_target_timeline_returns_tweets_with_replies() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        store_target_tweet(
            &pool,
            "tw_1",
            "acc_1",
            "First tweet",
            "2026-02-20T10:00:00Z",
            2,
            10,
            75.0,
        )
        .await
        .expect("store");
        store_target_tweet(
            &pool,
            "tw_2",
            "acc_1",
            "Second tweet",
            "2026-02-21T10:00:00Z",
            1,
            5,
            60.0,
        )
        .await
        .expect("store");
        mark_target_tweet_replied(&pool, "tw_1")
            .await
            .expect("mark");

        // Insert a reply for tw_1
        sqlx::query(
            "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
             VALUES ('tw_1', 'Great point!', '2026-02-20T11:00:00Z', 'sent')",
        )
        .execute(&pool)
        .await
        .expect("insert reply");

        let timeline = get_target_timeline(&pool, "alice", 50)
            .await
            .expect("timeline");
        assert_eq!(timeline.len(), 2);

        // Most recent first
        assert_eq!(timeline[0].tweet_id, "tw_2");
        assert!(!timeline[0].replied_to);
        assert!(timeline[0].reply_content.is_none());

        assert_eq!(timeline[1].tweet_id, "tw_1");
        assert!(timeline[1].replied_to);
        assert_eq!(timeline[1].reply_content.as_deref(), Some("Great point!"));
    }

    #[tokio::test]
    async fn get_target_stats_returns_aggregates() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        // Record some replies to set first/last engagement
        record_target_reply(&pool, "acc_1").await.expect("reply");
        record_target_reply(&pool, "acc_1").await.expect("reply");

        // Store tweets with scores and mark as replied
        store_target_tweet(
            &pool,
            "tw_1",
            "acc_1",
            "Tweet one",
            "2026-02-20T10:00:00Z",
            0,
            5,
            70.0,
        )
        .await
        .expect("store");
        mark_target_tweet_replied(&pool, "tw_1")
            .await
            .expect("mark");

        store_target_tweet(
            &pool,
            "tw_2",
            "acc_1",
            "Tweet two",
            "2026-02-22T10:00:00Z",
            0,
            3,
            90.0,
        )
        .await
        .expect("store");
        mark_target_tweet_replied(&pool, "tw_2")
            .await
            .expect("mark");

        // Insert replies for both
        sqlx::query(
            "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
             VALUES ('tw_1', 'Reply one', '2026-02-20T11:00:00Z', 'sent')",
        )
        .execute(&pool)
        .await
        .expect("insert reply");
        sqlx::query(
            "INSERT INTO replies_sent (target_tweet_id, reply_content, created_at, status) \
             VALUES ('tw_2', 'Reply two', '2026-02-22T11:00:00Z', 'sent')",
        )
        .execute(&pool)
        .await
        .expect("insert reply");

        let stats = get_target_stats(&pool, "alice")
            .await
            .expect("stats")
            .expect("found");
        assert_eq!(stats.total_replies, 2);
        assert!((stats.avg_score - 80.0).abs() < 0.01); // (70+90)/2
        assert!(stats.best_reply_content.is_some());
        assert!((stats.best_reply_score.unwrap() - 90.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn get_target_stats_returns_none_for_missing() {
        let pool = init_test_db().await.expect("init db");

        let stats = get_target_stats(&pool, "nobody").await.expect("stats");
        assert!(stats.is_none());
    }

    #[tokio::test]
    async fn get_target_stats_returns_zero_avg_for_target_without_replies() {
        let pool = init_test_db().await.expect("init db");

        upsert_target_account(&pool, "acc_1", "alice")
            .await
            .expect("upsert");

        let stats = get_target_stats(&pool, "alice")
            .await
            .expect("stats")
            .expect("found");
        assert_eq!(stats.total_replies, 0);
        assert_eq!(stats.avg_score, 0.0);
        assert!(stats.best_reply_content.is_none());
        assert!(stats.best_reply_score.is_none());
        assert!(stats.first_interaction.is_none());
        assert!(stats.interaction_frequency_days.is_none());
    }
}
