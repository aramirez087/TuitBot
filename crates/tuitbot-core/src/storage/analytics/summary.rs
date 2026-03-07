use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use super::content_scores::{
    get_avg_reply_engagement_for, get_avg_tweet_engagement_for, get_performance_counts_for,
    get_top_topics_for, ContentScore,
};
use super::snapshots::get_follower_snapshots_for;
use crate::error::StorageError;
use chrono::{NaiveDate, Utc};

/// Follower growth metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FollowerSummary {
    pub current: i64,
    pub change_7d: i64,
    pub change_30d: i64,
}

/// Today's action breakdown.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionsSummary {
    pub replies: i64,
    pub tweets: i64,
    pub threads: i64,
}

/// Engagement overview.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EngagementSummary {
    pub avg_reply_score: f64,
    pub avg_tweet_score: f64,
    pub total_replies_sent: i64,
    pub total_tweets_posted: i64,
}

/// Combined analytics summary for the dashboard.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalyticsSummary {
    pub followers: FollowerSummary,
    pub actions_today: ActionsSummary,
    pub engagement: EngagementSummary,
    pub top_topics: Vec<ContentScore>,
}

/// Get a combined analytics summary for the dashboard for a specific account.
///
/// Aggregates follower deltas, today's action counts, and engagement stats
/// into a single struct to minimise round-trips from the frontend.
pub async fn get_analytics_summary_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<AnalyticsSummary, StorageError> {
    // --- Follower data ---
    let snapshots = get_follower_snapshots_for(pool, account_id, 90).await?;
    let current = snapshots.first().map_or(0, |s| s.follower_count);

    // Find the first snapshot whose date is at least N days ago (handles gaps from
    // downtime or weekends).  Snapshots are ordered newest-first.
    let today = Utc::now().date_naive();
    let follower_at_or_before = |days: i64| -> i64 {
        snapshots
            .iter()
            .find(|s| {
                NaiveDate::parse_from_str(&s.snapshot_date, "%Y-%m-%d")
                    .map(|d| (today - d).num_days() >= days)
                    .unwrap_or(false)
            })
            .map_or(current, |s| s.follower_count)
    };

    let change_7d = if snapshots.len() >= 2 {
        current - follower_at_or_before(7)
    } else {
        0
    };
    let change_30d = if snapshots.len() >= 2 {
        current - follower_at_or_before(30)
    } else {
        0
    };

    // --- Today's actions (from action_log) ---
    let today = Utc::now().format("%Y-%m-%dT00:00:00Z").to_string();
    let counts = super::super::action_log::get_action_counts_since(pool, &today).await?;
    let actions_today = ActionsSummary {
        replies: *counts.get("reply").unwrap_or(&0),
        tweets: *counts.get("tweet").unwrap_or(&0),
        threads: *counts.get("thread").unwrap_or(&0),
    };

    // --- Engagement ---
    let avg_reply_score = get_avg_reply_engagement_for(pool, account_id).await?;
    let avg_tweet_score = get_avg_tweet_engagement_for(pool, account_id).await?;
    let (total_replies_sent, total_tweets_posted) =
        get_performance_counts_for(pool, account_id).await?;

    // --- Top topics ---
    let top_topics = get_top_topics_for(pool, account_id, 5).await?;

    Ok(AnalyticsSummary {
        followers: FollowerSummary {
            current,
            change_7d,
            change_30d,
        },
        actions_today,
        engagement: EngagementSummary {
            avg_reply_score,
            avg_tweet_score,
            total_replies_sent,
            total_tweets_posted,
        },
        top_topics,
    })
}

/// Get a combined analytics summary for the dashboard.
///
/// Aggregates follower deltas, today's action counts, and engagement stats
/// into a single struct to minimise round-trips from the frontend.
pub async fn get_analytics_summary(pool: &DbPool) -> Result<AnalyticsSummary, StorageError> {
    get_analytics_summary_for(pool, DEFAULT_ACCOUNT_ID).await
}
