use std::fs;
use std::path::Path;

use anyhow::Result;

use super::content_sources;
use super::group::UpgradeGroup;
use super::patch::patch_config;
use super::UpgradeAnswers;
use crate::output::CliOutput;

pub(crate) fn apply_defaults(
    config_path: &Path,
    missing: &[UpgradeGroup],
    out: CliOutput,
) -> Result<()> {
    let answers = UpgradeAnswers {
        persona: if missing.contains(&UpgradeGroup::Persona) {
            Some((vec![], vec![], vec![]))
        } else {
            None
        },
        targets: if missing.contains(&UpgradeGroup::Targets) {
            Some(vec![])
        } else {
            None
        },
        approval_mode: if missing.contains(&UpgradeGroup::ApprovalMode) {
            Some(false)
        } else {
            None
        },
        enhanced_limits: if missing.contains(&UpgradeGroup::EnhancedLimits) {
            Some((
                1,
                vec![
                    "check out".to_string(),
                    "you should try".to_string(),
                    "I recommend".to_string(),
                    "link in bio".to_string(),
                ],
                0.2,
            ))
        } else {
            None
        },
        deployment_mode: if missing.contains(&UpgradeGroup::DeploymentMode) {
            // Check env var first; default to "desktop".
            Some(std::env::var("TUITBOT_DEPLOYMENT_MODE").unwrap_or_else(|_| "desktop".to_string()))
        } else {
            None
        },
        connectors: if missing.contains(&UpgradeGroup::Connectors) {
            // Non-interactive: add empty scaffold (ready for env-var override).
            Some(Some(("".to_string(), "".to_string())))
        } else {
            None
        },
        content_sources_noticed: missing.contains(&UpgradeGroup::ContentSources),
    };

    patch_config(config_path, missing, &answers)?;

    out.info("Applied default values for new features:");
    for group in missing {
        out.info(&format!("  * {}", group.display_name()));
    }
    out.info(&format!("Backup saved to {}.bak", config_path.display()));

    // Print legacy SA-key notice (only in text mode).
    if !out.quiet && !out.is_json() {
        let content = fs::read_to_string(config_path).unwrap_or_default();
        content_sources::print_legacy_sa_key_notice(&content);
    }

    Ok(())
}
