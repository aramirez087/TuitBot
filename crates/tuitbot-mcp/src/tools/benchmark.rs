//! Baseline benchmark for migrated MCP tools.
//!
//! Runs each migrated tool multiple times, validates the envelope schema,
//! computes timing statistics, and writes artifact files.

use std::time::Instant;

use serde_json::Value;

use tuitbot_core::config::{Config, IntervalsConfig, LimitsConfig};
use tuitbot_core::error::XApiError;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;
use tuitbot_core::x_api::types::*;
use tuitbot_core::x_api::XApiClient;

use super::response::ToolResponse;
use crate::contract::ProviderError;
use crate::provider::SocialReadProvider;

/// Number of iterations per tool for the benchmark.
const ITERATIONS: usize = 5;

/// P95 latency threshold in milliseconds. The benchmark fails if exceeded.
const P95_THRESHOLD_MS: f64 = 50.0;

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

// ── Benchmark mock provider ──────────────────────────────────────────

struct BenchMockProvider;

#[async_trait::async_trait]
impl SocialReadProvider for BenchMockProvider {
    async fn get_tweet(&self, tid: &str) -> Result<Tweet, ProviderError> {
        Ok(Tweet {
            id: tid.to_string(),
            text: "Mock".to_string(),
            author_id: "a1".to_string(),
            created_at: "2026-02-25T00:00:00Z".to_string(),
            public_metrics: PublicMetrics::default(),
            conversation_id: None,
        })
    }
    async fn get_user_by_username(&self, u: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: "u1".to_string(),
            username: u.to_string(),
            name: "Mock".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
    async fn search_tweets(
        &self,
        _q: &str,
        _m: u32,
        _s: Option<&str>,
        _p: Option<&str>,
    ) -> Result<SearchResponse, ProviderError> {
        Ok(SearchResponse {
            data: vec![],
            includes: None,
            meta: SearchMeta {
                newest_id: None,
                oldest_id: None,
                result_count: 0,
                next_token: None,
            },
        })
    }
    async fn get_followers(
        &self,
        _u: &str,
        _m: u32,
        _p: Option<&str>,
    ) -> Result<UsersResponse, ProviderError> {
        Ok(UsersResponse {
            data: vec![],
            meta: UsersMeta {
                result_count: 0,
                next_token: None,
            },
        })
    }
    async fn get_user_by_id(&self, uid: &str) -> Result<User, ProviderError> {
        Ok(User {
            id: uid.to_string(),
            username: "bench".to_string(),
            name: "Bench".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
    async fn get_me(&self) -> Result<User, ProviderError> {
        Ok(User {
            id: "me".to_string(),
            username: "bench".to_string(),
            name: "Bench".to_string(),
            public_metrics: UserMetrics::default(),
        })
    }
}

struct BenchMockXApiClient;

#[async_trait::async_trait]
impl XApiClient for BenchMockXApiClient {
    async fn search_tweets(
        &self,
        _q: &str,
        _m: u32,
        _s: Option<&str>,
        _p: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }
    async fn get_mentions(
        &self,
        _u: &str,
        _s: Option<&str>,
        _p: Option<&str>,
    ) -> Result<MentionResponse, XApiError> {
        unimplemented!()
    }
    async fn post_tweet(&self, text: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "p1".to_string(),
            text: text.to_string(),
        })
    }
    async fn reply_to_tweet(&self, text: &str, _r: &str) -> Result<PostedTweet, XApiError> {
        Ok(PostedTweet {
            id: "r1".to_string(),
            text: text.to_string(),
        })
    }
    async fn get_tweet(&self, _id: &str) -> Result<Tweet, XApiError> {
        unimplemented!()
    }
    async fn get_me(&self) -> Result<User, XApiError> {
        unimplemented!()
    }
    async fn get_user_tweets(
        &self,
        _u: &str,
        _m: u32,
        _p: Option<&str>,
    ) -> Result<SearchResponse, XApiError> {
        unimplemented!()
    }
    async fn get_user_by_username(&self, _u: &str) -> Result<User, XApiError> {
        unimplemented!()
    }
}

/// Benchmark a single tool: run ITERATIONS, validate envelope, collect times.
async fn bench_tool<F, Fut>(name: &'static str, runs: &mut Vec<ToolRun>, f: F)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let result = f().await;
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        validate_envelope(&result, name);
    }
    runs.push(ToolRun {
        name,
        times_ms: times,
    });
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
                super::workflow::capabilities::get_capabilities(&pool, &config, true, false, None)
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
            let result = super::workflow::health::health_check(&pool, None, &config).await;
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
            let result = super::workflow::analytics::get_stats(&pool, 7, &config).await;
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
            let result = super::workflow::approval::list_pending(&pool, &config).await;
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
            let result = super::workflow::discovery::list_unreplied_tweets_with_limit(
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

#[tokio::test]
async fn session09_expanded_benchmark() {
    let pool = setup_db().await;
    let config = Config::default();
    let mut config_with_keywords = Config::default();
    config_with_keywords.business.product_keywords = vec!["rust".to_string()];
    config_with_keywords.business.industry_topics = vec!["software".to_string()];

    let mut runs: Vec<ToolRun> = Vec::new();

    // ── Kernel read tools (via MockProvider) ──────────────────────
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

    // ── Kernel write tools (via MockXApiClient) ──────────────────
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
                let input = super::scoring::ScoreTweetInput {
                    text: "Rust is great for async",
                    author_username: "dev",
                    author_followers: 1000,
                    likes: 5,
                    retweets: 2,
                    replies: 1,
                    created_at: "2026-02-24T12:00:00Z",
                };
                super::scoring::score_tweet(c, &input)
            }
        })
        .await;
    }

    {
        let cfg = config.clone();
        bench_tool("get_config", &mut runs, || {
            let c = &cfg;
            async move { super::config::get_config(c) }
        })
        .await;
    }

    {
        let cfg = config.clone();
        bench_tool("validate_config", &mut runs, || {
            let c = &cfg;
            async move { super::config::validate_config(c) }
        })
        .await;
    }

    // ── Telemetry tools ──────────────────────────────────────────
    bench_tool("get_mcp_tool_metrics", &mut runs, || {
        let p = pool.clone();
        async move { super::workflow::telemetry::get_mcp_tool_metrics(&p, 24).await }
    })
    .await;

    bench_tool("get_mcp_error_breakdown", &mut runs, || {
        let p = pool.clone();
        async move { super::workflow::telemetry::get_mcp_error_breakdown(&p, 24).await }
    })
    .await;

    // ── Workflow tools (existing) ────────────────────────────────
    bench_tool("get_capabilities", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move {
            super::workflow::capabilities::get_capabilities(&p, &c, true, false, None).await
        }
    })
    .await;

    bench_tool("health_check", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { super::workflow::health::health_check(&p, None, &c).await }
    })
    .await;

    bench_tool("get_stats", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { super::workflow::analytics::get_stats(&p, 7, &c).await }
    })
    .await;

    bench_tool("list_pending", &mut runs, || {
        let p = pool.clone();
        let c = config.clone();
        async move { super::workflow::approval::list_pending(&p, &c).await }
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

    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/roadmap/artifacts");
    std::fs::create_dir_all(&dir).expect("create artifacts dir");
    std::fs::write(dir.join("session-09-latency-report.md"), &md).expect("write latency report");

    assert!(
        global_p95 < P95_THRESHOLD_MS,
        "P95 {global_p95:.3} ms exceeds threshold {P95_THRESHOLD_MS:.1} ms"
    );
}
