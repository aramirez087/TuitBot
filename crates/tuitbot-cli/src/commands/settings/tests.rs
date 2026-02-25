use tuitbot_core::config::Config;
use tuitbot_core::safety::redact::mask_optional_secret as mask_secret;

use super::helpers::*;
use super::render::render_config;
use super::show::*;

#[test]
fn mask_secret_long() {
    let s = Some("sk-1234567890abcdef".to_string());
    assert_eq!(mask_secret(&s), "sk-1...cdef");
}

#[test]
fn mask_secret_short() {
    let s = Some("abc".to_string());
    assert_eq!(mask_secret(&s), "****");
}

#[test]
fn mask_secret_none() {
    assert_eq!(mask_secret(&None), "(not set)");
}

#[test]
fn format_list_empty() {
    assert_eq!(format_list(&[]), "(none)");
}

#[test]
fn format_list_items() {
    let items = vec!["a".to_string(), "b".to_string()];
    assert_eq!(format_list(&items), "a, b");
}

#[test]
fn format_duration_seconds() {
    assert_eq!(format_duration(30), "30 seconds");
}

#[test]
fn format_duration_minutes() {
    assert_eq!(format_duration(300), "5 min");
    assert_eq!(format_duration(330), "5 min 30 sec");
}

#[test]
fn format_duration_hours() {
    assert_eq!(format_duration(3600), "1 hour");
    assert_eq!(format_duration(10800), "3 hours");
    assert_eq!(format_duration(5400), "1 hour 30 min");
}

#[test]
fn format_duration_days() {
    assert_eq!(format_duration(86400), "1 day");
    assert_eq!(format_duration(604800), "7 days");
    assert_eq!(format_duration(90000), "1 day 1 hour");
}

#[test]
fn parse_duration_input_minutes() {
    assert_eq!(parse_duration_input("15").unwrap(), 900);
}

#[test]
fn parse_duration_input_hours() {
    assert_eq!(parse_duration_input("3h").unwrap(), 10800);
    assert_eq!(parse_duration_input("3H").unwrap(), 10800);
}

#[test]
fn parse_duration_input_days() {
    assert_eq!(parse_duration_input("7d").unwrap(), 604800);
}

#[test]
fn parse_bool_values() {
    assert!(parse_bool("true").unwrap());
    assert!(parse_bool("yes").unwrap());
    assert!(parse_bool("1").unwrap());
    assert!(parse_bool("on").unwrap());
    assert!(!parse_bool("false").unwrap());
    assert!(!parse_bool("no").unwrap());
    assert!(!parse_bool("0").unwrap());
    assert!(!parse_bool("off").unwrap());
    assert!(parse_bool("maybe").is_err());
}

#[test]
fn change_tracker_only_records_changes() {
    let mut tracker = ChangeTracker::new();
    tracker.record("section", "field", "old", "new");
    tracker.record("section", "field", "same", "same");
    assert_eq!(tracker.changes.len(), 1);
}

#[test]
fn render_config_roundtrip() {
    let mut config = Config::default();
    config.business.product_name = "TestProduct".to_string();
    config.business.product_description = "A test product".to_string();
    config.business.target_audience = "developers".to_string();
    config.business.product_keywords = vec!["rust".to_string(), "cli".to_string()];
    config.business.industry_topics = vec!["Rust dev".to_string()];
    config.x_api.client_id = "test-client-id".to_string();
    config.llm.provider = "ollama".to_string();
    config.llm.model = "llama3.2".to_string();

    let toml_str = render_config(&config);
    let parsed: Config = toml::from_str(&toml_str).expect("rendered config should parse");

    assert_eq!(parsed.business.product_name, "TestProduct");
    assert_eq!(parsed.x_api.client_id, "test-client-id");
    assert_eq!(parsed.llm.provider, "ollama");
    assert_eq!(parsed.scoring.threshold, config.scoring.threshold);
    assert_eq!(
        parsed.limits.max_replies_per_day,
        config.limits.max_replies_per_day
    );
}

#[test]
fn render_config_with_all_fields() {
    let mut config = Config::default();
    config.business.product_name = "FullApp".to_string();
    config.business.product_description = "Complete app".to_string();
    config.business.product_url = Some("https://example.com".to_string());
    config.business.target_audience = "everyone".to_string();
    config.business.product_keywords = vec!["test".to_string()];
    config.business.competitor_keywords = vec!["rival".to_string()];
    config.business.industry_topics = vec!["topic".to_string()];
    config.business.brand_voice = Some("Friendly".to_string());
    config.business.reply_style = Some("Helpful".to_string());
    config.business.content_style = Some("Practical".to_string());
    config.business.persona_opinions = vec!["Strong opinion".to_string()];
    config.business.persona_experiences = vec!["Built stuff".to_string()];
    config.business.content_pillars = vec!["Dev tools".to_string()];
    config.x_api.client_id = "cid".to_string();
    config.x_api.client_secret = Some("secret".to_string());
    config.llm.provider = "openai".to_string();
    config.llm.api_key = Some("sk-test".to_string());
    config.llm.model = "gpt-4o-mini".to_string();
    config.llm.base_url = Some("https://api.openai.com".to_string());
    config.targets.accounts = vec!["user1".to_string()];
    config.approval_mode = true;

    let toml_str = render_config(&config);
    let parsed: Config = toml::from_str(&toml_str).expect("rendered config should parse");

    assert_eq!(
        parsed.business.product_url,
        Some("https://example.com".to_string())
    );
    assert_eq!(parsed.business.brand_voice, Some("Friendly".to_string()));
    assert_eq!(parsed.targets.accounts, vec!["user1"]);
    assert!(parsed.approval_mode);
    assert_eq!(parsed.llm.api_key, Some("sk-test".to_string()));
}

#[test]
fn render_config_escapes_special_chars() {
    let mut config = Config::default();
    config.business.product_name = "My \"App\"".to_string();
    config.business.product_description = "line\\break".to_string();
    config.business.target_audience = "devs".to_string();
    config.business.product_keywords = vec!["say \"hi\"".to_string()];
    config.business.industry_topics = vec!["topic".to_string()];
    config.x_api.client_id = "id-\"test\"".to_string();
    config.llm.provider = "ollama".to_string();
    config.llm.model = "llama3.2".to_string();

    let toml_str = render_config(&config);
    let parsed: Config = toml::from_str(&toml_str).expect("config with special chars should parse");

    assert_eq!(parsed.business.product_name, "My \"App\"");
    assert_eq!(parsed.business.product_description, "line\\break");
    assert_eq!(parsed.x_api.client_id, "id-\"test\"");
}
