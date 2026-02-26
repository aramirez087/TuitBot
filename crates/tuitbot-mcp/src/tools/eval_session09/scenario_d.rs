use serde_json::Value;

use super::helpers::validate_schema;
use super::{ScenarioResult, StepResult};
use crate::kernel::{read, utils};
use crate::tools::test_mocks::MockProvider;

pub async fn run_scenario_d() -> ScenarioResult {
    let mut steps = Vec::new();

    // Step 1: get_tweet
    let start = std::time::Instant::now();
    let json = read::get_tweet(&MockProvider, "t42").await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "get_tweet".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    // Step 2: search_tweets
    let start = std::time::Instant::now();
    let json = read::search_tweets(&MockProvider, "rust", 10, None, None).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "search_tweets".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    // Step 3: get_followers
    let start = std::time::Instant::now();
    let json = read::get_followers(&MockProvider, "u1", 10, None).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "get_followers".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    // Step 4: get_me
    let start = std::time::Instant::now();
    let json = utils::get_me(&MockProvider).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "get_me".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    ScenarioResult {
        scenario: "D".to_string(),
        description: "Direct kernel read flow: get_tweet, search, followers, me".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_d_direct_kernel_reads() {
    let result = run_scenario_d().await;
    assert!(result.success, "Scenario D failed");
    assert!(result.schema_valid, "Scenario D schema validation failed");
}
