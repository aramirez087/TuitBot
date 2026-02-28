/// `tuitbot init` — interactive setup wizard or template copy.
///
/// Walks new users through X API credentials, business profile, and LLM
/// provider configuration in eight guided steps. Falls back to copying
/// `config.example.toml` with `--non-interactive`.
///
/// After writing the config the wizard offers to continue seamlessly
/// through `auth → test → preview` so the user doesn't have to remember
/// three separate commands.
mod display;
mod helpers;
mod prompts;
mod render;
mod steps;
mod wizard;

#[cfg(test)]
mod tests;

use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use dialoguer::Confirm;
use tuitbot_core::config::{Config, LlmConfig};
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::startup::data_dir;

use display::{
    print_llm_validation_fail, print_llm_validation_ok, print_quickstart_banner,
    print_quickstart_summary, print_remaining_steps, print_summary, print_welcome_banner,
};
use render::render_config_toml;
use steps::{
    step_approval_mode, step_brand_voice, step_business_profile, step_llm_provider, step_persona,
    step_quickstart, step_schedule, step_target_accounts, step_x_api,
};
use wizard::WizardResult;

use super::{auth, test, tick, OutputFormat, TickArgs};

// Re-export prompt functions used by the upgrade command.
pub(crate) use prompts::{
    prompt_approval_mode, prompt_enhanced_limits, prompt_persona, prompt_target_accounts,
};

/// Embedded copy of the example config shipped with the repo.
const EXAMPLE_CONFIG: &str = include_str!("../../../config.example.toml");

/// Run the init command.
pub async fn execute(force: bool, non_interactive: bool, advanced: bool) -> Result<()> {
    let dir = data_dir();
    let config_path: PathBuf = dir.join("config.toml");

    if config_path.exists() && !force {
        eprintln!(
            "Configuration already exists at {}\n\
             Use --force to overwrite.",
            config_path.display()
        );
        return Ok(());
    }

    if non_interactive {
        return write_template(&dir, &config_path);
    }

    // Guard: must be a real terminal for interactive mode.
    if !std::io::stdin().is_terminal() {
        bail!(
            "Interactive wizard requires a terminal.\n\
             Use --non-interactive to copy the template config instead."
        );
    }

    if advanced {
        run_advanced_wizard(&dir, &config_path).await
    } else {
        run_quickstart(&dir, &config_path).await
    }
}

/// Non-interactive path: copy the embedded template.
fn write_template(dir: &PathBuf, config_path: &PathBuf) -> Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(config_path, EXAMPLE_CONFIG)?;

    eprintln!("Created {}\n", config_path.display());
    eprintln!("Next steps:");
    eprintln!(
        "  1. Edit {} with your X API and LLM credentials",
        config_path.display()
    );
    eprintln!("  2. tuitbot auth    — authenticate with X");
    eprintln!("  3. tuitbot test    — validate configuration");
    eprintln!("  4. tuitbot run     — start the agent");

    Ok(())
}

/// Validate the LLM provider is reachable (non-blocking — continues on failure).
async fn validate_llm(result: &WizardResult) {
    let llm_config = LlmConfig {
        provider: result.llm_provider.clone(),
        api_key: result.llm_api_key.clone(),
        model: result.llm_model.clone(),
        base_url: result.llm_base_url.clone(),
    };

    let provider = match create_provider(&llm_config) {
        Ok(p) => p,
        Err(e) => {
            print_llm_validation_fail(&result.llm_provider, &format!("{e}"));
            return;
        }
    };

    let start = Instant::now();
    match tokio::time::timeout(std::time::Duration::from_secs(10), provider.health_check()).await {
        Ok(Ok(())) => {
            let latency = start.elapsed().as_millis();
            print_llm_validation_ok(provider.name(), &result.llm_model, latency);
        }
        Ok(Err(e)) => {
            print_llm_validation_fail(provider.name(), &format!("{e}"));
        }
        Err(_) => {
            print_llm_validation_fail(provider.name(), "connection timed out (10s)");
        }
    }
}

/// Quickstart path: 5 prompts → usable config → auth → test → preview.
async fn run_quickstart(dir: &PathBuf, config_path: &PathBuf) -> Result<()> {
    print_quickstart_banner();

    let result = step_quickstart()?;

    validate_llm(&result).await;

    print_quickstart_summary(&result);

    let confirm = Confirm::new()
        .with_prompt("Save configuration?")
        .default(true)
        .interact()?;

    if !confirm {
        eprintln!("Aborted. No files were written.");
        return Ok(());
    }

    fs::create_dir_all(dir)?;
    let toml = render_config_toml(&result);
    fs::write(config_path, &toml)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    eprintln!("\nWrote {}", config_path.display());

    let config_path_str = config_path.display().to_string();
    let config = Config::load(Some(&config_path_str))
        .context("Failed to reload the config we just wrote")?;

    chain_post_config(&config, &config_path_str).await?;

    Ok(())
}

/// Advanced wizard: full 8-step setup with auth → test → preview chaining.
async fn run_advanced_wizard(dir: &PathBuf, config_path: &PathBuf) -> Result<()> {
    print_welcome_banner();

    let result = step_x_api()?;
    let result = step_business_profile(result)?;
    let result = step_brand_voice(result)?;
    let result = step_llm_provider(result)?;
    let result = step_persona(result)?;
    let result = step_target_accounts(result)?;
    let result = step_approval_mode(result)?;
    let result = step_schedule(result)?;

    validate_llm(&result).await;

    print_summary(&result);

    let confirm = Confirm::new()
        .with_prompt("Write this configuration?")
        .default(true)
        .interact()?;

    if !confirm {
        eprintln!("Aborted. No files were written.");
        return Ok(());
    }

    fs::create_dir_all(dir)?;
    let toml = render_config_toml(&result);
    fs::write(config_path, &toml)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    eprintln!("\nWrote {}", config_path.display());

    let config_path_str = config_path.display().to_string();
    let config = Config::load(Some(&config_path_str))
        .context("Failed to reload the config we just wrote")?;

    chain_post_config(&config, &config_path_str).await?;

    Ok(())
}

/// Shared post-config chaining: auth → test → preview (dry run).
///
/// Used by both quickstart and advanced flows. On failure at any step,
/// prints actionable remaining steps and returns Ok (does not propagate).
async fn chain_post_config(config: &Config, config_path_str: &str) -> Result<()> {
    // Step A: Authenticate
    let do_auth = Confirm::new()
        .with_prompt("Connect your X account now?")
        .default(true)
        .interact()?;

    if !do_auth {
        print_remaining_steps(&[
            "tuitbot auth           — connect your X account",
            "tuitbot test           — verify everything works",
            "tuitbot tick --dry-run — preview the bot (no posts)",
        ]);
        return Ok(());
    }

    if let Err(e) = auth::execute(config, None).await {
        eprintln!("\nAuth failed: {e:#}");
        print_remaining_steps(&[
            "tuitbot auth           — retry authentication",
            "tuitbot test           — verify everything works",
            "tuitbot tick --dry-run — preview the bot (no posts)",
        ]);
        return Ok(());
    }

    // Step B: Validate
    let do_test = Confirm::new()
        .with_prompt("Verify everything works?")
        .default(true)
        .interact()?;

    if !do_test {
        print_remaining_steps(&[
            "tuitbot test           — verify everything works",
            "tuitbot tick --dry-run — preview the bot (no posts)",
        ]);
        return Ok(());
    }

    let all_passed = test::run_checks(config, config_path_str).await;
    if !all_passed {
        eprintln!("Fix the issues above, then:");
        print_remaining_steps(&[
            "tuitbot test           — re-validate configuration",
            "tuitbot tick --dry-run — preview the bot (no posts)",
        ]);
        return Ok(());
    }

    // Step C: Preview (dry run — defaults No, safe for new users)
    let do_preview = Confirm::new()
        .with_prompt("Preview the bot? (dry run, no posts)")
        .default(false)
        .interact()?;

    if !do_preview {
        print_remaining_steps(&["tuitbot tick --dry-run — preview the bot (no posts)"]);
        return Ok(());
    }

    let tick_args = TickArgs {
        dry_run: true,
        ignore_schedule: true,
        loops: Some(vec!["discovery".into(), "content".into()]),
        require_approval: false,
    };

    if let Err(e) = tick::execute(config, tick_args, OutputFormat::Text).await {
        eprintln!("\nPreview failed: {e:#}");
        print_remaining_steps(&["tuitbot tick --dry-run — retry preview"]);
    }

    Ok(())
}
