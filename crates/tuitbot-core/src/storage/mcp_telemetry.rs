//! MCP execution telemetry storage.
//!
//! Records tool invocations with latency, success/failure, error codes,
//! and policy decisions. Provides windowed aggregation queries for
//! observability MCP tools.

use super::DbPool;
use crate::error::StorageError;
use serde::Serialize;
use std::collections::HashMap;

/// A single telemetry record.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct TelemetryEntry {
    pub id: i64,
    pub tool_name: String,
    pub category: String,
    pub latency_ms: i64,
    pub success: bool,
    pub error_code: Option<String>,
    pub policy_decision: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Parameters for inserting a telemetry entry.
pub struct TelemetryParams<'a> {
    pub tool_name: &'a str,
    pub category: &'a str,
    pub latency_ms: u64,
    pub success: bool,
    pub error_code: Option<&'a str>,
    pub policy_decision: Option<&'a str>,
    pub metadata: Option<&'a str>,
}

/// Insert a telemetry entry.
pub async fn log_telemetry(
    pool: &DbPool,
    params: &TelemetryParams<'_>,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO mcp_telemetry \
         (tool_name, category, latency_ms, success, error_code, policy_decision, metadata) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.tool_name)
    .bind(params.category)
    .bind(params.latency_ms as i64)
    .bind(params.success)
    .bind(params.error_code)
    .bind(params.policy_decision)
    .bind(params.metadata)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Aggregated metrics for a single tool.
#[derive(Debug, Clone, Serialize)]
pub struct ToolMetrics {
    pub tool_name: String,
    pub category: String,
    pub total_calls: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub min_latency_ms: i64,
    pub max_latency_ms: i64,
}

/// Raw aggregate row from the metrics query.
#[derive(sqlx::FromRow)]
struct MetricsAggRow {
    tool_name: String,
    category: String,
    total: i64,
    successes: i64,
    failures: i64,
    avg_lat: f64,
    min_lat: i64,
    max_lat: i64,
}

/// Get aggregated metrics per tool in a time window.
pub async fn get_metrics_since(
    pool: &DbPool,
    since: &str,
) -> Result<Vec<ToolMetrics>, StorageError> {
    // First get basic aggregates per tool
    let rows: Vec<MetricsAggRow> = sqlx::query_as(
        "SELECT tool_name, category, \
         COUNT(*) as total, \
         SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successes, \
         SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failures, \
         AVG(latency_ms) as avg_lat, \
         MIN(latency_ms) as min_lat, \
         MAX(latency_ms) as max_lat \
         FROM mcp_telemetry WHERE created_at >= ? \
         GROUP BY tool_name, category ORDER BY total DESC",
    )
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let MetricsAggRow {
            tool_name,
            category,
            total,
            successes,
            failures,
            avg_lat,
            min_lat,
            max_lat,
        } = row;
        // Compute percentiles by fetching sorted latencies for this tool
        let latencies: Vec<(i64,)> = sqlx::query_as(
            "SELECT latency_ms FROM mcp_telemetry \
             WHERE created_at >= ? AND tool_name = ? ORDER BY latency_ms ASC",
        )
        .bind(since)
        .bind(&tool_name)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        let p50 = percentile(&latencies, 50);
        let p95 = percentile(&latencies, 95);
        let success_rate = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };

        results.push(ToolMetrics {
            tool_name,
            category,
            total_calls: total,
            success_count: successes,
            failure_count: failures,
            success_rate,
            avg_latency_ms: avg_lat,
            p50_latency_ms: p50 as f64,
            p95_latency_ms: p95 as f64,
            min_latency_ms: min_lat,
            max_latency_ms: max_lat,
        });
    }

    Ok(results)
}

/// Error breakdown: error_code → count, grouped by tool.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBreakdown {
    pub tool_name: String,
    pub error_code: String,
    pub count: i64,
    pub latest_at: String,
}

/// Get error distribution since a timestamp.
pub async fn get_error_breakdown(
    pool: &DbPool,
    since: &str,
) -> Result<Vec<ErrorBreakdown>, StorageError> {
    let rows: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT tool_name, COALESCE(error_code, 'unknown') as err, \
         COUNT(*) as cnt, MAX(created_at) as latest \
         FROM mcp_telemetry \
         WHERE created_at >= ? AND success = 0 \
         GROUP BY tool_name, error_code \
         ORDER BY cnt DESC",
    )
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(tool_name, error_code, count, latest_at)| ErrorBreakdown {
            tool_name,
            error_code,
            count,
            latest_at,
        })
        .collect())
}

/// Summary statistics across all tools.
#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySummary {
    pub total_calls: i64,
    pub total_successes: i64,
    pub total_failures: i64,
    pub overall_success_rate: f64,
    pub avg_latency_ms: f64,
    pub unique_tools: i64,
    pub policy_decisions: HashMap<String, i64>,
}

/// Get summary statistics since a timestamp.
pub async fn get_summary(pool: &DbPool, since: &str) -> Result<TelemetrySummary, StorageError> {
    let (total, successes, failures, avg_lat): (i64, i64, i64, f64) = sqlx::query_as(
        "SELECT COUNT(*), \
         SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), \
         SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END), \
         COALESCE(AVG(latency_ms), 0.0) \
         FROM mcp_telemetry WHERE created_at >= ?",
    )
    .bind(since)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let (unique_tools,): (i64,) =
        sqlx::query_as("SELECT COUNT(DISTINCT tool_name) FROM mcp_telemetry WHERE created_at >= ?")
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Query { source: e })?;

    let policy_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(policy_decision, 'none') as pd, COUNT(*) \
         FROM mcp_telemetry WHERE created_at >= ? \
         GROUP BY policy_decision",
    )
    .bind(since)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    let overall_success_rate = if total > 0 {
        successes as f64 / total as f64
    } else {
        0.0
    };

    Ok(TelemetrySummary {
        total_calls: total,
        total_successes: successes,
        total_failures: failures,
        overall_success_rate,
        avg_latency_ms: avg_lat,
        unique_tools,
        policy_decisions: policy_rows.into_iter().collect(),
    })
}

/// Get recent telemetry entries, ordered newest-first.
pub async fn get_recent_entries(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<TelemetryEntry>, StorageError> {
    sqlx::query_as::<_, TelemetryEntry>(
        "SELECT id, tool_name, category, latency_ms, success, \
         error_code, policy_decision, metadata, created_at \
         FROM mcp_telemetry ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })
}

/// Compute a percentile from sorted latency values.
fn percentile(sorted: &[(i64,)], pct: u32) -> i64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((pct as f64 / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx].0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    async fn log(
        pool: &DbPool,
        tool: &str,
        cat: &str,
        ms: u64,
        ok: bool,
        err: Option<&str>,
        policy: Option<&str>,
    ) {
        log_telemetry(
            pool,
            &TelemetryParams {
                tool_name: tool,
                category: cat,
                latency_ms: ms,
                success: ok,
                error_code: err,
                policy_decision: policy,
                metadata: None,
            },
        )
        .await
        .expect("log telemetry");
    }

    #[tokio::test]
    async fn log_and_retrieve_telemetry() {
        let pool = init_test_db().await.expect("init db");

        log(&pool, "get_stats", "analytics", 42, true, None, None).await;

        let metrics = get_metrics_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("metrics");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].tool_name, "get_stats");
        assert_eq!(metrics[0].total_calls, 1);
        assert_eq!(metrics[0].success_count, 1);
        assert_eq!(metrics[0].failure_count, 0);
    }

    #[tokio::test]
    async fn error_breakdown_groups_by_code() {
        let pool = init_test_db().await.expect("init db");

        log(
            &pool,
            "compose_tweet",
            "mutation",
            100,
            false,
            Some("policy_denied_blocked"),
            Some("deny"),
        )
        .await;
        log(
            &pool,
            "compose_tweet",
            "mutation",
            50,
            false,
            Some("policy_denied_blocked"),
            Some("deny"),
        )
        .await;
        log(
            &pool,
            "compose_tweet",
            "mutation",
            80,
            false,
            Some("db_error"),
            None,
        )
        .await;

        let errors = get_error_breakdown(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("errors");
        assert_eq!(errors.len(), 2);
        // Sorted by count desc
        assert_eq!(errors[0].error_code, "policy_denied_blocked");
        assert_eq!(errors[0].count, 2);
        assert_eq!(errors[1].error_code, "db_error");
        assert_eq!(errors[1].count, 1);
    }

    #[tokio::test]
    async fn summary_aggregates_correctly() {
        let pool = init_test_db().await.expect("init db");

        log(&pool, "get_stats", "analytics", 10, true, None, None).await;
        log(&pool, "get_stats", "analytics", 20, true, None, None).await;
        log(
            &pool,
            "compose_tweet",
            "mutation",
            50,
            false,
            Some("err"),
            Some("deny"),
        )
        .await;

        let summary = get_summary(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("summary");
        assert_eq!(summary.total_calls, 3);
        assert_eq!(summary.total_successes, 2);
        assert_eq!(summary.total_failures, 1);
        assert_eq!(summary.unique_tools, 2);
    }

    #[tokio::test]
    async fn empty_telemetry_returns_empty() {
        let pool = init_test_db().await.expect("init db");

        let metrics = get_metrics_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("metrics");
        assert!(metrics.is_empty());

        let errors = get_error_breakdown(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("errors");
        assert!(errors.is_empty());

        let summary = get_summary(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("summary");
        assert_eq!(summary.total_calls, 0);
        assert_eq!(summary.overall_success_rate, 0.0);
    }

    #[tokio::test]
    async fn percentile_calculation() {
        let pool = init_test_db().await.expect("init db");

        for ms in [10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
            log(&pool, "test_tool", "test", ms, true, None, None).await;
        }

        let metrics = get_metrics_since(&pool, "2000-01-01T00:00:00Z")
            .await
            .expect("metrics");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].total_calls, 10);
        assert_eq!(metrics[0].min_latency_ms, 10);
        assert_eq!(metrics[0].max_latency_ms, 100);
        // p50 ~ 50-60, p95 ~ 90-100
        assert!(metrics[0].p50_latency_ms >= 50.0);
        assert!(metrics[0].p95_latency_ms >= 90.0);
    }
}
