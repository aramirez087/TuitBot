/// `replyguy init` — interactive setup wizard or template copy.
///
/// Walks new users through X API credentials, business profile, and LLM
/// provider configuration in three guided steps. Falls back to copying
/// `config.example.toml` with `--non-interactive`.
///
/// After writing the config the wizard offers to continue seamlessly
/// through `auth → test → run` so the user doesn't have to remember
/// three separate commands.
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use console::Style;
use dialoguer::{Confirm, Input, Select};
use replyguy_core::config::Config;
use replyguy_core::startup::data_dir;

use super::{auth, run, test};

/// Embedded copy of the example config shipped with the repo.
const EXAMPLE_CONFIG: &str = include_str!("../../../../config.example.toml");

/// Collected wizard answers.
struct WizardResult {
    // X API
    client_id: String,
    client_secret: Option<String>,
    // Business
    product_name: String,
    product_description: String,
    product_url: Option<String>,
    target_audience: String,
    product_keywords: Vec<String>,
    industry_topics: Vec<String>,
    // Brand voice
    brand_voice: Option<String>,
    reply_style: Option<String>,
    content_style: Option<String>,
    // Persona
    persona_opinions: Vec<String>,
    persona_experiences: Vec<String>,
    content_pillars: Vec<String>,
    // Target accounts
    target_accounts: Vec<String>,
    auto_follow: bool,
    // Approval mode
    approval_mode: bool,
    // LLM
    llm_provider: String,
    llm_api_key: Option<String>,
    llm_model: String,
    llm_base_url: Option<String>,
}

/// Run the init command.
pub async fn execute(force: bool, non_interactive: bool) -> Result<()> {
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

    run_wizard(&dir, &config_path).await
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
    eprintln!("  2. replyguy auth    — authenticate with X");
    eprintln!("  3. replyguy test    — validate configuration");
    eprintln!("  4. replyguy run     — start the agent");

    Ok(())
}

/// Interactive wizard: collect credentials, write config, then offer to
/// continue through auth → test → run inline.
async fn run_wizard(dir: &PathBuf, config_path: &PathBuf) -> Result<()> {
    print_welcome_banner();

    let result = WizardResult {
        // Step 1
        ..step_x_api()?
    };
    let result = WizardResult {
        // Step 2
        ..step_business_profile(result)?
    };
    let result = WizardResult {
        // Step 3
        ..step_brand_voice(result)?
    };
    let result = WizardResult {
        // Step 4
        ..step_llm_provider(result)?
    };
    let result = WizardResult {
        // Step 5
        ..step_persona(result)?
    };
    let result = WizardResult {
        // Step 6
        ..step_target_accounts(result)?
    };
    let result = WizardResult {
        // Step 7
        ..step_approval_mode(result)?
    };

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

    // --- Seamless chaining: auth → test → run ---

    let config_path_str = config_path.display().to_string();
    let config = Config::load(Some(&config_path_str))
        .context("Failed to reload the config we just wrote")?;

    // Step A: Authenticate
    let do_auth = Confirm::new()
        .with_prompt("Authenticate with X now?")
        .default(true)
        .interact()?;

    if !do_auth {
        print_remaining_steps(&[
            "replyguy auth    — authenticate with X",
            "replyguy test    — validate configuration",
            "replyguy run     — start the agent",
        ]);
        return Ok(());
    }

    if let Err(e) = auth::execute(&config, None).await {
        eprintln!("\nAuth failed: {e:#}");
        print_remaining_steps(&[
            "replyguy auth    — retry authentication",
            "replyguy test    — validate configuration",
            "replyguy run     — start the agent",
        ]);
        return Ok(());
    }

    // Step B: Validate
    let do_test = Confirm::new()
        .with_prompt("Validate configuration now?")
        .default(true)
        .interact()?;

    if !do_test {
        print_remaining_steps(&[
            "replyguy test    — validate configuration",
            "replyguy run     — start the agent",
        ]);
        return Ok(());
    }

    let all_passed = test::run_checks(&config, &config_path_str).await;
    if !all_passed {
        eprintln!("Fix the issues above, then:");
        print_remaining_steps(&[
            "replyguy test    — re-validate configuration",
            "replyguy run     — start the agent",
        ]);
        return Ok(());
    }

    // Step C: Run (defaults No — bigger commitment)
    let do_run = Confirm::new()
        .with_prompt("Start the agent now?")
        .default(false)
        .interact()?;

    if !do_run {
        print_remaining_steps(&["replyguy run     — start the agent"]);
        return Ok(());
    }

    run::execute(&config, 0).await?;

    Ok(())
}

fn print_welcome_banner() {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Welcome to ReplyGuy Setup"));
    eprintln!(
        "{}",
        dim.apply_to("This wizard will create your configuration in 7 steps.")
    );
    eprintln!();
    eprintln!("{}", dim.apply_to("You'll need:"));
    eprintln!(
        "{}",
        dim.apply_to("  - X API credentials (https://developer.x.com)")
    );
    eprintln!("{}", dim.apply_to("  - Your product/business details"));
    eprintln!(
        "{}",
        dim.apply_to("  - An LLM API key (OpenAI, Anthropic, or Ollama)")
    );
    eprintln!();
    eprintln!(
        "{}",
        dim.apply_to("Tip: Defaults shown in [brackets] — just press Enter to accept them.")
    );
    eprintln!();
}

/// Step 1/7: X API credentials.
fn step_x_api() -> Result<WizardResult> {
    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to("Step 1/7: X API Credentials"));
    eprintln!();

    let client_id: String = Input::new()
        .with_prompt("OAuth 2.0 Client ID")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("Client ID cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let has_secret = Confirm::new()
        .with_prompt("Do you have a client secret? (confidential clients only)")
        .default(false)
        .interact()?;

    let client_secret = if has_secret {
        let secret: String = Input::new().with_prompt("Client Secret").interact_text()?;
        Some(secret)
    } else {
        None
    };

    eprintln!();

    Ok(WizardResult {
        client_id: client_id.trim().to_string(),
        client_secret,
        product_name: String::new(),
        product_description: String::new(),
        product_url: None,
        target_audience: String::new(),
        product_keywords: vec![],
        industry_topics: vec![],
        brand_voice: None,
        reply_style: None,
        content_style: None,
        persona_opinions: vec![],
        persona_experiences: vec![],
        content_pillars: vec![],
        target_accounts: vec![],
        auto_follow: false,
        approval_mode: false,
        llm_provider: String::new(),
        llm_api_key: None,
        llm_model: String::new(),
        llm_base_url: None,
    })
}

/// Step 2/7: Business profile.
fn step_business_profile(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to("Step 2/7: Business Profile"));
    eprintln!();

    let product_name: String = Input::new()
        .with_prompt("Product name")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("Product name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let product_description: String = Input::new()
        .with_prompt("One-line description")
        .interact_text()?;

    let product_url: String = Input::new()
        .with_prompt("Product URL (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let target_audience: String = Input::new()
        .with_prompt("Target audience")
        .interact_text()?;

    let keywords_raw: String = Input::new()
        .with_prompt("Discovery keywords (comma-separated)")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if parse_csv(input).is_empty() {
                Err("At least one keyword is required")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let topics_raw: String = Input::new()
        .with_prompt("Content topics (comma-separated)")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if parse_csv(input).is_empty() {
                Err("At least one topic is required")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    eprintln!();

    Ok(WizardResult {
        product_name: product_name.trim().to_string(),
        product_description: product_description.trim().to_string(),
        product_url: if product_url.trim().is_empty() {
            None
        } else {
            Some(product_url.trim().to_string())
        },
        target_audience: target_audience.trim().to_string(),
        product_keywords: parse_csv(&keywords_raw),
        industry_topics: parse_csv(&topics_raw),
        ..prev
    })
}

/// Step 3/7: Brand voice and style.
fn step_brand_voice(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    eprintln!("{}", bold.apply_to("Step 3/7: Brand Voice & Style"));
    eprintln!(
        "{}",
        dim.apply_to("These shape how the bot sounds when it replies and posts.")
    );
    eprintln!(
        "{}",
        dim.apply_to("All fields are optional — press Enter to skip and use defaults.")
    );
    eprintln!();

    let brand_voice: String = Input::new()
        .with_prompt("Brand voice / personality (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let reply_style: String = Input::new()
        .with_prompt("Reply style guidelines (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let content_style: String = Input::new()
        .with_prompt("Content style for tweets & threads (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    eprintln!();

    Ok(WizardResult {
        brand_voice: non_empty(brand_voice),
        reply_style: non_empty(reply_style),
        content_style: non_empty(content_style),
        ..prev
    })
}

/// Step 4/7: LLM provider.
fn step_llm_provider(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to("Step 4/7: LLM Provider"));
    eprintln!();

    let providers = &["openai", "anthropic", "ollama"];
    let selection = Select::new()
        .with_prompt("LLM provider")
        .items(providers)
        .default(0)
        .interact()?;
    let provider = providers[selection].to_string();

    let api_key = if provider == "ollama" {
        None
    } else {
        let key: String = Input::new()
            .with_prompt(format!("{} API key", capitalize(&provider)))
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                if input.trim().is_empty() {
                    Err("API key is required for this provider")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;
        Some(key.trim().to_string())
    };

    let default_model = match provider.as_str() {
        "openai" => "gpt-4o-mini",
        "anthropic" => "claude-sonnet-4-6",
        "ollama" => "llama3.2",
        _ => "",
    };

    let model: String = Input::new()
        .with_prompt("Model name (Enter to accept default)")
        .default(default_model.to_string())
        .interact_text()?;

    let base_url = if provider == "ollama" {
        let url: String = Input::new()
            .with_prompt("Ollama base URL (Enter to accept default)")
            .default("http://localhost:11434/v1".to_string())
            .interact_text()?;
        Some(url.trim().to_string())
    } else {
        None
    };

    eprintln!();

    Ok(WizardResult {
        llm_provider: provider,
        llm_api_key: api_key,
        llm_model: model.trim().to_string(),
        llm_base_url: base_url,
        ..prev
    })
}

/// Step 5/7: Persona (optional).
fn step_persona(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    eprintln!("{}", bold.apply_to("Step 5/7: Persona (optional)"));
    eprintln!(
        "{}",
        dim.apply_to("Strong opinions, experiences, and pillars make content more authentic.")
    );
    eprintln!(
        "{}",
        dim.apply_to("All fields are optional — press Enter to skip.")
    );
    eprintln!();

    let (opinions, experiences, pillars) = prompt_persona()?;

    Ok(WizardResult {
        persona_opinions: opinions,
        persona_experiences: experiences,
        content_pillars: pillars,
        ..prev
    })
}

/// Collect persona fields interactively.
/// Returns (opinions, experiences, content_pillars).
pub(crate) fn prompt_persona() -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
    let opinions_raw: String = Input::new()
        .with_prompt("Strong opinions, comma-separated (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let experiences_raw: String = Input::new()
        .with_prompt("Personal experiences, comma-separated (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let pillars_raw: String = Input::new()
        .with_prompt("Core content topics, comma-separated (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    eprintln!();

    Ok((
        parse_csv(&opinions_raw),
        parse_csv(&experiences_raw),
        parse_csv(&pillars_raw),
    ))
}

/// Step 6/7: Target Accounts (optional).
fn step_target_accounts(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    let dim = Style::new().dim();
    eprintln!("{}", bold.apply_to("Step 6/7: Target Accounts (optional)"));
    eprintln!(
        "{}",
        dim.apply_to("Monitor specific accounts and reply to their conversations.")
    );
    eprintln!();

    let (accounts, auto_follow) = prompt_target_accounts()?;

    Ok(WizardResult {
        target_accounts: accounts,
        auto_follow,
        ..prev
    })
}

/// Collect target account fields interactively.
/// Returns (accounts, auto_follow).
pub(crate) fn prompt_target_accounts() -> Result<(Vec<String>, bool)> {
    let accounts_raw: String = Input::new()
        .with_prompt("Accounts to monitor, comma-separated @usernames (Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let accounts: Vec<String> = parse_csv(&accounts_raw)
        .into_iter()
        .map(|a| a.trim_start_matches('@').to_string())
        .collect();

    let auto_follow = if !accounts.is_empty() {
        Confirm::new()
            .with_prompt("Auto-follow target accounts?")
            .default(false)
            .interact()?
    } else {
        false
    };

    eprintln!();

    Ok((accounts, auto_follow))
}

/// Step 7/7: Approval Mode.
fn step_approval_mode(prev: WizardResult) -> Result<WizardResult> {
    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to("Step 7/7: Approval Mode"));
    eprintln!();

    let approval_mode = prompt_approval_mode()?;

    Ok(WizardResult {
        approval_mode,
        ..prev
    })
}

/// Collect approval mode preference interactively.
pub(crate) fn prompt_approval_mode() -> Result<bool> {
    let approval_mode = Confirm::new()
        .with_prompt("Queue posts for review before posting?")
        .default(false)
        .interact()?;

    eprintln!();

    Ok(approval_mode)
}

/// Collect enhanced safety limit fields interactively.
/// Returns (max_replies_per_author_per_day, banned_phrases, product_mention_ratio).
pub(crate) fn prompt_enhanced_limits() -> Result<(u32, Vec<String>, f32)> {
    let max_replies: String = Input::new()
        .with_prompt("Max replies to same author per day")
        .default("1".to_string())
        .interact_text()?;
    let max_replies: u32 = max_replies.trim().parse().unwrap_or(1);

    let banned_raw: String = Input::new()
        .with_prompt("Banned phrases, comma-separated (Enter for defaults)")
        .default("check out, you should try, I recommend, link in bio".to_string())
        .interact_text()?;
    let banned_phrases = parse_csv(&banned_raw);

    let ratio_raw: String = Input::new()
        .with_prompt("Product mention ratio (0.0–1.0)")
        .default("0.2".to_string())
        .interact_text()?;
    let ratio: f32 = ratio_raw.trim().parse().unwrap_or(0.2);

    eprintln!();

    Ok((max_replies, banned_phrases, ratio))
}

/// Display a summary of all collected values.
fn print_summary(result: &WizardResult) {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!("{}", bold.apply_to("Configuration Summary"));
    eprintln!("{}", dim.apply_to("─────────────────────"));

    eprintln!("  X API Client ID:   {}", result.client_id);
    eprintln!(
        "  Client Secret:     {}",
        if result.client_secret.is_some() {
            "(set)"
        } else {
            "(none)"
        }
    );

    eprintln!();
    eprintln!("  Product:           {}", result.product_name);
    eprintln!("  Description:       {}", result.product_description);
    eprintln!(
        "  URL:               {}",
        result.product_url.as_deref().unwrap_or("(none)")
    );
    eprintln!("  Audience:          {}", result.target_audience);
    eprintln!(
        "  Keywords:          {}",
        result.product_keywords.join(", ")
    );
    eprintln!("  Topics:            {}", result.industry_topics.join(", "));

    eprintln!();
    eprintln!(
        "  Brand Voice:       {}",
        result.brand_voice.as_deref().unwrap_or("(default)")
    );
    eprintln!(
        "  Reply Style:       {}",
        result.reply_style.as_deref().unwrap_or("(default)")
    );
    eprintln!(
        "  Content Style:     {}",
        result.content_style.as_deref().unwrap_or("(default)")
    );

    eprintln!();
    eprintln!(
        "  Opinions:          {}",
        if result.persona_opinions.is_empty() {
            "(none)".to_string()
        } else {
            result.persona_opinions.join(", ")
        }
    );
    eprintln!(
        "  Experiences:       {}",
        if result.persona_experiences.is_empty() {
            "(none)".to_string()
        } else {
            result.persona_experiences.join(", ")
        }
    );
    eprintln!(
        "  Content Pillars:   {}",
        if result.content_pillars.is_empty() {
            "(none)".to_string()
        } else {
            result.content_pillars.join(", ")
        }
    );

    eprintln!();
    eprintln!(
        "  Target Accounts:   {}",
        if result.target_accounts.is_empty() {
            "(none)".to_string()
        } else {
            result.target_accounts.join(", ")
        }
    );
    if !result.target_accounts.is_empty() {
        eprintln!(
            "  Auto-follow:       {}",
            if result.auto_follow { "yes" } else { "no" }
        );
    }
    eprintln!(
        "  Approval Mode:     {}",
        if result.approval_mode { "yes" } else { "no" }
    );

    eprintln!();
    eprintln!("  LLM Provider:      {}", result.llm_provider);
    eprintln!(
        "  API Key:           {}",
        if result.llm_api_key.is_some() {
            "(set)"
        } else {
            "(none)"
        }
    );
    eprintln!("  Model:             {}", result.llm_model);
    if let Some(url) = &result.llm_base_url {
        eprintln!("  Base URL:          {}", url);
    }

    eprintln!();
}

fn print_remaining_steps(steps: &[&str]) {
    let bold = Style::new().bold();

    eprintln!();
    eprintln!("{}", bold.apply_to("Remaining steps:"));
    for (i, step) in steps.iter().enumerate() {
        eprintln!("  {}. {step}", i + 1);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

/// Convert a trimmed string to `Some` or `None` if empty.
fn non_empty(s: String) -> Option<String> {
    let trimmed = s.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Split a comma-separated string into trimmed, non-empty values.
fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Escape special TOML characters inside a double-quoted string value.
fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Format a `Vec<String>` as a TOML inline array: `["a", "b", "c"]`.
fn format_toml_array(items: &[String]) -> String {
    let inner: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", escape_toml(s)))
        .collect();
    format!("[{}]", inner.join(", "))
}

/// Render a complete, well-commented TOML config from wizard answers.
fn render_config_toml(r: &WizardResult) -> String {
    let client_secret_line = match &r.client_secret {
        Some(secret) => format!("client_secret = \"{}\"", escape_toml(secret)),
        None => "# client_secret = \"your-client-secret-here\"".to_string(),
    };

    let product_url_line = match &r.product_url {
        Some(url) => format!("product_url = \"{}\"", escape_toml(url)),
        None => "# product_url = \"https://example.com\"".to_string(),
    };

    let brand_voice_line = match &r.brand_voice {
        Some(v) => format!("brand_voice = \"{}\"", escape_toml(v)),
        None => {
            "# brand_voice = \"Friendly technical expert. Casual, occasionally witty.\"".to_string()
        }
    };

    let reply_style_line = match &r.reply_style {
        Some(s) => format!("reply_style = \"{}\"", escape_toml(s)),
        None => "# reply_style = \"Lead with genuine help. Only mention our product if relevant.\""
            .to_string(),
    };

    let content_style_line = match &r.content_style {
        Some(s) => format!("content_style = \"{}\"", escape_toml(s)),
        None => "# content_style = \"Share practical tips with real examples.\"".to_string(),
    };

    let persona_opinions_line = if r.persona_opinions.is_empty() {
        "# persona_opinions = [\"Your strong opinion here\"]".to_string()
    } else {
        format!(
            "persona_opinions = {}",
            format_toml_array(&r.persona_opinions)
        )
    };

    let persona_experiences_line = if r.persona_experiences.is_empty() {
        "# persona_experiences = [\"Your personal experience here\"]".to_string()
    } else {
        format!(
            "persona_experiences = {}",
            format_toml_array(&r.persona_experiences)
        )
    };

    let content_pillars_line = if r.content_pillars.is_empty() {
        "# content_pillars = [\"Your core topic here\"]".to_string()
    } else {
        format!(
            "content_pillars = {}",
            format_toml_array(&r.content_pillars)
        )
    };

    let targets_section = if r.target_accounts.is_empty() {
        "# --- Target Accounts ---\n\
         # Monitor specific accounts and reply to their conversations.\n\
         # [targets]\n\
         # accounts = [\"elonmusk\", \"levelsio\"]\n\
         # auto_follow = false"
            .to_string()
    } else {
        format!(
            "# --- Target Accounts ---\n\
             # Monitor specific accounts and reply to their conversations.\n\
             [targets]\n\
             accounts = {accounts}\n\
             auto_follow = {auto_follow}",
            accounts = format_toml_array(&r.target_accounts),
            auto_follow = r.auto_follow,
        )
    };

    let api_key_line = match &r.llm_api_key {
        Some(key) => format!("api_key = \"{}\"", escape_toml(key)),
        None => "# api_key = \"your-api-key-here\"".to_string(),
    };

    let base_url_line = match &r.llm_base_url {
        Some(url) => format!("base_url = \"{}\"", escape_toml(url)),
        None => "# base_url = \"http://localhost:11434/v1\"".to_string(),
    };

    format!(
        r#"# =============================================================================
# ReplyGuy Configuration
# =============================================================================
# Generated by `replyguy init` setup wizard.
# Edit this file to tune scoring, limits, and intervals.
# Docs: https://github.com/your-org/replyguy
# =============================================================================

# Queue posts for review before posting (use `replyguy approve` to review).
approval_mode = {approval_mode}

# --- X API Credentials ---
# Get your credentials from https://developer.x.com/en/portal/dashboard
[x_api]
client_id = "{client_id}"
{client_secret_line}

# --- Authentication Settings ---
[auth]
# Auth mode: "local_callback" (auto-catch via local server) or "manual" (paste code from browser).
mode = "local_callback"
callback_host = "127.0.0.1"
callback_port = 8080

# --- Business Profile ---
# Describe your product so ReplyGuy can find relevant conversations
# and generate on-brand content.
[business]
product_name = "{product_name}"
product_description = "{product_description}"
{product_url_line}
target_audience = "{target_audience}"

# Keywords for tweet discovery (ReplyGuy searches for tweets containing these).
product_keywords = {product_keywords}

# Optional: competitor keywords for discovery.
competitor_keywords = []

# Topics for original content generation (tweets and threads).
industry_topics = {industry_topics}

# Brand voice and style — shapes how the bot sounds in all generated content.
{brand_voice_line}
{reply_style_line}
{content_style_line}

# Persona — strong opinions, experiences, and pillars make content more authentic.
{persona_opinions_line}
{persona_experiences_line}
{content_pillars_line}

# --- Scoring Engine ---
# Controls how tweets are scored for reply-worthiness (0-100 scale).
# Weights should sum to ~100 for balanced scoring.
[scoring]
threshold = 70
keyword_relevance_max = 40.0
follower_count_max = 20.0
recency_max = 15.0
engagement_rate_max = 25.0

# --- Safety Limits ---
# Prevent aggressive posting that could trigger account restrictions.
[limits]
max_replies_per_day = 5
max_tweets_per_day = 6
max_threads_per_week = 1
min_action_delay_seconds = 45
max_action_delay_seconds = 180
max_replies_per_author_per_day = 1
product_mention_ratio = 0.2
banned_phrases = ["check out", "you should try", "I recommend", "link in bio"]

# --- Automation Intervals ---
# How often each loop runs. Shorter intervals use more API quota.
[intervals]
mentions_check_seconds = 300
discovery_search_seconds = 900
content_post_window_seconds = 10800
thread_interval_seconds = 604800

{targets_section}

# --- LLM Provider ---
# Supported: "openai", "anthropic", "ollama"
[llm]
provider = "{llm_provider}"
{api_key_line}
model = "{llm_model}"
{base_url_line}

# --- Data Storage ---
[storage]
db_path = "~/.replyguy/replyguy.db"
retention_days = 90

# --- Logging ---
[logging]
# Seconds between periodic status summaries (0 = disabled).
status_interval_seconds = 0
"#,
        approval_mode = r.approval_mode,
        client_id = escape_toml(&r.client_id),
        client_secret_line = client_secret_line,
        product_name = escape_toml(&r.product_name),
        product_description = escape_toml(&r.product_description),
        product_url_line = product_url_line,
        target_audience = escape_toml(&r.target_audience),
        product_keywords = format_toml_array(&r.product_keywords),
        industry_topics = format_toml_array(&r.industry_topics),
        brand_voice_line = brand_voice_line,
        reply_style_line = reply_style_line,
        content_style_line = content_style_line,
        persona_opinions_line = persona_opinions_line,
        persona_experiences_line = persona_experiences_line,
        content_pillars_line = content_pillars_line,
        targets_section = targets_section,
        llm_provider = escape_toml(&r.llm_provider),
        api_key_line = api_key_line,
        llm_model = escape_toml(&r.llm_model),
        base_url_line = base_url_line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_basic() {
        assert_eq!(parse_csv("rust, cli, tools"), vec!["rust", "cli", "tools"]);
    }

    #[test]
    fn parse_csv_trims_and_filters_empty() {
        assert_eq!(parse_csv("  a , , b ,  "), vec!["a", "b"]);
    }

    #[test]
    fn parse_csv_empty_string() {
        assert!(parse_csv("").is_empty());
        assert!(parse_csv("   ").is_empty());
        assert!(parse_csv(",,,").is_empty());
    }

    #[test]
    fn escape_toml_special_chars() {
        assert_eq!(escape_toml(r#"hello "world""#), r#"hello \"world\""#);
        assert_eq!(escape_toml("back\\slash"), "back\\\\slash");
        assert_eq!(escape_toml("line\nbreak"), "line\\nbreak");
        assert_eq!(escape_toml("tab\there"), "tab\\there");
    }

    #[test]
    fn escape_toml_plain_string() {
        assert_eq!(escape_toml("hello world"), "hello world");
    }

    #[test]
    fn format_toml_array_basic() {
        let items = vec!["a".to_string(), "b".to_string()];
        assert_eq!(format_toml_array(&items), r#"["a", "b"]"#);
    }

    #[test]
    fn format_toml_array_escapes() {
        let items = vec!["say \"hi\"".to_string()];
        assert_eq!(format_toml_array(&items), r#"["say \"hi\""]"#);
    }

    #[test]
    fn render_config_toml_is_valid_toml() {
        let result = WizardResult {
            client_id: "test-client-id".to_string(),
            client_secret: Some("test-secret".to_string()),
            product_name: "TestProduct".to_string(),
            product_description: "A test product for devs".to_string(),
            product_url: Some("https://example.com".to_string()),
            target_audience: "developers".to_string(),
            product_keywords: vec!["rust".to_string(), "cli".to_string()],
            industry_topics: vec!["Rust development".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "openai".to_string(),
            llm_api_key: Some("sk-test-key".to_string()),
            llm_model: "gpt-4o-mini".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);

        // Must parse as valid TOML
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        // Roundtrip: verify key fields survive
        assert_eq!(config.x_api.client_id, "test-client-id");
        assert_eq!(config.x_api.client_secret, Some("test-secret".to_string()));
        assert_eq!(config.business.product_name, "TestProduct");
        assert_eq!(
            config.business.product_description,
            "A test product for devs"
        );
        assert_eq!(
            config.business.product_url,
            Some("https://example.com".to_string())
        );
        assert_eq!(config.business.target_audience, "developers");
        assert_eq!(config.business.product_keywords, vec!["rust", "cli"]);
        assert_eq!(config.business.industry_topics, vec!["Rust development"]);
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.api_key, Some("sk-test-key".to_string()));
        assert_eq!(config.llm.model, "gpt-4o-mini");
        assert!(config.llm.base_url.is_none());
    }

    #[test]
    fn render_config_toml_ollama_with_base_url() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "MyApp".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: Some("http://localhost:11434/v1".to_string()),
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert_eq!(config.llm.provider, "ollama");
        assert!(config.llm.api_key.is_none());
        assert_eq!(
            config.llm.base_url,
            Some("http://localhost:11434/v1".to_string())
        );
        // client_secret should be None (was commented out)
        assert!(config.x_api.client_secret.is_none());
        // product_url should be None (was commented out)
        assert!(config.business.product_url.is_none());
    }

    #[test]
    fn render_config_toml_escapes_special_chars() {
        let result = WizardResult {
            client_id: "id-with-\"quotes\"".to_string(),
            client_secret: None,
            product_name: "My \"App\"".to_string(),
            product_description: "A tool for\\devs".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["say \"hi\"".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("TOML with special chars should parse");

        assert_eq!(config.x_api.client_id, "id-with-\"quotes\"");
        assert_eq!(config.business.product_name, "My \"App\"");
        assert_eq!(config.business.product_description, "A tool for\\devs");
        assert_eq!(config.business.product_keywords, vec!["say \"hi\""]);
    }

    #[test]
    fn render_config_toml_with_brand_voice() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "VoiceApp".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: Some("Friendly technical expert. Casual, occasionally witty.".to_string()),
            reply_style: Some("Lead with genuine help. Ask follow-up questions.".to_string()),
            content_style: Some("Share practical tips with real examples.".to_string()),
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert_eq!(
            config.business.brand_voice,
            Some("Friendly technical expert. Casual, occasionally witty.".to_string())
        );
        assert_eq!(
            config.business.reply_style,
            Some("Lead with genuine help. Ask follow-up questions.".to_string())
        );
        assert_eq!(
            config.business.content_style,
            Some("Share practical tips with real examples.".to_string())
        );
    }

    #[test]
    fn render_config_toml_without_brand_voice() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "App".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        // When None, lines are commented out → deserialized as None
        assert!(config.business.brand_voice.is_none());
        assert!(config.business.reply_style.is_none());
        assert!(config.business.content_style.is_none());
    }

    #[test]
    fn render_config_toml_with_persona() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "App".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec!["Rust is the future".to_string(), "TDD matters".to_string()],
            persona_experiences: vec!["Built 3 startups".to_string()],
            content_pillars: vec!["Developer tools".to_string(), "Productivity".to_string()],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert_eq!(
            config.business.persona_opinions,
            vec!["Rust is the future", "TDD matters"]
        );
        assert_eq!(
            config.business.persona_experiences,
            vec!["Built 3 startups"]
        );
        assert_eq!(
            config.business.content_pillars,
            vec!["Developer tools", "Productivity"]
        );
    }

    #[test]
    fn render_config_toml_with_targets() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "App".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec!["elonmusk".to_string(), "levelsio".to_string()],
            auto_follow: true,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert_eq!(config.targets.accounts, vec!["elonmusk", "levelsio"]);
        assert!(config.targets.auto_follow);
    }

    #[test]
    fn render_config_toml_with_approval_mode() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "App".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: true,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert!(config.approval_mode);
    }

    #[test]
    fn render_config_toml_updated_defaults() {
        let result = WizardResult {
            client_id: "cid".to_string(),
            client_secret: None,
            product_name: "App".to_string(),
            product_description: "desc".to_string(),
            product_url: None,
            target_audience: "devs".to_string(),
            product_keywords: vec!["test".to_string()],
            industry_topics: vec!["topic".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
            target_accounts: vec![],
            auto_follow: false,
            approval_mode: false,
            llm_provider: "ollama".to_string(),
            llm_api_key: None,
            llm_model: "llama3.2".to_string(),
            llm_base_url: None,
        };

        let toml_str = render_config_toml(&result);
        let config: replyguy_core::config::Config =
            toml::from_str(&toml_str).expect("rendered TOML should parse");

        assert_eq!(config.limits.max_replies_per_day, 5);
        assert_eq!(config.limits.max_tweets_per_day, 6);
        assert_eq!(config.intervals.content_post_window_seconds, 10800);
        assert_eq!(config.logging.status_interval_seconds, 0);
        assert_eq!(config.limits.max_replies_per_author_per_day, 1);
        assert!((config.limits.product_mention_ratio - 0.2).abs() < f32::EPSILON);
        assert_eq!(
            config.limits.banned_phrases,
            vec!["check out", "you should try", "I recommend", "link in bio"]
        );
    }
}
