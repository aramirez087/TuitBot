use tuitbot_core::storage;

use super::mocks::*;
use super::{ScenarioResult, StepResult};
use crate::requests::ProposeItem;

pub async fn run_scenario_c() -> ScenarioResult {
    let client = MockXApiClient::with_results(vec![], vec![]);
    let llm = MockLlmProvider::new("This will be blocked!");
    let state = make_test_state(
        Some(Box::new(client)),
        Some(Box::new(llm)),
        blocked_policy_config(),
    )
    .await;
    seed_discovered_tweet(&state, "t1", "Rust topic", "dev").await;

    let mut steps = Vec::new();

    // Step 1: Try to propose (blocked by policy)
    let start = std::time::Instant::now();
    let items = vec![ProposeItem {
        candidate_id: "t1".to_string(),
        pre_drafted_text: Some("This reply should be blocked".to_string()),
    }];
    let result =
        crate::tools::workflow::composite::propose_queue::execute(&state, &items, false).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&result);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
    let is_denied = parsed["error"]["code"]
        .as_str()
        .map(|c| c.starts_with("policy_denied"))
        .unwrap_or(false);

    steps.push(StepResult {
        tool_name: "propose_and_queue_replies".to_string(),
        latency_ms: elapsed,
        success: false,
        response_valid: valid,
        error_code: parsed["error"]["code"].as_str().map(String::from),
        policy_decision: Some("deny".to_string()),
    });

    // Step 2: Verify telemetry captured the denial
    let start = std::time::Instant::now();
    let metrics_result =
        crate::tools::workflow::telemetry::get_mcp_error_breakdown(&state.pool, 24).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&metrics_result);
    let parsed: serde_json::Value = serde_json::from_str(&metrics_result).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);

    steps.push(StepResult {
        tool_name: "get_mcp_error_breakdown".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
        policy_decision: None,
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    let telemetry_count = storage::mcp_telemetry::get_summary(&state.pool, "2000-01-01T00:00:00Z")
        .await
        .map(|s| s.total_calls as u64)
        .unwrap_or(0);

    ScenarioResult {
        scenario: "C".to_string(),
        description: "Blocked-by-policy mutation with telemetry verification".to_string(),
        total_latency_ms: total,
        success: is_denied,
        telemetry_entries: telemetry_count,
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_c_policy_blocked() {
    let result = run_scenario_c().await;
    assert!(result.success, "Scenario C: policy should have blocked");
    assert!(result.schema_valid, "Scenario C schema validation failed");
    assert!(
        result.telemetry_entries > 0,
        "Scenario C should have telemetry entries for blocked mutation"
    );
}
