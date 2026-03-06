use std::fs;
use std::path::Path;

use anyhow::Result;
use console::Style;

use super::content_sources;
use super::group::UpgradeGroup;
use super::patch::patch_config;
use super::UpgradeAnswers;
use crate::commands::init::{
    prompt_approval_mode, prompt_enhanced_limits, prompt_persona, prompt_target_accounts,
};

pub(crate) fn run_upgrade_wizard(config_path: &Path, missing: &[UpgradeGroup]) -> Result<()> {
    let bold = Style::new().bold();

    eprintln!();
    eprintln!("{}", bold.apply_to("Upgrade Wizard"));
    eprintln!();

    let mut answers = UpgradeAnswers {
        persona: None,
        targets: None,
        approval_mode: None,
        enhanced_limits: None,
        deployment_mode: None,
        connectors: None,
        content_sources_noticed: false,
    };

    for group in missing {
        eprintln!("{}", bold.apply_to(group.display_name()));
        eprintln!("  {}", group.description());
        eprintln!();

        match group {
            UpgradeGroup::Persona => {
                answers.persona = Some(prompt_persona()?);
            }
            UpgradeGroup::Targets => {
                answers.targets = Some(prompt_target_accounts()?);
            }
            UpgradeGroup::ApprovalMode => {
                answers.approval_mode = Some(prompt_approval_mode()?);
            }
            UpgradeGroup::EnhancedLimits => {
                answers.enhanced_limits = Some(prompt_enhanced_limits()?);
            }
            UpgradeGroup::DeploymentMode => {
                answers.deployment_mode = Some(content_sources::prompt_deployment_mode()?);
            }
            UpgradeGroup::Connectors => {
                answers.connectors = Some(content_sources::prompt_connectors()?);
            }
            UpgradeGroup::ContentSources => {
                let dim = Style::new().dim();
                eprintln!(
                    "{}",
                    dim.apply_to(
                        "  Configure content sources in the dashboard:\n  \
                         Settings > Content Sources"
                    )
                );
                eprintln!();
                answers.content_sources_noticed = true;
            }
        }
    }

    patch_config(config_path, missing, &answers)?;

    // Print legacy SA-key notice after patching.
    let content = fs::read_to_string(config_path).unwrap_or_default();
    content_sources::print_legacy_sa_key_notice(&content);

    eprintln!("{}", bold.apply_to("Config updated successfully!"));
    eprintln!("  Backup saved to {}.bak", config_path.display());
    eprintln!();

    Ok(())
}
