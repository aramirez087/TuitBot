//! Report computation — orchestrates metric queries and recommendation generation
//! into a complete `StrategyReportRow` for persistence.

use chrono::{Datelike, NaiveDate, Utc};

use crate::config::Config;
use crate::error::StorageError;
use crate::storage::strategy::StrategyReportRow;
use crate::storage::DbPool;

use super::metrics;
use super::recommendations::{self, WeekMetrics};

/// Compute a fresh strategy report for the week containing `week_of`.
pub async fn compute_report(
    pool: &DbPool,
    config: &Config,
    week_of: NaiveDate,
) -> Result<StrategyReportRow, StorageError> {
    let (monday, sunday) = week_bounds(week_of);
    let start = format!("{monday}T00:00:00Z");
    let end_date = sunday.succ_opt().unwrap_or(sunday);
    let end = format!("{end_date}T00:00:00Z");

    // --- Gather all metrics in parallel-ish (sequential for simplicity) ---
    let actions = metrics::count_actions_in_range(pool, &start, &end).await?;
    let follower_start = metrics::get_follower_at_date(pool, &monday.to_string())
        .await?
        .unwrap_or(0);
    let follower_end = metrics::get_follower_at_date(pool, &sunday.to_string())
        .await?
        .unwrap_or(follower_start);
    let follower_delta = follower_end - follower_start;

    let avg_reply_score = metrics::avg_reply_score_in_range(pool, &start, &end).await?;
    let avg_tweet_score = metrics::avg_tweet_score_in_range(pool, &start, &end).await?;
    let acceptance_rate = metrics::reply_acceptance_rate(pool, &start, &end).await?;
    let top_topics = metrics::top_topics_in_range(pool, &start, &end, 5).await?;
    let bottom_topics = metrics::bottom_topics_in_range(pool, &start, &end, 5).await?;
    let top_content = metrics::top_content_in_range(pool, &start, &end, 5).await?;
    let distinct_topic_count = metrics::distinct_topic_count(pool, &start, &end).await?;

    // --- Estimated follow conversion ---
    let total_output = actions.replies + actions.tweets + actions.threads + actions.target_replies;
    let estimated_follow_conversion = if total_output > 0 {
        follower_delta.max(0) as f64 / total_output as f64
    } else {
        0.0
    };

    // --- Previous week report (for W-o-W comparison) ---
    let prev_monday = monday - chrono::Duration::days(7);
    let previous =
        crate::storage::strategy::get_strategy_report(pool, &prev_monday.to_string()).await?;

    // --- Generate recommendations ---
    let week_metrics = WeekMetrics {
        replies_sent: actions.replies,
        tweets_posted: actions.tweets,
        threads_posted: actions.threads,
        target_replies: actions.target_replies,
        follower_delta,
        avg_reply_score,
        avg_tweet_score,
        reply_acceptance_rate: acceptance_rate,
        top_topics: top_topics.clone(),
        bottom_topics: bottom_topics.clone(),
        distinct_topic_count,
        max_replies_per_week: i64::from(config.limits.max_replies_per_day) * 7,
        max_tweets_per_week: i64::from(config.limits.max_tweets_per_day) * 7,
    };
    let recs = recommendations::generate(&week_metrics, previous.as_ref());

    // --- Serialize JSON columns ---
    let top_topics_json = serde_json::to_string(&top_topics).unwrap_or_else(|_| "[]".to_string());
    let bottom_topics_json =
        serde_json::to_string(&bottom_topics).unwrap_or_else(|_| "[]".to_string());
    let top_content_json = serde_json::to_string(&top_content).unwrap_or_else(|_| "[]".to_string());
    let recommendations_json = serde_json::to_string(&recs).unwrap_or_else(|_| "[]".to_string());

    Ok(StrategyReportRow {
        id: 0,
        week_start: monday.to_string(),
        week_end: sunday.to_string(),
        replies_sent: actions.replies,
        tweets_posted: actions.tweets,
        threads_posted: actions.threads,
        target_replies: actions.target_replies,
        follower_start,
        follower_end,
        follower_delta,
        avg_reply_score,
        avg_tweet_score,
        reply_acceptance_rate: acceptance_rate,
        estimated_follow_conversion,
        top_topics_json,
        bottom_topics_json,
        top_content_json,
        recommendations_json,
        created_at: String::new(),
    })
}

/// Get the current week's report, computing it if missing or stale.
pub async fn get_or_compute_current(
    pool: &DbPool,
    config: &Config,
) -> Result<StrategyReportRow, StorageError> {
    let today = Utc::now().date_naive();
    let (monday, _sunday) = week_bounds(today);

    // Always recompute for current (in-progress) week
    let report = compute_report(pool, config, today).await?;
    crate::storage::strategy::insert_strategy_report(pool, &report).await?;

    // Re-read to get the assigned id and created_at
    crate::storage::strategy::get_strategy_report(pool, &monday.to_string())
        .await
        .map(|opt| opt.unwrap_or(report))
}

/// Force-recompute the current week's report (deletes existing, then recomputes).
pub async fn refresh_current(
    pool: &DbPool,
    config: &Config,
) -> Result<StrategyReportRow, StorageError> {
    let today = Utc::now().date_naive();
    let (monday, _) = week_bounds(today);
    crate::storage::strategy::delete_strategy_report(pool, &monday.to_string()).await?;
    get_or_compute_current(pool, config).await
}

/// Return the Monday and Sunday bounding the ISO week containing `date`.
fn week_bounds(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday = date.weekday();
    let days_since_monday = weekday.num_days_from_monday();
    let monday = date - chrono::Duration::days(i64::from(days_since_monday));
    let sunday = monday + chrono::Duration::days(6);
    (monday, sunday)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn week_bounds_monday() {
        let d = NaiveDate::from_ymd_opt(2026, 2, 23).unwrap(); // Monday
        let (mon, sun) = week_bounds(d);
        assert_eq!(mon.weekday(), Weekday::Mon);
        assert_eq!(sun.weekday(), Weekday::Sun);
        assert_eq!(mon, d);
    }

    #[test]
    fn week_bounds_wednesday() {
        let d = NaiveDate::from_ymd_opt(2026, 2, 25).unwrap(); // Wednesday
        let (mon, sun) = week_bounds(d);
        assert_eq!(mon, NaiveDate::from_ymd_opt(2026, 2, 23).unwrap());
        assert_eq!(sun, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
    }

    #[test]
    fn week_bounds_sunday() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(); // Sunday
        let (mon, sun) = week_bounds(d);
        assert_eq!(mon, NaiveDate::from_ymd_opt(2026, 2, 23).unwrap());
        assert_eq!(sun, d);
    }

    #[tokio::test]
    async fn compute_report_empty_db() {
        let pool = crate::storage::init_test_db().await.expect("init db");
        let config = Config::default();
        let d = NaiveDate::from_ymd_opt(2026, 2, 25).unwrap();
        let report = compute_report(&pool, &config, d).await.expect("compute");
        assert_eq!(report.week_start, "2026-02-23");
        assert_eq!(report.week_end, "2026-03-01");
        assert_eq!(report.replies_sent, 0);
        assert_eq!(report.follower_delta, 0);
    }
}
