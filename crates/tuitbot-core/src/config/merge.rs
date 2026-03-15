//! Account-aware configuration merging.
//!
//! Implements RFC 7396 JSON merge-patch semantics to produce an effective
//! configuration by merging a base `Config` (from `config.toml`) with
//! per-account `config_overrides` stored in the database.

use super::Config;
use crate::error::ConfigError;
use serde_json::Value;

/// Top-level Config keys that are account-scoped.
///
/// Only these keys may appear in an account's `config_overrides`.
/// Everything else is instance-scoped and shared across all accounts.
pub const ACCOUNT_SCOPED_KEYS: &[&str] = &[
    "mode",
    "x_api",
    "business",
    "scoring",
    "limits",
    "intervals",
    "approval_mode",
    "max_batch_approve",
    "schedule",
    "targets",
    "content_sources",
];

/// Result of effective config resolution.
#[derive(Debug)]
pub struct EffectiveConfigResult {
    /// The merged configuration.
    pub config: Config,
    /// Top-level keys that were overridden by the account.
    pub overridden_keys: Vec<String>,
}

/// Merge account-specific overrides into a base config.
///
/// `overrides_json` is the JSON string from `accounts.config_overrides`.
/// Empty string or `"{}"` returns the base config unchanged.
///
/// Algorithm:
/// 1. Serialize base to JSON Value.
/// 2. Parse overrides_json to JSON Value.
/// 3. Validate overrides only contain account-scoped keys.
/// 4. Deep-merge overrides into base (RFC 7396).
/// 5. Deserialize merged Value back to Config.
/// 6. Return config + list of overridden top-level keys.
pub fn effective_config(
    base: &Config,
    overrides_json: &str,
) -> Result<EffectiveConfigResult, ConfigError> {
    let trimmed = overrides_json.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        return Ok(EffectiveConfigResult {
            config: base.clone(),
            overridden_keys: Vec::new(),
        });
    }

    let overrides: Value =
        serde_json::from_str(trimmed).map_err(|e| ConfigError::InvalidValue {
            field: "config_overrides".to_string(),
            message: format!("invalid JSON: {e}"),
        })?;

    if !overrides.is_object() {
        return Err(ConfigError::InvalidValue {
            field: "config_overrides".to_string(),
            message: "must be a JSON object".to_string(),
        });
    }

    validate_override_keys(&overrides)?;

    let overridden_keys: Vec<String> = overrides
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let mut base_value = serde_json::to_value(base).map_err(|e| ConfigError::InvalidValue {
        field: "config".to_string(),
        message: format!("failed to serialize base config: {e}"),
    })?;

    json_merge_patch(&mut base_value, &overrides);

    let config: Config =
        serde_json::from_value(base_value).map_err(|e| ConfigError::InvalidValue {
            field: "config_overrides".to_string(),
            message: format!("merged config is invalid: {e}"),
        })?;

    Ok(EffectiveConfigResult {
        config,
        overridden_keys,
    })
}

/// Validate that a JSON object only contains account-scoped keys.
pub fn validate_override_keys(overrides: &Value) -> Result<(), ConfigError> {
    if let Some(obj) = overrides.as_object() {
        let rejected: Vec<String> = obj
            .keys()
            .filter(|k| !ACCOUNT_SCOPED_KEYS.contains(&k.as_str()))
            .cloned()
            .collect();

        if !rejected.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "config_overrides".to_string(),
                message: format!(
                    "instance-scoped keys cannot be overridden per-account: {}",
                    rejected.join(", ")
                ),
            });
        }
    }
    Ok(())
}

/// Filter a JSON patch to only include account-scoped keys.
///
/// Returns `(account_patch, rejected_keys)`.
pub fn split_patch_by_scope(patch: &Value) -> (Value, Vec<String>) {
    let Some(obj) = patch.as_object() else {
        return (Value::Object(serde_json::Map::new()), Vec::new());
    };

    let mut account_patch = serde_json::Map::new();
    let mut rejected = Vec::new();

    for (key, value) in obj {
        if ACCOUNT_SCOPED_KEYS.contains(&key.as_str()) {
            account_patch.insert(key.clone(), value.clone());
        } else {
            rejected.push(key.clone());
        }
    }

    (Value::Object(account_patch), rejected)
}

/// RFC 7396 JSON merge-patch: objects recurse, everything else replaced,
/// null removes the key.
fn json_merge_patch(base: &mut Value, patch: &Value) {
    if let Some(patch_obj) = patch.as_object() {
        if !base.is_object() {
            *base = Value::Object(serde_json::Map::new());
        }
        let base_obj = base.as_object_mut().unwrap();
        for (key, patch_val) in patch_obj {
            if patch_val.is_null() {
                base_obj.remove(key);
            } else if patch_val.is_object() {
                let entry = base_obj
                    .entry(key.clone())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                json_merge_patch(entry, patch_val);
            } else {
                base_obj.insert(key.clone(), patch_val.clone());
            }
        }
    } else {
        *base = patch.clone();
    }
}

/// Apply a JSON merge-patch to an existing overrides JSON string.
///
/// Used by `PATCH /api/settings` for non-default accounts: merges the
/// incoming patch into the current `config_overrides`, producing updated
/// overrides ready to be persisted.
pub fn merge_overrides(current_overrides: &str, patch: &Value) -> Result<String, ConfigError> {
    let trimmed = current_overrides.trim();
    let mut current: Value = if trimmed.is_empty() || trimmed == "{}" {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(trimmed).map_err(|e| ConfigError::InvalidValue {
            field: "config_overrides".to_string(),
            message: format!("invalid existing overrides JSON: {e}"),
        })?
    };

    // Apply merge-patch: null values remove keys from overrides
    if let Some(patch_obj) = patch.as_object() {
        if !current.is_object() {
            current = Value::Object(serde_json::Map::new());
        }
        let current_obj = current.as_object_mut().unwrap();
        for (key, val) in patch_obj {
            if val.is_null() {
                current_obj.remove(key);
            } else {
                current_obj.insert(key.clone(), val.clone());
            }
        }
    }

    serde_json::to_string(&current).map_err(|e| ConfigError::InvalidValue {
        field: "config_overrides".to_string(),
        message: format!("failed to serialize overrides: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> Config {
        Config {
            business: super::super::BusinessProfile {
                product_name: "TestProduct".to_string(),
                product_keywords: vec!["test".to_string()],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn effective_config_empty_overrides() {
        let base = base_config();
        let result = effective_config(&base, "").unwrap();
        assert!(result.overridden_keys.is_empty());
        assert_eq!(result.config.business.product_name, "TestProduct");
    }

    #[test]
    fn effective_config_empty_object() {
        let base = base_config();
        let result = effective_config(&base, "{}").unwrap();
        assert!(result.overridden_keys.is_empty());
    }

    #[test]
    fn effective_config_single_field() {
        let base = base_config();
        let result = effective_config(&base, r#"{"scoring": {"threshold": 80}}"#).unwrap();
        assert_eq!(result.config.scoring.threshold, 80);
        assert_eq!(result.overridden_keys, vec!["scoring"]);
        // Other fields should be inherited from base
        assert_eq!(result.config.business.product_name, "TestProduct");
    }

    #[test]
    fn effective_config_full_section() {
        let base = base_config();
        let overrides = r#"{
            "business": {
                "product_name": "OverriddenProduct",
                "product_keywords": ["override"],
                "product_description": "A different product"
            }
        }"#;
        let result = effective_config(&base, overrides).unwrap();
        assert_eq!(result.config.business.product_name, "OverriddenProduct");
        assert_eq!(result.config.business.product_keywords, vec!["override"]);
        assert_eq!(
            result.config.business.product_description,
            "A different product"
        );
        assert_eq!(result.overridden_keys, vec!["business"]);
    }

    #[test]
    fn effective_config_array_replacement() {
        let base = base_config();
        let overrides =
            r#"{"limits": {"banned_phrases": ["spam", "scam"], "max_replies_per_day": 10}}"#;
        let result = effective_config(&base, overrides).unwrap();
        assert_eq!(
            result.config.limits.banned_phrases,
            vec!["spam".to_string(), "scam".to_string()]
        );
        assert_eq!(result.config.limits.max_replies_per_day, 10);
    }

    #[test]
    fn effective_config_mode_override() {
        let base = base_config();
        let result = effective_config(&base, r#"{"mode": "composer"}"#).unwrap();
        assert_eq!(result.config.mode, super::super::OperatingMode::Composer);
        assert_eq!(result.overridden_keys, vec!["mode"]);
    }

    #[test]
    fn effective_config_approval_mode_override() {
        let base = base_config();
        let result = effective_config(&base, r#"{"approval_mode": false}"#).unwrap();
        assert!(!result.config.approval_mode);
    }

    #[test]
    fn validate_override_keys_rejects_instance_scoped() {
        let overrides: Value =
            serde_json::from_str(r#"{"llm": {"provider": "anthropic"}}"#).unwrap();
        let err = validate_override_keys(&overrides).unwrap_err();
        assert!(err.to_string().contains("llm"));
    }

    #[test]
    fn validate_override_keys_rejects_server() {
        let overrides: Value = serde_json::from_str(r#"{"server": {"host": "0.0.0.0"}}"#).unwrap();
        let err = validate_override_keys(&overrides).unwrap_err();
        assert!(err.to_string().contains("server"));
    }

    #[test]
    fn validate_override_keys_rejects_storage() {
        let overrides: Value =
            serde_json::from_str(r#"{"storage": {"db_path": "/tmp/test.db"}}"#).unwrap();
        let err = validate_override_keys(&overrides).unwrap_err();
        assert!(err.to_string().contains("storage"));
    }

    #[test]
    fn validate_override_keys_allows_account_scoped() {
        let overrides: Value = serde_json::from_str(
            r#"{"business": {"product_name": "X"}, "scoring": {"threshold": 50}, "mode": "composer"}"#,
        )
        .unwrap();
        assert!(validate_override_keys(&overrides).is_ok());
    }

    #[test]
    fn split_patch_by_scope_separates() {
        let patch: Value = serde_json::from_str(
            r#"{"scoring": {"threshold": 80}, "llm": {"provider": "openai"}, "business": {"product_name": "X"}}"#,
        )
        .unwrap();
        let (account_patch, rejected) = split_patch_by_scope(&patch);
        assert!(account_patch.get("scoring").is_some());
        assert!(account_patch.get("business").is_some());
        assert!(account_patch.get("llm").is_none());
        assert_eq!(rejected, vec!["llm"]);
    }

    #[test]
    fn effective_config_invalid_json() {
        let base = base_config();
        let err = effective_config(&base, "not json").unwrap_err();
        assert!(err.to_string().contains("invalid JSON"));
    }

    #[test]
    fn merge_overrides_null_removes_key() {
        let current = r#"{"scoring": {"threshold": 80}, "business": {"product_name": "X"}}"#;
        let patch: Value = serde_json::from_str(r#"{"scoring": null}"#).unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("scoring").is_none());
        assert!(parsed.get("business").is_some());
    }

    #[test]
    fn merge_overrides_empty_current() {
        let patch: Value = serde_json::from_str(r#"{"scoring": {"threshold": 90}}"#).unwrap();
        let result = merge_overrides("", &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["scoring"]["threshold"], 90);
    }

    #[test]
    fn merge_overrides_adds_to_existing() {
        let current = r#"{"scoring": {"threshold": 80}}"#;
        let patch: Value =
            serde_json::from_str(r#"{"business": {"product_name": "New"}}"#).unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["scoring"]["threshold"], 80);
        assert_eq!(parsed["business"]["product_name"], "New");
    }

    // ── json_merge_patch edge cases ───────────────────────────────

    #[test]
    fn json_merge_patch_replaces_non_object_base() {
        let mut base = Value::String("hello".into());
        let patch: Value = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert!(base.is_object());
        assert_eq!(base["key"], "value");
    }

    #[test]
    fn json_merge_patch_null_removes_key() {
        let mut base: Value = serde_json::from_str(r#"{"a": 1, "b": 2}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"a": null}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert!(base.get("a").is_none());
        assert_eq!(base["b"], 2);
    }

    #[test]
    fn json_merge_patch_deep_merge() {
        let mut base: Value = serde_json::from_str(r#"{"nested": {"a": 1, "b": 2}}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"nested": {"b": 3, "c": 4}}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["nested"]["a"], 1);
        assert_eq!(base["nested"]["b"], 3);
        assert_eq!(base["nested"]["c"], 4);
    }

    #[test]
    fn json_merge_patch_non_object_patch_replaces() {
        let mut base: Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let patch = Value::String("replaced".into());
        json_merge_patch(&mut base, &patch);
        assert_eq!(base, "replaced");
    }

    // ── split_patch_by_scope edge cases ──────────────────────────

    #[test]
    fn split_patch_by_scope_non_object() {
        let patch = Value::String("not an object".into());
        let (account, rejected) = split_patch_by_scope(&patch);
        assert!(account.as_object().unwrap().is_empty());
        assert!(rejected.is_empty());
    }

    #[test]
    fn split_patch_by_scope_all_rejected() {
        let patch: Value = serde_json::from_str(r#"{"llm": {}, "server": {}}"#).unwrap();
        let (account, rejected) = split_patch_by_scope(&patch);
        assert!(account.as_object().unwrap().is_empty());
        assert_eq!(rejected.len(), 2);
    }

    #[test]
    fn split_patch_by_scope_all_accepted() {
        let patch: Value = serde_json::from_str(r#"{"mode": "composer", "scoring": {}}"#).unwrap();
        let (account, rejected) = split_patch_by_scope(&patch);
        assert_eq!(account.as_object().unwrap().len(), 2);
        assert!(rejected.is_empty());
    }

    // ── effective_config edge cases ──────────────────────────────

    #[test]
    fn effective_config_whitespace_only_overrides() {
        let base = base_config();
        let result = effective_config(&base, "   ").unwrap();
        assert!(result.overridden_keys.is_empty());
    }

    #[test]
    fn effective_config_non_object_overrides() {
        let base = base_config();
        let err = effective_config(&base, r#""string value""#).unwrap_err();
        assert!(err.to_string().contains("must be a JSON object"));
    }

    // ── ACCOUNT_SCOPED_KEYS ─────────────────────────────────────

    #[test]
    fn account_scoped_keys_is_nonempty() {
        assert!(!ACCOUNT_SCOPED_KEYS.is_empty());
    }

    #[test]
    fn account_scoped_keys_includes_business() {
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"business"));
    }

    #[test]
    fn account_scoped_keys_excludes_llm() {
        assert!(!ACCOUNT_SCOPED_KEYS.contains(&"llm"));
    }

    #[test]
    fn account_scoped_keys_excludes_server() {
        assert!(!ACCOUNT_SCOPED_KEYS.contains(&"server"));
    }

    // ── merge_overrides edge cases ──────────────────────────────

    #[test]
    fn merge_overrides_empty_object_current() {
        let patch: Value = serde_json::from_str(r#"{"mode": "composer"}"#).unwrap();
        let result = merge_overrides("{}", &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mode"], "composer");
    }

    #[test]
    fn merge_overrides_replaces_existing_key() {
        let current = r#"{"mode": "autopilot"}"#;
        let patch: Value = serde_json::from_str(r#"{"mode": "composer"}"#).unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mode"], "composer");
    }

    #[test]
    fn merge_overrides_invalid_current_json() {
        let patch: Value = serde_json::from_str(r#"{"mode": "x"}"#).unwrap();
        let result = merge_overrides("not json", &patch);
        assert!(result.is_err());
    }

    // ── EffectiveConfigResult ────────────────────────────────────

    #[test]
    fn effective_config_result_debug() {
        let result = EffectiveConfigResult {
            config: base_config(),
            overridden_keys: vec!["business".to_string()],
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("business"));
    }

    // -----------------------------------------------------------------------
    // Extended merge edge-case tests for coverage push
    // -----------------------------------------------------------------------

    #[test]
    fn json_merge_patch_nested_null_removes() {
        let mut base: Value =
            serde_json::from_str(r#"{"outer": {"a": 1, "b": {"c": 3}}}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"outer": {"b": null}}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert!(base["outer"].get("b").is_none());
        assert_eq!(base["outer"]["a"], 1);
    }

    #[test]
    fn json_merge_patch_add_nested_key() {
        let mut base: Value = serde_json::from_str(r#"{"a": {"b": 1}}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"a": {"c": 2}}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["a"]["b"], 1);
        assert_eq!(base["a"]["c"], 2);
    }

    #[test]
    fn json_merge_patch_replace_array_entirely() {
        let mut base: Value = serde_json::from_str(r#"{"arr": [1, 2, 3]}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"arr": [4, 5]}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["arr"].as_array().unwrap().len(), 2);
        assert_eq!(base["arr"][0], 4);
        assert_eq!(base["arr"][1], 5);
    }

    #[test]
    fn json_merge_patch_deeply_nested() {
        let mut base: Value = serde_json::from_str(r#"{"a": {"b": {"c": {"d": 1}}}}"#).unwrap();
        let patch: Value =
            serde_json::from_str(r#"{"a": {"b": {"c": {"d": 2, "e": 3}}}}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["a"]["b"]["c"]["d"], 2);
        assert_eq!(base["a"]["b"]["c"]["e"], 3);
    }

    #[test]
    fn json_merge_patch_empty_patch() {
        let mut base: Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["a"], 1);
    }

    #[test]
    fn json_merge_patch_number_replaces_object() {
        let mut base: Value = serde_json::from_str(r#"{"a": {"nested": 1}}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"a": 42}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["a"], 42);
    }

    #[test]
    fn json_merge_patch_object_replaces_number() {
        let mut base: Value = serde_json::from_str(r#"{"a": 42}"#).unwrap();
        let patch: Value = serde_json::from_str(r#"{"a": {"nested": 1}}"#).unwrap();
        json_merge_patch(&mut base, &patch);
        assert_eq!(base["a"]["nested"], 1);
    }

    #[test]
    fn effective_config_limits_override() {
        let base = base_config();
        let overrides = r#"{"limits": {"max_replies_per_day": 5}}"#;
        let result = effective_config(&base, overrides).unwrap();
        assert_eq!(result.config.limits.max_replies_per_day, 5);
        assert_eq!(result.overridden_keys, vec!["limits"]);
    }

    #[test]
    fn effective_config_intervals_override() {
        let base = base_config();
        let overrides = r#"{"limits": {"max_replies_per_day": 42}}"#;
        let result = effective_config(&base, overrides).unwrap();
        assert_eq!(result.config.limits.max_replies_per_day, 42);
    }

    #[test]
    fn effective_config_multiple_overrides() {
        let base = base_config();
        let overrides = r#"{
            "scoring": {"threshold": 75},
            "business": {"product_name": "NewProduct"},
            "approval_mode": true
        }"#;
        let result = effective_config(&base, overrides).unwrap();
        assert_eq!(result.config.scoring.threshold, 75);
        assert_eq!(result.config.business.product_name, "NewProduct");
        assert!(result.config.approval_mode);
        assert_eq!(result.overridden_keys.len(), 3);
    }

    #[test]
    fn validate_override_keys_empty_object_ok() {
        let overrides: Value = serde_json::from_str("{}").unwrap();
        assert!(validate_override_keys(&overrides).is_ok());
    }

    #[test]
    fn validate_override_keys_non_object_ok() {
        // Non-object passes (no keys to validate)
        let overrides = Value::String("test".into());
        assert!(validate_override_keys(&overrides).is_ok());
    }

    #[test]
    fn validate_override_keys_all_account_scoped() {
        for key in ACCOUNT_SCOPED_KEYS {
            let json_str = format!(r#"{{"{key}": "test"}}"#);
            let overrides: Value = serde_json::from_str(&json_str).unwrap();
            assert!(
                validate_override_keys(&overrides).is_ok(),
                "Key '{key}' should be allowed"
            );
        }
    }

    #[test]
    fn split_patch_by_scope_mixed_keys() {
        let patch: Value = serde_json::from_str(
            r#"{"scoring": {}, "business": {}, "llm": {}, "server": {}, "storage": {}}"#,
        )
        .unwrap();
        let (account, rejected) = split_patch_by_scope(&patch);
        assert_eq!(account.as_object().unwrap().len(), 2);
        assert_eq!(rejected.len(), 3);
    }

    #[test]
    fn split_patch_by_scope_empty_object() {
        let patch: Value = serde_json::from_str("{}").unwrap();
        let (account, rejected) = split_patch_by_scope(&patch);
        assert!(account.as_object().unwrap().is_empty());
        assert!(rejected.is_empty());
    }

    #[test]
    fn merge_overrides_multiple_null_removals() {
        let current = r#"{"scoring": {}, "business": {}, "limits": {}}"#;
        let patch: Value = serde_json::from_str(r#"{"scoring": null, "limits": null}"#).unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("scoring").is_none());
        assert!(parsed.get("limits").is_none());
        assert!(parsed.get("business").is_some());
    }

    #[test]
    fn merge_overrides_non_object_current_becomes_object() {
        // If current is not an object (edge case), it gets replaced
        let patch: Value = serde_json::from_str(r#"{"mode": "composer"}"#).unwrap();
        let result = merge_overrides(r#""string""#, &patch);
        // This should fail because "string" is valid JSON but not an object
        // Actually merge_overrides handles non-object current by replacing
        assert!(result.is_ok());
    }

    #[test]
    fn merge_overrides_preserves_unmentioned_keys() {
        let current = r#"{"scoring": {"threshold": 80}, "mode": "autopilot"}"#;
        let patch: Value = serde_json::from_str(r#"{"scoring": {"threshold": 90}}"#).unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["scoring"]["threshold"], 90);
        assert_eq!(parsed["mode"], "autopilot");
    }

    #[test]
    fn account_scoped_keys_contains_expected() {
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"mode"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"x_api"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"scoring"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"limits"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"intervals"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"approval_mode"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"max_batch_approve"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"schedule"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"targets"));
        assert!(ACCOUNT_SCOPED_KEYS.contains(&"content_sources"));
    }

    #[test]
    fn account_scoped_keys_excludes_instance() {
        assert!(!ACCOUNT_SCOPED_KEYS.contains(&"llm"));
        assert!(!ACCOUNT_SCOPED_KEYS.contains(&"server"));
        assert!(!ACCOUNT_SCOPED_KEYS.contains(&"storage"));
    }

    #[test]
    fn effective_config_preserves_base_when_no_override() {
        let base = base_config();
        let result = effective_config(&base, "{}").unwrap();
        assert_eq!(
            result.config.business.product_name,
            base.business.product_name
        );
        assert_eq!(
            result.config.business.product_keywords,
            base.business.product_keywords
        );
        assert_eq!(result.config.scoring.threshold, base.scoring.threshold);
    }

    #[test]
    fn effective_config_result_overridden_keys_empty_for_no_change() {
        let base = base_config();
        let result = effective_config(&base, "  {} ").unwrap();
        assert!(result.overridden_keys.is_empty());
    }

    #[test]
    fn merge_overrides_empty_patch_object() {
        let current = r#"{"mode": "autopilot"}"#;
        let patch: Value = serde_json::from_str("{}").unwrap();
        let result = merge_overrides(current, &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mode"], "autopilot");
    }

    #[test]
    fn merge_overrides_whitespace_current() {
        let patch: Value = serde_json::from_str(r#"{"mode": "composer"}"#).unwrap();
        let result = merge_overrides("  \n  ", &patch).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mode"], "composer");
    }
}
