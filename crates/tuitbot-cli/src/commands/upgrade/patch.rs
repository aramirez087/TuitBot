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
