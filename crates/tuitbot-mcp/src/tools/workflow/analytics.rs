//! Analytics tools: stats dashboard and follower trend.

use std::time::Instant;

use serde::Serialize;

use tuitbot_core::config::Config;
use tuitbot_core::storage;
use tuitbot_core::storage::analytics::ContentScore;
use tuitbot_core::storage::DbPool;

use crate::tools::response::{ToolMeta, ToolResponse};

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
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
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
    let meta = ToolMeta::new(elapsed)
        .with_workflow(config.mode.to_string(), config.effective_approval_mode());

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
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching follower trend: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}

/// Get a deep analytics summary including heatmap and content breakdown.
pub async fn get_analytics_summary(pool: &DbPool, config: &Config) -> String {
    let start = Instant::now();

    let summary = match storage::analytics::get_analytics_summary(pool).await {
        Ok(s) => s,
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            return ToolResponse::db_error(format!("Error loading analytics summary: {e}"))
                .with_meta(meta)
                .to_json();
        }
    };

    let heatmap = storage::analytics::get_heatmap(pool)
        .await
        .unwrap_or_default();
    let content_breakdown = storage::analytics::get_content_breakdown(pool)
        .await
        .unwrap_or_default();
    let best_times = storage::analytics::get_best_times(pool)
        .await
        .unwrap_or_default();

    #[derive(Serialize)]
    struct SummaryOut {
        followers: FollowersSummaryOut,
        engagement: EngagementOut,
        top_topics: Vec<TopicOut>,
        best_times: Vec<BestTimeOut>,
        heatmap: Vec<HeatmapCellOut>,
        content_breakdown: Vec<ContentBreakdownOut>,
    }
    #[derive(Serialize)]
    struct FollowersSummaryOut {
        current: i64,
        change_7d: i64,
        change_30d: i64,
    }
    #[derive(Serialize)]
    struct EngagementOut {
        avg_reply_score: f64,
        avg_tweet_score: f64,
        total_replies: i64,
        total_tweets: i64,
    }
    #[derive(Serialize)]
    struct BestTimeOut {
        day_of_week: i32,
        day_name: String,
        hour: i32,
        avg_engagement: f64,
        sample_size: i64,
    }
    #[derive(Serialize)]
    struct HeatmapCellOut {
        day_of_week: i32,
        hour: i32,
        avg_engagement: f64,
        sample_size: i64,
    }
    #[derive(Serialize)]
    struct ContentBreakdownOut {
        content_type: String,
        count: i64,
        avg_performance: f64,
        total_impressions: i64,
    }

    let out = SummaryOut {
        followers: FollowersSummaryOut {
            current: summary.followers.current,
            change_7d: summary.followers.change_7d,
            change_30d: summary.followers.change_30d,
        },
        engagement: EngagementOut {
            avg_reply_score: summary.engagement.avg_reply_score,
            avg_tweet_score: summary.engagement.avg_tweet_score,
            total_replies: summary.engagement.total_replies_sent,
            total_tweets: summary.engagement.total_tweets_posted,
        },
        top_topics: topics_to_out(summary.top_topics),
        best_times: best_times
            .into_iter()
            .take(10)
            .map(|s| BestTimeOut {
                day_of_week: s.day_of_week,
                day_name: s.day_name,
                hour: s.hour,
                avg_engagement: s.avg_engagement,
                sample_size: s.sample_size,
            })
            .collect(),
        heatmap: heatmap
            .into_iter()
            .map(|c| HeatmapCellOut {
                day_of_week: c.day_of_week,
                hour: c.hour,
                avg_engagement: c.avg_engagement,
                sample_size: c.sample_size,
            })
            .collect(),
        content_breakdown: content_breakdown
            .into_iter()
            .map(|b| ContentBreakdownOut {
                content_type: b.content_type,
                count: b.count,
                avg_performance: b.avg_performance,
                total_impressions: b.total_impressions,
            })
            .collect(),
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let meta = ToolMeta::new(elapsed)
        .with_workflow(config.mode.to_string(), config.effective_approval_mode());

    ToolResponse::success(out).with_meta(meta).to_json()
}

/// Get top-performing topics from analytics.
pub async fn get_top_topics(pool: &DbPool, limit: u32, config: &Config) -> String {
    let start = Instant::now();

    match storage::analytics::get_top_topics(pool, limit).await {
        Ok(topics) => {
            let out = topics_to_out(topics);
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::success(out).with_meta(meta).to_json()
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let meta = ToolMeta::new(elapsed)
                .with_workflow(config.mode.to_string(), config.effective_approval_mode());
            ToolResponse::db_error(format!("Error fetching top topics: {e}"))
                .with_meta(meta)
                .to_json()
        }
    }
}
