//! Implementation of the `tuitbot stats` command.
//!
//! Displays analytics dashboard: follower trend, top-performing topics,
//! engagement rates, and weekly volume.

use serde::Serialize;
use tuitbot_core::config::Config;
use tuitbot_core::storage;

use crate::output::CliOutput;

#[derive(Serialize)]
struct FollowerSnapshotJson {
    date: String,
    follower_count: i64,
    following_count: i64,
    tweet_count: i64,
}

#[derive(Serialize)]
struct TopicJson {
    topic: String,
    format: String,
    total_posts: i64,
    avg_performance: f64,
}

#[derive(Serialize)]
struct EngagementJson {
    avg_reply_score: f64,
    avg_tweet_score: f64,
}

#[derive(Serialize)]
struct ContentMeasuredJson {
    replies: i64,
    tweets: i64,
}

#[derive(Serialize)]
struct StatsOutput {
    follower_trend: Vec<FollowerSnapshotJson>,
    net_follower_change: Option<i64>,
    top_topics: Vec<TopicJson>,
    engagement: EngagementJson,
    content_measured: ContentMeasuredJson,
}

/// Execute the `tuitbot stats` command.
pub async fn execute(config: &Config, out: CliOutput) -> anyhow::Result<()> {
    let pool = storage::init_db(&config.storage.db_path).await?;

    if out.is_json() {
        let result = collect_stats_json(&pool).await;
        pool.close().await;
        let stats = result?;
        out.json(&stats)?;
        return Ok(());
    }

    out.info("");
    out.info("=== Tuitbot Analytics ===");
    out.info("");

    if !out.quiet {
        // 1. Follower trend (7 days)
        print_follower_trend(&pool).await;

        // 2. Top performing topics
        print_top_topics(&pool).await;

        // 3. Engagement rates
        print_engagement_rates(&pool).await;

        // 4. Performance counts
        print_performance_counts(&pool).await;

        eprintln!();
    }

    pool.close().await;
    Ok(())
}

async fn collect_stats_json(pool: &storage::DbPool) -> anyhow::Result<StatsOutput> {
    let snapshots = storage::analytics::get_follower_snapshots(pool, 7)
        .await
        .unwrap_or_default();

    let net_follower_change = if snapshots.len() >= 2 {
        Some(snapshots[0].follower_count - snapshots[snapshots.len() - 1].follower_count)
    } else {
        None
    };

    let follower_trend: Vec<FollowerSnapshotJson> = snapshots
        .iter()
        .rev()
        .map(|s| FollowerSnapshotJson {
            date: s.snapshot_date.clone(),
            follower_count: s.follower_count,
            following_count: s.following_count,
            tweet_count: s.tweet_count,
        })
        .collect();

    let topics = storage::analytics::get_top_topics(pool, 10)
        .await
        .unwrap_or_default();
    let top_topics: Vec<TopicJson> = topics
        .into_iter()
        .map(|t| TopicJson {
            topic: t.topic,
            format: t.format,
            total_posts: t.total_posts,
            avg_performance: t.avg_performance,
        })
        .collect();

    let avg_reply_score = storage::analytics::get_avg_reply_engagement(pool)
        .await
        .unwrap_or(0.0);
    let avg_tweet_score = storage::analytics::get_avg_tweet_engagement(pool)
        .await
        .unwrap_or(0.0);

    let (replies, tweets) = storage::analytics::get_performance_counts(pool)
        .await
        .unwrap_or((0, 0));

    Ok(StatsOutput {
        follower_trend,
        net_follower_change,
        top_topics,
        engagement: EngagementJson {
            avg_reply_score,
            avg_tweet_score,
        },
        content_measured: ContentMeasuredJson { replies, tweets },
    })
}

async fn print_follower_trend(pool: &storage::DbPool) {
    eprintln!("--- Follower Trend (7 days) ---");

    match storage::analytics::get_follower_snapshots(pool, 7).await {
        Ok(snapshots) if snapshots.is_empty() => {
            eprintln!("  No data yet. Run the agent to collect snapshots.");
        }
        Ok(snapshots) => {
            // snapshots are newest first, reverse for chronological display
            for snap in snapshots.iter().rev() {
                eprintln!(
                    "  {} | Followers: {:>6} | Following: {:>5} | Tweets: {:>6}",
                    snap.snapshot_date, snap.follower_count, snap.following_count, snap.tweet_count
                );
            }

            if snapshots.len() >= 2 {
                let newest = &snapshots[0];
                let oldest = &snapshots[snapshots.len() - 1];
                let diff = newest.follower_count - oldest.follower_count;
                let sign = if diff >= 0 { "+" } else { "" };
                eprintln!(
                    "  Net change: {sign}{diff} followers over {} day(s)",
                    snapshots.len()
                );
            }
        }
        Err(e) => {
            eprintln!("  Error fetching snapshots: {e}");
        }
    }
    eprintln!();
}

async fn print_top_topics(pool: &storage::DbPool) {
    eprintln!("--- Top Performing Topics ---");

    match storage::analytics::get_top_topics(pool, 10).await {
        Ok(topics) if topics.is_empty() => {
            eprintln!("  No topic data yet.");
        }
        Ok(topics) => {
            for (i, topic) in topics.iter().enumerate() {
                eprintln!(
                    "  {}. {} (format: {}) | Posts: {} | Avg score: {:.1}",
                    i + 1,
                    topic.topic,
                    if topic.format.is_empty() {
                        "-"
                    } else {
                        &topic.format
                    },
                    topic.total_posts,
                    topic.avg_performance,
                );
            }
        }
        Err(e) => {
            eprintln!("  Error fetching topics: {e}");
        }
    }
    eprintln!();
}

async fn print_engagement_rates(pool: &storage::DbPool) {
    eprintln!("--- Engagement Rates ---");

    match storage::analytics::get_avg_reply_engagement(pool).await {
        Ok(avg) => eprintln!("  Avg reply score:  {avg:.1}"),
        Err(e) => eprintln!("  Reply engagement: error ({e})"),
    }

    match storage::analytics::get_avg_tweet_engagement(pool).await {
        Ok(avg) => eprintln!("  Avg tweet score:  {avg:.1}"),
        Err(e) => eprintln!("  Tweet engagement: error ({e})"),
    }

    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FollowerSnapshotJson ──────────────────────────────────────────

    #[test]
    fn follower_snapshot_json_serializes() {
        let snap = FollowerSnapshotJson {
            date: "2025-01-15".to_string(),
            follower_count: 1000,
            following_count: 500,
            tweet_count: 200,
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("\"date\":\"2025-01-15\""));
        assert!(json.contains("\"follower_count\":1000"));
        assert!(json.contains("\"following_count\":500"));
        assert!(json.contains("\"tweet_count\":200"));
    }

    #[test]
    fn follower_snapshot_json_zero_counts() {
        let snap = FollowerSnapshotJson {
            date: "2025-01-01".to_string(),
            follower_count: 0,
            following_count: 0,
            tweet_count: 0,
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("\"follower_count\":0"));
    }

    // ── TopicJson ─────────────────────────────────────────────────────

    #[test]
    fn topic_json_serializes() {
        let topic = TopicJson {
            topic: "rust".to_string(),
            format: "tweet".to_string(),
            total_posts: 15,
            avg_performance: 72.5,
        };
        let json = serde_json::to_string(&topic).unwrap();
        assert!(json.contains("\"topic\":\"rust\""));
        assert!(json.contains("\"format\":\"tweet\""));
        assert!(json.contains("\"total_posts\":15"));
        assert!(json.contains("72.5"));
    }

    #[test]
    fn topic_json_empty_format() {
        let topic = TopicJson {
            topic: "general".to_string(),
            format: String::new(),
            total_posts: 0,
            avg_performance: 0.0,
        };
        let json = serde_json::to_string(&topic).unwrap();
        assert!(json.contains("\"format\":\"\""));
    }

    // ── EngagementJson ────────────────────────────────────────────────

    #[test]
    fn engagement_json_serializes() {
        let engagement = EngagementJson {
            avg_reply_score: 45.2,
            avg_tweet_score: 67.8,
        };
        let json = serde_json::to_string(&engagement).unwrap();
        assert!(json.contains("45.2"));
        assert!(json.contains("67.8"));
    }

    #[test]
    fn engagement_json_zeros() {
        let engagement = EngagementJson {
            avg_reply_score: 0.0,
            avg_tweet_score: 0.0,
        };
        let json = serde_json::to_string(&engagement).unwrap();
        assert!(json.contains("\"avg_reply_score\":0.0"));
        assert!(json.contains("\"avg_tweet_score\":0.0"));
    }

    // ── ContentMeasuredJson ───────────────────────────────────────────

    #[test]
    fn content_measured_json_serializes() {
        let content = ContentMeasuredJson {
            replies: 42,
            tweets: 10,
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"replies\":42"));
        assert!(json.contains("\"tweets\":10"));
    }

    // ── StatsOutput ───────────────────────────────────────────────────

    #[test]
    fn stats_output_serializes_empty() {
        let stats = StatsOutput {
            follower_trend: vec![],
            net_follower_change: None,
            top_topics: vec![],
            engagement: EngagementJson {
                avg_reply_score: 0.0,
                avg_tweet_score: 0.0,
            },
            content_measured: ContentMeasuredJson {
                replies: 0,
                tweets: 0,
            },
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"follower_trend\":[]"));
        assert!(json.contains("\"net_follower_change\":null"));
        assert!(json.contains("\"top_topics\":[]"));
    }

    #[test]
    fn stats_output_serializes_with_data() {
        let stats = StatsOutput {
            follower_trend: vec![
                FollowerSnapshotJson {
                    date: "2025-01-01".to_string(),
                    follower_count: 100,
                    following_count: 50,
                    tweet_count: 20,
                },
                FollowerSnapshotJson {
                    date: "2025-01-02".to_string(),
                    follower_count: 110,
                    following_count: 52,
                    tweet_count: 22,
                },
            ],
            net_follower_change: Some(10),
            top_topics: vec![TopicJson {
                topic: "rust".to_string(),
                format: "thread".to_string(),
                total_posts: 5,
                avg_performance: 80.0,
            }],
            engagement: EngagementJson {
                avg_reply_score: 65.0,
                avg_tweet_score: 70.0,
            },
            content_measured: ContentMeasuredJson {
                replies: 30,
                tweets: 15,
            },
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"net_follower_change\":10"));
        assert!(json.contains("\"rust\""));
    }

    // ── Net follower change computation ───────────────────────────────

    #[test]
    fn net_follower_change_positive() {
        let snapshots = vec![
            FollowerSnapshotJson {
                date: "d1".to_string(),
                follower_count: 110,
                following_count: 0,
                tweet_count: 0,
            },
            FollowerSnapshotJson {
                date: "d2".to_string(),
                follower_count: 100,
                following_count: 0,
                tweet_count: 0,
            },
        ];
        // Newest first in the snapshots vec
        let diff = snapshots[0].follower_count - snapshots[snapshots.len() - 1].follower_count;
        assert_eq!(diff, 10);
    }

    #[test]
    fn net_follower_change_negative() {
        let snapshots = vec![
            FollowerSnapshotJson {
                date: "d1".to_string(),
                follower_count: 90,
                following_count: 0,
                tweet_count: 0,
            },
            FollowerSnapshotJson {
                date: "d2".to_string(),
                follower_count: 100,
                following_count: 0,
                tweet_count: 0,
            },
        ];
        let diff = snapshots[0].follower_count - snapshots[snapshots.len() - 1].follower_count;
        assert_eq!(diff, -10);
    }

    #[test]
    fn net_follower_change_sign_formatting() {
        let diff = 10i64;
        let sign = if diff >= 0 { "+" } else { "" };
        assert_eq!(format!("{sign}{diff}"), "+10");

        let diff = -5i64;
        let sign = if diff >= 0 { "+" } else { "" };
        assert_eq!(format!("{sign}{diff}"), "-5");
    }

    // ── Topic display formatting ──────────────────────────────────────

    #[test]
    fn topic_format_dash_for_empty() {
        let format = "";
        let display = if format.is_empty() { "-" } else { format };
        assert_eq!(display, "-");
    }

    #[test]
    fn topic_format_shows_value() {
        let format = "thread";
        let display = if format.is_empty() { "-" } else { format };
        assert_eq!(display, "thread");
    }

    // ── JSON round-trip ───────────────────────────────────────────────

    #[test]
    fn stats_output_json_round_trip() {
        let stats = StatsOutput {
            follower_trend: vec![],
            net_follower_change: Some(42),
            top_topics: vec![],
            engagement: EngagementJson {
                avg_reply_score: 1.0,
                avg_tweet_score: 2.0,
            },
            content_measured: ContentMeasuredJson {
                replies: 3,
                tweets: 4,
            },
        };
        let json_str = serde_json::to_string(&stats).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["net_follower_change"], 42);
        assert_eq!(parsed["content_measured"]["replies"], 3);
        assert_eq!(parsed["content_measured"]["tweets"], 4);
    }
}

async fn print_performance_counts(pool: &storage::DbPool) {
    eprintln!("--- Content Measured ---");

    match storage::analytics::get_performance_counts(pool).await {
        Ok((replies, tweets)) => {
            eprintln!("  Replies measured: {replies}");
            eprintln!("  Tweets measured:  {tweets}");
        }
        Err(e) => {
            eprintln!("  Error fetching counts: {e}");
        }
    }
}
