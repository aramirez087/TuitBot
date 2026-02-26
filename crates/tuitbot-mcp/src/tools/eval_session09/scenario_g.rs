use serde_json::Value;

use super::helpers::validate_schema;
use super::{ScenarioResult, StepResult};
use crate::kernel::read;
use crate::provider::scraper::ScraperReadProvider;
use crate::tools::test_mocks::MockProvider;

pub async fn run_scenario_g() -> ScenarioResult {
    let mut steps = Vec::new();
    let scraper = ScraperReadProvider::new();

    // Step 1: MockProvider get_tweet -> success
    let start = std::time::Instant::now();
    let json = read::get_tweet(&MockProvider, "t1").await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "get_tweet_mock_provider".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    // Step 2: ScraperReadProvider get_tweet -> error (stub)
    let start = std::time::Instant::now();
    let json = read::get_tweet(&scraper, "t1").await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let code = parsed["error"]["code"].as_str().map(String::from);
    let expected_scraper_error = code.as_deref() == Some("x_api_error");
    steps.push(StepResult {
        tool_name: "get_tweet_scraper_provider".to_string(),
        latency_ms: elapsed,
        success: expected_scraper_error,
        response_valid: valid,
        error_code: code,
    });

    // Step 3: ScraperReadProvider get_bookmarks -> NotConfigured
    let start = std::time::Instant::now();
    let json = read::get_bookmarks(&scraper, "u1", 10, None).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&json);
    let parsed: Value = serde_json::from_str(&json).unwrap_or_default();
    let code = parsed["error"]["code"].as_str().map(String::from);
    let expected_not_configured = code.as_deref() == Some("x_not_configured");
    steps.push(StepResult {
        tool_name: "get_bookmarks_scraper_provider".to_string(),
        latency_ms: elapsed,
        success: expected_not_configured,
        response_valid: valid,
        error_code: code,
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    ScenarioResult {
        scenario: "G".to_string(),
        description: "Provider switching: MockProvider vs ScraperReadProvider".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_g_provider_switching() {
    let result = run_scenario_g().await;
    assert!(result.success, "Scenario G failed");
    assert!(result.schema_valid, "Scenario G schema validation failed");
    assert_eq!(
        result.steps[1].error_code.as_deref(),
        Some("x_api_error"),
        "Scraper stub should return x_api_error"
    );
    assert_eq!(
        result.steps[2].error_code.as_deref(),
        Some("x_not_configured"),
        "Scraper auth-gated should return x_not_configured"
    );
}
