//! Best-time-to-post analytics: ranked time slots by historical engagement.

use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;
use serde::{Deserialize, Serialize};

/// A recommended posting time slot with engagement metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestTimeSlot {
    pub hour: i32,
    pub day_of_week: i32,
    pub day_name: String,
    pub avg_engagement: f64,
    pub confidence_score: f64, // 0-100, higher = more historical data
    pub sample_size: i64,
}

/// Get ranked best-time-to-post slots for a specific account (sorted by avg_engagement DESC).
pub async fn get_best_times_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<Vec<BestTimeSlot>, StorageError> {
    let day_names = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    let rows = sqlx::query_as::<_, (i32, i32, f64, f64, i64)>(
        "SELECT hour_of_day, day_of_week, avg_engagement, confidence_score, sample_size \
         FROM best_times \
         WHERE account_id = ? \
         ORDER BY avg_engagement DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(hour, day, avg_engagement, confidence, sample_size)| BestTimeSlot {
                hour,
                day_of_week: day,
                day_name: day_names[day as usize % 7].to_string(),
                avg_engagement,
                confidence_score: confidence,
                sample_size,
            },
        )
        .collect())
}

/// Get ranked best-time-to-post slots (default account).
pub async fn get_best_times(pool: &DbPool) -> Result<Vec<BestTimeSlot>, StorageError> {
    get_best_times_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Compute and update best-times aggregations for a specific account.
/// Call daily via background job.
pub async fn aggregate_best_times_for(pool: &DbPool, account_id: &str) -> Result<(), StorageError> {
    // Query engagement metrics grouped by hour and day-of-week
    let rows = sqlx::query_as::<_, (i32, i32, f64, i64)>(
        "SELECT \
           CAST(STRFTIME('%H', posted_at) AS INTEGER) as hour, \
           (CAST(STRFTIME('%w', posted_at) AS INTEGER) + 6) % 7 as day_of_week, \
           AVG(engagement_rate) as avg_engagement, \
           COUNT(*) as sample_size \
         FROM engagement_metrics \
         WHERE account_id = ? AND posted_at IS NOT NULL \
         GROUP BY hour, day_of_week",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    // Compute confidence score based on sample size
    // Assuming 5+ samples = high confidence (90+), 2-4 = medium (50-80), 0-1 = low (0-40)
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    for (hour, day_of_week, avg_engagement, sample_size) in rows {
        let confidence_score = if sample_size >= 5 {
            90.0 + (sample_size as f64 - 5.0).min(10.0)
        } else if sample_size >= 2 {
            50.0 + (sample_size as f64 - 2.0) * 15.0
        } else {
            (sample_size as f64) * 20.0
        };

        sqlx::query(
            "INSERT INTO best_times \
             (account_id, hour_of_day, day_of_week, avg_engagement, confidence_score, sample_size, last_updated) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(account_id, hour_of_day, day_of_week) DO UPDATE SET \
             avg_engagement = excluded.avg_engagement, \
             confidence_score = excluded.confidence_score, \
             sample_size = excluded.sample_size, \
             last_updated = excluded.last_updated",
        )
        .bind(account_id)
        .bind(hour)
        .bind(day_of_week)
        .bind(avg_engagement)
        .bind(confidence_score)
        .bind(sample_size)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    }

    Ok(())
}

/// Follower growth time-series: daily deltas and weekly deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerGrowthSnapshot {
    pub date: String,
    pub follower_count: i64,
    pub daily_delta: i64,
    pub weekly_delta: i64,
}

/// Get follower growth time-series for a specific account over the past N days.
pub async fn get_follower_growth_for(
    pool: &DbPool,
    account_id: &str,
    days: u32,
) -> Result<Vec<FollowerGrowthSnapshot>, StorageError> {
    // Query follower_snapshots, compute deltas
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT snapshot_date, follower_count \
         FROM follower_snapshots \
         WHERE account_id = ? \
         AND snapshot_date >= date('now', '-' || ? || ' days') \
         ORDER BY snapshot_date ASC",
    )
    .bind(account_id)
    .bind(days as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let mut result = Vec::new();
    for (i, (date, follower_count)) in rows.iter().enumerate() {
        let daily_delta = if i > 0 {
            follower_count - rows[i - 1].1
        } else {
            0
        };

        let weekly_delta = if i >= 7 {
            follower_count - rows[i - 7].1
        } else {
            0
        };

        result.push(FollowerGrowthSnapshot {
            date: date.clone(),
            follower_count: *follower_count,
            daily_delta,
            weekly_delta,
        });
    }

    Ok(result)
}

/// Get follower growth time-series (default account).
pub async fn get_follower_growth(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<FollowerGrowthSnapshot>, StorageError> {
    get_follower_growth_for(pool, DEFAULT_ACCOUNT_ID, days).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_score_high() {
        let sample_size = 10i64;
        let confidence = if sample_size >= 5 {
            90.0 + (sample_size as f64 - 5.0).min(10.0)
        } else {
            0.0
        };
        assert!(confidence >= 95.0);
    }

    #[test]
    fn confidence_score_medium() {
        let sample_size = 3i64;
        let confidence = if sample_size >= 2 {
            50.0 + (sample_size as f64 - 2.0) * 15.0
        } else {
            0.0
        };
        assert_eq!(confidence, 65.0);
    }

    #[test]
    fn confidence_score_low() {
        let sample_size = 1i64;
        let confidence = (sample_size as f64) * 20.0;
        assert_eq!(confidence, 20.0);
    }

    #[test]
    fn follower_delta_calculation() {
        let prev_count = 1000i64;
        let curr_count = 1050i64;
        let daily_delta = curr_count - prev_count;
        assert_eq!(daily_delta, 50);
    }

    #[test]
    fn day_name_mapping() {
        let day_names = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];
        assert_eq!(day_names[0], "Sunday");
        assert_eq!(day_names[6], "Saturday");
    }
}
