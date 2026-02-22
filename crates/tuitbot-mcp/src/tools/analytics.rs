//! Analytics tools: stats dashboard and follower trend.

use serde::Serialize;

use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

#[derive(Serialize)]
struct FollowerSnapshotOut {
    date: String,
    follower_count: i64,
    following_count: i64,
    tweet_count: i64,
}

#[derive(Serialize)]
struct TopicOut {
    topic: String,
    format: String,
    total_posts: i64,
    avg_performance: f64,
}

#[derive(Serialize)]
struct StatsOut {
    follower_trend: Vec<FollowerSnapshotOut>,
    net_follower_change: Option<i64>,
    top_topics: Vec<TopicOut>,
    avg_reply_engagement: f64,
    avg_tweet_engagement: f64,
    replies_measured: i64,
    tweets_measured: i64,
}

/// Collect analytics stats for the given number of days.
pub async fn get_stats(pool: &DbPool, days: u32) -> String {
    let snapshots = storage::analytics::get_follower_snapshots(pool, days)
        .await
        .unwrap_or_default();

    let net_follower_change = if snapshots.len() >= 2 {
        Some(snapshots[0].follower_count - snapshots[snapshots.len() - 1].follower_count)
    } else {
        None
    };

    let follower_trend: Vec<FollowerSnapshotOut> = snapshots
        .iter()
        .rev()
        .map(|s| FollowerSnapshotOut {
            date: s.snapshot_date.clone(),
            follower_count: s.follower_count,
            following_count: s.following_count,
            tweet_count: s.tweet_count,
        })
        .collect();

    let topics = storage::analytics::get_top_topics(pool, 10)
        .await
        .unwrap_or_default();
    let top_topics: Vec<TopicOut> = topics
        .into_iter()
        .map(|t| TopicOut {
            topic: t.topic,
            format: t.format,
            total_posts: t.total_posts,
            avg_performance: t.avg_performance,
        })
        .collect();

    let avg_reply_engagement = storage::analytics::get_avg_reply_engagement(pool)
        .await
        .unwrap_or(0.0);
    let avg_tweet_engagement = storage::analytics::get_avg_tweet_engagement(pool)
        .await
        .unwrap_or(0.0);
    let (replies_measured, tweets_measured) = storage::analytics::get_performance_counts(pool)
        .await
        .unwrap_or((0, 0));

    let out = StatsOut {
        follower_trend,
        net_follower_change,
        top_topics,
        avg_reply_engagement,
        avg_tweet_engagement,
        replies_measured,
        tweets_measured,
    };

    serde_json::to_string_pretty(&out).unwrap_or_else(|e| format!("Error serializing stats: {e}"))
}

/// Get follower snapshots over time.
pub async fn get_follower_trend(pool: &DbPool, limit: u32) -> String {
    let snapshots = storage::analytics::get_follower_snapshots(pool, limit)
        .await
        .unwrap_or_default();

    let out: Vec<FollowerSnapshotOut> = snapshots
        .iter()
        .rev()
        .map(|s| FollowerSnapshotOut {
            date: s.snapshot_date.clone(),
            follower_count: s.follower_count,
            following_count: s.following_count,
            tweet_count: s.tweet_count,
        })
        .collect();

    serde_json::to_string_pretty(&out)
        .unwrap_or_else(|e| format!("Error serializing follower trend: {e}"))
}
