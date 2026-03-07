/// Feature groups that may be missing from older config files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeGroup {
    /// business.persona_opinions, .persona_experiences, .content_pillars
    Persona,
    /// [targets] section
    Targets,
    /// approval_mode (top-level)
    ApprovalMode,
    /// limits.max_replies_per_author_per_day, .banned_phrases, .product_mention_ratio
    EnhancedLimits,
    /// deployment_mode (top-level)
    DeploymentMode,
    /// [connectors.google_drive] section
    Connectors,
    /// [content_sources] section
    ContentSources,
}

impl UpgradeGroup {
    /// All upgrade groups in recommended configuration order.
    pub(crate) fn all() -> &'static [UpgradeGroup] {
        &[
            UpgradeGroup::Persona,
            UpgradeGroup::Targets,
            UpgradeGroup::ApprovalMode,
            UpgradeGroup::EnhancedLimits,
            UpgradeGroup::DeploymentMode,
            UpgradeGroup::Connectors,
            UpgradeGroup::ContentSources,
        ]
    }

    /// TOML key paths that belong to this group.
    pub(crate) fn key_paths(&self) -> &[&str] {
        match self {
            UpgradeGroup::Persona => &[
                "business.persona_opinions",
                "business.persona_experiences",
                "business.content_pillars",
            ],
            UpgradeGroup::Targets => &["targets"],
            UpgradeGroup::ApprovalMode => &["approval_mode"],
            UpgradeGroup::EnhancedLimits => &[
                "limits.max_replies_per_author_per_day",
                "limits.banned_phrases",
                "limits.product_mention_ratio",
            ],
            UpgradeGroup::DeploymentMode => &["deployment_mode"],
            UpgradeGroup::Connectors => &["connectors.google_drive.client_id"],
            UpgradeGroup::ContentSources => &["content_sources"],
        }
    }

    /// Human-readable name for display.
    pub(crate) fn display_name(&self) -> &str {
        match self {
            UpgradeGroup::Persona => "Persona",
            UpgradeGroup::Targets => "Target Accounts",
            UpgradeGroup::ApprovalMode => "Approval Mode",
            UpgradeGroup::EnhancedLimits => "Enhanced Safety Limits",
            UpgradeGroup::DeploymentMode => "Deployment Mode",
            UpgradeGroup::Connectors => "Google Drive Connector",
            UpgradeGroup::ContentSources => "Content Sources",
        }
    }

    /// One-line description of the feature.
    pub(crate) fn description(&self) -> &str {
        match self {
            UpgradeGroup::Persona => {
                "Strong opinions, experiences, and content pillars for authentic content"
            }
            UpgradeGroup::Targets => "Monitor specific accounts and reply to their conversations",
            UpgradeGroup::ApprovalMode => "Queue posts for human review before posting",
            UpgradeGroup::EnhancedLimits => {
                "Per-author reply limits, banned phrases, and product mention ratio"
            }
            UpgradeGroup::DeploymentMode => {
                "Declare how Tuitbot runs (desktop, self-hosted, or cloud)"
            }
            UpgradeGroup::Connectors => "OAuth credentials for Google Drive integration",
            UpgradeGroup::ContentSources => {
                "Configure content ingestion from local folders or Google Drive"
            }
        }
    }
}
