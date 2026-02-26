use super::scenario_a::run_scenario_a;
use super::scenario_b::run_scenario_b;
use super::scenario_c::run_scenario_c;
use super::{EvalResults, QualityGates};
use crate::tools::test_mocks::artifacts_dir;

#[tokio::test]
async fn eval_harness_all_scenarios() {
    let scenario_a = run_scenario_a().await;
    let scenario_b = run_scenario_b().await;
    let scenario_c = run_scenario_c().await;

    let scenarios = vec![scenario_a, scenario_b, scenario_c];

    // Compute quality gates
    let total_steps: usize = scenarios.iter().map(|s| s.steps.len()).sum();
    let valid_steps: usize = scenarios
        .iter()
        .flat_map(|s| &s.steps)
        .filter(|s| s.response_valid)
        .count();
    let schema_validation_rate = if total_steps > 0 {
        valid_steps as f64 / total_steps as f64
    } else {
        0.0
    };

    let error_steps: usize = scenarios
        .iter()
        .flat_map(|s| &s.steps)
        .filter(|s| {
            s.error_code
                .as_ref()
                .map(|c| c == "unknown" || c.is_empty())
                .unwrap_or(false)
        })
        .count();
    let unknown_error_rate = if total_steps > 0 {
        error_steps as f64 / total_steps as f64
    } else {
        0.0
    };

    let quality_gates = QualityGates {
        schema_validation_rate,
        schema_validation_threshold: 0.95,
        schema_validation_pass: schema_validation_rate >= 0.95,
        unknown_error_rate,
        unknown_error_threshold: 0.05,
        unknown_error_pass: unknown_error_rate <= 0.05,
        all_pass: schema_validation_rate >= 0.95 && unknown_error_rate <= 0.05,
    };

    let results = EvalResults {
        eval_name: "task-07-observability-evals".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        scenarios,
        quality_gates,
    };

    // Write eval results JSON
    let json = serde_json::to_string_pretty(&results).expect("serialize results");
    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");

    let json_path = dir.join("task-07-eval-results.json");
    std::fs::write(&json_path, &json).expect("write eval results");

    // Write eval summary markdown
    let mut md = String::new();
    md.push_str("# Task 07 — Observability Eval Results\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));
    md.push_str("## Scenarios\n\n");
    md.push_str("| Scenario | Description | Steps | Total (ms) | Success | Schema Valid | Telemetry Entries |\n");
    md.push_str("|----------|-------------|-------|------------|---------|--------------|-------------------|\n");
    for s in &results.scenarios {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            s.scenario,
            s.description,
            s.steps.len(),
            s.total_latency_ms,
            if s.success { "PASS" } else { "FAIL" },
            if s.schema_valid { "PASS" } else { "FAIL" },
            s.telemetry_entries,
        ));
    }
    md.push_str("\n## Step Details\n\n");
    for s in &results.scenarios {
        md.push_str(&format!(
            "### Scenario {}: {}\n\n",
            s.scenario, s.description
        ));
        md.push_str("| Tool | Latency (ms) | Success | Schema Valid | Error | Policy |\n");
        md.push_str("|------|-------------|---------|--------------|-------|--------|\n");
        for step in &s.steps {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                step.tool_name,
                step.latency_ms,
                if step.success { "PASS" } else { "FAIL" },
                if step.response_valid { "PASS" } else { "FAIL" },
                step.error_code.as_deref().unwrap_or("-"),
                step.policy_decision.as_deref().unwrap_or("-"),
            ));
        }
        md.push('\n');
    }

    md.push_str("## Quality Gates\n\n");
    md.push_str("| Gate | Rate | Threshold | Status |\n");
    md.push_str("|------|------|-----------|--------|\n");
    md.push_str(&format!(
        "| Schema validation | {:.1}% | {:.0}% | {} |\n",
        results.quality_gates.schema_validation_rate * 100.0,
        results.quality_gates.schema_validation_threshold * 100.0,
        if results.quality_gates.schema_validation_pass {
            "PASS"
        } else {
            "FAIL"
        },
    ));
    md.push_str(&format!(
        "| Unknown errors | {:.1}% | {:.0}% | {} |\n",
        results.quality_gates.unknown_error_rate * 100.0,
        results.quality_gates.unknown_error_threshold * 100.0,
        if results.quality_gates.unknown_error_pass {
            "PASS"
        } else {
            "FAIL"
        },
    ));
    md.push_str(&format!(
        "\n**Overall: {}**\n",
        if results.quality_gates.all_pass {
            "ALL GATES PASS"
        } else {
            "GATES FAILED"
        },
    ));

    let md_path = dir.join("task-07-eval-summary.md");
    std::fs::write(&md_path, &md).expect("write eval summary");

    // Assert quality gates
    assert!(
        results.quality_gates.schema_validation_pass,
        "Schema validation rate {:.1}% below threshold {:.0}%",
        results.quality_gates.schema_validation_rate * 100.0,
        results.quality_gates.schema_validation_threshold * 100.0,
    );
    assert!(
        results.quality_gates.unknown_error_pass,
        "Unknown error rate {:.1}% above threshold {:.0}%",
        results.quality_gates.unknown_error_rate * 100.0,
        results.quality_gates.unknown_error_threshold * 100.0,
    );
    assert!(results.quality_gates.all_pass, "Quality gates failed");
}
