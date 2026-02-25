//! Analytics tools: stats dashboard and follower trend.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::analytics::ContentScore;
use tuitbot_core::storage::DbPool;

use super::response::{ToolMeta, ToolResponse};

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
    net_follower_change_7d: i64,
    net_follower_change_30d: i64,
    top_topics: Vec<TopicOut>,
    avg_reply_engagement: f64,
    avg_tweet_engagement: f64,
    replies_measured: i64,
    tweets_measured: i64,
}

fn topics_to_out(topics: Vec<ContentScore>) -> Vec<TopicOut> {
    topics
        .into_iter()
        .map(|t| TopicOut {
            topic: t.topic,
            format: t.format,
            total_posts: t.total_posts,
            avg_performance: t.avg_performance,
        })
        .collect()
}

/// Collect analytics stats using the consolidated summary from storage.
pub async fn get_stats(pool: &DbPool, days: u32, config: &Config) -> String {
    let start = Instant::now();

    // Use the consolidated summary to avoid data drift with the dashboard
    let summary = match storage::analytics::get_analytics_summary(pool).await {
        Ok(s) => s,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            return ToolResponse::db_error(format!("Error loading analytics summary: {e}"))
                .with_meta(meta)
                .to_json();
        }
    };

    let snapshots = storage::analytics::get_follower_snapshots(pool, days)
        .await
        .unwrap_or_default();

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

    let out = StatsOut {
        follower_trend,
        net_follower_change_7d: summary.followers.change_7d,
        net_follower_change_30d: summary.followers.change_30d,
        top_topics: topics_to_out(summary.top_topics),
        avg_reply_engagement: summary.engagement.avg_reply_score,
        avg_tweet_engagement: summary.engagement.avg_tweet_score,
        replies_measured: summary.engagement.total_replies_sent,
        tweets_measured: summary.engagement.total_tweets_posted,
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let meta =
        ToolMeta::new(elapsed).with_mode(config.mode.to_string(), config.effective_approval_mode());

    ToolResponse::success(out).with_meta(meta).to_json()
}

/// Get follower snapshots over time.
pub async fn get_follower_trend(pool: &DbPool, limit: u32, config: &Config) -> String {
    let start = Instant::now();

    match storage::analytics::get_follower_snapshots(pool, limit).await {
        Ok(snapshots) => {
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
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching follower trend: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get top-performing topics from analytics.
pub async fn get_top_topics(pool: &DbPool, limit: u32, config: &Config) -> String {
    let start = Instant::now();

    match storage::analytics::get_top_topics(pool, limit).await {
        Ok(topics) => {
            let out = topics_to_out(topics);
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_mode(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching top topics: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
