/// CLI subcommand argument definitions for ReplyGuy.
///
/// Each subcommand struct defines its flags and arguments
/// matching the CLI interface contract.
use clap::Args;

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
