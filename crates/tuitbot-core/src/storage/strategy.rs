//! CRUD operations for the `strategy_reports` table.
//!
//! Stores weekly strategy reports that aggregate engagement metrics,
//! follower growth, top/bottom topics, and actionable recommendations.

use super::DbPool;
use crate::error::StorageError;

/// A persisted weekly strategy report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct StrategyReportRow {
    pub id: i64,
    pub week_start: String,
    pub week_end: String,
    // Output volume
    pub replies_sent: i64,
    pub tweets_posted: i64,
    pub threads_posted: i64,
    pub target_replies: i64,
    // Follower metrics
    pub follower_start: i64,
    pub follower_end: i64,
    pub follower_delta: i64,
    // Engagement metrics
    pub avg_reply_score: f64,
    pub avg_tweet_score: f64,
    pub reply_acceptance_rate: f64,
    pub estimated_follow_conversion: f64,
    // JSON blobs (callers deserialize as needed)
    pub top_topics_json: String,
    pub bottom_topics_json: String,
    pub top_content_json: String,
    pub recommendations_json: String,
    // Metadata
    pub created_at: String,
}

/// Insert a new strategy report (or replace if the same `week_start` exists).
///
/// Returns the row id of the inserted report.
pub async fn insert_strategy_report(
    pool: &DbPool,
    report: &StrategyReportRow,
) -> Result<i64, StorageError> {
    let result = sqlx::query(
        "INSERT INTO strategy_reports \
         (week_start, week_end, replies_sent, tweets_posted, threads_posted, target_replies, \
          follower_start, follower_end, follower_delta, \
          avg_reply_score, avg_tweet_score, reply_acceptance_rate, estimated_follow_conversion, \
          top_topics_json, bottom_topics_json, top_content_json, recommendations_json) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(week_start) DO UPDATE SET \
         week_end = excluded.week_end, \
         replies_sent = excluded.replies_sent, \
         tweets_posted = excluded.tweets_posted, \
         threads_posted = excluded.threads_posted, \
         target_replies = excluded.target_replies, \
         follower_start = excluded.follower_start, \
         follower_end = excluded.follower_end, \
         follower_delta = excluded.follower_delta, \
         avg_reply_score = excluded.avg_reply_score, \
         avg_tweet_score = excluded.avg_tweet_score, \
         reply_acceptance_rate = excluded.reply_acceptance_rate, \
         estimated_follow_conversion = excluded.estimated_follow_conversion, \
         top_topics_json = excluded.top_topics_json, \
         bottom_topics_json = excluded.bottom_topics_json, \
         top_content_json = excluded.top_content_json, \
         recommendations_json = excluded.recommendations_json",
    )
    .bind(&report.week_start)
    .bind(&report.week_end)
    .bind(report.replies_sent)
    .bind(report.tweets_posted)
    .bind(report.threads_posted)
    .bind(report.target_replies)
    .bind(report.follower_start)
    .bind(report.follower_end)
    .bind(report.follower_delta)
    .bind(report.avg_reply_score)
    .bind(report.avg_tweet_score)
    .bind(report.reply_acceptance_rate)
    .bind(report.estimated_follow_conversion)
    .bind(&report.top_topics_json)
    .bind(&report.bottom_topics_json)
    .bind(&report.top_content_json)
    .bind(&report.recommendations_json)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(result.last_insert_rowid())
}

/// Get a strategy report by its `week_start` date (ISO date string, e.g. "2026-02-24").
pub async fn get_strategy_report(
    pool: &DbPool,
    week_start: &str,
) -> Result<Option<StrategyReportRow>, StorageError> {
    sqlx::query_as::<_, StrategyReportRow>("SELECT * FROM strategy_reports WHERE week_start = ?")
        .bind(week_start)
        .fetch_optional(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })
}

/// Get recent strategy reports, newest first.
pub async fn get_recent_reports(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<StrategyReportRow>, StorageError> {
    sqlx::query_as::<_, StrategyReportRow>(
        "SELECT * FROM strategy_reports ORDER BY week_start DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Delete a strategy report by `week_start` (used for refresh/recompute).
pub async fn delete_strategy_report(pool: &DbPool, week_start: &str) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM strategy_reports WHERE week_start = ?")
        .bind(week_start)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    fn sample_report(week_start: &str, week_end: &str) -> StrategyReportRow {
        StrategyReportRow {
            id: 0,
            week_start: week_start.to_string(),
            week_end: week_end.to_string(),
            replies_sent: 42,
            tweets_posted: 10,
            threads_posted: 2,
            target_replies: 5,
            follower_start: 1000,
            follower_end: 1050,
            follower_delta: 50,
            avg_reply_score: 65.5,
            avg_tweet_score: 72.3,
            reply_acceptance_rate: 0.25,
            estimated_follow_conversion: 0.012,
            top_topics_json: r#"[{"topic":"rust","avg_score":80}]"#.to_string(),
            bottom_topics_json: "[]".to_string(),
            top_content_json: "[]".to_string(),
            recommendations_json: "[]".to_string(),
            created_at: String::new(),
        }
    }

    #[tokio::test]
    async fn insert_and_get_report() {
        let pool = init_test_db().await.expect("init db");
        let report = sample_report("2026-02-24", "2026-03-02");

        let id = insert_strategy_report(&pool, &report)
            .await
            .expect("insert");
        assert!(id > 0);

        let fetched = get_strategy_report(&pool, "2026-02-24")
            .await
            .expect("get")
            .expect("should exist");

        assert_eq!(fetched.week_start, "2026-02-24");
        assert_eq!(fetched.replies_sent, 42);
        assert_eq!(fetched.follower_delta, 50);
    }

    #[tokio::test]
    async fn upsert_overwrites_existing() {
        let pool = init_test_db().await.expect("init db");
        let mut report = sample_report("2026-02-24", "2026-03-02");

        insert_strategy_report(&pool, &report)
            .await
            .expect("insert");

        report.replies_sent = 100;
        insert_strategy_report(&pool, &report)
            .await
            .expect("upsert");

        let fetched = get_strategy_report(&pool, "2026-02-24")
            .await
            .expect("get")
            .expect("should exist");
        assert_eq!(fetched.replies_sent, 100);
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let pool = init_test_db().await.expect("init db");
        let result = get_strategy_report(&pool, "2099-01-01").await.expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_recent_reports_ordering() {
        let pool = init_test_db().await.expect("init db");

        insert_strategy_report(&pool, &sample_report("2026-02-17", "2026-02-23"))
            .await
            .expect("insert");
        insert_strategy_report(&pool, &sample_report("2026-02-24", "2026-03-02"))
            .await
            .expect("insert");

        let reports = get_recent_reports(&pool, 10).await.expect("get");
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].week_start, "2026-02-24"); // newest first
        assert_eq!(reports[1].week_start, "2026-02-17");
    }

    #[tokio::test]
    async fn delete_strategy_report_works() {
        let pool = init_test_db().await.expect("init db");
        let report = sample_report("2026-02-24", "2026-03-02");

        insert_strategy_report(&pool, &report)
            .await
            .expect("insert");
        delete_strategy_report(&pool, "2026-02-24")
            .await
            .expect("delete");

        let result = get_strategy_report(&pool, "2026-02-24").await.expect("get");
        assert!(result.is_none());
    }
}
