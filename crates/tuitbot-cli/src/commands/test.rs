//! Implementation of the `tuitbot test` command.
//!
//! Validates configuration, credentials, and connectivity before
//! running the agent. Each check runs independently -- a failure
//! in one does not skip others.

use serde::Serialize;
use tuitbot_core::config::Config;
use tuitbot_core::startup::{expand_tilde, load_tokens_from_file, StartupError, StoredTokens};

use super::OutputFormat;

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
    let results = collect_checks(config, config_path);

    // Print results.
    eprintln!();
    for result in &results {
        eprintln!("{result}");
    }
    eprintln!();

    let all_passed = results.iter().all(|r| r.passed);
    if all_passed {
        eprintln!("All checks passed.");
    } else {
        let failed = results.iter().filter(|r| !r.passed).count();
        eprintln!("{failed} check(s) failed.");
    }

    all_passed
}

/// Execute the `tuitbot test` command.
///
/// Runs all diagnostic checks and reports results. Exits with code 1
/// if any check fails.
pub async fn execute(
    config: &Config,
    config_path: &str,
    output: OutputFormat,
) -> anyhow::Result<()> {
    if output.is_json() {
        let auth = evaluate_auth(load_tokens_from_file());
        let checks = collect_checks_with_auth(config, config_path, auth.checks);
        let output = build_test_output(checks, Some(auth.details));
        println!("{}", serde_json::to_string(&output)?);
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
    let topic_count = config.business.industry_topics.len();

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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_tokens() -> StoredTokens {
        StoredTokens {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(chrono::Utc::now() + chrono::TimeDelta::hours(1)),
            scopes: vec![
                "tweet.read".to_string(),
                "tweet.write".to_string(),
                "users.read".to_string(),
                "follows.read".to_string(),
                "follows.write".to_string(),
                "like.read".to_string(),
                "like.write".to_string(),
                "offline.access".to_string(),
            ],
        }
    }

    #[test]
    fn check_auth_expired_token_fails_token_status() {
        let mut tokens = valid_tokens();
        tokens.expires_at = Some(chrono::Utc::now() - chrono::TimeDelta::minutes(5));

        let auth = evaluate_auth(Ok(tokens));

        let token_check = auth
            .checks
            .iter()
            .find(|check| check.label == "X API token")
            .expect("token check should exist");
        assert!(!token_check.passed);
    }

    #[test]
    fn check_auth_missing_refresh_token_fails() {
        let mut tokens = valid_tokens();
        tokens.refresh_token = None;

        let auth = evaluate_auth(Ok(tokens));

        let refresh_check = auth
            .checks
            .iter()
            .find(|check| check.label == "X API refresh")
            .expect("refresh check should exist");
        assert!(!refresh_check.passed);
        assert!(refresh_check.message.contains("offline.access"));
    }

    #[test]
    fn check_auth_full_scopes_all_ok() {
        let auth = evaluate_auth(Ok(valid_tokens()));

        assert_eq!(auth.checks.len(), 3);
        assert!(auth.checks.iter().all(|check| check.passed));
        assert!(auth.details.missing_scopes.is_empty());
        assert!(auth.details.degraded_features.is_empty());
    }

    #[test]
    fn check_auth_partial_scopes_reports_degraded_features() {
        let mut tokens = valid_tokens();
        tokens.scopes.retain(|scope| scope != "like.write");

        let auth = evaluate_auth(Ok(tokens));

        let scope_check = auth
            .checks
            .iter()
            .find(|check| check.label == "X API scopes")
            .expect("scope check should exist");
        assert!(!scope_check.passed);
        assert!(scope_check.message.contains("like.write"));
        assert!(scope_check.message.contains("Like/unlike"));
        assert!(auth
            .details
            .degraded_features
            .contains(&"Like/unlike".to_string()));
    }

    #[test]
    fn check_auth_no_tokens_returns_single_fail() {
        let auth = evaluate_auth(Err(StartupError::AuthRequired));

        assert_eq!(auth.checks.len(), 1);
        assert!(!auth.checks[0].passed);
        assert_eq!(auth.checks[0].label, "X API auth");
    }

    #[test]
    fn check_auth_legacy_tokens_report_scope_not_tracked() {
        let mut tokens = valid_tokens();
        tokens.scopes.clear();

        let auth = evaluate_auth(Ok(tokens));

        let scope_check = auth
            .checks
            .iter()
            .find(|check| check.label == "X API scopes")
            .expect("scope check should exist");
        assert!(scope_check.passed);
        assert!(scope_check.message.contains("not tracked"));
        assert!(!auth.details.scopes_tracked);
    }

    #[test]
    fn json_output_contains_auth_details() {
        let auth = evaluate_auth(Ok(valid_tokens()));
        let output = build_test_output(auth.checks, Some(auth.details));
        let value = serde_json::to_value(output).expect("serialize output");

        assert!(value.get("auth_details").is_some());
        assert!(value["auth_details"].is_object());
    }
}
