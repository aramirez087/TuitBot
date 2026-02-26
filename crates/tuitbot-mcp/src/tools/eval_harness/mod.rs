//! Eval harness for MCP observability quality gates.
//!
//! Runs three scenarios and captures telemetry results:
//! - Scenario A: raw direct reply flow (single tool)
//! - Scenario B: composite find -> draft -> queue flow
//! - Scenario C: blocked-by-policy mutation
//!
//! Results are written to `roadmap/artifacts/task-07-eval-results.json`
//! and summarized in `task-07-eval-summary.md`.

#[cfg(test)]
mod aggregate;
#[cfg(test)]
mod mocks;
#[cfg(test)]
mod scenario_a;
#[cfg(test)]
mod scenario_b;
#[cfg(test)]
mod scenario_c;

#[cfg(test)]
pub(crate) use types::*;

#[cfg(test)]
mod types {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct EvalResults {
        pub eval_name: String,
        pub timestamp: String,
        pub scenarios: Vec<ScenarioResult>,
        pub quality_gates: QualityGates,
    }

    #[derive(Debug, Serialize)]
    pub struct ScenarioResult {
        pub scenario: String,
        pub description: String,
        pub steps: Vec<StepResult>,
        pub total_latency_ms: u64,
        pub success: bool,
        pub telemetry_entries: u64,
        pub schema_valid: bool,
    }

    #[derive(Debug, Serialize)]
    pub struct StepResult {
        pub tool_name: String,
        pub latency_ms: u64,
        pub success: bool,
        pub response_valid: bool,
        pub error_code: Option<String>,
        pub policy_decision: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct QualityGates {
        pub schema_validation_rate: f64,
        pub schema_validation_threshold: f64,
        pub schema_validation_pass: bool,
        pub unknown_error_rate: f64,
        pub unknown_error_threshold: f64,
        pub unknown_error_pass: bool,
        pub all_pass: bool,
    }
}
