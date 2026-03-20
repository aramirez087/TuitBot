//! Shared helpers: config I/O, patch merging, TOML conversion, and
//! credential cleanup. No Axum handlers here — pure utility functions.

use std::path::Path;

use serde_json::Value;
use tuitbot_core::config::Config;

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Config file helpers
// ---------------------------------------------------------------------------

/// Read the config file, merge a JSON patch into it, and parse the result.
///
/// Returns `(merged_toml_string, parsed_config)` on success.
pub fn merge_patch_and_parse(
    config_path: &Path,
    patch: &Value,
) -> Result<(String, Config), ApiError> {
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

/// Load and parse the base config from the TOML file.
pub(crate) fn load_base_config(config_path: &Path) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            config_path.display()
        ))
    })?;

    toml::from_str(&contents)
        .map_err(|e| ApiError::BadRequest(format!("failed to parse config: {e}")))
}

/// Replace any non-null `service_account_key` values in `content_sources.sources`
/// with `"[redacted]"` so secrets are never returned in API responses.
pub(super) fn redact_service_account_keys(json: &mut Value) {
    if let Some(sources) = json
        .get_mut("content_sources")
        .and_then(|cs| cs.get_mut("sources"))
        .and_then(|s| s.as_array_mut())
    {
        for source in sources {
            if let Some(key) = source.get_mut("service_account_key") {
                if !key.is_null() {
                    *key = serde_json::Value::String("[redacted]".to_string());
                }
            }
        }
    }
}

/// Delete all credential files produced during normal operation.
///
/// Removes root-level `scraper_session.json` / `tokens.json` (default
/// account) plus the entire `accounts/` subdirectory tree (per-account
/// credentials). Returns `true` if anything was actually removed.
pub(super) fn delete_all_credentials(data_dir: &std::path::Path) -> bool {
    let mut deleted = false;

    // Root-level credential files (default account).
    for name in &["scraper_session.json", "tokens.json"] {
        let path = data_dir.join(name);
        match std::fs::remove_file(&path) {
            Ok(()) => deleted = true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "failed to delete credential file")
            }
        }
    }

    // Per-account data directories (accounts/{uuid}/).
    let accounts_dir = data_dir.join("accounts");
    match std::fs::remove_dir_all(&accounts_dir) {
        Ok(()) => deleted = true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => tracing::warn!(error = %e, "failed to delete accounts directory"),
    }

    deleted
}

// ---------------------------------------------------------------------------
// TOML utilities
// ---------------------------------------------------------------------------

/// Recursively merge `patch` into `base`. Tables are merged key-by-key;
/// scalar values in `patch` overwrite those in `base`.
pub(super) fn merge_toml(base: &mut toml::Value, patch: &toml::Value) {
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
pub(super) fn json_to_toml(json: &serde_json::Value) -> Result<toml::Value, String> {
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

// ---------------------------------------------------------------------------
// test_llm + factory_reset handlers (low-coupling, grouped here)
// ---------------------------------------------------------------------------

use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use tuitbot_core::config::LlmConfig;
use tuitbot_core::llm::factory::create_provider;

use crate::state::AppState;

use super::{TestLlmRequest, TestResult};

/// `GET /api/settings/defaults` — return the built-in default configuration.
pub async fn get_defaults() -> Result<Json<serde_json::Value>, crate::error::ApiError> {
    let defaults = tuitbot_core::config::Config::default();
    let json = serde_json::to_value(defaults).map_err(|e| {
        crate::error::ApiError::BadRequest(format!("failed to serialize defaults: {e}"))
    })?;
    Ok(Json(json))
}

/// `POST /api/settings/test-llm` — test LLM provider connectivity.
pub async fn test_llm(
    Json(body): Json<TestLlmRequest>,
) -> Result<Json<serde_json::Value>, crate::error::ApiError> {
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

use super::{FactoryResetCleared, FactoryResetRequest, FactoryResetResponse, FACTORY_RESET_PHRASE};

/// `POST /api/settings/factory-reset` -- erase all Tuitbot-managed data.
///
/// Requires authentication (bearer or session+CSRF). Validates a typed
/// confirmation phrase before proceeding. Stops runtimes, clears all 31
/// DB tables in a single transaction, deletes config/passphrase/media files,
/// clears in-memory state, and returns a response that also clears the
/// session cookie.
pub async fn factory_reset(
    State(state): State<Arc<AppState>>,
    Json(body): Json<FactoryResetRequest>,
) -> Result<impl IntoResponse, crate::error::ApiError> {
    if body.confirmation != FACTORY_RESET_PHRASE {
        return Err(crate::error::ApiError::BadRequest(
            "incorrect confirmation phrase".to_string(),
        ));
    }

    let runtimes_stopped = {
        let mut runtimes = state.runtimes.lock().await;
        let count = runtimes.len() as u32;
        for (_, mut rt) in runtimes.drain() {
            rt.shutdown().await;
        }
        count
    };

    if let Some(cancel) = state.watchtower_cancel.write().await.take() {
        cancel.cancel();
    }

    let reset_stats = tuitbot_core::storage::reset::factory_reset(&state.db).await?;
    tuitbot_core::storage::accounts::ensure_default_account(&state.db).await?;

    let config_deleted = match std::fs::remove_file(&state.config_path) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            tracing::warn!(error = %e, "failed to delete config file");
            false
        }
    };

    let passphrase_path = state.data_dir.join("passphrase_hash");
    let passphrase_deleted = match std::fs::remove_file(&passphrase_path) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            tracing::warn!(error = %e, "failed to delete passphrase hash");
            false
        }
    };

    let media_dir = state.data_dir.join("media");
    let media_deleted = match std::fs::remove_dir_all(&media_dir) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            tracing::warn!(error = %e, "failed to delete media directory");
            false
        }
    };

    let credentials_deleted = delete_all_credentials(&state.data_dir);

    *state.passphrase_hash.write().await = None;
    *state.passphrase_hash_mtime.write().await = None;
    state.content_generators.lock().await.clear();
    state.login_attempts.lock().await.clear();
    state.pending_oauth.lock().await.clear();
    state.token_managers.lock().await.clear();

    tracing::info!(
        tables = reset_stats.tables_cleared,
        rows = reset_stats.rows_deleted,
        config = config_deleted,
        passphrase = passphrase_deleted,
        media = media_deleted,
        credentials = credentials_deleted,
        runtimes = runtimes_stopped,
        "Factory reset completed"
    );

    let response = FactoryResetResponse {
        status: "reset_complete".to_string(),
        cleared: FactoryResetCleared {
            tables_cleared: reset_stats.tables_cleared,
            rows_deleted: reset_stats.rows_deleted,
            config_deleted,
            passphrase_deleted,
            media_deleted,
            credentials_deleted,
            runtimes_stopped,
        },
    };

    let cookie = "tuitbot_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0";
    Ok((
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(serde_json::to_value(response).unwrap()),
    ))
}
