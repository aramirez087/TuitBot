//! Implementation of the `replyguy stats` command.
//!
//! Displays analytics dashboard: follower trend, top-performing topics,
//! engagement rates, and weekly volume.

use replyguy_core::config::Config;
use replyguy_core::storage;

/// Execute the `replyguy stats` command.
pub async fn execute(config: &Config) -> anyhow::Result<()> {
    let pool = storage::init_db(&config.storage.db_path).await?;

    eprintln!();
    eprintln!("=== ReplyGuy Analytics ===");
    eprintln!();

    // 1. Follower trend (7 days)
    print_follower_trend(&pool).await;

    // 2. Top performing topics
    print_top_topics(&pool).await;

    // 3. Engagement rates
    print_engagement_rates(&pool).await;

    // 4. Performance counts
    print_performance_counts(&pool).await;

    eprintln!();

    pool.close().await;
    Ok(())
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
