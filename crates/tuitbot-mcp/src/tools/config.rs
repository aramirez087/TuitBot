//! Configuration tools: get_config, validate_config.

use std::time::Instant;

use tuitbot_core::config::Config;

use super::response::{ToolMeta, ToolResponse};

/// Get current config with secrets redacted.
pub fn get_config(config: &Config) -> String {
    let start = Instant::now();
    let mut redacted = config.clone();

    // Redact sensitive fields
    if !redacted.x_api.client_id.is_empty() {
        redacted.x_api.client_id = "***REDACTED***".to_string();
    }
    if redacted.x_api.client_secret.is_some() {
        redacted.x_api.client_secret = Some("***REDACTED***".to_string());
    }
    if redacted.llm.api_key.is_some() {
        redacted.llm.api_key = Some("***REDACTED***".to_string());
    }

    let elapsed = start.elapsed().as_millis() as u64;
    let meta =
        ToolMeta::new(elapsed).with_mode(config.mode.to_string(), config.effective_approval_mode());
    ToolResponse::success(redacted).with_meta(meta).to_json()
}

/// Validate the current configuration and report any errors.
pub fn validate_config(config: &Config) -> String {
    let start = Instant::now();

    let result = match config.validate() {
        Ok(()) => serde_json::json!({
            "valid": true,
            "errors": [],
        }),
        Err(errors) => {
            let error_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            serde_json::json!({
                "valid": false,
                "errors": error_msgs,
            })
        }
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let meta =
        ToolMeta::new(elapsed).with_mode(config.mode.to_string(), config.effective_approval_mode());
    ToolResponse::success(result).with_meta(meta).to_json()
}
