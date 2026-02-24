//! LLM usage tracking — stores per-call token counts and costs.

use crate::error::StorageError;

use super::DbPool;

/// Summary of costs across multiple time windows.
#[derive(Debug, serde::Serialize)]
pub struct CostSummary {
    pub cost_today: f64,
    pub cost_7d: f64,
    pub cost_30d: f64,
    pub cost_all_time: f64,
    pub calls_today: i64,
    pub calls_7d: i64,
    pub calls_30d: i64,
    pub calls_all_time: i64,
}

/// Daily cost aggregation for chart data.
#[derive(Debug, serde::Serialize)]
pub struct DailyCostSummary {
    pub date: String,
    pub cost: f64,
    pub calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

/// Cost breakdown by provider + model.
#[derive(Debug, serde::Serialize)]
pub struct ModelCostBreakdown {
    pub provider: String,
    pub model: String,
    pub cost: f64,
    pub calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

/// Cost breakdown by generation type (reply/tweet/thread).
#[derive(Debug, serde::Serialize)]
pub struct TypeCostBreakdown {
    pub generation_type: String,
    pub cost: f64,
    pub calls: i64,
    pub avg_cost: f64,
}

/// Insert a new LLM usage record.
pub async fn insert_llm_usage(
    pool: &DbPool,
    generation_type: &str,
    provider: &str,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
    cost_usd: f64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO llm_usage (generation_type, provider, model, input_tokens, output_tokens, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(generation_type)
    .bind(provider)
    .bind(model)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(cost_usd)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get cost summary across time windows.
pub async fn get_cost_summary(pool: &DbPool) -> Result<CostSummary, StorageError> {
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
        FROM llm_usage",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(CostSummary {
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

/// Get daily cost aggregation for chart data.
pub async fn get_daily_costs(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<DailyCostSummary>, StorageError> {
    let rows: Vec<(String, f64, i64, i64, i64)> = sqlx::query_as(
        "SELECT
            date(created_at) as day,
            COALESCE(SUM(cost_usd), 0.0),
            COUNT(*),
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0)
        FROM llm_usage
        WHERE created_at >= date('now', '-' || ?1 || ' days')
        GROUP BY day
        ORDER BY day",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(date, cost, calls, input_tokens, output_tokens)| DailyCostSummary {
                date,
                cost,
                calls,
                input_tokens,
                output_tokens,
            },
        )
        .collect())
}

/// Get cost breakdown by provider + model.
pub async fn get_model_breakdown(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<ModelCostBreakdown>, StorageError> {
    let rows: Vec<(String, String, f64, i64, i64, i64)> = sqlx::query_as(
        "SELECT
            provider,
            model,
            COALESCE(SUM(cost_usd), 0.0),
            COUNT(*),
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0)
        FROM llm_usage
        WHERE created_at >= date('now', '-' || ?1 || ' days')
        GROUP BY provider, model
        ORDER BY SUM(cost_usd) DESC",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(
            |(provider, model, cost, calls, input_tokens, output_tokens)| ModelCostBreakdown {
                provider,
                model,
                cost,
                calls,
                input_tokens,
                output_tokens,
            },
        )
        .collect())
}

/// Get cost breakdown by generation type.
pub async fn get_type_breakdown(
    pool: &DbPool,
    days: u32,
) -> Result<Vec<TypeCostBreakdown>, StorageError> {
    let rows: Vec<(String, f64, i64)> = sqlx::query_as(
        "SELECT
            generation_type,
            COALESCE(SUM(cost_usd), 0.0),
            COUNT(*)
        FROM llm_usage
        WHERE created_at >= date('now', '-' || ?1 || ' days')
        GROUP BY generation_type
        ORDER BY SUM(cost_usd) DESC",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|(generation_type, cost, calls)| {
            let avg_cost = if calls > 0 { cost / calls as f64 } else { 0.0 };
            TypeCostBreakdown {
                generation_type,
                cost,
                calls,
                avg_cost,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn insert_and_query_summary() {
        let pool = init_test_db().await.expect("init db");

        insert_llm_usage(&pool, "reply", "openai", "gpt-4o-mini", 100, 50, 0.000045)
            .await
            .expect("insert");

        insert_llm_usage(&pool, "tweet", "openai", "gpt-4o-mini", 200, 80, 0.000063)
            .await
            .expect("insert");

        let summary = get_cost_summary(&pool).await.expect("summary");
        assert_eq!(summary.calls_all_time, 2);
        assert!(summary.cost_all_time > 0.0);
    }

    #[tokio::test]
    async fn model_breakdown_groups_correctly() {
        let pool = init_test_db().await.expect("init db");

        insert_llm_usage(&pool, "reply", "openai", "gpt-4o", 100, 50, 0.001)
            .await
            .expect("insert");
        insert_llm_usage(&pool, "reply", "openai", "gpt-4o", 100, 50, 0.001)
            .await
            .expect("insert");
        insert_llm_usage(&pool, "reply", "anthropic", "claude-sonnet", 100, 50, 0.002)
            .await
            .expect("insert");

        let breakdown = get_model_breakdown(&pool, 30).await.expect("breakdown");
        assert_eq!(breakdown.len(), 2);
    }

    #[tokio::test]
    async fn type_breakdown_groups_correctly() {
        let pool = init_test_db().await.expect("init db");

        insert_llm_usage(&pool, "reply", "openai", "gpt-4o", 100, 50, 0.001)
            .await
            .expect("insert");
        insert_llm_usage(&pool, "tweet", "openai", "gpt-4o", 100, 50, 0.001)
            .await
            .expect("insert");
        insert_llm_usage(&pool, "thread", "openai", "gpt-4o", 100, 50, 0.001)
            .await
            .expect("insert");

        let breakdown = get_type_breakdown(&pool, 30).await.expect("breakdown");
        assert_eq!(breakdown.len(), 3);
    }

    #[tokio::test]
    async fn empty_table_returns_zero_summary() {
        let pool = init_test_db().await.expect("init db");

        let summary = get_cost_summary(&pool).await.expect("summary");
        assert_eq!(summary.calls_all_time, 0);
        assert!((summary.cost_all_time).abs() < f64::EPSILON);
    }
}
