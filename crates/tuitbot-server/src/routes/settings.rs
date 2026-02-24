//! Settings endpoints for reading and updating the configuration.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::Value;
use tuitbot_core::config::Config;

use crate::error::ApiError;
use crate::state::AppState;

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

    // Read the current config file as a TOML value for merging.
    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    let mut toml_value: toml::Value = contents.parse().map_err(|e: toml::de::Error| {
        ApiError::BadRequest(format!("failed to parse existing config: {e}"))
    })?;

    // Convert the JSON patch to a TOML value for merging.
    let patch_toml = json_to_toml(&patch)
        .map_err(|e| ApiError::BadRequest(format!("patch contains invalid values: {e}")))?;

    // Deep-merge the patch into the existing config.
    merge_toml(&mut toml_value, &patch_toml);

    // Validate the merged result parses as a valid Config.
    let merged_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize merged config: {e}")))?;
    let _config: Config = toml::from_str(&merged_str)
        .map_err(|e| ApiError::BadRequest(format!("merged config is invalid: {e}")))?;

    // Write the merged config back to disk.
    std::fs::write(&state.config_path, &merged_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    // Return the updated config as JSON.
    let updated: Config = toml::from_str(&merged_str)
        .map_err(|e| ApiError::BadRequest(format!("failed to re-parse config: {e}")))?;
    let json = serde_json::to_value(updated)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize config: {e}")))?;

    Ok(Json(json))
}

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
fn json_to_toml(json: &serde_json::Value) -> Result<toml::Value, String> {
    match json {
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (key, val) in map {
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
        serde_json::Value::Null => Err("null values are not supported in TOML".to_string()),
    }
}
