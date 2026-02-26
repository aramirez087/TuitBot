//! X API usage tracking — stores per-call endpoint, method, status, and estimated cost.

use crate::error::StorageError;

use super::accounts::DEFAULT_ACCOUNT_ID;
use super::DbPool;

/// Known per-call costs for X API endpoints (pay-per-use pricing, Feb 2026).
///
/// Source: <https://developer.x.com/#pricing>
/// - Reading a post: $0.005
/// - User profile lookup: $0.010
/// - Creating a post: $0.010
///
/// Endpoints not yet priced default to $0.0 — update as X publishes more rates.
pub fn estimate_cost(endpoint: &str, method: &str) -> f64 {
    match (method, endpoint) {
        // Post reads
        ("GET", e) if e.starts_with("/tweets/search") => 0.005,
        ("GET", e) if e.starts_with("/tweets/") || e == "/tweets" => 0.005,
        ("GET", e) if e.contains("/mentions") => 0.005,
        ("GET", e) if e.contains("/tweets") && e.starts_with("/users/") => 0.005,

        // Bookmarks reads
        ("GET", e) if e.contains("/bookmarks") => 0.005,
        // Liked tweets reads
        ("GET", e) if e.contains("/liked_tweets") => 0.005,
        // Liking users reads
        ("GET", e) if e.contains("/liking_users") => 0.005,

        // User profile lookups
        ("GET", "/users/me") => 0.010,
        ("GET", "/users") => 0.010, // batch user lookup
        ("GET", e) if e.starts_with("/users/by/username/") => 0.010,
        ("GET", e)
            if e.starts_with("/users/") && !e.contains("/tweets") && !e.contains("/mentions") =>
        {
            0.010
        }

        // Post creation (tweet or reply)
        ("POST", "/tweets") => 0.010,

        // Like/unlike a tweet
        ("POST", e) if e.contains("/likes") => 0.010,
        ("DELETE", e) if e.contains("/likes") => 0.010,

        // Follow/unfollow a user
        ("POST", e) if e.contains("/following") => 0.010,
        ("DELETE", e) if e.contains("/following") => 0.010,

        // Bookmark/unbookmark a tweet
        ("POST", e) if e.contains("/bookmarks") => 0.010,
        ("DELETE", e) if e.contains("/bookmarks") => 0.010,

        // Unknown — default to zero until pricing is published
        _ => 0.0,
    }
}

/// Summary of X API usage across multiple time windows.
#[derive(Debug, serde::Serialize)]
pub struct XApiUsageSummary {
    pub cost_today: f64,
    pub cost_7d: f64,
    pub cost_30d: f64,
    pub cost_all_time: f64,
    pub calls_today: i64,
    pub calls_7d: i64,
    pub calls_30d: i64,
    pub calls_all_time: i64,
}

/// Daily X API usage aggregation for chart data.
#[derive(Debug, serde::Serialize)]
pub struct DailyXApiUsage {
    pub date: String,
    pub calls: i64,
    pub cost: f64,
}

/// Breakdown of X API usage by endpoint + method.
#[derive(Debug, serde::Serialize)]
pub struct EndpointBreakdown {
    pub endpoint: String,
    pub method: String,
    pub calls: i64,
    pub cost: f64,
    pub error_count: i64,
}

/// Insert a new X API usage record for a specific account.
pub async fn insert_x_api_usage_for(
    pool: &DbPool,
    account_id: &str,
    endpoint: &str,
    method: &str,
    status_code: i32,
    cost_usd: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO x_api_usage (account_id, endpoint, method, status_code, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(account_id)
    .bind(endpoint)
    .bind(method)
    .bind(status_code)
    .bind(cost_usd)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Insert a new X API usage record.
pub async fn insert_x_api_usage(
    pool: &DbPool,
    endpoint: &str,
    method: &str,
    status_code: i32,
    cost_usd: f64,
) -> Result<(), StorageError> {
    insert_x_api_usage_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        endpoint,
        method,
        status_code,
        cost_usd,
    )
    .await
}

/// Get usage summary across time windows for a specific account.
pub async fn get_usage_summary_for(
    pool: &DbPool,
    account_id: &str,
) -> Result<XApiUsageSummary, StorageError> {
    let row: (f64, i64, f64, i64, f64, i64, f64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN created_at >= date('now') THEN cost_usd ELSE 0.0 END), 0.0),
            COALESCE(SUM(CASE WHEN created_at >= date('now') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN created_at >= date('now', '-7 days') THEN cost_usd ELSE 0.0 END), 0.0),
            COALESCE(SUM(CASE WHEN created_at >= date('now', '-7 days') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN created_at >= date('now', '-30 days') THEN cost_usd ELSE 0.0 END), 0.0),
            COALESCE(SUM(CASE WHEN created_at >= date('now', '-30 days') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(cost_usd), 0.0),
            COUNT(*)
        FROM x_api_usage
        WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(XApiUsageSummary {
        cost_today: row.0,
        calls_today: row.1,
        cost_7d: row.2,
        calls_7d: row.3,
        cost_30d: row.4,
        calls_30d: row.5,
        cost_all_time: row.6,
        calls_all_time: row.7,
    })
}

/// Get usage summary across time windows.
pub async fn get_usage_summary(pool: &DbPool) -> Result<XApiUsageSummary, StorageError> {
    get_usage_summary_for(pool, DEFAULT_ACCOUNT_ID).await
}

/// Get daily usage aggregation for chart data for a specific account.
pub async fn get_daily_usage_for(
    pool: &DbPool,
    account_id: &str,
    days: u32,
) -> Result<Vec<DailyXApiUsage>, StorageError> {
    let rows: Vec<(String, i64, f64)> = sqlx::query_as(
        "SELECT
            date(created_at) as day,
            COUNT(*),
            COALESCE(SUM(cost_usd), 0.0)
        FROM x_api_usage
        WHERE account_id = ? AND created_at >= date('now', '-' || ? || ' days')
        GROUP BY day
        ORDER BY day",
    )
    .bind(account_id)
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(date, calls, cost)| DailyXApiUsage { date, calls, cost })
        .collect())
}

/// Get daily usage aggregation for chart data.
pub async fn get_daily_usage(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<DailyXApiUsage>, StorageError> {
    get_daily_usage_for(pool, DEFAULT_ACCOUNT_ID, days).await
}

/// Get usage breakdown by endpoint + method for a specific account.
pub async fn get_endpoint_breakdown_for(
    pool: &DbPool,
    account_id: &str,
    days: u32,
) -> Result<Vec<EndpointBreakdown>, StorageError> {
    let rows: Vec<(String, String, i64, f64, i64)> = sqlx::query_as(
        "SELECT
            endpoint,
            method,
            COUNT(*),
            COALESCE(SUM(cost_usd), 0.0),
            COALESCE(SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END), 0)
        FROM x_api_usage
        WHERE account_id = ? AND created_at >= date('now', '-' || ? || ' days')
        GROUP BY endpoint, method
        ORDER BY COUNT(*) DESC",
    )
    .bind(account_id)
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(endpoint, method, calls, cost, error_count)| EndpointBreakdown {
                endpoint,
                method,
                calls,
                cost,
                error_count,
            },
        )
        .collect())
}

/// Get usage breakdown by endpoint + method.
pub async fn get_endpoint_breakdown(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<EndpointBreakdown>, StorageError> {
    get_endpoint_breakdown_for(pool, DEFAULT_ACCOUNT_ID, days).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn insert_and_query_summary() {
        let pool = init_test_db().await.expect("init db");

        insert_x_api_usage(&pool, "/tweets/search/recent", "GET", 200, 0.005)
            .await
            .expect("insert");

        insert_x_api_usage(&pool, "/tweets", "POST", 201, 0.010)
            .await
            .expect("insert");

        let summary = get_usage_summary(&pool).await.expect("summary");
        assert_eq!(summary.calls_all_time, 2);
        assert!((summary.cost_all_time - 0.015).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn endpoint_breakdown_groups_correctly() {
        let pool = init_test_db().await.expect("init db");

        insert_x_api_usage(&pool, "/tweets/search/recent", "GET", 200, 0.005)
            .await
            .expect("insert");
        insert_x_api_usage(&pool, "/tweets/search/recent", "GET", 200, 0.005)
            .await
            .expect("insert");
        insert_x_api_usage(&pool, "/tweets", "POST", 201, 0.010)
            .await
            .expect("insert");
        insert_x_api_usage(&pool, "/tweets/search/recent", "GET", 429, 0.0)
            .await
            .expect("insert error");

        let breakdown = get_endpoint_breakdown(&pool, 30).await.expect("breakdown");
        assert_eq!(breakdown.len(), 2);

        let search = breakdown
            .iter()
            .find(|b| b.endpoint == "/tweets/search/recent")
            .unwrap();
        assert_eq!(search.calls, 3);
        assert_eq!(search.error_count, 1);

        let post = breakdown.iter().find(|b| b.method == "POST").unwrap();
        assert_eq!(post.calls, 1);
        assert_eq!(post.error_count, 0);
    }

    #[tokio::test]
    async fn empty_table_returns_zero_summary() {
        let pool = init_test_db().await.expect("init db");

        let summary = get_usage_summary(&pool).await.expect("summary");
        assert_eq!(summary.calls_all_time, 0);
        assert!(summary.cost_all_time.abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn estimate_cost_known_endpoints() {
        assert!((estimate_cost("/tweets/search/recent", "GET") - 0.005).abs() < f64::EPSILON);
        assert!((estimate_cost("/tweets/12345", "GET") - 0.005).abs() < f64::EPSILON);
        assert!((estimate_cost("/users/me", "GET") - 0.010).abs() < f64::EPSILON);
        assert!((estimate_cost("/users/by/username/jack", "GET") - 0.010).abs() < f64::EPSILON);
        assert!((estimate_cost("/tweets", "POST") - 0.010).abs() < f64::EPSILON);
        assert!((estimate_cost("/unknown", "DELETE") - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn daily_usage_returns_data() {
        let pool = init_test_db().await.expect("init db");

        insert_x_api_usage(&pool, "/tweets/search/recent", "GET", 200, 0.005)
            .await
            .expect("insert");

        let daily = get_daily_usage(&pool, 30).await.expect("daily");
        assert_eq!(daily.len(), 1);
        assert_eq!(daily[0].calls, 1);
    }
}
