use super::helpers::{capitalize, escape_toml, format_toml_array, non_empty, parse_csv};
use super::render::render_config_toml;
use super::wizard::WizardResult;
use super::EXAMPLE_CONFIG;

// ============================================================================
// EXAMPLE_CONFIG validation
// ============================================================================

#[test]
fn example_config_is_valid_toml() {
    let _config: tuitbot_core::config::Config =
        toml::from_str(EXAMPLE_CONFIG).expect("config.example.toml should be valid TOML");
}

#[test]
fn example_config_has_required_sections() {
    assert!(EXAMPLE_CONFIG.contains("[x_api]"));
    assert!(EXAMPLE_CONFIG.contains("[business]"));
    assert!(EXAMPLE_CONFIG.contains("[llm]"));
    assert!(EXAMPLE_CONFIG.contains("[scoring]"));
    assert!(EXAMPLE_CONFIG.contains("[limits]"));
    assert!(EXAMPLE_CONFIG.contains("[intervals]"));
    assert!(EXAMPLE_CONFIG.contains("[storage]"));
    assert!(EXAMPLE_CONFIG.contains("[schedule]"));
}

#[test]
fn example_config_has_sensible_defaults() {
    let config: tuitbot_core::config::Config =
        toml::from_str(EXAMPLE_CONFIG).expect("config.example.toml should parse");
    assert!(config.scoring.threshold <= 100);
    assert!(config.limits.max_replies_per_day > 0);
    assert!(config.limits.min_action_delay_seconds <= config.limits.max_action_delay_seconds);
    assert!(config.schedule.active_hours_start <= 23);
    assert!(config.schedule.active_hours_end <= 23);
    assert!(config.storage.retention_days > 0);
}

#[test]
fn example_config_is_not_empty() {
    assert!(!EXAMPLE_CONFIG.is_empty());
    assert!(
        EXAMPLE_CONFIG.len() > 100,
        "config.example.toml seems too small"
    );
}

// ============================================================================
// Helper function edge cases
// ============================================================================

#[test]
fn capitalize_unicode() {
    // Unicode characters should not panic
    let result = capitalize("uber");
    assert_eq!(result, "Uber");
}

#[test]
fn non_empty_only_spaces_returns_none() {
    assert_eq!(non_empty("   \t  ".to_string()), None);
}

#[test]
fn non_empty_newline_returns_trimmed() {
    assert_eq!(
        non_empty("\n hello \n".to_string()),
        Some("hello".to_string())
    );
}

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

/// Helper to create a default WizardResult for tests.
fn test_wizard_result() -> WizardResult {
    WizardResult {
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
        approval_mode: false,
        llm_provider: "ollama".to_string(),
        llm_api_key: None,
        llm_model: "llama3.2".to_string(),
        llm_base_url: None,
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".into(),
            "Tue".into(),
            "Wed".into(),
            "Thu".into(),
            "Fri".into(),
            "Sat".into(),
            "Sun".into(),
        ],
    }
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
        approval_mode: false,
        llm_provider: "openai".to_string(),
        llm_api_key: Some("sk-test-key".to_string()),
        llm_model: "gpt-4o-mini".to_string(),
        llm_base_url: None,
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".into(),
            "Tue".into(),
            "Wed".into(),
            "Thu".into(),
            "Fri".into(),
            "Sat".into(),
            "Sun".into(),
        ],
    };

    let toml_str = render_config_toml(&result);

    // Must parse as valid TOML
    let config: tuitbot_core::config::Config =
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
        llm_base_url: Some("http://localhost:11434/v1".to_string()),
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
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
        product_name: "My \"App\"".to_string(),
        product_description: "A tool for\\devs".to_string(),
        product_keywords: vec!["say \"hi\"".to_string()],
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("TOML with special chars should parse");

    assert_eq!(config.x_api.client_id, "id-with-\"quotes\"");
    assert_eq!(config.business.product_name, "My \"App\"");
    assert_eq!(config.business.product_description, "A tool for\\devs");
    assert_eq!(config.business.product_keywords, vec!["say \"hi\""]);
}

#[test]
fn render_config_toml_with_brand_voice() {
    let result = WizardResult {
        product_name: "VoiceApp".to_string(),
        brand_voice: Some("Friendly technical expert. Casual, occasionally witty.".to_string()),
        reply_style: Some("Lead with genuine help. Ask follow-up questions.".to_string()),
        content_style: Some("Share practical tips with real examples.".to_string()),
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
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
    let result = test_wizard_result();

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("rendered TOML should parse");

    // When None, lines are commented out → deserialized as None
    assert!(config.business.brand_voice.is_none());
    assert!(config.business.reply_style.is_none());
    assert!(config.business.content_style.is_none());
}

#[test]
fn render_config_toml_with_persona() {
    let result = WizardResult {
        persona_opinions: vec!["Rust is the future".to_string(), "TDD matters".to_string()],
        persona_experiences: vec!["Built 3 startups".to_string()],
        content_pillars: vec!["Developer tools".to_string(), "Productivity".to_string()],
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
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
        target_accounts: vec!["elonmusk".to_string(), "levelsio".to_string()],
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("rendered TOML should parse");

    assert_eq!(config.targets.accounts, vec!["elonmusk", "levelsio"]);
}

#[test]
fn render_config_toml_with_approval_mode() {
    let result = WizardResult {
        approval_mode: true,
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("rendered TOML should parse");

    assert!(config.approval_mode);
}

#[test]
fn render_config_toml_updated_defaults() {
    let result = test_wizard_result();

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
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

#[test]
fn render_quickstart_minimal_is_valid_toml() {
    let result = WizardResult {
        client_id: "qs-client".to_string(),
        client_secret: None,
        product_name: "QuickApp".to_string(),
        product_description: String::new(),
        product_url: None,
        target_audience: String::new(),
        product_keywords: vec!["rust cli".to_string()],
        industry_topics: vec![],
        brand_voice: None,
        reply_style: None,
        content_style: None,
        persona_opinions: vec![],
        persona_experiences: vec![],
        content_pillars: vec![],
        target_accounts: vec![],
        approval_mode: true,
        llm_provider: "ollama".to_string(),
        llm_api_key: None,
        llm_model: "llama3.2".to_string(),
        llm_base_url: None,
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".into(),
            "Tue".into(),
            "Wed".into(),
            "Thu".into(),
            "Fri".into(),
            "Sat".into(),
            "Sun".into(),
        ],
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart-minimal TOML should parse");

    // Quickstart fields survive roundtrip
    assert_eq!(config.business.product_name, "QuickApp");
    assert_eq!(config.business.product_keywords, vec!["rust cli"]);
    assert_eq!(config.llm.provider, "ollama");

    // Empty optional fields should be absent / default
    assert!(config.business.product_description.is_empty());
    assert!(config.business.target_audience.is_empty());
    assert!(config.business.industry_topics.is_empty());

    // Minimal quickstart omits product_description and industry_topics,
    // which are now required — validation should flag them.
    let errors = config.validate().unwrap_err();
    let fields: Vec<&str> = errors
        .iter()
        .filter_map(|e| match e {
            tuitbot_core::error::ConfigError::MissingField { field } => Some(field.as_str()),
            _ => None,
        })
        .collect();
    assert!(fields.contains(&"business.product_description"));
    assert!(fields.contains(&"business.industry_topics"));
}

#[test]
fn render_quickstart_omits_empty_fields() {
    let result = WizardResult {
        product_description: String::new(),
        target_audience: String::new(),
        industry_topics: vec![],
        ..test_wizard_result()
    };

    let toml_str = render_config_toml(&result);

    // Empty fields should be rendered as comments
    assert!(
        toml_str.contains("# product_description"),
        "empty product_description should be commented"
    );
    assert!(
        toml_str.contains("# target_audience"),
        "empty target_audience should be commented"
    );
    assert!(
        toml_str.contains("# industry_topics"),
        "empty industry_topics should be commented"
    );

    // Should still parse as valid TOML
    let _config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("TOML with commented fields should parse");
}

/// Helper to create a WizardResult matching quickstart defaults.
fn quickstart_wizard_result() -> WizardResult {
    WizardResult {
        client_id: "qs-cid".to_string(),
        client_secret: None,
        product_name: "QuickApp".to_string(),
        product_description: String::new(),
        product_url: None,
        target_audience: String::new(),
        product_keywords: vec!["rust".to_string(), "cli".to_string()],
        industry_topics: vec![],
        brand_voice: None,
        reply_style: None,
        content_style: None,
        persona_opinions: vec![],
        persona_experiences: vec![],
        content_pillars: vec![],
        target_accounts: vec![],
        approval_mode: true,
        llm_provider: "openai".to_string(),
        llm_api_key: Some("sk-test".to_string()),
        llm_model: "gpt-4o-mini".to_string(),
        llm_base_url: None,
        timezone: "UTC".to_string(),
        active_hours_start: 8,
        active_hours_end: 22,
        active_days: vec![
            "Mon".into(),
            "Tue".into(),
            "Wed".into(),
            "Thu".into(),
            "Fri".into(),
            "Sat".into(),
            "Sun".into(),
        ],
    }
}

#[test]
fn quickstart_wizard_result_renders_valid_toml() {
    let result = quickstart_wizard_result();
    let toml_str = render_config_toml(&result);

    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart TOML should parse");

    assert_eq!(config.business.product_name, "QuickApp");
    assert_eq!(config.business.product_keywords, vec!["rust", "cli"]);
    assert_eq!(config.llm.provider, "openai");
    assert_eq!(config.llm.model, "gpt-4o-mini");
    assert_eq!(config.x_api.client_id, "qs-cid");
    assert!(config.approval_mode);

    // Empty optional fields should be absent / default
    assert!(config.business.product_description.is_empty());
    assert!(config.business.target_audience.is_empty());
    assert!(config.business.industry_topics.is_empty());

    // Quickstart omits product_description and industry_topics,
    // which are now required — validation should flag them.
    let errors = config.validate().unwrap_err();
    let fields: Vec<&str> = errors
        .iter()
        .filter_map(|e| match e {
            tuitbot_core::error::ConfigError::MissingField { field } => Some(field.as_str()),
            _ => None,
        })
        .collect();
    assert!(fields.contains(&"business.product_description"));
    assert!(fields.contains(&"business.industry_topics"));
}

#[test]
fn quickstart_wizard_result_defaults_are_correct() {
    let result = quickstart_wizard_result();

    assert!(result.approval_mode);
    assert_eq!(result.timezone, "UTC");
    assert_eq!(result.active_hours_start, 8);
    assert_eq!(result.active_hours_end, 22);
    assert_eq!(result.active_days.len(), 7);
    assert_eq!(result.active_days[0], "Mon");
    assert_eq!(result.active_days[6], "Sun");

    // Enrichment fields should all be empty
    assert!(result.product_description.is_empty());
    assert!(result.target_audience.is_empty());
    assert!(result.industry_topics.is_empty());
    assert!(result.brand_voice.is_none());
    assert!(result.reply_style.is_none());
    assert!(result.content_style.is_none());
    assert!(result.persona_opinions.is_empty());
    assert!(result.persona_experiences.is_empty());
    assert!(result.content_pillars.is_empty());
    assert!(result.target_accounts.is_empty());
    assert!(result.client_secret.is_none());
    assert!(result.product_url.is_none());
}

#[test]
fn advanced_wizard_result_still_renders_valid_toml() {
    // Simulates a fully-filled advanced wizard result
    let result = WizardResult {
        client_id: "adv-cid".to_string(),
        client_secret: Some("adv-secret".to_string()),
        product_name: "AdvancedApp".to_string(),
        product_description: "A full-featured app for devs".to_string(),
        product_url: Some("https://advanced.example.com".to_string()),
        target_audience: "developers and founders".to_string(),
        product_keywords: vec!["devtools".to_string(), "productivity".to_string()],
        industry_topics: vec!["Developer tools".to_string(), "SaaS".to_string()],
        brand_voice: Some("Friendly expert".to_string()),
        reply_style: Some("Lead with help".to_string()),
        content_style: Some("Practical tips".to_string()),
        persona_opinions: vec!["Rust is great".to_string()],
        persona_experiences: vec!["Built 3 startups".to_string()],
        content_pillars: vec!["Dev tools".to_string()],
        target_accounts: vec!["levelsio".to_string()],
        approval_mode: false,
        llm_provider: "anthropic".to_string(),
        llm_api_key: Some("sk-ant-test".to_string()),
        llm_model: "claude-sonnet-4-6".to_string(),
        llm_base_url: None,
        timezone: "America/New_York".to_string(),
        active_hours_start: 9,
        active_hours_end: 21,
        active_days: vec![
            "Mon".into(),
            "Tue".into(),
            "Wed".into(),
            "Thu".into(),
            "Fri".into(),
        ],
    };

    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("advanced TOML should parse");

    assert_eq!(config.business.product_name, "AdvancedApp");
    assert_eq!(
        config.business.product_description,
        "A full-featured app for devs"
    );
    assert_eq!(config.business.target_audience, "developers and founders");
    assert_eq!(
        config.business.industry_topics,
        vec!["Developer tools", "SaaS"]
    );
    assert_eq!(
        config.business.brand_voice,
        Some("Friendly expert".to_string())
    );
    assert_eq!(config.targets.accounts, vec!["levelsio"]);
    assert!(!config.approval_mode);
    assert_eq!(config.schedule.timezone, "America/New_York");
    assert_eq!(config.schedule.active_hours_start, 9);
    assert_eq!(config.schedule.active_hours_end, 21);
    assert_eq!(config.schedule.active_days.len(), 5);

    assert!(config.validate().is_ok());
}

// ============================================================================
// Quickstart → enrichment invariant tests
// ============================================================================

use tuitbot_core::config::EnrichmentStage;

#[test]
fn quickstart_config_not_enriched() {
    let result = quickstart_wizard_result();
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart TOML should parse");

    let pc = config.profile_completeness();
    assert!(
        !pc.is_fully_enriched(),
        "quickstart config should NOT be fully enriched"
    );
}

#[test]
fn quickstart_config_all_enrichment_stages_incomplete() {
    let result = quickstart_wizard_result();
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart TOML should parse");

    let pc = config.profile_completeness();
    assert_eq!(
        pc.completed_count(),
        0,
        "quickstart config should have 0 completed enrichment stages"
    );
}

#[test]
fn quickstart_config_next_incomplete_is_voice() {
    let result = quickstart_wizard_result();
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart TOML should parse");

    let pc = config.profile_completeness();
    assert_eq!(
        pc.next_incomplete(),
        Some(EnrichmentStage::Voice),
        "first incomplete stage should be Voice"
    );
}

#[test]
fn quickstart_config_one_line_summary_all_dashes() {
    let result = quickstart_wizard_result();
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("quickstart TOML should parse");

    let pc = config.profile_completeness();
    assert_eq!(pc.one_line_summary(), "Voice --  Persona --  Targeting --");
}

#[test]
fn advanced_config_voice_shows_partial_enrichment() {
    let result = WizardResult {
        brand_voice: Some("Friendly technical expert".to_string()),
        ..quickstart_wizard_result()
    };
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("advanced TOML should parse");

    let pc = config.profile_completeness();
    assert_eq!(pc.completed_count(), 1);
    assert_eq!(pc.one_line_summary(), "Voice OK  Persona --  Targeting --");
}

#[test]
fn advanced_config_fully_enriched() {
    let result = WizardResult {
        brand_voice: Some("Friendly".to_string()),
        reply_style: Some("Helpful".to_string()),
        content_style: Some("Practical".to_string()),
        persona_opinions: vec!["Rust is great".to_string()],
        persona_experiences: vec!["Built startups".to_string()],
        content_pillars: vec!["Dev tools".to_string()],
        target_accounts: vec!["levelsio".to_string()],
        ..quickstart_wizard_result()
    };
    let toml_str = render_config_toml(&result);
    let config: tuitbot_core::config::Config =
        toml::from_str(&toml_str).expect("fully enriched TOML should parse");

    let pc = config.profile_completeness();
    assert!(pc.is_fully_enriched());
    assert_eq!(pc.completed_count(), 3);
    assert!(pc.next_incomplete().is_none());
}
