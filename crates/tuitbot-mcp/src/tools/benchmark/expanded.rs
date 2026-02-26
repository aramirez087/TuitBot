use tuitbot_core::config::Config;

use super::mocks::{BenchMockProvider, BenchMockXApiClient};
use super::{bench_tool, setup_db, ToolRun, P95_THRESHOLD_MS};
use crate::tools::test_mocks::artifacts_dir;

#[tokio::test]
async fn session09_expanded_benchmark() {
    let pool = setup_db().await;
    let config = Config::default();
    let mut config_with_keywords = Config::default();
    config_with_keywords.business.product_keywords = vec!["rust".to_string()];
    config_with_keywords.business.industry_topics = vec!["software".to_string()];

    let mut runs: Vec<ToolRun> = Vec::new();

    // ── Kernel read tools (via BenchMockProvider) ────────────────
    use crate::kernel::{read, utils, write};

    bench_tool("kernel::get_tweet", &mut runs, || async {
        read::get_tweet(&BenchMockProvider, "t1").await
    })
    .await;

    bench_tool("kernel::search_tweets", &mut runs, || async {
        read::search_tweets(&BenchMockProvider, "q", 10, None, None).await
    })
    .await;

    bench_tool("kernel::get_followers", &mut runs, || async {
        read::get_followers(&BenchMockProvider, "u1", 10, None).await
    })
    .await;

    bench_tool("kernel::get_user_by_id", &mut runs, || async {
        read::get_user_by_id(&BenchMockProvider, "u1").await
    })
    .await;

    bench_tool("kernel::get_me", &mut runs, || async {
        utils::get_me(&BenchMockProvider).await
    })
    .await;

    // ── Kernel write tools (via BenchMockXApiClient) ─────────────
    bench_tool("kernel::post_tweet", &mut runs, || async {
        write::post_tweet(&BenchMockXApiClient, "Hello!", None).await
    })
    .await;

    bench_tool("kernel::reply_to_tweet", &mut runs, || async {
        write::reply_to_tweet(&BenchMockXApiClient, "Great!", "t1", None).await
    })
    .await;

    // ── Config tools ─────────────────────────────────────────────
    {
        let cfg = config_with_keywords.clone();
        bench_tool("score_tweet", &mut runs, || {
            let c = &cfg;
            async move {
                let input = crate::tools::scoring::ScoreTweetInput {
                    text: "Rust is great for async",
                    author_username: "dev",
                    author_followers: 1000,
                    likes: 5,
                    retweets: 2,
                    replies: 1,
                    created_at: "2026-02-24T12:00:00Z",
                };
                crate::tools::scoring::score_tweet(c, &input)
            }
        })
        .await;
    }

    {
        let cfg = config.clone();
        bench_tool("get_config", &mut runs, || {
            let c = &cfg;
            async move { crate::tools::config::get_config(c) }
        })
        .await;
    }

    {
        let cfg = config.clone();
        bench_tool("validate_config", &mut runs, || {
            let c = &cfg;
            async move { crate::tools::config::validate_config(c) }
        })
        .await;
    }

    // ── Telemetry tools ──────────────────────────────────────────
    bench_tool("get_mcp_tool_metrics", &mut runs, || {
        let p = pool.clone();
        async move { crate::tools::workflow::telemetry::get_mcp_tool_metrics(&p, 24).await }
    })
    .await;

    bench_tool("get_mcp_error_breakdown", &mut runs, || {
        let p = pool.clone();
        async move { crate::tools::workflow::telemetry::get_mcp_error_breakdown(&p, 24).await }
    })
    .await;

    // ── Workflow tools (existing) ────────────────────────────────
    bench_tool("get_capabilities", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move {
            crate::tools::workflow::capabilities::get_capabilities(&p, &c, true, false, None, &[])
                .await
        }
    })
    .await;

    bench_tool("health_check", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { crate::tools::workflow::health::health_check(&p, None, &c).await }
    })
    .await;

    bench_tool("get_stats", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { crate::tools::workflow::analytics::get_stats(&p, 7, &c).await }
    })
    .await;

    bench_tool("list_pending", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { crate::tools::workflow::approval::list_pending(&p, &c).await }
    })
    .await;

    // ── Aggregate stats ──────────────────────────────────────────
    let mut sorted: Vec<f64> = runs.iter().flat_map(|r| r.times_ms.clone()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let global_p50 = sorted[sorted.len() / 2];
    let global_p95_idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    let global_p95 = sorted[global_p95_idx.min(sorted.len() - 1)];
    let global_min = sorted.first().copied().unwrap_or(0.0);
    let global_max = sorted.last().copied().unwrap_or(0.0);

    // Build categories for the report
    let kernel_read: Vec<&ToolRun> = runs
        .iter()
        .filter(|r| r.name.starts_with("kernel::get") || r.name.starts_with("kernel::search"))
        .collect();
    let kernel_write: Vec<&ToolRun> = runs
        .iter()
        .filter(|r| r.name.starts_with("kernel::post") || r.name.starts_with("kernel::reply"))
        .collect();
    let config_tools: Vec<&ToolRun> = runs
        .iter()
        .filter(|r| {
            r.name == "score_tweet" || r.name == "get_config" || r.name == "validate_config"
        })
        .collect();
    let telemetry: Vec<&ToolRun> = runs
        .iter()
        .filter(|r| r.name.starts_with("get_mcp_"))
        .collect();

    fn category_p95(tools: &[&ToolRun]) -> f64 {
        let mut all: Vec<f64> = tools.iter().flat_map(|r| r.times_ms.clone()).collect();
        if all.is_empty() {
            return 0.0;
        }
        all.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((all.len() as f64) * 0.95).ceil() as usize - 1;
        all[idx.min(all.len() - 1)]
    }

    // Write latency report
    let mut md = String::from("# Session 09 — Latency Report\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));
    md.push_str(&format!("**Tools benchmarked:** {}\n\n", runs.len()));
    md.push_str("## Per-tool Results\n\n");
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

    md.push_str("\n## Category Breakdown\n\n");
    md.push_str("| Category | Tools | P95 (ms) |\n");
    md.push_str("|----------|-------|----------|\n");
    md.push_str(&format!(
        "| Kernel read | {} | {:.3} |\n",
        kernel_read.len(),
        category_p95(&kernel_read)
    ));
    md.push_str(&format!(
        "| Kernel write | {} | {:.3} |\n",
        kernel_write.len(),
        category_p95(&kernel_write)
    ));
    md.push_str(&format!(
        "| Config | {} | {:.3} |\n",
        config_tools.len(),
        category_p95(&config_tools)
    ));
    md.push_str(&format!(
        "| Telemetry | {} | {:.3} |\n",
        telemetry.len(),
        category_p95(&telemetry)
    ));

    md.push_str(&format!(
        "\n## Aggregate\n\n\
         **P50:** {global_p50:.3} ms | **P95:** {global_p95:.3} ms | \
         **Min:** {global_min:.3} ms | **Max:** {global_max:.3} ms\n\n\
         ## P95 Gate\n\n**Global P95:** {global_p95:.3} ms\n\
         **Threshold:** {P95_THRESHOLD_MS:.1} ms\n\
         **Status:** {}\n",
        if global_p95 < P95_THRESHOLD_MS {
            "PASS"
        } else {
            "FAIL"
        }
    ));

    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");
    std::fs::write(dir.join("session-09-latency-report.md"), &md).expect("write latency report");

    assert!(
        global_p95 < P95_THRESHOLD_MS,
        "P95 {global_p95:.3} ms exceeds threshold {P95_THRESHOLD_MS:.1} ms"
    );
}
