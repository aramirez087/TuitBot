//! Session 09 eval scenarios D-G extending the eval_harness pattern.
//!
//! - Scenario D: Direct kernel read flow
//! - Scenario E: Mutation with idempotency enforcement
//! - Scenario F: Rate-limited and auth error behavior
//! - Scenario G: Provider switching behavior (MockProvider vs Scraper)

#[cfg(test)]
mod aggregate;
#[cfg(test)]
mod helpers;
#[cfg(test)]
mod scenario_d;
#[cfg(test)]
mod scenario_e;
#[cfg(test)]
mod scenario_f;
#[cfg(test)]
mod scenario_g;

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
        pub schema_valid: bool,
    }

    #[derive(Debug, Serialize)]
    pub struct StepResult {
        pub tool_name: String,
        pub latency_ms: u64,
        pub success: bool,
        pub response_valid: bool,
        pub error_code: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct QualityGates {
        pub schema_validation_rate: f64,
        pub schema_validation_threshold: f64,
        pub schema_validation_pass: bool,
        pub unknown_error_rate: f64,
        pub unknown_error_threshold: f64,
        pub unknown_error_pass: bool,
        pub kernel_conformance_rate: f64,
        pub kernel_conformance_pass: bool,
        pub error_code_accuracy_rate: f64,
        pub error_code_accuracy_pass: bool,
        pub all_pass: bool,
    }
}
