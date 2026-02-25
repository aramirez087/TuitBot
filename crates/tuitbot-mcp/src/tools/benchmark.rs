//! Baseline benchmark for migrated MCP tools.
//!
//! Runs each migrated tool multiple times, validates the envelope schema,
//! computes timing statistics, and writes artifact files.

use std::time::Instant;

use serde_json::Value;

use tuitbot_core::config::{Config, IntervalsConfig, LimitsConfig};
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

use super::response::ToolResponse;

/// Number of iterations per tool for the benchmark.
const ITERATIONS: usize = 5;

struct ToolRun {
    name: &'static str,
    times_ms: Vec<f64>,
}

impl ToolRun {
    fn avg(&self) -> f64 {
        let sum: f64 = self.times_ms.iter().sum();
        sum / self.times_ms.len() as f64
    }
    fn min(&self) -> f64 {
        self.times_ms.iter().cloned().fold(f64::INFINITY, f64::min)
    }
    fn max(&self) -> f64 {
        self.times_ms
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }
    fn p50(&self) -> f64 {
        let mut sorted = self.times_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    }
    fn p95(&self) -> f64 {
        let mut sorted = self.times_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
        sorted[idx.min(sorted.len() - 1)]
    }
}

async fn setup_db() -> DbPool {
    let pool = storage::init_test_db().await.expect("init db");
    let limits = LimitsConfig {
        max_replies_per_day: 5,
        max_tweets_per_day: 6,
        max_threads_per_week: 1,
        min_action_delay_seconds: 30,
        max_action_delay_seconds: 120,
        max_replies_per_author_per_day: 1,
        banned_phrases: vec![],
        product_mention_ratio: 0.2,
    };
    let intervals = IntervalsConfig {
        mentions_check_seconds: 300,
        discovery_search_seconds: 600,
        content_post_window_seconds: 14400,
        thread_interval_seconds: 604800,
    };
    storage::rate_limits::init_rate_limits(&pool, &limits, &intervals)
        .await
        .expect("init rate limits");
    pool
}

fn validate_envelope(json_str: &str, tool_name: &str) {
    let parsed: ToolResponse =
        serde_json::from_str(json_str).unwrap_or_else(|e| panic!("{tool_name}: invalid JSON: {e}"));
    assert!(
        parsed.success,
        "{tool_name}: expected success=true, got false"
    );
    // Also check the raw JSON has the expected top-level key
    let raw: Value = serde_json::from_str(json_str).unwrap();
    assert!(
        raw.get("success").is_some(),
        "{tool_name}: missing top-level 'success' key"
    );
    assert!(
        raw.get("data").is_some(),
        "{tool_name}: missing top-level 'data' key"
    );
}

#[tokio::test]
async fn generate_baseline_benchmark() {
    let pool = setup_db().await;
    let config = Config::default();

    let mut runs: Vec<ToolRun> = Vec::new();

    // 1. get_capabilities
    {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let result =
                super::capabilities::get_capabilities(&pool, &config, true, false, None).await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "get_capabilities");
        }
        runs.push(ToolRun {
            name: "get_capabilities",
            times_ms: times,
        });
    }

    // 2. health_check
    {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let result = super::health::health_check(&pool, None, &config).await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "health_check");
        }
        runs.push(ToolRun {
            name: "health_check",
            times_ms: times,
        });
    }

    // 3. get_stats
    {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let result = super::analytics::get_stats(&pool, 7, &config).await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "get_stats");
        }
        runs.push(ToolRun {
            name: "get_stats",
            times_ms: times,
        });
    }

    // 4. list_pending
    {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let result = super::approval::list_pending(&pool, &config).await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "list_pending");
        }
        runs.push(ToolRun {
            name: "list_pending",
            times_ms: times,
        });
    }

    // 5. list_unreplied_tweets_with_limit
    {
        let mut times = Vec::new();
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let result =
                super::discovery::list_unreplied_tweets_with_limit(&pool, 0.0, 10, &config).await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "list_unreplied_tweets_with_limit");
        }
        runs.push(ToolRun {
            name: "list_unreplied_tweets_with_limit",
            times_ms: times,
        });
    }

    // Compute aggregate stats across all tools
    let all_times: Vec<f64> = runs.iter().flat_map(|r| r.times_ms.clone()).collect();
    let mut sorted = all_times.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let global_p50 = sorted[sorted.len() / 2];
    let global_p95_idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    let global_p95 = sorted[global_p95_idx.min(sorted.len() - 1)];
    let global_min = sorted.first().copied().unwrap_or(0.0);
    let global_max = sorted.last().copied().unwrap_or(0.0);

    // Build JSON report
    let tool_results: Vec<Value> = runs
        .iter()
        .map(|r| {
            serde_json::json!({
                "tool": r.name,
                "iterations": ITERATIONS,
                "avg_ms": format!("{:.3}", r.avg()),
                "min_ms": format!("{:.3}", r.min()),
                "max_ms": format!("{:.3}", r.max()),
                "p50_ms": format!("{:.3}", r.p50()),
                "p95_ms": format!("{:.3}", r.p95()),
            })
        })
        .collect();

    let report = serde_json::json!({
        "benchmark": "task-01-baseline",
        "migrated_tools": 5,
        "total_tools": 27,
        "iterations_per_tool": ITERATIONS,
        "schema_pass_rate": "100%",
        "aggregate": {
            "p50_ms": format!("{global_p50:.3}"),
            "p95_ms": format!("{global_p95:.3}"),
            "min_ms": format!("{global_min:.3}"),
            "max_ms": format!("{global_max:.3}"),
        },
        "tools": tool_results,
    });

    let json_path = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/roadmap/artifacts/task-01-baseline-benchmark.json"
    ));
    if let Some(parent) = json_path.parent() {
        std::fs::create_dir_all(parent).expect("create benchmark artifacts directory");
    }
    std::fs::write(&json_path, serde_json::to_string_pretty(&report).unwrap())
        .expect("write benchmark JSON");

    // Build markdown summary
    let mut md = String::from("# Task 01 — Baseline Benchmark\n\n");
    md.push_str("| Tool | Avg (ms) | P50 (ms) | P95 (ms) | Min (ms) | Max (ms) |\n");
    md.push_str("|------|----------|----------|----------|----------|----------|\n");
    for r in &runs {
        md.push_str(&format!(
            "| {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |\n",
            r.name,
            r.avg(),
            r.p50(),
            r.p95(),
            r.min(),
            r.max(),
        ));
    }
    md.push_str(&format!(
        "\n**Aggregate** — P50: {global_p50:.3} ms, P95: {global_p95:.3} ms, Min: {global_min:.3} ms, Max: {global_max:.3} ms\n"
    ));
    md.push_str(&format!(
        "\nMigrated: 5 / 27 tools — Schema pass rate: 100%\n"
    ));

    let md_path = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/roadmap/artifacts/task-01-summary.md"
    ));
    if let Some(parent) = md_path.parent() {
        std::fs::create_dir_all(parent).expect("create benchmark artifacts directory");
    }
    std::fs::write(md_path, md).expect("write benchmark summary");
}
