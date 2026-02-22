/// CLI subcommand argument definitions and implementations for ReplyGuy.
///
/// Each subcommand struct defines its flags and arguments
/// matching the CLI interface contract.
pub mod approve;
pub mod auth;
pub mod init;
pub mod run;
pub mod settings;
pub mod stats;
pub mod test;
pub mod upgrade;

use clap::Args;

/// Arguments for the `init` subcommand.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Overwrite existing config file
    #[arg(long)]
    pub force: bool,

    /// Skip interactive wizard and copy template config
    #[arg(long)]
    pub non_interactive: bool,
}

/// Arguments for the `run` subcommand.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Print periodic status summary (0 = disabled)
    #[arg(long, default_value = "0")]
    pub status_interval: u64,
}

/// Arguments for the `auth` subcommand.
#[derive(Debug, Args)]
pub struct AuthArgs {
    /// Auth mode override
    #[arg(long, value_parser = ["manual", "local_callback"])]
    pub mode: Option<String>,
}

/// Arguments for the `test` subcommand.
#[derive(Debug, Args)]
pub struct TestArgs;

/// Arguments for the `discover` subcommand.
#[derive(Debug, Args)]
pub struct DiscoverArgs {
    /// Search and score tweets without posting replies
    #[arg(long)]
    pub dry_run: bool,

    /// Maximum tweets to process
    #[arg(long, default_value = "50")]
    pub limit: u32,
}

/// Arguments for the `mentions` subcommand.
#[derive(Debug, Args)]
pub struct MentionsArgs {
    /// Retrieve mentions and generate replies without posting
    #[arg(long)]
    pub dry_run: bool,

    /// Maximum mentions to process
    #[arg(long, default_value = "20")]
    pub limit: u32,
}

/// Arguments for the `post` subcommand.
#[derive(Debug, Args)]
pub struct PostArgs {
    /// Generate tweet without posting
    #[arg(long)]
    pub dry_run: bool,

    /// Override topic (default: random from industry_topics)
    #[arg(long)]
    pub topic: Option<String>,
}

/// Arguments for the `thread` subcommand.
#[derive(Debug, Args)]
pub struct ThreadArgs {
    /// Generate thread without posting
    #[arg(long)]
    pub dry_run: bool,

    /// Override topic (default: random from industry_topics)
    #[arg(long)]
    pub topic: Option<String>,

    /// Number of tweets in thread
    #[arg(long)]
    pub count: Option<u32>,
}

/// Arguments for the `score` subcommand.
#[derive(Debug, Args)]
pub struct ScoreArgs {
    /// The X tweet ID to score
    pub tweet_id: String,
}

/// Arguments for the `stats` subcommand.
#[derive(Debug, Args)]
pub struct StatsArgs;

/// Arguments for the `approve` subcommand.
#[derive(Debug, Args)]
pub struct ApproveArgs;

/// Arguments for the `settings` subcommand.
#[derive(Debug, Args)]
pub struct SettingsArgs {
    /// Show current configuration (read-only)
    #[arg(long)]
    pub show: bool,

    /// Set a config value directly (e.g., --set scoring.threshold=80)
    #[arg(long)]
    pub set: Option<String>,

    /// Jump directly to a specific category
    #[arg(value_name = "CATEGORY")]
    pub category: Option<String>,
}

/// Arguments for the `upgrade` subcommand.
#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Skip interactive prompts and apply default values for new features
    #[arg(long)]
    pub non_interactive: bool,
}
