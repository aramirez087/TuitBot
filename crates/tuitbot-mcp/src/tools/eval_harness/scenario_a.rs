use tuitbot_core::storage;

use super::mocks::*;
use super::{ScenarioResult, StepResult};
use crate::requests::ProposeItem;

pub async fn run_scenario_a() -> ScenarioResult {
    let llm = MockLlmProvider::new("Great point about Rust async!");
    let client = MockXApiClient::with_results(vec![], vec![]);
    let state = make_test_state(Some(Box::new(client)), Some(Box::new(llm)), test_config()).await;
    seed_discovered_tweet(
        &state,
        "t1",
        "Rust async programming is fascinating",
        "rustdev",
    )
    .await;

    let mut steps = Vec::new();

    // Step 1: Draft a reply for a known candidate
    let start = std::time::Instant::now();
    let ids = vec!["t1".to_string()];
    let result =
        crate::tools::workflow::composite::draft_replies::execute(&state, &ids, None, false).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&result);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);

    steps.push(StepResult {
        tool_name: "draft_replies_for_candidates".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
        policy_decision: None,
    });

    // Step 2: Queue the reply (in approval mode)
    let state2 = make_test_state(
        Some(Box::new(MockXApiClient::with_results(vec![], vec![]))),
        Some(Box::new(MockLlmProvider::new("Great point!"))),
        approval_config(),
    )
    .await;
    seed_discovered_tweet(&state2, "t1", "Rust async programming", "rustdev").await;

    let start = std::time::Instant::now();
    let items = vec![ProposeItem {
        candidate_id: "t1".to_string(),
        pre_drafted_text: Some("Great point about Rust async!".to_string()),
    }];
    let result =
        crate::tools::workflow::composite::propose_queue::execute(&state2, &items, false).await;
    let elapsed2 = start.elapsed().as_millis() as u64;
    let valid2 = validate_schema(&result);
    let parsed2: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
    let success2 = parsed2["success"].as_bool().unwrap_or(false);

    steps.push(StepResult {
        tool_name: "propose_and_queue_replies".to_string(),
        latency_ms: elapsed2,
        success: success2,
        response_valid: valid2,
        error_code: None,
        policy_decision: Some("allow".to_string()),
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    let telemetry_count = storage::mcp_telemetry::get_summary(&state2.pool, "2000-01-01T00:00:00Z")
        .await
        .map(|s| s.total_calls as u64)
        .unwrap_or(0);

    ScenarioResult {
        scenario: "A".to_string(),
        description: "Raw direct reply flow: draft -> queue".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        telemetry_entries: telemetry_count,
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_a_raw_reply() {
    let result = run_scenario_a().await;
    assert!(result.success, "Scenario A failed");
    assert!(result.schema_valid, "Scenario A schema validation failed");
}
