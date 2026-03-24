use anyhow::Result;
use dialoguer::{Input, Select};
use tuitbot_core::config::Config;
use tuitbot_core::safety::redact::mask_optional_secret;

use super::{
    edit_and_record_list, edit_and_record_opt_string, edit_and_record_string, edit_and_record_u32,
    edit_and_record_u64, print_category_header,
};
use crate::commands::settings::helpers::{edit_f32, edit_list, ChangeTracker};
use crate::commands::settings::show::format_list;

// ---------------------------------------------------------------------------
// Category editors: product / voice / persona / llm / xapi / targets / limits
// ---------------------------------------------------------------------------

pub(crate) fn edit_category_product(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Your Product");

    let fields = &[
        format!("Product name:        {}", config.business.product_name),
        format!(
            "Description:         {}",
            config.business.product_description
        ),
        format!(
            "Product URL:         {}",
            config
                .business
                .product_url
                .as_deref()
                .unwrap_or("(not set)")
        ),
        format!("Target audience:     {}", config.business.target_audience),
        format!(
            "Discovery keywords:  {}",
            format_list(&config.business.product_keywords)
        ),
        format!(
            "Competitor keywords: {}",
            format_list(&config.business.competitor_keywords)
        ),
        format!(
            "Content topics:      {}",
            format_list(&config.business.industry_topics)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => edit_and_record_string(
            tracker,
            &mut config.business.product_name,
            "business",
            "product_name",
            "Product name",
        )?,
        1 => edit_and_record_string(
            tracker,
            &mut config.business.product_description,
            "business",
            "product_description",
            "Description",
        )?,
        2 => edit_and_record_opt_string(
            tracker,
            &mut config.business.product_url,
            "business",
            "product_url",
            "Product URL",
        )?,
        3 => edit_and_record_string(
            tracker,
            &mut config.business.target_audience,
            "business",
            "target_audience",
            "Target audience",
        )?,
        4 => edit_and_record_list(
            tracker,
            &mut config.business.product_keywords,
            "business",
            "product_keywords",
            "Discovery keywords",
        )?,
        5 => edit_and_record_list(
            tracker,
            &mut config.business.competitor_keywords,
            "business",
            "competitor_keywords",
            "Competitor keywords",
        )?,
        6 => edit_and_record_list(
            tracker,
            &mut config.business.industry_topics,
            "business",
            "industry_topics",
            "Content topics",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_voice(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("Brand Voice");

    let fields = &[
        format!(
            "Personality:   {}",
            config
                .business
                .brand_voice
                .as_deref()
                .unwrap_or("(default)")
        ),
        format!(
            "Reply style:   {}",
            config
                .business
                .reply_style
                .as_deref()
                .unwrap_or("(default)")
        ),
        format!(
            "Content style: {}",
            config
                .business
                .content_style
                .as_deref()
                .unwrap_or("(default)")
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => edit_and_record_opt_string(
            tracker,
            &mut config.business.brand_voice,
            "business",
            "brand_voice",
            "Personality",
        )?,
        1 => edit_and_record_opt_string(
            tracker,
            &mut config.business.reply_style,
            "business",
            "reply_style",
            "Reply style",
        )?,
        2 => edit_and_record_opt_string(
            tracker,
            &mut config.business.content_style,
            "business",
            "content_style",
            "Content style",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_persona(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Persona");

    let fields = &[
        format!(
            "Opinions:        {}",
            format_list(&config.business.persona_opinions)
        ),
        format!(
            "Experiences:     {}",
            format_list(&config.business.persona_experiences)
        ),
        format!(
            "Content pillars: {}",
            format_list(&config.business.content_pillars)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => edit_and_record_list(
            tracker,
            &mut config.business.persona_opinions,
            "business",
            "persona_opinions",
            "Opinions",
        )?,
        1 => edit_and_record_list(
            tracker,
            &mut config.business.persona_experiences,
            "business",
            "persona_experiences",
            "Experiences",
        )?,
        2 => edit_and_record_list(
            tracker,
            &mut config.business.content_pillars,
            "business",
            "content_pillars",
            "Content pillars",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_llm(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("AI Provider");

    let fields = &[
        format!("Provider:  {}", config.llm.provider),
        format!("API key:   {}", mask_optional_secret(&config.llm.api_key)),
        format!("Model:     {}", config.llm.model),
        format!(
            "Base URL:  {}",
            config.llm.base_url.as_deref().unwrap_or("(default)")
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let providers = &["openai", "anthropic", "ollama"];
            let current = providers
                .iter()
                .position(|p| *p == config.llm.provider)
                .unwrap_or(0);
            let idx = Select::new()
                .with_prompt("LLM provider")
                .items(providers)
                .default(current)
                .interact()?;
            let new_val = providers[idx].to_string();
            tracker.record("llm", "provider", &config.llm.provider, &new_val);
            config.llm.provider = new_val;
        }
        1 => {
            let new_val: String = Input::new()
                .with_prompt("API key")
                .allow_empty(true)
                .interact_text()?;
            if new_val.trim().is_empty() {
                tracker.record("llm", "api_key", "(hidden)", "(cleared)");
                config.llm.api_key = None;
            } else {
                tracker.record("llm", "api_key", "(hidden)", "(updated)");
                config.llm.api_key = Some(new_val.trim().to_string());
            }
        }
        2 => edit_and_record_string(tracker, &mut config.llm.model, "llm", "model", "Model")?,
        3 => edit_and_record_opt_string(
            tracker,
            &mut config.llm.base_url,
            "llm",
            "base_url",
            "Base URL",
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_xapi(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("X API Credentials");

    let fields = &[
        format!("Client ID:     {}", config.x_api.client_id),
        format!(
            "Client secret: {}",
            mask_optional_secret(&config.x_api.client_secret)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => edit_and_record_string(
            tracker,
            &mut config.x_api.client_id,
            "x_api",
            "client_id",
            "Client ID",
        )?,
        1 => {
            let new_val: String = Input::new()
                .with_prompt("Client secret (Enter to clear)")
                .allow_empty(true)
                .interact_text()?;
            if new_val.trim().is_empty() {
                tracker.record("x_api", "client_secret", "(hidden)", "(cleared)");
                config.x_api.client_secret = None;
            } else {
                tracker.record("x_api", "client_secret", "(hidden)", "(updated)");
                config.x_api.client_secret = Some(new_val.trim().to_string());
            }
        }
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_targets(
    config: &mut Config,
    tracker: &mut ChangeTracker,
) -> Result<()> {
    print_category_header("Target Accounts");

    let fields = &[
        format!(
            "Accounts:          {}",
            format_list(&config.targets.accounts)
        ),
        format!(
            "Max replies/day:   {}",
            config.targets.max_target_replies_per_day
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let new_val = edit_list("Target accounts (without @)", &config.targets.accounts)?;
            let cleaned: Vec<String> = new_val
                .into_iter()
                .map(|a| a.trim_start_matches('@').to_string())
                .collect();
            let old_display = config.targets.accounts.join(", ");
            let new_display = cleaned.join(", ");
            tracker.record("targets", "accounts", &old_display, &new_display);
            config.targets.accounts = cleaned;
        }
        1 => edit_and_record_u32(
            tracker,
            &mut config.targets.max_target_replies_per_day,
            "targets",
            "max_target_replies_per_day",
            "Max target replies per day",
            None,
        )?,
        _ => {} // Back
    }

    Ok(())
}

pub(crate) fn edit_category_limits(config: &mut Config, tracker: &mut ChangeTracker) -> Result<()> {
    print_category_header("Posting Limits");

    let fields = &[
        format!("Replies/day:         {}", config.limits.max_replies_per_day),
        format!("Tweets/day:          {}", config.limits.max_tweets_per_day),
        format!(
            "Threads/week:        {}",
            config.limits.max_threads_per_week
        ),
        format!(
            "Min action delay:    {} sec",
            config.limits.min_action_delay_seconds
        ),
        format!(
            "Max action delay:    {} sec",
            config.limits.max_action_delay_seconds
        ),
        format!(
            "Replies/author/day:  {}",
            config.limits.max_replies_per_author_per_day
        ),
        format!(
            "Product mention %:   {:.0}%",
            config.limits.product_mention_ratio * 100.0
        ),
        format!(
            "Banned phrases:      {}",
            format_list(&config.limits.banned_phrases)
        ),
        "Back to categories".to_string(),
    ];

    let selection = Select::new()
        .with_prompt("Which field to change?")
        .items(fields)
        .default(0)
        .interact()?;

    match selection {
        0 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_replies_per_day,
            "limits",
            "max_replies_per_day",
            "Max replies per day",
            Some("How many tweets to reply to per day"),
        )?,
        1 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_tweets_per_day,
            "limits",
            "max_tweets_per_day",
            "Max tweets per day",
            Some("How many original tweets to post per day"),
        )?,
        2 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_threads_per_week,
            "limits",
            "max_threads_per_week",
            "Max threads per week",
            Some("How many educational threads to post per week"),
        )?,
        3 => edit_and_record_u64(
            tracker,
            &mut config.limits.min_action_delay_seconds,
            "limits",
            "min_action_delay_seconds",
            "Min action delay (seconds)",
            Some("Minimum wait between actions"),
        )?,
        4 => edit_and_record_u64(
            tracker,
            &mut config.limits.max_action_delay_seconds,
            "limits",
            "max_action_delay_seconds",
            "Max action delay (seconds)",
            Some("Maximum wait between actions"),
        )?,
        5 => edit_and_record_u32(
            tracker,
            &mut config.limits.max_replies_per_author_per_day,
            "limits",
            "max_replies_per_author_per_day",
            "Max replies per author per day",
            Some("Avoid spamming the same person"),
        )?,
        6 => {
            let new_val = edit_f32(
                "Product mention ratio (0.0-1.0)",
                config.limits.product_mention_ratio,
                Some("What fraction of replies may mention your product (e.g., 0.2 = 20%)"),
            )?;
            tracker.record(
                "limits",
                "product_mention_ratio",
                &format!("{:.2}", config.limits.product_mention_ratio),
                &format!("{:.2}", new_val),
            );
            config.limits.product_mention_ratio = new_val;
        }
        7 => edit_and_record_list(
            tracker,
            &mut config.limits.banned_phrases,
            "limits",
            "banned_phrases",
            "Banned phrases",
        )?,
        _ => {} // Back
    }

    Ok(())
}
