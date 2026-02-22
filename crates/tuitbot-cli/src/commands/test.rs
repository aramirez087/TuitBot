//! Implementation of the `tuitbot test` command.
//!
//! Validates configuration, credentials, and connectivity before
//! running the agent. Each check runs independently -- a failure
//! in one does not skip others.

use tuitbot_core::config::Config;
use tuitbot_core::startup::{expand_tilde, load_tokens_from_file};

/// A single diagnostic check result.
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

/// Run all diagnostic checks and print results.
///
/// Returns `true` if all checks pass, `false` if any fail.
/// Does **not** call `process::exit` — callers decide what to do on failure.
pub async fn run_checks(config: &Config, config_path: &str) -> bool {
    let results = vec![
        check_config(config, config_path),
        check_business_profile(config),
        check_auth(),
        check_llm_config(config),
        check_database(config),
    ];

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
pub async fn execute(config: &Config, config_path: &str) -> anyhow::Result<()> {
    if !run_checks(config, config_path).await {
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
fn check_auth() -> CheckResult {
    match load_tokens_from_file() {
        Ok(tokens) => {
            if tokens.is_expired() {
                CheckResult::fail(
                    "X API auth",
                    "token expired, run `tuitbot auth` to re-authenticate",
                )
            } else {
                CheckResult::ok(
                    "X API auth",
                    format!("token valid, expires in {}", tokens.format_expiry()),
                )
            }
        }
        Err(_) => CheckResult::fail("X API auth", "no tokens found, run `tuitbot auth` first"),
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
