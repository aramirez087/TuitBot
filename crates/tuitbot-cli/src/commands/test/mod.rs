//! Implementation of the `tuitbot test` command.
//!
//! Validates configuration, credentials, and connectivity before
//! running the agent. Each check runs independently -- a failure
//! in one does not skip others.

#[cfg(test)]
mod tests;

use serde::Serialize;
use tuitbot_core::config::Config;
use tuitbot_core::error::LlmError;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::startup::{expand_tilde, load_tokens_from_file, StartupError, StoredTokens};

use crate::output::CliOutput;

/// A single diagnostic check result.
#[derive(Clone, Serialize)]
struct CheckResult {
    label: &'static str,
    passed: bool,
    message: String,
}

impl CheckResult {
    fn ok(label: &'static str, message: impl Into<String>) -> Self {
        Self {
            label,
            passed: true,
            message: message.into(),
        }
    }

    fn fail(label: &'static str, message: impl Into<String>) -> Self {
        Self {
            label,
            passed: false,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.passed { "OK" } else { "FAIL" };
        write!(
            f,
            "{:<18}{status} ({})",
            format!("{}:", self.label),
            self.message
        )
    }
}

#[derive(Clone, Serialize)]
struct AuthDetails {
    token_valid: bool,
    expires_at: Option<String>,
    expires_in_seconds: Option<i64>,
    has_refresh_token: bool,
    scopes_tracked: bool,
    granted_scopes: Vec<String>,
    missing_scopes: Vec<String>,
    degraded_features: Vec<String>,
    fix_action: Option<String>,
}

#[derive(Serialize)]
struct TestOutput {
    passed: bool,
    checks: Vec<CheckResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_details: Option<AuthDetails>,
}

struct AuthEvaluation {
    checks: Vec<CheckResult>,
    details: AuthDetails,
}

/// Run all diagnostic checks and return results.
fn collect_checks(config: &Config, config_path: &str) -> Vec<CheckResult> {
    let mut checks = vec![
        check_config(config, config_path),
        check_business_profile(config),
    ];
    checks.extend(check_auth());
    checks.push(check_llm_config(config));
    checks.push(check_database(config));
    checks
}

fn collect_checks_with_auth(
    config: &Config,
    config_path: &str,
    auth_checks: Vec<CheckResult>,
) -> Vec<CheckResult> {
    let mut checks = vec![
        check_config(config, config_path),
        check_business_profile(config),
    ];
    checks.extend(auth_checks);
    checks.push(check_llm_config(config));
    checks.push(check_database(config));
    checks
}

fn build_test_output(checks: Vec<CheckResult>, auth_details: Option<AuthDetails>) -> TestOutput {
    let passed = checks.iter().all(|r| r.passed);
    TestOutput {
        passed,
        checks,
        auth_details,
    }
}

/// Run all diagnostic checks and print results.
///
/// Returns `true` if all checks pass, `false` if any fail.
/// Does **not** call `process::exit` — callers decide what to do on failure.
pub async fn run_checks(config: &Config, config_path: &str) -> bool {
    let mut results = collect_checks(config, config_path);
    results.push(check_llm_connectivity(config).await);

    // Print results.
    eprintln!();
    for result in &results {
        eprintln!("{result}");
    }
    eprintln!();

    let all_passed = results.iter().all(|r| r.passed);
    if all_passed {
        eprintln!("All checks passed.");
        print_enrichment_hint(config);
    } else {
        let failed = results.iter().filter(|r| !r.passed).count();
        eprintln!("{failed} check(s) failed.");
    }

    if let Some(hint) = next_step_guidance(&results) {
        eprintln!("{hint}");
    }

    all_passed
}

/// Execute the `tuitbot test` command.
///
/// Runs all diagnostic checks and reports results. Exits with code 1
/// if any check fails.
pub async fn execute(config: &Config, config_path: &str, out: CliOutput) -> anyhow::Result<()> {
    if out.is_json() {
        let auth = evaluate_auth(load_tokens_from_file());
        let mut checks = collect_checks_with_auth(config, config_path, auth.checks);
        checks.push(check_llm_connectivity(config).await);
        let output = build_test_output(checks, Some(auth.details));
        out.json(&output)?;
        if !output.passed {
            std::process::exit(1);
        }
    } else if !run_checks(config, config_path).await {
        std::process::exit(1);
    }
    Ok(())
}

/// Check that configuration loaded successfully.
fn check_config(config: &Config, config_path: &str) -> CheckResult {
    let path = expand_tilde(config_path);
    match config.validate() {
        Ok(()) => CheckResult::ok("Configuration", format!("loaded from {}", path.display())),
        Err(errors) => {
            let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            CheckResult::fail("Configuration", msgs.join("; "))
        }
    }
}

/// Check that the business profile has required fields.
fn check_business_profile(config: &Config) -> CheckResult {
    let name = &config.business.product_name;
    if name.is_empty() {
        return CheckResult::fail("Business profile", "product_name is empty");
    }

    let keyword_count =
        config.business.product_keywords.len() + config.business.competitor_keywords.len();
    let topic_count = config.business.effective_industry_topics().len();

    if keyword_count == 0 {
        return CheckResult::fail(
            "Business profile",
            "no product_keywords or competitor_keywords configured",
        );
    }

    CheckResult::ok(
        "Business profile",
        format!("product_name: \"{name}\", {keyword_count} keywords, {topic_count} topics"),
    )
}

/// Check that OAuth tokens exist and are valid.
fn check_auth() -> Vec<CheckResult> {
    evaluate_auth(load_tokens_from_file()).checks
}

fn evaluate_auth(tokens_result: Result<StoredTokens, StartupError>) -> AuthEvaluation {
    match tokens_result {
        Ok(tokens) => evaluate_auth_from_tokens(tokens),
        Err(StartupError::AuthRequired) => AuthEvaluation {
            checks: vec![CheckResult::fail(
                "X API auth",
                "no tokens found, run `tuitbot auth` first",
            )],
            details: AuthDetails {
                token_valid: false,
                expires_at: None,
                expires_in_seconds: None,
                has_refresh_token: false,
                scopes_tracked: false,
                granted_scopes: vec![],
                missing_scopes: vec![],
                degraded_features: vec![],
                fix_action: Some("Run `tuitbot auth` to authenticate.".to_string()),
            },
        },
        Err(err) => AuthEvaluation {
            checks: vec![CheckResult::fail(
                "X API auth",
                format!("failed to load tokens ({err}), run `tuitbot auth` to re-authenticate"),
            )],
            details: AuthDetails {
                token_valid: false,
                expires_at: None,
                expires_in_seconds: None,
                has_refresh_token: false,
                scopes_tracked: false,
                granted_scopes: vec![],
                missing_scopes: vec![],
                degraded_features: vec![],
                fix_action: Some("Run `tuitbot auth` to re-authenticate.".to_string()),
            },
        },
    }
}

fn evaluate_auth_from_tokens(tokens: StoredTokens) -> AuthEvaluation {
    let token_valid = !tokens.is_expired();
    let expires_at = tokens.expires_at.map(|dt| dt.to_rfc3339());
    let expires_in_seconds = tokens.time_until_expiry().map(|d| d.num_seconds());
    let has_refresh_token = tokens
        .refresh_token
        .as_ref()
        .is_some_and(|token| !token.trim().is_empty());

    let mut checks = Vec::new();

    if token_valid {
        checks.push(CheckResult::ok(
            "X API token",
            format!("token valid, expires in {}", tokens.format_expiry()),
        ));
    } else {
        checks.push(CheckResult::fail(
            "X API token",
            "token expired, run `tuitbot auth` to re-authenticate",
        ));
    }

    if has_refresh_token {
        checks.push(CheckResult::ok("X API refresh", "refresh token present"));
    } else {
        checks.push(CheckResult::fail(
            "X API refresh",
            "refresh token missing; re-run `tuitbot auth` and grant `offline.access`",
        ));
    }

    let (scopes_tracked, granted_scopes, missing_scopes, degraded_features) =
        if tokens.has_scope_info() {
            let analysis = tokens.analyze_scopes();
            let degraded_names: Vec<String> = analysis
                .degraded_features
                .iter()
                .map(|feature| feature.feature.clone())
                .collect();

            if analysis.all_required_present {
                checks.push(CheckResult::ok(
                    "X API scopes",
                    format!(
                        "all required scopes granted ({})",
                        analysis.required.join(", ")
                    ),
                ));
            } else {
                let degraded_display = if degraded_names.is_empty() {
                    "(none)".to_string()
                } else {
                    degraded_names.join(", ")
                };
                checks.push(CheckResult::fail(
                    "X API scopes",
                    format!(
                        "missing scopes: {}; degraded features: {}",
                        analysis.missing.join(", "),
                        degraded_display
                    ),
                ));
            }

            (true, analysis.granted, analysis.missing, degraded_names)
        } else {
            checks.push(CheckResult::ok(
            "X API scopes",
            "scope metadata not tracked (legacy token file); run `tuitbot auth` to capture scopes",
        ));
            (false, vec![], vec![], vec![])
        };

    let fix_action = if !token_valid {
        Some("Run `tuitbot auth` to re-authenticate.".to_string())
    } else if !has_refresh_token {
        Some(
            "Re-run `tuitbot auth` and grant `offline.access` so refresh tokens are issued."
                .to_string(),
        )
    } else if scopes_tracked && !missing_scopes.is_empty() {
        Some(format!(
            "Run `tuitbot auth` to grant missing scopes: {}",
            missing_scopes.join(", ")
        ))
    } else if !scopes_tracked {
        Some("Re-run `tuitbot auth` to store granted scope metadata for diagnostics.".to_string())
    } else {
        None
    };

    AuthEvaluation {
        checks,
        details: AuthDetails {
            token_valid,
            expires_at,
            expires_in_seconds,
            has_refresh_token,
            scopes_tracked,
            granted_scopes,
            missing_scopes,
            degraded_features,
            fix_action,
        },
    }
}

/// Check LLM provider configuration.
fn check_llm_config(config: &Config) -> CheckResult {
    if config.llm.provider.is_empty() {
        return CheckResult::fail("LLM provider", "no provider configured");
    }

    match config.llm.provider.as_str() {
        "openai" | "anthropic" | "ollama" => {}
        other => {
            return CheckResult::fail("LLM provider", format!("unknown provider: {other}"));
        }
    }

    if matches!(config.llm.provider.as_str(), "openai" | "anthropic") {
        match &config.llm.api_key {
            Some(key) if !key.is_empty() => {}
            _ => {
                return CheckResult::fail(
                    "LLM provider",
                    format!("api_key required for {} provider", config.llm.provider),
                );
            }
        }
    }

    let model = if config.llm.model.is_empty() {
        "default"
    } else {
        &config.llm.model
    };

    CheckResult::ok(
        "LLM provider",
        format!("{}, model: {model}", config.llm.provider),
    )
}

/// Check database file status.
fn check_database(config: &Config) -> CheckResult {
    let db_path = expand_tilde(&config.storage.db_path);

    if db_path.exists() {
        match std::fs::metadata(&db_path) {
            Ok(meta) => {
                let size_mb = meta.len() as f64 / (1024.0 * 1024.0);
                let name = db_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| db_path.display().to_string());
                CheckResult::ok("Database", format!("{name}, {size_mb:.1} MB"))
            }
            Err(e) => CheckResult::fail(
                "Database",
                format!("cannot read {}: {e}", db_path.display()),
            ),
        }
    } else {
        CheckResult::ok(
            "Database",
            format!("will be created at {}", db_path.display()),
        )
    }
}

/// Check LLM connectivity by creating the provider and calling health_check.
async fn check_llm_connectivity(config: &Config) -> CheckResult {
    let provider = match create_provider(&config.llm) {
        Ok(p) => p,
        Err(LlmError::NotConfigured) => {
            return CheckResult::fail("LLM connectivity", "provider not configured");
        }
        Err(e) => {
            return CheckResult::fail("LLM connectivity", format!("{}: {e}", config.llm.provider));
        }
    };

    match provider.health_check().await {
        Ok(()) => CheckResult::ok(
            "LLM connectivity",
            format!("{}: reachable", provider.name()),
        ),
        Err(e) => CheckResult::fail("LLM connectivity", format!("{}: {e}", provider.name())),
    }
}

/// Synchronous provider-creation check for unit tests (no network call).
#[cfg(test)]
fn check_llm_connectivity_sync(config: &tuitbot_core::config::LlmConfig) -> CheckResult {
    match create_provider(config) {
        Ok(p) => CheckResult::ok(
            "LLM connectivity",
            format!("{}: created (not tested)", p.name()),
        ),
        Err(LlmError::NotConfigured) => {
            CheckResult::fail("LLM connectivity", "provider not configured")
        }
        Err(e) => CheckResult::fail("LLM connectivity", format!("{e}")),
    }
}

/// Return a next-step hint if all checks passed, or `None` if any failed.
fn next_step_guidance(checks: &[CheckResult]) -> Option<&'static str> {
    if checks.iter().all(|r| r.passed) {
        Some("Ready! Try: tuitbot tick --dry-run")
    } else {
        None
    }
}

/// Print an enrichment hint when all checks pass but profile isn't fully enriched.
fn print_enrichment_hint(config: &Config) {
    let completeness = config.profile_completeness();
    if completeness.is_fully_enriched() {
        return;
    }

    eprintln!();
    eprintln!("Profile: {}", completeness.one_line_summary());

    if let Some(stage) = completeness.next_incomplete() {
        eprintln!(
            "Tip: Run `tuitbot settings enrich` to configure {} ({})",
            stage.label().to_lowercase(),
            stage.description()
        );
    }
}
