use serde_json::Value;

use super::helpers::{approval_config, make_test_state, seed_discovered_tweet, validate_schema};
use super::{ScenarioResult, StepResult};
use crate::requests::ProposeItem;
use crate::tools::test_mocks::{MockLlmProvider, MockXApiClient};

pub async fn run_scenario_e() -> ScenarioResult {
    let state = make_test_state(
        Some(Box::new(MockXApiClient)),
        Some(Box::new(MockLlmProvider::new("Great point!"))),
        approval_config(),
    )
    .await;
    seed_discovered_tweet(&state, "t1", "Rust async programming", "rustdev").await;

    let mut steps = Vec::new();

    // Step 1: IdempotencyStore first call -> None (proceed)
    let params = r#"{"candidate_id":"t1","text":"Great point!"}"#;
    let first = state
        .idempotency
        .check_and_record("propose_and_queue_replies", params);
    steps.push(StepResult {
        tool_name: "idempotency_check_first".to_string(),
        latency_ms: 0,
        success: first.is_none(),
        response_valid: true,
        error_code: None,
    });

    // Step 2: propose_and_queue_replies -> succeeds, routes to approval
    let start = std::time::Instant::now();
    let items = vec![ProposeItem {
        candidate_id: "t1".to_string(),
        pre_drafted_text: Some("Great point about Rust!".to_string()),
    }];
    let result =
        crate::tools::workflow::composite::propose_queue::execute(&state, &items, false).await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&result);
    let parsed: Value = serde_json::from_str(&result).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "propose_and_queue_replies".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
    });

    // Step 3: IdempotencyStore same params within 30s -> duplicate error
    let duplicate = state
        .idempotency
        .check_and_record("propose_and_queue_replies", params);
    let dup_valid = duplicate
        .as_ref()
        .map(|j| validate_schema(j))
        .unwrap_or(false);
    let dup_code = duplicate.as_ref().and_then(|j| {
        let p: Value = serde_json::from_str(j).unwrap_or_default();
        p["error"]["code"].as_str().map(String::from)
    });
    steps.push(StepResult {
        tool_name: "idempotency_check_duplicate".to_string(),
        latency_ms: 0,
        success: duplicate.is_some(), // success = correctly blocked
        response_valid: dup_valid,
        error_code: dup_code,
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    ScenarioResult {
        scenario: "E".to_string(),
        description: "Mutation with idempotency enforcement".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_e_idempotency_enforcement() {
    let result = run_scenario_e().await;
    assert!(result.success, "Scenario E failed");
    assert!(result.schema_valid, "Scenario E schema validation failed");
    let dup_step = &result.steps[2];
    assert_eq!(
        dup_step.error_code.as_deref(),
        Some("validation_error"),
        "Duplicate should produce validation_error"
    );
}
