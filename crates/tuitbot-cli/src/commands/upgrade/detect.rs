use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::group::UpgradeGroup;

/// Detect which feature groups are missing from the config file.
pub fn detect_missing_features(config_path: &Path) -> Result<Vec<UpgradeGroup>> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    detect_missing_features_from_str(&content)
}

/// Detect missing features from a TOML string (testable without filesystem).
pub(crate) fn detect_missing_features_from_str(content: &str) -> Result<Vec<UpgradeGroup>> {
    let table: toml::Value = content.parse().context("Failed to parse config as TOML")?;
    let root = table
        .as_table()
        .context("Config root is not a TOML table")?;

    let mut missing = Vec::new();

    for group in UpgradeGroup::all() {
        // A group is missing if ANY of its key paths are absent
        let any_missing = group.key_paths().iter().any(|p| !key_exists(root, p));
        if any_missing {
            missing.push(*group);
        }
    }

    Ok(missing)
}

/// Walk a dot-separated key path in a TOML table.
///
/// Public within the crate for test access via `key_exists_public`.
fn key_exists(table: &toml::value::Table, dotted_path: &str) -> bool {
    let segments: Vec<&str> = dotted_path.split('.').collect();
    let mut current: &toml::Value = &toml::Value::Table(table.clone());

    for segment in &segments {
        match current.as_table() {
            Some(t) => match t.get(*segment) {
                Some(v) => current = v,
                None => return false,
            },
            None => return false,
        }
    }

    true
}

/// Test-visible wrapper for `key_exists`.
#[cfg(test)]
pub(crate) fn key_exists_public(table: &toml::value::Table, dotted_path: &str) -> bool {
    key_exists(table, dotted_path)
}
