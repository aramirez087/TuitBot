//! Read-only queries for target account monitoring.

use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
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

type TargetAccountRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    String,
);

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

type StatsRow = (i64, f64, Option<String>, Option<String>);
type BestReplyRow = (String, f64);

/// Compute average days between interactions.
pub fn compute_frequency(first: &Option<String>, last: &Option<String>, total: i64) -> Option<f64> {
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
