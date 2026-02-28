/// CLI subcommand argument definitions and implementations for Tuitbot.
///
/// Each subcommand struct defines its flags and arguments
/// matching the CLI interface contract.
pub mod approve;
pub mod auth;
pub mod backup;
pub mod init;
pub mod mcp;
pub mod restore;
pub mod run;
pub mod settings;
pub mod stats;
pub mod test;
pub mod tick;
pub mod update;
pub mod upgrade;

use clap::Args;

/// Output format for machine-readable output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "json" => Self::Json,
            _ => Self::Text,
        }
    }

    pub fn is_json(self) -> bool {
        self == Self::Json
    }
}

/// Arguments for the `init` subcommand.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Overwrite existing config file
    #[arg(long)]
    pub force: bool,

    /// Skip interactive wizard and copy template config
    #[arg(long)]
    pub non_interactive: bool,

    /// Run the full 8-step advanced setup wizard
    #[arg(long)]
    pub advanced: bool,
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
pub struct ApproveArgs {
    /// List pending items without interactive review
    #[arg(long)]
    pub list: bool,

    /// Approve a specific item by ID
    #[arg(long)]
    pub approve: Option<i64>,

    /// Reject a specific item by ID
    #[arg(long)]
    pub reject: Option<i64>,

    /// Approve all pending items
    #[arg(long)]
    pub approve_all: bool,
}

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

/// Arguments for the `update` subcommand.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Skip all interactive prompts
    #[arg(long)]
    pub non_interactive: bool,

    /// Only check for updates, don't install
    #[arg(long)]
    pub check: bool,

    /// Skip binary update, only upgrade config
    #[arg(long)]
    pub config_only: bool,
}

/// Arguments for the `upgrade` subcommand.
#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Skip interactive prompts and apply default values for new features
    #[arg(long)]
    pub non_interactive: bool,
}

/// Arguments for the `tick` subcommand.
#[derive(Debug, Args)]
pub struct TickArgs {
    /// Run all loops but don't post anything
    #[arg(long)]
    pub dry_run: bool,

    /// Skip active-hours schedule check
    #[arg(long)]
    pub ignore_schedule: bool,

    /// Comma-separated loops to run (default: all enabled)
    /// Options: discovery, mentions, content, thread, target, analytics
    #[arg(long, value_delimiter = ',')]
    pub loops: Option<Vec<String>>,

    /// Force approval mode on for this tick (queue posts for human review)
    #[arg(long)]
    pub require_approval: bool,
}

/// Arguments for the `backup` subcommand.
#[derive(Debug, Args)]
pub struct BackupArgs {
    /// Custom output directory for the backup file
    #[arg(long)]
    pub output_dir: Option<String>,

    /// List existing backups
    #[arg(long)]
    pub list: bool,

    /// Keep N most recent backups, delete the rest
    #[arg(long)]
    pub prune: Option<usize>,
}

/// Arguments for the `restore` subcommand.
#[derive(Debug, Args)]
pub struct RestoreArgs {
    /// Path to the backup file to restore from
    pub backup_path: String,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,

    /// Validate backup without restoring
    #[arg(long)]
    pub validate_only: bool,
}

/// Arguments for the `mcp` subcommand.
#[derive(Debug, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpSubcommand,
}

/// MCP subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum McpSubcommand {
    /// Start the MCP server (stdio transport)
    Serve {
        /// Runtime profile: "write" (default), "readonly", "api-readonly", or "admin"
        #[arg(long, default_value = "write")]
        profile: String,
    },
    /// Print the tool manifest for a profile (JSON to stdout)
    Manifest {
        /// Runtime profile: "write" (default), "readonly", "api-readonly", or "admin"
        #[arg(long, default_value = "write")]
        profile: String,
    },
    /// Guided setup for MCP-only users (Client ID → auth → register)
    Setup,
}
