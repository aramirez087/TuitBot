//! Settings endpoints for reading and updating the configuration.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tuitbot_core::config::{Config, LlmConfig};
use tuitbot_core::error::ConfigError;
use tuitbot_core::llm::factory::create_provider;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ValidationResponse {
    valid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<ValidationErrorItem>,
}

#[derive(Serialize)]
struct ValidationErrorItem {
    field: String,
    message: String,
}

#[derive(Deserialize)]
pub struct TestLlmRequest {
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Serialize)]
struct TestResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read the config file, merge a JSON patch into it, and parse the result.
///
/// Returns `(merged_toml_string, parsed_config)` on success.
fn merge_patch_and_parse(config_path: &Path, patch: &Value) -> Result<(String, Config), ApiError> {
    let contents = std::fs::read_to_string(config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            config_path.display()
        ))
    })?;

    let mut toml_value: toml::Value = contents.parse().map_err(|e: toml::de::Error| {
        ApiError::BadRequest(format!("failed to parse existing config: {e}"))
    })?;

    let patch_toml = json_to_toml(patch)
        .map_err(|e| ApiError::BadRequest(format!("patch contains invalid values: {e}")))?;

    merge_toml(&mut toml_value, &patch_toml);

    let merged_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize merged config: {e}")))?;

    let config: Config = toml::from_str(&merged_str)
        .map_err(|e| ApiError::BadRequest(format!("merged config is invalid: {e}")))?;

    Ok((merged_str, config))
}

fn config_errors_to_response(errors: Vec<ConfigError>) -> Vec<ValidationErrorItem> {
    errors
        .into_iter()
        .map(|e| match e {
            ConfigError::MissingField { field } => ValidationErrorItem {
                field,
                message: "this field is required".to_string(),
            },
            ConfigError::InvalidValue { field, message } => ValidationErrorItem { field, message },
            other => ValidationErrorItem {
                field: String::new(),
                message: other.to_string(),
            },
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Onboarding endpoints (no auth required)
// ---------------------------------------------------------------------------

/// `GET /api/settings/status` — check if config exists.
///
/// Also returns `deployment_mode` and `capabilities` so unauthenticated
/// pages (e.g. onboarding) can adapt their source-type UI.
pub async fn config_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let configured = state.config_path.exists();
    let capabilities = state.deployment_mode.capabilities();
    Json(serde_json::json!({
        "configured": configured,
        "deployment_mode": state.deployment_mode,
        "capabilities": capabilities,
    }))
}

/// `POST /api/settings/init` — create initial config from JSON.
///
/// Accepts the full configuration as JSON, validates it, converts to TOML,
/// and writes to `config_path`. Returns 409 if config already exists.
pub async fn init_settings(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if state.config_path.exists() {
        return Err(ApiError::Conflict(
            "configuration already exists; use PATCH /api/settings to update".to_string(),
        ));
    }

    if !body.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    // Convert JSON to TOML.
    let toml_value = json_to_toml(&body)
        .map_err(|e| ApiError::BadRequest(format!("invalid config values: {e}")))?;

    let toml_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    // Validate by parsing through Config.
    let config: Config = toml::from_str(&toml_str)
        .map_err(|e| ApiError::BadRequest(format!("invalid config: {e}")))?;

    if let Err(errors) = config.validate() {
        let items = config_errors_to_response(errors);
        return Ok(Json(serde_json::json!({
            "status": "validation_failed",
            "errors": items
        })));
    }

    // Ensure parent directory exists and write.
    if let Some(parent) = state.config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ApiError::BadRequest(format!("failed to create config directory: {e}")))?;
    }

    std::fs::write(&state.config_path, &toml_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    // Set file permissions to 0600 on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ =
            std::fs::set_permissions(&state.config_path, std::fs::Permissions::from_mode(0o600));
    }

    let json = serde_json::to_value(config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "config": json
    })))
}

// ---------------------------------------------------------------------------
// Endpoints
// ---------------------------------------------------------------------------

/// `GET /api/settings` — return the current config as JSON.
pub async fn get_settings(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    let config: Config = toml::from_str(&contents)
        .map_err(|e| ApiError::BadRequest(format!("failed to parse config: {e}")))?;

    let json = serde_json::to_value(config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    Ok(Json(json))
}

/// `PATCH /api/settings` — merge partial JSON into the config and write back.
pub async fn patch_settings(
    State(state): State<Arc<AppState>>,
    Json(patch): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !patch.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    let (merged_str, config) = merge_patch_and_parse(&state.config_path, &patch)?;

    std::fs::write(&state.config_path, &merged_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    let json = serde_json::to_value(config)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    Ok(Json(json))
}

/// `POST /api/settings/validate` — validate a config change without saving.
pub async fn validate_settings(
    State(state): State<Arc<AppState>>,
    Json(patch): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !patch.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    let (_merged_str, config) = merge_patch_and_parse(&state.config_path, &patch)?;

    let response = match config.validate() {
        Ok(()) => ValidationResponse {
            valid: true,
            errors: Vec::new(),
        },
        Err(errors) => ValidationResponse {
            valid: false,
            errors: config_errors_to_response(errors),
        },
    };

    Ok(Json(serde_json::to_value(response).unwrap()))
}

/// `GET /api/settings/defaults` — return the built-in default configuration.
pub async fn get_defaults() -> Result<Json<Value>, ApiError> {
    let defaults = Config::default();
    let json = serde_json::to_value(defaults)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize defaults: {e}")))?;
    Ok(Json(json))
}

/// `POST /api/settings/test-llm` — test LLM provider connectivity.
pub async fn test_llm(Json(body): Json<TestLlmRequest>) -> Result<Json<Value>, ApiError> {
    let llm_config = LlmConfig {
        provider: body.provider,
        api_key: body.api_key,
        model: body.model,
        base_url: body.base_url,
    };

    let provider = match create_provider(&llm_config) {
        Ok(p) => p,
        Err(e) => {
            return Ok(Json(
                serde_json::to_value(TestResult {
                    success: false,
                    error: Some(e.to_string()),
                    latency_ms: None,
                })
                .unwrap(),
            ));
        }
    };

    let start = Instant::now();
    let latency_ms = |s: &Instant| s.elapsed().as_millis() as u64;

    match provider.health_check().await {
        Ok(()) => Ok(Json(
            serde_json::to_value(TestResult {
                success: true,
                error: None,
                latency_ms: Some(latency_ms(&start)),
            })
            .unwrap(),
        )),
        Err(e) => Ok(Json(
            serde_json::to_value(TestResult {
                success: false,
                error: Some(e.to_string()),
                latency_ms: Some(latency_ms(&start)),
            })
            .unwrap(),
        )),
    }
}

// ---------------------------------------------------------------------------
// TOML utilities
// ---------------------------------------------------------------------------

/// Recursively merge `patch` into `base`. Tables are merged key-by-key;
/// scalar values in `patch` overwrite those in `base`.
fn merge_toml(base: &mut toml::Value, patch: &toml::Value) {
    match (base, patch) {
        (toml::Value::Table(base_table), toml::Value::Table(patch_table)) => {
            for (key, patch_val) in patch_table {
                if let Some(base_val) = base_table.get_mut(key) {
                    merge_toml(base_val, patch_val);
                } else {
                    base_table.insert(key.clone(), patch_val.clone());
                }
            }
        }
        (base, _) => {
            *base = patch.clone();
        }
    }
}

/// Convert a `serde_json::Value` to a `toml::Value`.
///
/// Null values in objects are silently skipped (TOML has no null literal),
/// allowing the frontend to send `null` for optional fields to clear them.
/// Null values in arrays are rejected since arrays cannot have holes.
fn json_to_toml(json: &serde_json::Value) -> Result<toml::Value, String> {
    match json {
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (key, val) in map {
                if val.is_null() {
                    continue;
                }
                table.insert(key.clone(), json_to_toml(val)?);
            }
            Ok(toml::Value::Table(table))
        }
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>, _> = arr.iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(values?))
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(format!("unsupported number: {n}"))
            }
        }
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Null => Err("null values are not supported in TOML arrays".to_string()),
    }
}
