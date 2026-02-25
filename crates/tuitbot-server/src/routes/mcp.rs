//! MCP governance and telemetry endpoints.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tuitbot_core::config::Config;
use tuitbot_core::mcp_policy::templates;
use tuitbot_core::mcp_policy::types::PolicyTemplateName;
use tuitbot_core::storage::{mcp_telemetry, rate_limits};

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct TimeWindowQuery {
    /// Lookback window in hours (default: 24).
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

#[derive(Deserialize)]
pub struct RecentQuery {
    /// Number of recent entries to return (default: 50).
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

// ---------------------------------------------------------------------------
// Policy endpoints
// ---------------------------------------------------------------------------

/// `GET /api/mcp/policy` — current MCP policy config + rate limit usage + v2 fields.
pub async fn get_policy(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let config = read_config(&state)?;

    let rate_limit_info = match rate_limits::get_all_rate_limits(&state.db).await {
        Ok(limits) => {
            let mcp = limits.iter().find(|l| l.action_type == "mcp_mutation");
            match mcp {
                Some(rl) => json!({
                    "used": rl.request_count,
                    "max": rl.max_requests,
                    "period_seconds": rl.period_seconds,
                    "period_start": rl.period_start,
                }),
                None => json!({ "used": 0, "max": config.mcp_policy.max_mutations_per_hour }),
            }
        }
        Err(_) => json!({ "used": 0, "max": config.mcp_policy.max_mutations_per_hour }),
    };

    Ok(Json(json!({
        "enforce_for_mutations": config.mcp_policy.enforce_for_mutations,
        "require_approval_for": config.mcp_policy.require_approval_for,
        "blocked_tools": config.mcp_policy.blocked_tools,
        "dry_run_mutations": config.mcp_policy.dry_run_mutations,
        "max_mutations_per_hour": config.mcp_policy.max_mutations_per_hour,
        "mode": format!("{}", config.mode),
        "rate_limit": rate_limit_info,
        "template": config.mcp_policy.template,
        "rules": config.mcp_policy.rules,
        "rate_limits": config.mcp_policy.rate_limits,
    })))
}

/// `PATCH /api/mcp/policy` — update MCP policy config fields.
///
/// Accepts partial JSON with `mcp_policy` fields and merges into config.
pub async fn patch_policy(
    State(state): State<Arc<AppState>>,
    Json(patch): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    if !patch.is_object() {
        return Err(ApiError::BadRequest(
            "request body must be a JSON object".to_string(),
        ));
    }

    // Wrap the patch under `mcp_policy` key for the settings merge.
    let wrapped = json!({ "mcp_policy": patch });

    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    let mut toml_value: toml::Value = contents.parse().map_err(|e: toml::de::Error| {
        ApiError::BadRequest(format!("failed to parse existing config: {e}"))
    })?;

    let patch_toml = json_to_toml(&wrapped)
        .map_err(|e| ApiError::BadRequest(format!("patch contains invalid values: {e}")))?;

    merge_toml(&mut toml_value, &patch_toml);

    let merged_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize merged config: {e}")))?;

    let config: Config = toml::from_str(&merged_str)
        .map_err(|e| ApiError::BadRequest(format!("merged config is invalid: {e}")))?;

    std::fs::write(&state.config_path, &merged_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    Ok(Json(json!({
        "enforce_for_mutations": config.mcp_policy.enforce_for_mutations,
        "require_approval_for": config.mcp_policy.require_approval_for,
        "blocked_tools": config.mcp_policy.blocked_tools,
        "dry_run_mutations": config.mcp_policy.dry_run_mutations,
        "max_mutations_per_hour": config.mcp_policy.max_mutations_per_hour,
        "template": config.mcp_policy.template,
        "rules": config.mcp_policy.rules,
        "rate_limits": config.mcp_policy.rate_limits,
    })))
}

// ---------------------------------------------------------------------------
// Template endpoints
// ---------------------------------------------------------------------------

/// `GET /api/mcp/policy/templates` — list available policy templates.
pub async fn list_templates() -> Json<Value> {
    let templates = templates::list_templates();
    Json(json!(templates))
}

/// `POST /api/mcp/policy/templates/{name}` — apply a template.
pub async fn apply_template(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let template_name: PolicyTemplateName =
        name.parse().map_err(|e: String| ApiError::BadRequest(e))?;

    let template = templates::get_template(&template_name);

    // Build a patch that sets the template and its rules/rate_limits
    let patch = json!({
        "template": template_name,
        "rules": template.rules,
        "rate_limits": template.rate_limits,
    });

    // Wrap under mcp_policy and merge into config
    let wrapped = json!({ "mcp_policy": patch });

    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    let mut toml_value: toml::Value = contents.parse().map_err(|e: toml::de::Error| {
        ApiError::BadRequest(format!("failed to parse existing config: {e}"))
    })?;

    let patch_toml = json_to_toml(&wrapped)
        .map_err(|e| ApiError::BadRequest(format!("patch contains invalid values: {e}")))?;

    merge_toml(&mut toml_value, &patch_toml);

    let merged_str = toml::to_string_pretty(&toml_value)
        .map_err(|e| ApiError::BadRequest(format!("failed to serialize merged config: {e}")))?;

    let config: Config = toml::from_str(&merged_str)
        .map_err(|e| ApiError::BadRequest(format!("merged config is invalid: {e}")))?;

    std::fs::write(&state.config_path, &merged_str).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not write config file {}: {e}",
            state.config_path.display()
        ))
    })?;

    // Initialize rate limit rows for the new template limits
    if let Err(e) =
        rate_limits::init_policy_rate_limits(&state.db, &config.mcp_policy.rate_limits).await
    {
        tracing::warn!("Failed to initialize policy rate limits: {e}");
    }

    Ok(Json(json!({
        "applied_template": template_name,
        "description": template.description,
        "rules_count": config.mcp_policy.rules.len(),
        "rate_limits_count": config.mcp_policy.rate_limits.len(),
    })))
}

// ---------------------------------------------------------------------------
// Telemetry endpoints
// ---------------------------------------------------------------------------

/// `GET /api/mcp/telemetry/summary` — aggregate stats over a time window.
pub async fn telemetry_summary(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeWindowQuery>,
) -> Result<Json<Value>, ApiError> {
    let since = since_timestamp(params.hours);
    let summary = mcp_telemetry::get_summary(&state.db, &since).await?;
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

/// `GET /api/mcp/telemetry/metrics` — per-tool metrics over a time window.
pub async fn telemetry_metrics(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeWindowQuery>,
) -> Result<Json<Value>, ApiError> {
    let since = since_timestamp(params.hours);
    let metrics = mcp_telemetry::get_metrics_since(&state.db, &since).await?;
    Ok(Json(json!(metrics)))
}

/// `GET /api/mcp/telemetry/errors` — error breakdown over a time window.
pub async fn telemetry_errors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeWindowQuery>,
) -> Result<Json<Value>, ApiError> {
    let since = since_timestamp(params.hours);
    let errors = mcp_telemetry::get_error_breakdown(&state.db, &since).await?;
    Ok(Json(json!(errors)))
}

/// `GET /api/mcp/telemetry/recent` — recent tool executions.
pub async fn telemetry_recent(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentQuery>,
) -> Result<Json<Value>, ApiError> {
    let entries = mcp_telemetry::get_recent_entries(&state.db, params.limit).await?;
    Ok(Json(json!(entries)))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_config(state: &AppState) -> Result<Config, ApiError> {
    let contents = std::fs::read_to_string(&state.config_path).map_err(|e| {
        ApiError::BadRequest(format!(
            "could not read config file {}: {e}",
            state.config_path.display()
        ))
    })?;
    let config: Config = toml::from_str(&contents)
        .map_err(|e| ApiError::BadRequest(format!("failed to parse config: {e}")))?;
    Ok(config)
}

fn since_timestamp(hours: u32) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let since_epoch = now.saturating_sub(u64::from(hours) * 3600);

    // Convert epoch seconds to ISO-8601 UTC (YYYY-MM-DDTHH:MM:SSZ).
    let secs = since_epoch as i64;
    let days = secs.div_euclid(86400);
    let day_secs = secs.rem_euclid(86400);
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    // Days since epoch → date using the civil-from-days algorithm.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Recursively merge `patch` into `base`.
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

/// Convert JSON to TOML, skipping nulls in objects.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_timestamp_is_valid_utc() {
        let ts = since_timestamp(24);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }
}
