//! Baseline benchmark for migrated MCP tools.
//!
//! Runs each migrated tool multiple times, validates the envelope schema,
//! computes timing statistics, and writes artifact files.

#[cfg(test)]
mod baseline;
#[cfg(test)]
mod expanded;
#[cfg(test)]
mod mocks;

use std::time::Instant;

use serde_json::Value;

use super::response::ToolResponse;
use tuitbot_core::config::{IntervalsConfig, LimitsConfig};
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

/// Number of iterations per tool for the benchmark.
const ITERATIONS: usize = 5;

/// P95 latency threshold in milliseconds. The benchmark fails if exceeded.
const P95_THRESHOLD_MS: f64 = 50.0;

pub(super) struct ToolRun {
    pub name: &'static str,
    pub times_ms: Vec<f64>,
}

impl ToolRun {
    pub fn avg(&self) -> f64 {
        let sum: f64 = self.times_ms.iter().sum();
        sum / self.times_ms.len() as f64
    }
    pub fn min(&self) -> f64 {
        self.times_ms.iter().cloned().fold(f64::INFINITY, f64::min)
    }
    pub fn max(&self) -> f64 {
        self.times_ms
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }
    pub fn p50(&self) -> f64 {
        let mut sorted = self.times_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted[sorted.len() / 2]
    }
    pub fn p95(&self) -> f64 {
        let mut sorted = self.times_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
        sorted[idx.min(sorted.len() - 1)]
    }
}

pub(super) async fn setup_db() -> DbPool {
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

/// Benchmark a single tool: run ITERATIONS, validate envelope, collect times.
pub(super) async fn bench_tool<F, Fut>(name: &'static str, runs: &mut Vec<ToolRun>, f: F)
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

pub(super) fn validate_envelope(json_str: &str, tool_name: &str) {
    let parsed: ToolResponse =
        serde_json::from_str(json_str).unwrap_or_else(|e| panic!("{tool_name}: invalid JSON: {e}"));
    assert!(
        parsed.success,
        "{tool_name}: expected success=true, got false"
    );
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
