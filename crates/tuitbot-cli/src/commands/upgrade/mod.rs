/// `tuitbot upgrade` -- detect and configure new features in an existing config.
///
/// Parses the raw TOML file to find missing feature groups, then offers an
/// interactive mini-wizard to configure only the missing features. Uses
/// `toml_edit` to patch the file in-place, preserving user comments and
/// formatting.
mod content_sources;

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use console::Style;
use toml_edit::{value, Array, DocumentMut};

use super::init::{
    prompt_approval_mode, prompt_enhanced_limits, prompt_persona, prompt_target_accounts,
};

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
    fn all() -> &'static [UpgradeGroup] {
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
    fn key_paths(&self) -> &[&str] {
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

/// Collected answers from the upgrade wizard.
struct UpgradeAnswers {
    persona: Option<(Vec<String>, Vec<String>, Vec<String>)>,
    targets: Option<Vec<String>>,
    approval_mode: Option<bool>,
    enhanced_limits: Option<(u32, Vec<String>, f32)>,
    deployment_mode: Option<String>,
    connectors: Option<Option<(String, String)>>,
    content_sources_noticed: bool,
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Detect which feature groups are missing from the config file.
pub fn detect_missing_features(config_path: &Path) -> Result<Vec<UpgradeGroup>> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    detect_missing_features_from_str(&content)
}

/// Detect missing features from a TOML string (testable without filesystem).
fn detect_missing_features_from_str(content: &str) -> Result<Vec<UpgradeGroup>> {
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

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

/// Run the upgrade command explicitly.
///
/// **Deprecated:** Use `tuitbot update --config-only` instead. This command
/// will be removed in a future release.
pub async fn execute(non_interactive: bool, config_path_str: &str) -> Result<()> {
    let dim = Style::new().dim();
    eprintln!(
        "{}",
        Style::new()
            .yellow()
            .bold()
            .apply_to("Note: 'tuitbot upgrade' is deprecated. Use 'tuitbot update' instead.")
    );
    eprintln!(
        "{}",
        dim.apply_to("  'tuitbot update' also checks for new binary releases.")
    );
    eprintln!();

    let config_path = expand_tilde(config_path_str);

    if !config_path.exists() {
        bail!(
            "Config file not found: {}\nRun 'tuitbot init' first.",
            config_path.display()
        );
    }

    let missing = detect_missing_features(&config_path)?;
    if missing.is_empty() {
        // Still check for legacy SA-key notice even when all groups present.
        let content = fs::read_to_string(&config_path)?;
        content_sources::print_legacy_sa_key_notice(&content);

        eprintln!("Config is up to date -- no new features to configure.");
        return Ok(());
    }

    if non_interactive {
        return apply_defaults(&config_path, &missing);
    }

    if !std::io::stdin().is_terminal() {
        bail!(
            "Interactive upgrade requires a terminal.\n\
             Use --non-interactive to apply default values for new features."
        );
    }

    run_upgrade_wizard(&config_path, &missing)
}

/// Expand `~` at the start of a path to the user's home directory.
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

// ---------------------------------------------------------------------------
// Interactive wizard
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Non-interactive defaults
// ---------------------------------------------------------------------------

pub(crate) fn apply_defaults(config_path: &Path, missing: &[UpgradeGroup]) -> Result<()> {
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

    eprintln!("Applied default values for new features:");
    for group in missing {
        eprintln!("  * {}", group.display_name());
    }
    eprintln!("Backup saved to {}.bak", config_path.display());

    // Print legacy SA-key notice.
    let content = fs::read_to_string(config_path).unwrap_or_default();
    content_sources::print_legacy_sa_key_notice(&content);

    Ok(())
}

// ---------------------------------------------------------------------------
// TOML patching
// ---------------------------------------------------------------------------

fn patch_config(
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

fn to_toml_array(items: &[String]) -> Array {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_CONFIG: &str = r#"
# =============================================================================
# Tuitbot Configuration -- Docklet (@getdocklet)
# =============================================================================

# --- X API Credentials ---
[x_api]
client_id = "YOUR_CLIENT_ID"

# --- Authentication Settings ---
[auth]
mode = "local_callback"
callback_host = "127.0.0.1"
callback_port = 8080

# --- Business Profile ---
[business]
product_name = "Docklet"
product_description = "A floating command strip for macOS"
product_url = "https://getdocklet.app"
target_audience = "Mac power users"
product_keywords = ["macos productivity", "mac menu bar"]
competitor_keywords = ["notchnook", "bartender mac"]
industry_topics = ["Mac productivity tips"]
brand_voice = "Confident but not cocky."
reply_style = "Lead with genuine value."
content_style = "Share genuinely useful Mac tips."

# --- Scoring Engine ---
[scoring]
threshold = 65
keyword_relevance_max = 40.0
follower_count_max = 15.0
recency_max = 20.0
engagement_rate_max = 25.0

# --- Safety Limits ---
[limits]
max_replies_per_day = 15
max_tweets_per_day = 3
max_threads_per_week = 1
min_action_delay_seconds = 45
max_action_delay_seconds = 180

# --- Automation Intervals ---
[intervals]
mentions_check_seconds = 300
discovery_search_seconds = 900
content_post_window_seconds = 18000
thread_interval_seconds = 604800

# --- LLM Provider ---
[llm]
provider = "anthropic"
api_key = "YOUR_KEY"
model = "claude-sonnet-4-6"

# --- Data Storage ---
[storage]
db_path = "~/.tuitbot/tuitbot.db"
retention_days = 90

# --- Logging ---
[logging]
status_interval_seconds = 3600
"#;

    #[test]
    fn detect_missing_from_old_config() {
        let missing = detect_missing_features_from_str(OLD_CONFIG).unwrap();
        assert!(
            missing.contains(&UpgradeGroup::Persona),
            "should detect missing persona"
        );
        assert!(
            missing.contains(&UpgradeGroup::Targets),
            "should detect missing targets"
        );
        assert!(
            missing.contains(&UpgradeGroup::ApprovalMode),
            "should detect missing approval_mode"
        );
        assert!(
            missing.contains(&UpgradeGroup::EnhancedLimits),
            "should detect missing enhanced limits"
        );
        assert!(
            missing.contains(&UpgradeGroup::DeploymentMode),
            "should detect missing deployment_mode"
        );
        assert!(
            missing.contains(&UpgradeGroup::Connectors),
            "should detect missing connectors"
        );
        assert!(
            missing.contains(&UpgradeGroup::ContentSources),
            "should detect missing content_sources"
        );
        assert_eq!(missing.len(), 7);
    }

    #[test]
    fn detect_nothing_missing_from_full_config() {
        let full = r#"
approval_mode = false
deployment_mode = "desktop"

[x_api]
client_id = "cid"

[business]
product_name = "Test"
product_keywords = ["test"]
industry_topics = ["topic"]
persona_opinions = []
persona_experiences = []
content_pillars = []

[scoring]
threshold = 60

[limits]
max_replies_per_day = 5
max_tweets_per_day = 6
max_threads_per_week = 1
min_action_delay_seconds = 45
max_action_delay_seconds = 180
max_replies_per_author_per_day = 1
banned_phrases = ["check out"]
product_mention_ratio = 0.2

[intervals]
mentions_check_seconds = 300

[targets]
accounts = []

[llm]
provider = "ollama"
model = "llama3.2"

[storage]
db_path = "~/.tuitbot/tuitbot.db"

[logging]
status_interval_seconds = 0

[connectors.google_drive]
client_id = "test.apps.googleusercontent.com"

[content_sources]
"#;
        let missing = detect_missing_features_from_str(full).unwrap();
        assert!(
            missing.is_empty(),
            "full config should have no missing groups, got: {:?}",
            missing
        );
    }

    #[test]
    fn key_exists_helper() {
        let toml_str = r#"
[business]
product_name = "Test"

[limits]
max_replies_per_day = 5
"#;
        let table: toml::Value = toml_str.parse().unwrap();
        let root = table.as_table().unwrap();

        assert!(key_exists(root, "business"));
        assert!(key_exists(root, "business.product_name"));
        assert!(!key_exists(root, "business.persona_opinions"));
        assert!(key_exists(root, "limits"));
        assert!(key_exists(root, "limits.max_replies_per_day"));
        assert!(!key_exists(root, "limits.banned_phrases"));
        assert!(!key_exists(root, "targets"));
        assert!(!key_exists(root, "approval_mode"));
    }

    #[test]
    fn patch_config_preserves_comments() {
        let config_str = r#"# My custom header comment

# --- X API Credentials ---
[x_api]
client_id = "test-id"

# --- Business Profile ---
# This is my business section comment
[business]
product_name = "TestApp"

# --- Limits ---
[limits]
max_replies_per_day = 10
"#;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: Some((
                vec!["Rust is great".to_string()],
                vec!["Built 3 apps".to_string()],
                vec!["Dev tools".to_string()],
            )),
            targets: Some(vec!["elonmusk".to_string()]),
            approval_mode: Some(true),
            enhanced_limits: Some((
                2,
                vec!["check out".to_string(), "link in bio".to_string()],
                0.3,
            )),
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        };

        let groups = vec![
            UpgradeGroup::Persona,
            UpgradeGroup::Targets,
            UpgradeGroup::ApprovalMode,
            UpgradeGroup::EnhancedLimits,
        ];

        patch_config(tmp.path(), &groups, &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();

        // Original comments preserved
        assert!(
            result.contains("My custom header comment"),
            "header comment should be preserved"
        );
        assert!(
            result.contains("This is my business section comment"),
            "business section comment should be preserved"
        );

        // New keys are present and parseable
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        assert_eq!(config.business.persona_opinions, vec!["Rust is great"]);
        assert_eq!(config.business.persona_experiences, vec!["Built 3 apps"]);
        assert_eq!(config.business.content_pillars, vec!["Dev tools"]);
        assert_eq!(config.targets.accounts, vec!["elonmusk"]);
        assert!(config.approval_mode);
        assert_eq!(config.limits.max_replies_per_author_per_day, 2);
        assert_eq!(
            config.limits.banned_phrases,
            vec!["check out", "link in bio"]
        );
        assert!((config.limits.product_mention_ratio - 0.3).abs() < f32::EPSILON);

        // Original values preserved
        assert_eq!(config.x_api.client_id, "test-id");
        assert_eq!(config.business.product_name, "TestApp");
        assert_eq!(config.limits.max_replies_per_day, 10);

        // Backup was created
        let backup = tmp.path().with_extension("toml.bak");
        assert!(backup.exists(), "backup file should exist");
    }

    #[test]
    fn patch_config_persona_into_business() {
        let config_str = r#"
[business]
product_name = "App"
product_keywords = ["test"]
industry_topics = ["topic"]
"#;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: Some((
                vec!["opinion1".to_string()],
                vec!["experience1".to_string()],
                vec!["pillar1".to_string()],
            )),
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::Persona], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        assert_eq!(config.business.persona_opinions, vec!["opinion1"]);
        assert_eq!(config.business.persona_experiences, vec!["experience1"]);
        assert_eq!(config.business.content_pillars, vec!["pillar1"]);
        // Original values preserved
        assert_eq!(config.business.product_name, "App");
    }

    #[test]
    fn patch_config_targets_new_section() {
        let config_str = r#"
[x_api]
client_id = "test"

[business]
product_name = "App"
"#;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: Some(vec!["levelsio".to_string(), "naval".to_string()]),
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::Targets], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        assert_eq!(config.targets.accounts, vec!["levelsio", "naval"]);
        assert_eq!(config.targets.max_target_replies_per_day, 3);
    }

    #[test]
    fn patch_config_approval_mode_top_level() {
        let config_str = r#"
[x_api]
client_id = "test"
"#;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: Some(true),
            enhanced_limits: None,
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::ApprovalMode], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        assert!(config.approval_mode);
    }

    #[test]
    fn patch_config_partial_limits_already_present() {
        // Config that already has max_replies_per_author_per_day but missing the others
        let config_str = r#"
[limits]
max_replies_per_day = 10
max_replies_per_author_per_day = 2
"#;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: Some((1, vec!["check out".to_string()], 0.15)),
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::EnhancedLimits], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        // Existing value preserved (not overwritten)
        assert_eq!(config.limits.max_replies_per_author_per_day, 2);
        // New values inserted
        assert_eq!(config.limits.banned_phrases, vec!["check out"]);
        assert!((config.limits.product_mention_ratio - 0.15).abs() < f32::EPSILON);
        // Original value preserved
        assert_eq!(config.limits.max_replies_per_day, 10);
    }

    // --- Session 06: New group tests ---

    #[test]
    fn detect_missing_deployment_mode() {
        let config = r#"
approval_mode = false
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[connectors.google_drive]
client_id = "test"
[content_sources]
"#;
        let missing = detect_missing_features_from_str(config).unwrap();
        assert!(
            missing.contains(&UpgradeGroup::DeploymentMode),
            "should detect missing deployment_mode"
        );
        assert!(
            !missing.contains(&UpgradeGroup::Connectors),
            "connectors should not be missing"
        );
        assert!(
            !missing.contains(&UpgradeGroup::ContentSources),
            "content_sources should not be missing"
        );
    }

    #[test]
    fn detect_missing_connectors() {
        let config = r#"
approval_mode = false
deployment_mode = "desktop"
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[content_sources]
"#;
        let missing = detect_missing_features_from_str(config).unwrap();
        assert!(
            missing.contains(&UpgradeGroup::Connectors),
            "should detect missing connectors"
        );
        assert!(
            !missing.contains(&UpgradeGroup::DeploymentMode),
            "deployment_mode should not be missing"
        );
    }

    #[test]
    fn detect_missing_content_sources() {
        let config = r#"
approval_mode = false
deployment_mode = "self_host"
[business]
product_name = "Test"
persona_opinions = []
persona_experiences = []
content_pillars = []
product_keywords = ["test"]
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
[connectors.google_drive]
client_id = "test"
"#;
        let missing = detect_missing_features_from_str(config).unwrap();
        assert!(
            missing.contains(&UpgradeGroup::ContentSources),
            "should detect missing content_sources"
        );
    }

    #[test]
    fn patch_deployment_mode_top_level() {
        let config_str = r#"
[x_api]
client_id = "test"
"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: Some("self_host".to_string()),
            connectors: None,
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::DeploymentMode], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        assert!(result.contains("deployment_mode = \"self_host\""));

        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");
        assert_eq!(
            config.deployment_mode,
            tuitbot_core::config::DeploymentMode::SelfHost
        );
    }

    #[test]
    fn patch_connectors_section() {
        let config_str = r#"
[x_api]
client_id = "test"
"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: None,
            connectors: Some(Some((
                "my-client.apps.googleusercontent.com".to_string(),
                "GOCSPX-secret".to_string(),
            ))),
            content_sources_noticed: false,
        };

        patch_config(tmp.path(), &[UpgradeGroup::Connectors], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        assert!(result.contains("[connectors.google_drive]"));
        assert!(result.contains("my-client.apps.googleusercontent.com"));

        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");
        assert_eq!(
            config.connectors.google_drive.client_id.as_deref(),
            Some("my-client.apps.googleusercontent.com")
        );
        assert_eq!(
            config.connectors.google_drive.client_secret.as_deref(),
            Some("GOCSPX-secret")
        );
    }

    #[test]
    fn patch_content_sources_scaffold() {
        let config_str = r#"
[x_api]
client_id = "test"
"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: None,
            connectors: None,
            content_sources_noticed: true,
        };

        patch_config(tmp.path(), &[UpgradeGroup::ContentSources], &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        assert!(
            result.contains("[content_sources]"),
            "should contain content_sources section"
        );

        // Must be parseable as valid config.
        let _config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");
    }

    #[test]
    fn detect_legacy_sa_key_notice() {
        let config_with_legacy = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc"
service_account_key = "/keys/sa.json"
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
        assert!(
            content_sources::has_legacy_sa_key(config_with_legacy),
            "should detect legacy SA key"
        );

        let config_with_connection = r#"
[[content_sources.sources]]
source_type = "google_drive"
folder_id = "abc"
service_account_key = "/keys/sa.json"
connection_id = 1
watch = true
file_patterns = ["*.md"]
loop_back_enabled = false
"#;
        assert!(
            !content_sources::has_legacy_sa_key(config_with_connection),
            "should not flag when connection_id is present"
        );

        let config_no_sources = r#"
[business]
product_name = "Test"
"#;
        assert!(
            !content_sources::has_legacy_sa_key(config_no_sources),
            "should not flag when no sources"
        );
    }

    #[test]
    fn patch_all_new_groups_together() {
        let config_str = r#"
approval_mode = true
[x_api]
client_id = "test"
[business]
product_name = "App"
product_keywords = ["test"]
persona_opinions = []
persona_experiences = []
content_pillars = []
[limits]
max_replies_per_author_per_day = 1
banned_phrases = []
product_mention_ratio = 0.2
[targets]
accounts = []
"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        fs::write(tmp.path(), config_str).unwrap();

        let answers = UpgradeAnswers {
            persona: None,
            targets: None,
            approval_mode: None,
            enhanced_limits: None,
            deployment_mode: Some("self_host".to_string()),
            connectors: Some(Some(("cid".to_string(), "csecret".to_string()))),
            content_sources_noticed: true,
        };

        let groups = vec![
            UpgradeGroup::DeploymentMode,
            UpgradeGroup::Connectors,
            UpgradeGroup::ContentSources,
        ];

        patch_config(tmp.path(), &groups, &answers).unwrap();

        let result = fs::read_to_string(tmp.path()).unwrap();
        let config: tuitbot_core::config::Config =
            toml::from_str(&result).expect("patched config should parse");

        assert_eq!(
            config.deployment_mode,
            tuitbot_core::config::DeploymentMode::SelfHost
        );
        assert_eq!(
            config.connectors.google_drive.client_id.as_deref(),
            Some("cid")
        );
        assert!(result.contains("[content_sources]"));
        // Original values preserved
        assert!(config.approval_mode);
        assert_eq!(config.business.product_name, "App");
    }
}
