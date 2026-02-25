//! Topic performance snapshot with double-down/reduce recommendations.
//!
//! Queries time-windowed performance data from original tweets and their
//! measured engagement to produce ranked topic analysis with actionable
//! recommendations grounded in stored data.

use crate::error::StorageError;
use crate::storage::DbPool;
use serde::Serialize;

/// Complete topic performance snapshot for a lookback window.
#[derive(Debug, Clone, Serialize)]
pub struct TopicSnapshot {
    pub lookback_days: u32,
    pub topics: Vec<TopicAnalysis>,
    pub overall_avg_performance: f64,
    pub total_posts_analyzed: i64,
}

/// Performance analysis for a single topic.
#[derive(Debug, Clone, Serialize)]
pub struct TopicAnalysis {
    pub topic: String,
    pub post_count: i64,
    pub avg_performance: f64,
    pub performance_vs_average: f64,
    pub recommendation: String,
    pub provenance: TopicProvenance,
}

/// Evidence trail showing the data behind the recommendation.
#[derive(Debug, Clone, Serialize)]
pub struct TopicProvenance {
    pub best_content_preview: String,
    pub best_performance_score: f64,
    pub worst_performance_score: f64,
}

type AggRow = (String, i64, f64, f64, f64);

/// Build a topic performance snapshot for the given lookback window.
///
/// Returns topics ranked by average performance, each annotated with
/// a recommendation ("double_down", "reduce", "maintain", or "experiment")
/// and provenance data showing which posts drove the score.
pub async fn get_topic_snapshot(
    pool: &DbPool,
    lookback_days: u32,
) -> Result<TopicSnapshot, StorageError> {
    let since = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(lookback_days as i64))
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // Aggregate topic performance within the window
    let agg_rows: Vec<AggRow> = sqlx::query_as(
        "SELECT ot.topic, \
                COUNT(*) as post_count, \
                AVG(COALESCE(tp.performance_score, 0.0)) as avg_perf, \
                MAX(COALESCE(tp.performance_score, 0.0)) as best, \
                MIN(COALESCE(tp.performance_score, 0.0)) as worst \
         FROM original_tweets ot \
         LEFT JOIN tweet_performance tp ON tp.tweet_id = ot.tweet_id \
         WHERE ot.created_at >= ? \
           AND ot.topic IS NOT NULL AND ot.topic != '' \
           AND ot.status = 'sent' \
         GROUP BY ot.topic \
         ORDER BY avg_perf DESC",
    )
    .bind(&since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    if agg_rows.is_empty() {
        return Ok(TopicSnapshot {
            lookback_days,
            topics: Vec::new(),
            overall_avg_performance: 0.0,
            total_posts_analyzed: 0,
        });
    }

    // Compute weighted overall average
    let total_posts: i64 = agg_rows.iter().map(|r| r.1).sum();
    let weighted_sum: f64 = agg_rows.iter().map(|r| r.2 * r.1 as f64).sum();
    let overall_avg = if total_posts > 0 {
        weighted_sum / total_posts as f64
    } else {
        0.0
    };

    // Build topic analysis with provenance
    let mut topics = Vec::with_capacity(agg_rows.len());
    for (topic, post_count, avg_perf, best, worst) in &agg_rows {
        let preview = query_best_content_preview(pool, topic, &since).await?;
        let vs_avg = if overall_avg > 0.0 {
            avg_perf / overall_avg
        } else {
            1.0
        };
        let recommendation = classify_topic(*post_count, vs_avg);

        topics.push(TopicAnalysis {
            topic: topic.clone(),
            post_count: *post_count,
            avg_performance: *avg_perf,
            performance_vs_average: vs_avg,
            recommendation,
            provenance: TopicProvenance {
                best_content_preview: preview,
                best_performance_score: *best,
                worst_performance_score: *worst,
            },
        });
    }

    Ok(TopicSnapshot {
        lookback_days,
        topics,
        overall_avg_performance: overall_avg,
        total_posts_analyzed: total_posts,
    })
}

async fn query_best_content_preview(
    pool: &DbPool,
    topic: &str,
    since: &str,
) -> Result<String, StorageError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT SUBSTR(ot.content, 1, 120) \
         FROM original_tweets ot \
         LEFT JOIN tweet_performance tp ON tp.tweet_id = ot.tweet_id \
         WHERE ot.topic = ? AND ot.created_at >= ? AND ot.status = 'sent' \
         ORDER BY COALESCE(tp.performance_score, 0.0) DESC \
         LIMIT 1",
    )
    .bind(topic)
    .bind(since)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.map(|(s,)| s).unwrap_or_default())
}

/// Classify a topic into an actionable recommendation.
///
/// - `double_down`: ≥3 posts and avg > 1.5× overall avg
/// - `reduce`: ≥3 posts and avg < 0.5× overall avg
/// - `experiment`: < 3 posts (insufficient data)
/// - `maintain`: everything else
fn classify_topic(post_count: i64, performance_vs_average: f64) -> String {
    if post_count < 3 {
        "experiment".to_string()
    } else if performance_vs_average > 1.5 {
        "double_down".to_string()
    } else if performance_vs_average < 0.5 {
        "reduce".to_string()
    } else {
        "maintain".to_string()
    }
}
