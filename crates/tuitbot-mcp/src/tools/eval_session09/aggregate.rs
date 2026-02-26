use super::scenario_d::run_scenario_d;
use super::scenario_e::run_scenario_e;
use super::scenario_f::run_scenario_f;
use super::scenario_g::run_scenario_g;
use super::{EvalResults, QualityGates, StepResult};
use crate::tools::test_mocks::artifacts_dir;

#[tokio::test]
async fn eval_session09_all_scenarios() {
    let scenario_d = run_scenario_d().await;
    let scenario_e = run_scenario_e().await;
    let scenario_f = run_scenario_f().await;
    let scenario_g = run_scenario_g().await;

    let scenarios = vec![scenario_d, scenario_e, scenario_f, scenario_g];

    // Compute quality gates
    let total_steps: usize = scenarios.iter().map(|s| s.steps.len()).sum();
    let valid_steps: usize = scenarios
        .iter()
        .flat_map(|s| &s.steps)
        .filter(|s| s.response_valid)
        .count();
    let schema_rate = if total_steps > 0 {
        valid_steps as f64 / total_steps as f64
    } else {
        0.0
    };

    let unknown_errors: usize = scenarios
        .iter()
        .flat_map(|s| &s.steps)
        .filter(|s| {
            s.error_code
                .as_ref()
                .map(|c| c == "unknown" || c.is_empty())
                .unwrap_or(false)
        })
        .count();
    let unknown_rate = if total_steps > 0 {
        unknown_errors as f64 / total_steps as f64
    } else {
        0.0
    };

    let conformance_passed = scenarios.iter().filter(|s| s.success).count();
    let conformance_rate = conformance_passed as f64 / scenarios.len() as f64;

    let error_steps: Vec<&StepResult> = scenarios
        .iter()
        .flat_map(|s| &s.steps)
        .filter(|s| s.error_code.is_some())
        .collect();
    let accurate_errors = error_steps.iter().filter(|s| s.success).count();
    let error_accuracy = if error_steps.is_empty() {
        1.0
    } else {
        accurate_errors as f64 / error_steps.len() as f64
    };

    let quality_gates = QualityGates {
        schema_validation_rate: schema_rate,
        schema_validation_threshold: 0.95,
        schema_validation_pass: schema_rate >= 0.95,
        unknown_error_rate: unknown_rate,
        unknown_error_threshold: 0.05,
        unknown_error_pass: unknown_rate <= 0.05,
        kernel_conformance_rate: conformance_rate,
        kernel_conformance_pass: (conformance_rate - 1.0).abs() < f64::EPSILON,
        error_code_accuracy_rate: error_accuracy,
        error_code_accuracy_pass: (error_accuracy - 1.0).abs() < f64::EPSILON,
        all_pass: schema_rate >= 0.95
            && unknown_rate <= 0.05
            && (conformance_rate - 1.0).abs() < f64::EPSILON
            && (error_accuracy - 1.0).abs() < f64::EPSILON,
    };

    let results = EvalResults {
        eval_name: "session-09-conformance-evals".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        scenarios,
        quality_gates,
    };

    // Write artifacts
    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");

    let json = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write(dir.join("session-09-eval-results.json"), &json).expect("write eval results");

    write_handoff(&results, &dir);

    // Assert gates
    assert!(
        results.quality_gates.schema_validation_pass,
        "Schema validation {:.1}% < 95%",
        results.quality_gates.schema_validation_rate * 100.0
    );
    assert!(
        results.quality_gates.unknown_error_pass,
        "Unknown error rate {:.1}% > 5%",
        results.quality_gates.unknown_error_rate * 100.0
    );
    assert!(
        results.quality_gates.kernel_conformance_pass,
        "Kernel conformance {:.1}% < 100%",
        results.quality_gates.kernel_conformance_rate * 100.0
    );
    assert!(
        results.quality_gates.error_code_accuracy_pass,
        "Error code accuracy {:.1}% < 100%",
        results.quality_gates.error_code_accuracy_rate * 100.0
    );
    assert!(results.quality_gates.all_pass, "Quality gates failed");
}

fn write_handoff(results: &EvalResults, dir: &std::path::PathBuf) {
    let mut md = String::from("# Session 09 — Handoff\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));

    md.push_str("## Scenarios\n\n");
    md.push_str("| Scenario | Description | Steps | Total (ms) | Success | Schema |\n");
    md.push_str("|----------|-------------|-------|------------|---------|--------|\n");
    for s in &results.scenarios {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            s.scenario,
            s.description,
            s.steps.len(),
            s.total_latency_ms,
            if s.success { "PASS" } else { "FAIL" },
            if s.schema_valid { "PASS" } else { "FAIL" },
        ));
    }

    md.push_str("\n## Quality Gates\n\n");
    md.push_str("| Gate | Rate | Threshold | Status |\n");
    md.push_str("|------|------|-----------|--------|\n");
    md.push_str(&format!(
        "| Schema validation | {:.1}% | 95% | {} |\n",
        results.quality_gates.schema_validation_rate * 100.0,
        if results.quality_gates.schema_validation_pass {
            "PASS"
        } else {
            "FAIL"
        },
    ));
    md.push_str(&format!(
        "| Unknown errors | {:.1}% | 5% | {} |\n",
        results.quality_gates.unknown_error_rate * 100.0,
        if results.quality_gates.unknown_error_pass {
            "PASS"
        } else {
            "FAIL"
        },
    ));
    md.push_str(&format!(
        "| Kernel conformance | {:.1}% | 100% | {} |\n",
        results.quality_gates.kernel_conformance_rate * 100.0,
        if results.quality_gates.kernel_conformance_pass {
            "PASS"
        } else {
            "FAIL"
        },
    ));
    md.push_str(&format!(
        "| Error code accuracy | {:.1}% | 100% | {} |\n",
        results.quality_gates.error_code_accuracy_rate * 100.0,
        if results.quality_gates.error_code_accuracy_pass {
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

    md.push_str("\n## Session 09 Artifacts\n\n");
    md.push_str("- `session-09-conformance-results.md` — kernel tool conformance\n");
    md.push_str("- `session-09-golden-fixtures.json` — schema golden fixtures\n");
    md.push_str("- `session-09-schema-golden-report.md` — golden fixture report\n");
    md.push_str("- `session-09-eval-results.json` — eval scenario results\n");
    md.push_str("- `session-09-latency-report.md` — benchmark latency gates\n");

    md.push_str("\n## What Session 10 Must Finalize\n\n");
    md.push_str("1. Release documentation (README, CHANGELOG, API docs)\n");
    md.push_str(
        "2. Final manifest regeneration (`cargo test -p tuitbot-mcp manifest -- --ignored`)\n",
    );
    md.push_str("3. Version bump and crates.io publish preparation\n");
    md.push_str("4. End-to-end integration test with real X API sandbox (if available)\n");

    std::fs::write(dir.join("session-09-handoff.md"), &md).expect("write handoff");
}
