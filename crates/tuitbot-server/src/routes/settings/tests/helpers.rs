//! Tests for credentials cleanup, config_errors_to_response, request types,
//! and serialization helpers.

use tuitbot_core::error::ConfigError;

use crate::routes::settings::validation::{delete_all_credentials, redact_service_account_keys};
use crate::routes::settings::{
    config_errors_to_response, FactoryResetRequest, TestLlmRequest, ValidationErrorItem,
    ValidationResponse, FACTORY_RESET_PHRASE,
};

// ── config_errors_to_response tests ───────────────────────────────

#[test]
fn config_errors_to_response_missing_field() {
    let errors = vec![ConfigError::MissingField {
        field: "api_key".to_string(),
    }];
    let items = config_errors_to_response(errors);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].field, "api_key");
    assert!(items[0].message.contains("required"));
}

#[test]
fn config_errors_to_response_invalid_value() {
    let errors = vec![ConfigError::InvalidValue {
        field: "port".to_string(),
        message: "must be positive".to_string(),
    }];
    let items = config_errors_to_response(errors);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].field, "port");
    assert_eq!(items[0].message, "must be positive");
}

#[test]
fn config_errors_to_response_parse_error_variant() {
    let parse_err: toml::de::Error =
        toml::from_str::<toml::Value>("invalid [[[").expect_err("should fail");
    let errors = vec![ConfigError::ParseError { source: parse_err }];
    let items = config_errors_to_response(errors);
    assert_eq!(items.len(), 1);
    assert!(items[0].field.is_empty());
    assert!(!items[0].message.is_empty());
}

#[test]
fn config_errors_to_response_empty() {
    let items = config_errors_to_response(vec![]);
    assert!(items.is_empty());
}

#[test]
fn config_errors_to_response_multiple() {
    let errors = vec![
        ConfigError::MissingField {
            field: "a".to_string(),
        },
        ConfigError::InvalidValue {
            field: "b".to_string(),
            message: "too large".to_string(),
        },
    ];
    let items = config_errors_to_response(errors);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].field, "a");
    assert_eq!(items[1].field, "b");
}

#[test]
fn config_errors_to_response_preserves_order() {
    let errors = vec![
        ConfigError::MissingField {
            field: "first".to_string(),
        },
        ConfigError::MissingField {
            field: "second".to_string(),
        },
        ConfigError::MissingField {
            field: "third".to_string(),
        },
    ];
    let items = config_errors_to_response(errors);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].field, "first");
    assert_eq!(items[1].field, "second");
    assert_eq!(items[2].field, "third");
}

// ── delete_all_credentials tests ──────────────────────────────────

#[test]
fn delete_all_credentials_empty_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!delete_all_credentials(dir.path()), "nothing to delete");
}

#[test]
fn delete_all_credentials_with_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("scraper_session.json"), "{}").expect("write");
    std::fs::write(dir.path().join("tokens.json"), "{}").expect("write");

    assert!(delete_all_credentials(dir.path()));
    assert!(!dir.path().join("scraper_session.json").exists());
    assert!(!dir.path().join("tokens.json").exists());
}

#[test]
fn delete_all_credentials_with_accounts_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let accounts_dir = dir.path().join("accounts");
    std::fs::create_dir_all(accounts_dir.join("uuid1")).expect("create");
    std::fs::write(accounts_dir.join("uuid1").join("tokens.json"), "{}").expect("write");

    assert!(delete_all_credentials(dir.path()));
    assert!(!accounts_dir.exists());
}

#[test]
fn delete_all_credentials_partial_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("tokens.json"), "{}").expect("write");

    assert!(delete_all_credentials(dir.path()));
    assert!(!dir.path().join("tokens.json").exists());
}

#[test]
fn delete_all_credentials_nonexistent_dir() {
    let path = std::env::temp_dir().join("tuitbot_test_nonexistent_delete_creds");
    assert!(!delete_all_credentials(&path));
}

#[test]
fn delete_all_credentials_only_scraper_session() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("scraper_session.json"), "{}").expect("write");

    assert!(delete_all_credentials(dir.path()));
    assert!(!dir.path().join("scraper_session.json").exists());
}

#[test]
fn delete_all_credentials_nested_accounts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let accounts_dir = dir.path().join("accounts");
    std::fs::create_dir_all(accounts_dir.join("uuid-1")).expect("create");
    std::fs::create_dir_all(accounts_dir.join("uuid-2")).expect("create");
    std::fs::write(accounts_dir.join("uuid-1").join("tokens.json"), "{}").expect("write");
    std::fs::write(accounts_dir.join("uuid-2").join("tokens.json"), "{}").expect("write");

    assert!(delete_all_credentials(dir.path()));
    assert!(!accounts_dir.exists());
}

// ── FACTORY_RESET_PHRASE tests ────────────────────────────────────

#[test]
fn factory_reset_phrase_is_expected_value() {
    assert_eq!(FACTORY_RESET_PHRASE, "RESET TUITBOT");
}

#[test]
fn factory_reset_phrase_constant() {
    assert!(!FACTORY_RESET_PHRASE.is_empty());
    assert!(FACTORY_RESET_PHRASE.len() > 5);
}

#[test]
fn factory_reset_request_deserialize() {
    let json = r#"{"confirmation": "RESET TUITBOT"}"#;
    let req: FactoryResetRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.confirmation, "RESET TUITBOT");
}

#[test]
fn factory_reset_request_wrong_phrase() {
    let json = r#"{"confirmation": "wrong phrase"}"#;
    let req: FactoryResetRequest = serde_json::from_str(json).unwrap();
    assert_ne!(req.confirmation, FACTORY_RESET_PHRASE);
}

// ── TestLlmRequest tests ──────────────────────────────────────────

#[test]
fn test_llm_request_deserialize() {
    let json = r#"{"provider": "openai", "model": "gpt-4"}"#;
    let req: TestLlmRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.provider, "openai");
    assert_eq!(req.model, "gpt-4");
    assert!(req.api_key.is_none());
    assert!(req.base_url.is_none());
}

#[test]
fn test_llm_request_with_all_fields() {
    let json = r#"{"provider":"ollama","api_key":"sk-test","model":"llama2","base_url":"http://localhost:11434"}"#;
    let req: TestLlmRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.provider, "ollama");
    assert_eq!(req.api_key.as_deref(), Some("sk-test"));
    assert_eq!(req.base_url.as_deref(), Some("http://localhost:11434"));
}

#[test]
fn test_llm_request_minimal() {
    let json = r#"{"provider": "openai", "model": "gpt-3.5-turbo"}"#;
    let req: TestLlmRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.provider, "openai");
    assert_eq!(req.model, "gpt-3.5-turbo");
    assert!(req.api_key.is_none());
    assert!(req.base_url.is_none());
}

// ── Serialization tests for response types ────────────────────────

#[test]
fn validation_response_serialization() {
    let resp = ValidationResponse {
        valid: true,
        errors: Vec::new(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["valid"], true);
}

#[test]
fn validation_error_item_fields() {
    let item = ValidationErrorItem {
        field: "test_field".to_string(),
        message: "test message".to_string(),
    };
    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["field"], "test_field");
    assert_eq!(json["message"], "test message");
}

// ── redact edge cases (shared with toml.rs, different focus) ──────

#[test]
fn redact_sources_not_in_json() {
    let mut json = serde_json::json!({"other_key": "value"});
    redact_service_account_keys(&mut json);
    assert_eq!(json["other_key"], "value");
}
