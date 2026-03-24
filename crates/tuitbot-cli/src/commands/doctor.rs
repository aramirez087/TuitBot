//! Implementation of the `tuitbot doctor` command.
//!
//! Runs a self-diagnosis checklist and prints a pass/fail report.
//! Exits 0 if all checks pass, 1 if any fail.

use std::time::Duration;

use tuitbot_core::config::Config;
use tuitbot_core::startup::expand_tilde;

struct Check {
    passed: bool,
    message: String,
}

impl Check {
    fn pass(msg: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: msg.into(),
        }
    }

    fn fail(msg: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: msg.into(),
        }
    }

    fn print(&self) {
        let icon = if self.passed { '✓' } else { '✗' };
        println!("  {} {}", icon, self.message);
    }
}

/// Execute `tuitbot doctor`.
///
/// Loads and validates the config independently so we can report per-check
/// failures instead of short-circuiting on the first error.
pub async fn execute(config_path: &str) -> anyhow::Result<()> {
    println!("tuitbot doctor");

    let mut checks: Vec<Check> = Vec::new();

    // ── 1. Config file exists and parses ─────────────────────────────────────
    let config_opt = {
        let expanded = expand_tilde(config_path);
        let path_display = expanded.display().to_string();

        match Config::load(Some(config_path)) {
            Ok(cfg) => {
                checks.push(Check::pass(format!(
                    "Config file found and valid ({})",
                    path_display
                )));
                Some(cfg)
            }
            Err(e) => {
                checks.push(Check::fail(format!(
                    "Config file invalid or missing ({path_display}): {e}"
                )));
                None
            }
        }
    };

    // ── 2. X API credentials ──────────────────────────────────────────────────
    {
        let present = config_opt
            .as_ref()
            .map(|cfg| {
                !cfg.x_api.client_id.is_empty()
                    || cfg
                        .x_api
                        .client_secret
                        .as_deref()
                        .is_some_and(|s| !s.is_empty())
            })
            .unwrap_or(false);

        if present {
            checks.push(Check::pass("X API credentials present"));
        } else {
            checks.push(Check::fail(
                "X API credentials not configured (set x_api.client_id in config)",
            ));
        }
    }

    // ── 3. LLM provider ──────────────────────────────────────────────────────
    {
        let configured = config_opt
            .as_ref()
            .map(|cfg| !cfg.llm.provider.is_empty())
            .unwrap_or(false);

        if configured {
            checks.push(Check::pass("LLM provider configured"));
        } else {
            checks.push(Check::fail(
                "LLM provider not configured (set llm.provider in config)",
            ));
        }
    }

    // ── 4. Database path ──────────────────────────────────────────────────────
    {
        match config_opt.as_ref() {
            Some(cfg) if !cfg.storage.db_path.is_empty() => {
                let expanded = expand_tilde(&cfg.storage.db_path);
                checks.push(Check::pass(format!(
                    "Database path configured ({})",
                    expanded.display()
                )));
            }
            Some(_) => {
                checks.push(Check::fail(
                    "Database path not configured (set storage.db_path in config)",
                ));
            }
            None => {
                checks.push(Check::fail("Database path unknown (config not loaded)"));
            }
        }
    }

    // ── 5. Network: api.x.com ─────────────────────────────────────────────────
    {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build();

        let reachable = match client {
            Ok(c) => c
                .head("https://api.x.com")
                .send()
                .await
                .map(|_| true)
                .unwrap_or(false),
            Err(_) => false,
        };

        if reachable {
            checks.push(Check::pass("Network: api.x.com reachable"));
        } else {
            checks.push(Check::fail(
                "Network: api.x.com unreachable (check internet connection)",
            ));
        }
    }

    // ── Print results ─────────────────────────────────────────────────────────
    for check in &checks {
        check.print();
    }

    let all_passed = checks.iter().all(|c| c.passed);
    if all_passed {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
