use std::time::Instant;

use serde_json::Value;
use tuitbot_core::config::Config;

use super::{setup_db, validate_envelope, ToolRun, ITERATIONS};

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
            let result = crate::tools::workflow::capabilities::get_capabilities(
                &pool, &config, true, false, None,
            )
            .await;
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
            let result = crate::tools::workflow::health::health_check(&pool, None, &config).await;
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
            let result = crate::tools::workflow::analytics::get_stats(&pool, 7, &config).await;
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
            let result = crate::tools::workflow::approval::list_pending(&pool, &config).await;
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
            let result = crate::tools::workflow::discovery::list_unreplied_tweets_with_limit(
                &pool, 0.0, 10, &config,
            )
            .await;
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            validate_envelope(&result, "list_unreplied_tweets_with_limit");
        }
        runs.push(ToolRun {
            name: "list_unreplied_tweets_with_limit",
            times_ms: times,
        });
    }

    // Compute aggregate stats
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

    let artifacts = crate::tools::test_mocks::artifacts_dir();
    std::fs::create_dir_all(&artifacts).expect("create benchmark artifacts directory");

    let json_path = artifacts.join("task-01-baseline-benchmark.json");
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
        "\n**Aggregate** — P50: {global_p50:.3} ms, P95: {global_p95:.3} ms, \
         Min: {global_min:.3} ms, Max: {global_max:.3} ms\n"
    ));
    md.push_str(&format!(
        "\nMigrated: 5 / 27 tools — Schema pass rate: 100%\n"
    ));

    let md_path = artifacts.join("task-01-summary.md");
    std::fs::write(md_path, md).expect("write benchmark summary");
}
