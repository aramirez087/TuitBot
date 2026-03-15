use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use toml_edit::{value, Array, DocumentMut};

use super::content_sources;
use super::group::UpgradeGroup;
use super::UpgradeAnswers;

pub(crate) fn patch_config(
    config_path: &Path,
    missing: &[UpgradeGroup],
    answers: &UpgradeAnswers,
) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    // Backup before writing
    let backup_path = config_path.with_extension("toml.bak");
    fs::write(&backup_path, &content)
        .with_context(|| format!("Failed to write backup to {}", backup_path.display()))?;

    let mut doc: DocumentMut = content
        .parse()
        .context("Failed to parse config for editing")?;

    for group in missing {
        match group {
            UpgradeGroup::Persona => {
                if let Some((opinions, experiences, pillars)) = &answers.persona {
                    patch_persona(&mut doc, opinions, experiences, pillars);
                }
            }
            UpgradeGroup::Targets => {
                if let Some(accounts) = &answers.targets {
                    patch_targets(&mut doc, accounts);
                }
            }
            UpgradeGroup::ApprovalMode => {
                if let Some(approval_mode) = answers.approval_mode {
                    patch_approval_mode(&mut doc, approval_mode);
                }
            }
            UpgradeGroup::EnhancedLimits => {
                if let Some((max_replies, banned, ratio)) = &answers.enhanced_limits {
                    patch_enhanced_limits(&mut doc, *max_replies, banned, *ratio);
                }
            }
            UpgradeGroup::DeploymentMode => {
                if let Some(mode) = &answers.deployment_mode {
                    content_sources::patch_deployment_mode(&mut doc, mode);
                }
            }
            UpgradeGroup::Connectors => {
                if let Some(Some((client_id, client_secret))) = &answers.connectors {
                    content_sources::patch_connectors(&mut doc, client_id, client_secret);
                }
            }
            UpgradeGroup::ContentSources => {
                if answers.content_sources_noticed {
                    content_sources::patch_content_sources(&mut doc);
                }
            }
        }
    }

    fs::write(config_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    Ok(())
}

pub(crate) fn to_toml_array(items: &[String]) -> Array {
    let mut arr = Array::new();
    for item in items {
        arr.push(item.as_str());
    }
    arr
}

fn patch_persona(
    doc: &mut DocumentMut,
    opinions: &[String],
    experiences: &[String],
    pillars: &[String],
) {
    // Ensure [business] table exists
    if doc.get("business").is_none() {
        doc["business"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    let business = doc["business"].as_table_mut().unwrap();

    if !business.contains_key("persona_opinions") {
        business.insert("persona_opinions", value(to_toml_array(opinions)));
        if let Some(mut key) = business.key_mut("persona_opinions") {
            key.leaf_decor_mut().set_prefix(
                "\n# Persona -- strong opinions, experiences, and pillars make content more authentic.\n",
            );
        }
    }

    if !business.contains_key("persona_experiences") {
        business.insert("persona_experiences", value(to_toml_array(experiences)));
    }

    if !business.contains_key("content_pillars") {
        business.insert("content_pillars", value(to_toml_array(pillars)));
    }
}

fn patch_targets(doc: &mut DocumentMut, accounts: &[String]) {
    if doc.get("targets").is_some() {
        return;
    }

    let mut table = toml_edit::Table::new();
    table.insert("accounts", value(to_toml_array(accounts)));
    table.insert("max_target_replies_per_day", value(3i64));

    table.decor_mut().set_prefix(
        "\n# --- Target Accounts ---\n# Monitor specific accounts and reply to their conversations.\n",
    );

    doc.insert("targets", toml_edit::Item::Table(table));
}

fn patch_approval_mode(doc: &mut DocumentMut, approval_mode: bool) {
    if doc.get("approval_mode").is_some() {
        return;
    }

    doc.insert("approval_mode", value(approval_mode));

    if let Some(mut key) = doc.key_mut("approval_mode") {
        key.leaf_decor_mut().set_prefix(
            "# Queue posts for review before posting (use `tuitbot approve` to review).\n",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_toml_array_empty() {
        let arr = to_toml_array(&[]);
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn to_toml_array_single_item() {
        let items = vec!["hello".to_string()];
        let arr = to_toml_array(&items);
        assert_eq!(arr.len(), 1);
        assert_eq!(arr.get(0).unwrap().as_str().unwrap(), "hello");
    }

    #[test]
    fn to_toml_array_multiple_items() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let arr = to_toml_array(&items);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get(0).unwrap().as_str().unwrap(), "a");
        assert_eq!(arr.get(1).unwrap().as_str().unwrap(), "b");
        assert_eq!(arr.get(2).unwrap().as_str().unwrap(), "c");
    }

    #[test]
    fn to_toml_array_preserves_content() {
        let items = vec!["rust programming".to_string(), "CLI tools".to_string()];
        let arr = to_toml_array(&items);
        assert_eq!(arr.get(0).unwrap().as_str().unwrap(), "rust programming");
        assert_eq!(arr.get(1).unwrap().as_str().unwrap(), "CLI tools");
    }

    fn empty_answers() -> UpgradeAnswers {
        UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        }
    }

    #[test]
    fn patch_config_creates_backup() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let _ = patch_config(&config_path, &[], &empty_answers());

        let backup_path = config_path.with_extension("toml.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn patch_config_empty_missing_is_noop() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let original = "[business]\nproduct_name = \"test\"\n";
        fs::write(&config_path, original).unwrap();

        patch_config(&config_path, &[], &empty_answers()).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn patch_config_adds_approval_mode() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.approval_mode = Some(true);

        patch_config(&config_path, &[UpgradeGroup::ApprovalMode], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("approval_mode = true"));
    }

    #[test]
    fn patch_config_adds_targets() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.targets = Some(vec!["user1".to_string(), "user2".to_string()]);

        patch_config(&config_path, &[UpgradeGroup::Targets], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("[targets]"));
        assert!(result.contains("user1"));
        assert!(result.contains("user2"));
    }

    #[test]
    fn patch_config_adds_persona() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.persona = Some((
            vec!["strong opinion".to_string()],
            vec!["built something".to_string()],
            vec!["dev tools".to_string()],
        ));

        patch_config(&config_path, &[UpgradeGroup::Persona], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("persona_opinions"));
        assert!(result.contains("persona_experiences"));
        assert!(result.contains("content_pillars"));
    }

    #[test]
    fn patch_config_adds_enhanced_limits() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[limits]\nmax_replies_per_day = 5\n").unwrap();

        let mut answers = empty_answers();
        answers.enhanced_limits = Some((2, vec!["check out".to_string()], 0.2));

        patch_config(&config_path, &[UpgradeGroup::EnhancedLimits], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("max_replies_per_author_per_day"));
        assert!(result.contains("banned_phrases"));
        assert!(result.contains("product_mention_ratio"));
    }

    #[test]
    fn patch_config_adds_deployment_mode() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.deployment_mode = Some("self_host".to_string());

        patch_config(&config_path, &[UpgradeGroup::DeploymentMode], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("deployment_mode = \"self_host\""));
    }

    #[test]
    fn patch_config_adds_connectors() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.connectors = Some(Some(("cid".to_string(), "csecret".to_string())));

        patch_config(&config_path, &[UpgradeGroup::Connectors], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("[connectors"));
        assert!(result.contains("client_id"));
    }

    #[test]
    fn patch_config_adds_content_sources() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.content_sources_noticed = true;

        patch_config(&config_path, &[UpgradeGroup::ContentSources], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("[content_sources]"));
    }

    #[test]
    fn patch_config_skips_connectors_when_none() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.connectors = Some(None); // User skipped

        patch_config(&config_path, &[UpgradeGroup::Connectors], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(!result.contains("[connectors"));
    }

    #[test]
    fn patch_config_multiple_groups() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[business]\nproduct_name = \"test\"\n").unwrap();

        let mut answers = empty_answers();
        answers.approval_mode = Some(false);
        answers.deployment_mode = Some("desktop".to_string());

        patch_config(
            &config_path,
            &[UpgradeGroup::ApprovalMode, UpgradeGroup::DeploymentMode],
            &answers,
        )
        .unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(result.contains("approval_mode = false"));
        assert!(result.contains("deployment_mode = \"desktop\""));
    }

    #[test]
    fn patch_config_preserves_existing_persona() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(
            &config_path,
            "[business]\nproduct_name = \"test\"\npersona_opinions = [\"existing\"]\n",
        )
        .unwrap();

        let mut answers = empty_answers();
        answers.persona = Some((
            vec!["new opinion".to_string()],
            vec!["new exp".to_string()],
            vec!["new pillar".to_string()],
        ));

        patch_config(&config_path, &[UpgradeGroup::Persona], &answers).unwrap();

        let result = fs::read_to_string(&config_path).unwrap();
        assert!(
            result.contains("existing"),
            "should preserve existing opinions"
        );
    }
}

fn patch_enhanced_limits(doc: &mut DocumentMut, max_replies: u32, banned: &[String], ratio: f32) {
    // Ensure [limits] table exists
    if doc.get("limits").is_none() {
        doc["limits"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    let limits = doc["limits"].as_table_mut().unwrap();

    if !limits.contains_key("max_replies_per_author_per_day") {
        limits.insert(
            "max_replies_per_author_per_day",
            value(i64::from(max_replies)),
        );
        if let Some(mut key) = limits.key_mut("max_replies_per_author_per_day") {
            key.leaf_decor_mut()
                .set_prefix("\n# Enhanced safety limits\n");
        }
    }

    if !limits.contains_key("banned_phrases") {
        limits.insert("banned_phrases", value(to_toml_array(banned)));
    }

    if !limits.contains_key("product_mention_ratio") {
        limits.insert("product_mention_ratio", value(f64::from(ratio)));
    }
}
