/// Display helpers for the setup wizard — banners, summaries, step headers.
use console::Style;

use super::wizard::WizardResult;

/// Print the step header: "Step N/8: Title\n".
pub(super) fn print_step_header(step: u8, title: &str) {
    let bold = Style::new().bold();
    eprintln!("{}", bold.apply_to(format!("Step {step}/8: {title}")));
    eprintln!();
}

/// Print a subtitle under a step header.
pub(super) fn print_step_subtitle(lines: &[&str]) {
    let dim = Style::new().dim();
    for line in lines {
        eprintln!("{}", dim.apply_to(*line));
    }
    eprintln!();
}

pub(super) fn print_welcome_banner() {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Welcome to Tuitbot Setup"));
    eprintln!(
        "{}",
        dim.apply_to("This wizard will create your configuration in 8 steps.")
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

/// Display a summary of all collected values.
pub(super) fn print_summary(result: &WizardResult) {
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
    eprintln!(
        "  Approval Mode:     {}",
        if result.approval_mode { "yes" } else { "no" }
    );

    eprintln!();
    eprintln!("  Timezone:          {}", result.timezone);
    eprintln!(
        "  Active Hours:      {}:00 – {}:00",
        result.active_hours_start, result.active_hours_end
    );
    eprintln!("  Active Days:       {}", result.active_days.join(", "));

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

pub(super) fn print_remaining_steps(steps: &[&str]) {
    let bold = Style::new().bold();

    eprintln!();
    eprintln!("{}", bold.apply_to("Remaining steps:"));
    for (i, step) in steps.iter().enumerate() {
        eprintln!("  {}. {step}", i + 1);
    }
}

/// Print the quickstart welcome banner (replaces the 8-step banner).
pub(super) fn print_quickstart_banner() {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Tuitbot Quick Setup"));
    eprintln!("{}", dim.apply_to("───────────────────"));
    eprintln!("{}", dim.apply_to("Before we start, have these ready:"));
    eprintln!(
        "{}",
        dim.apply_to("  • X API Client ID  — from https://developer.x.com")
    );
    eprintln!(
        "{}",
        dim.apply_to("  • An LLM API key   — OpenAI, Anthropic, or Ollama (free, local)")
    );
    eprintln!();
    eprintln!(
        "{}",
        dim.apply_to("5 questions to get you running. Use --advanced for full configuration.")
    );
    eprintln!();
}

/// Print inline guide for obtaining an X API Client ID.
pub(super) fn print_x_api_guide() {
    let dim = Style::new().dim();

    eprintln!();
    eprintln!(
        "{}",
        dim.apply_to("To get your Client ID (takes ~2 minutes):")
    );
    eprintln!(
        "{}",
        dim.apply_to("  1. Go to https://developer.x.com/en/portal/dashboard")
    );
    eprintln!(
        "{}",
        dim.apply_to("  2. Create a Project and App (or select an existing one)")
    );
    eprintln!(
        "{}",
        dim.apply_to("  3. Under \"User authentication settings\", enable OAuth 2.0:")
    );
    eprintln!(
        "{}",
        dim.apply_to("     Type: \"Web App\" — Callback URL: http://127.0.0.1:8080/callback")
    );
    eprintln!(
        "{}",
        dim.apply_to("  4. Copy the Client ID from the \"Keys and tokens\" tab")
    );
    eprintln!();
}

/// Print a green checkmark for a successful LLM validation.
pub(super) fn print_llm_validation_ok(provider: &str, model: &str, latency_ms: u128) {
    let green = Style::new().green();
    eprintln!(
        "{}",
        green.apply_to(format!(
            "  ✓ Connected to {provider} ({model}, {latency_ms}ms)"
        ))
    );
}

/// Print a yellow warning for a failed LLM validation.
pub(super) fn print_llm_validation_fail(provider: &str, error: &str) {
    let yellow = Style::new().yellow();
    eprintln!(
        "{}",
        yellow.apply_to(format!("  ⚠ Could not connect to {provider}: {error}"))
    );
}

/// Print a compact summary of quickstart-collected values.
pub(super) fn print_quickstart_summary(result: &WizardResult) {
    let bold = Style::new().bold();
    let dim = Style::new().dim();

    eprintln!();
    eprintln!("{}", bold.apply_to("Configuration Summary"));
    eprintln!("{}", dim.apply_to("─────────────────────"));
    eprintln!("  Product:     {}", result.product_name);
    eprintln!("  Keywords:    {}", result.product_keywords.join(", "));
    eprintln!(
        "  LLM:         {} ({})",
        result.llm_provider, result.llm_model
    );
    eprintln!("  X API:       {} (Client ID set)", result.client_id);
    eprintln!("  Approval:    on (all posts queued for review)");
    eprintln!();
    eprintln!(
        "  {}",
        dim.apply_to("Defaults applied: UTC schedule, no brand voice, no persona.")
    );
    eprintln!(
        "  {}",
        dim.apply_to("Customize later: tuitbot init --advanced  or  tuitbot settings")
    );
    eprintln!();
}
