/// Interactive wizard steps: quickstart (5 prompts) and advanced (8 steps).
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};

use super::display::{print_step_header, print_step_subtitle, print_x_api_guide};
use super::helpers::{capitalize, non_empty, parse_csv};
use super::prompts;
use super::wizard::WizardResult;

/// Quickstart: collect 5 critical inputs, default everything else.
pub(super) fn step_quickstart() -> Result<WizardResult> {
    // Prompt 1: Product name (easy, builds momentum)
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

    // Prompt 2: Discovery keywords (CSV, at least 1)
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
    let keywords = parse_csv(&keywords_raw);

    // Prompt 3: LLM provider
    let providers = &["openai", "anthropic", "ollama"];
    let selection = Select::new()
        .with_prompt("LLM provider")
        .items(providers)
        .default(0)
        .interact()?;
    let provider = providers[selection].to_string();

    // Prompt 4: API key (skipped for ollama)
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

    let (default_model, base_url) = match provider.as_str() {
        "openai" => ("gpt-4o-mini", None),
        "anthropic" => ("claude-sonnet-4-6", None),
        "ollama" => ("llama3.2", Some("http://localhost:11434/v1".to_string())),
        _ => ("", None),
    };

    // Prompt 5: X API Client ID (highest friction last)
    print_x_api_guide();

    let client_id: String = Input::new()
        .with_prompt("X API Client ID")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.trim().is_empty() {
                Err("Client ID cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    Ok(WizardResult {
        client_id: client_id.trim().to_string(),
        client_secret: None,
        product_name: product_name.trim().to_string(),
        product_description: String::new(),
        product_url: None,
        target_audience: String::new(),
        product_keywords: keywords,
        industry_topics: vec![], // empty → effective_industry_topics() derives from keywords
        brand_voice: None,
        reply_style: None,
        content_style: None,
        persona_opinions: vec![],
        persona_experiences: vec![],
        content_pillars: vec![],
        target_accounts: vec![],
        approval_mode: true, // safe default
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        llm_provider: provider,
        llm_api_key: api_key,
        llm_model: default_model.to_string(),
        llm_base_url: base_url,
    })
}

/// Step 1/8: X API credentials.
pub(super) fn step_x_api() -> Result<WizardResult> {
    print_step_header(1, "X API Credentials");

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
        approval_mode: true,
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        llm_provider: String::new(),
        llm_api_key: None,
        llm_model: String::new(),
        llm_base_url: None,
    })
}

/// Step 2/8: Business profile.
pub(super) fn step_business_profile(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(2, "Business Profile");

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

/// Step 3/8: Brand voice and style.
pub(super) fn step_brand_voice(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(3, "Brand Voice & Style");
    print_step_subtitle(&[
        "These shape how the bot sounds when it replies and posts.",
        "All fields are optional — press Enter to skip and use defaults.",
    ]);

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

/// Step 4/8: LLM provider.
pub(super) fn step_llm_provider(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(4, "LLM Provider");

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

/// Step 5/8: Persona (optional).
pub(super) fn step_persona(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(5, "Persona (optional)");
    print_step_subtitle(&[
        "Strong opinions, experiences, and pillars make content more authentic.",
        "All fields are optional — press Enter to skip.",
    ]);

    let (opinions, experiences, pillars) = prompts::prompt_persona()?;

    Ok(WizardResult {
        persona_opinions: opinions,
        persona_experiences: experiences,
        content_pillars: pillars,
        ..prev
    })
}

/// Step 6/8: Target Accounts (optional).
pub(super) fn step_target_accounts(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(6, "Target Accounts (optional)");
    print_step_subtitle(&["Monitor specific accounts and reply to their conversations."]);

    let accounts = prompts::prompt_target_accounts()?;

    Ok(WizardResult {
        target_accounts: accounts,
        ..prev
    })
}

/// Step 7/8: Approval Mode.
pub(super) fn step_approval_mode(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(7, "Approval Mode");

    let approval_mode = prompts::prompt_approval_mode()?;

    Ok(WizardResult {
        approval_mode,
        ..prev
    })
}

/// Step 8/8: Active Hours Schedule.
pub(super) fn step_schedule(prev: WizardResult) -> Result<WizardResult> {
    print_step_header(8, "Active Hours Schedule");
    print_step_subtitle(&[
        "Configure when the bot is active. Outside these hours it sleeps.",
        "Press Enter to accept defaults (UTC, 8 AM – 10 PM, every day).",
    ]);

    let timezone: String = Input::new()
        .with_prompt("Timezone (IANA name, e.g. America/New_York)")
        .default("UTC".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), String> {
            input
                .trim()
                .parse::<chrono_tz::Tz>()
                .map(|_| ())
                .map_err(|_| format!("Unknown timezone: {input}. Use IANA names like America/New_York, Europe/London, Asia/Tokyo."))
        })
        .interact_text()?;

    let start_raw: String = Input::new()
        .with_prompt("Active hours start (0-23)")
        .default("8".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            input
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|&h| h <= 23)
                .map(|_| ())
                .ok_or("Must be 0-23")
        })
        .interact_text()?;
    let active_hours_start: u8 = start_raw.trim().parse().unwrap_or(8);

    let end_raw: String = Input::new()
        .with_prompt("Active hours end (0-23)")
        .default("22".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            input
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|&h| h <= 23)
                .map(|_| ())
                .ok_or("Must be 0-23")
        })
        .interact_text()?;
    let active_hours_end: u8 = end_raw.trim().parse().unwrap_or(22);

    let all_days = &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let use_all_days = Confirm::new()
        .with_prompt("Active every day of the week?")
        .default(true)
        .interact()?;

    let active_days: Vec<String> = if use_all_days {
        all_days.iter().map(|d| d.to_string()).collect()
    } else {
        let selections = dialoguer::MultiSelect::new()
            .with_prompt("Select active days")
            .items(all_days)
            .defaults(&[true, true, true, true, true, false, false])
            .interact()?;
        selections
            .iter()
            .map(|&i| all_days[i].to_string())
            .collect()
    };

    eprintln!();

    Ok(WizardResult {
        timezone: timezone.trim().to_string(),
        active_hours_start,
        active_hours_end,
        active_days,
        ..prev
    })
}
