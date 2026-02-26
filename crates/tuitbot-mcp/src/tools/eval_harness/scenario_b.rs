use tuitbot_core::storage;

use super::mocks::*;
use super::{ScenarioResult, StepResult};
use crate::requests::ProposeItem;

pub async fn run_scenario_b() -> ScenarioResult {
    let tweets = vec![
        sample_tweet("t1", "Learning rust async programming today!", "a1"),
        sample_tweet("t2", "Async patterns in rust are powerful", "a2"),
    ];
    let users = vec![
        sample_user("a1", "rustdev", 5000),
        sample_user("a2", "asyncfan", 3000),
    ];
    let client = MockXApiClient::with_results(tweets, users);
    let llm = MockLlmProvider::new("Excellent insight on async Rust!");
    let state = make_test_state(
        Some(Box::new(client)),
        Some(Box::new(llm)),
        approval_config(),
    )
    .await;

    let mut steps = Vec::new();

    // Step 1: find_reply_opportunities
    let start = std::time::Instant::now();
    let result = crate::tools::workflow::composite::find_opportunities::execute(
        &state,
        Some("rust async"),
        None,
        Some(10),
        None,
    )
    .await;
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&result);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "find_reply_opportunities".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
        policy_decision: None,
    });

    // Collect discovered candidate IDs
    let candidate_ids: Vec<String> = parsed["data"]["candidates"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c["tweet_id"].as_str().map(String::from))
        .collect();

    // Step 2: draft_replies_for_candidates
    let start = std::time::Instant::now();
    let result = crate::tools::workflow::composite::draft_replies::execute(
        &state,
        &candidate_ids,
        None,
        false,
    )
    .await;
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

    // Extract drafts
    let draft_items: Vec<ProposeItem> = parsed["data"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|d| {
            if d["status"] == "success" {
                Some(ProposeItem {
                    candidate_id: d["candidate_id"].as_str()?.to_string(),
                    pre_drafted_text: d["draft_text"].as_str().map(String::from),
                })
            } else {
                None
            }
        })
        .collect();

    // Step 3: propose_and_queue_replies
    let start = std::time::Instant::now();
    let result = if draft_items.is_empty() {
        let fallback = vec![ProposeItem {
            candidate_id: candidate_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "t1".to_string()),
            pre_drafted_text: Some("Great insight!".to_string()),
        }];
        crate::tools::workflow::composite::propose_queue::execute(&state, &fallback, false).await
    } else {
        crate::tools::workflow::composite::propose_queue::execute(&state, &draft_items, false).await
    };
    let elapsed = start.elapsed().as_millis() as u64;
    let valid = validate_schema(&result);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
    let success = parsed["success"].as_bool().unwrap_or(false);
    steps.push(StepResult {
        tool_name: "propose_and_queue_replies".to_string(),
        latency_ms: elapsed,
        success,
        response_valid: valid,
        error_code: None,
        policy_decision: Some("allow".to_string()),
    });

    let total = steps.iter().map(|s| s.latency_ms).sum();
    let telemetry_count = storage::mcp_telemetry::get_summary(&state.pool, "2000-01-01T00:00:00Z")
        .await
        .map(|s| s.total_calls as u64)
        .unwrap_or(0);

    ScenarioResult {
        scenario: "B".to_string(),
        description: "Composite flow: find -> draft -> queue".to_string(),
        total_latency_ms: total,
        success: steps.iter().all(|s| s.success),
        telemetry_entries: telemetry_count,
        schema_valid: steps.iter().all(|s| s.response_valid),
        steps,
    }
}

#[tokio::test]
async fn scenario_b_composite_flow() {
    let result = run_scenario_b().await;
    assert!(result.success, "Scenario B failed");
    assert!(result.schema_valid, "Scenario B schema validation failed");
    assert!(
        result.telemetry_entries > 0,
        "Scenario B should have telemetry entries"
    );
}
