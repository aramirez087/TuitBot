//! Configuration tools: get_config, validate_config.

use tuitbot_core::config::Config;

/// Get current config with secrets redacted.
pub fn get_config(config: &Config) -> String {
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

    match serde_json::to_string_pretty(&redacted) {
        Ok(json) => json,
        Err(e) => format!("Error serializing config: {e}"),
    }
}

/// Validate the current configuration and report any errors.
pub fn validate_config(config: &Config) -> String {
    match config.validate() {
        Ok(()) => serde_json::json!({
            "valid": true,
            "errors": [],
        })
        .to_string(),
        Err(errors) => {
            let error_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            serde_json::json!({
                "valid": false,
                "errors": error_msgs,
            })
            .to_string()
        }
    }
}
