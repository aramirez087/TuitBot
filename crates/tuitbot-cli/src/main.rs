/// ReplyGuy CLI - Autonomous X growth assistant.
///
/// Entry point for the replyguy binary. Parses CLI arguments,
/// initializes logging, and dispatches to subcommand handlers.
mod commands;

use std::io::IsTerminal;

use clap::Parser;
use replyguy_core::config::Config;
use tracing_subscriber::EnvFilter;

/// Autonomous X growth assistant
#[derive(Parser)]
#[command(name = "replyguy")]
#[command(version)]
#[command(about = "Autonomous X growth assistant")]
#[command(after_help = "\
Quick start:
  1. replyguy init     — interactive setup wizard
  2. replyguy auth     — authenticate with X
  3. replyguy test     — validate configuration
  4. replyguy run      — start the agent")]
struct Cli {
    /// Path to config.toml
    #[arg(
        short = 'c',
        long,
        global = true,
        default_value = "~/.replyguy/config.toml"
    )]
    config: String,

    /// Enable verbose logging (debug level)
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Set up configuration (interactive wizard)
    Init(commands::InitArgs),
    /// Start the autonomous agent
    Run(commands::RunArgs),
    /// Authenticate with X API
    Auth(commands::AuthArgs),
    /// Validate configuration and connectivity
    Test(commands::TestArgs),
    /// Run discovery loop once
    Discover(commands::DiscoverArgs),
    /// Check and reply to mentions
    Mentions(commands::MentionsArgs),
    /// Generate and post an original tweet
    Post(commands::PostArgs),
    /// Generate and post an educational thread
    Thread(commands::ThreadArgs),
    /// Edit configuration interactively
    Settings(commands::SettingsArgs),
    /// Score a specific tweet
    Score(commands::ScoreArgs),
    /// Show analytics dashboard
    Stats(commands::StatsArgs),
    /// Review and approve queued posts
    Approve(commands::ApproveArgs),
    /// Configure new features added since last setup
    Upgrade(commands::UpgradeArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing-subscriber.
    //
    // Priority: RUST_LOG env var > --verbose/--quiet flags > default (warn).
    // - Default: warn level, compact format with timestamps.
    // - Verbose (-v): debug level, includes module paths.
    // - Quiet (-q): error level, minimal format.
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else if cli.verbose {
        EnvFilter::new("replyguy=debug,replyguy_core=debug,info")
    } else if cli.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("replyguy=info,replyguy_core=info,warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(cli.verbose)
        .compact()
        .init();

    // Handle `init`, `upgrade`, and `settings` before general config loading
    // (they manage their own config lifecycle).
    if let Commands::Init(args) = cli.command {
        return commands::init::execute(args.force, args.non_interactive).await;
    }
    if let Commands::Upgrade(args) = cli.command {
        return commands::upgrade::execute(args.non_interactive, &cli.config).await;
    }
    if let Commands::Settings(args) = cli.command {
        return commands::settings::execute(args, &cli.config).await;
    }

    // Load configuration.
    let config = Config::load(Some(&cli.config)).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load configuration: {e}\n\
             Hint: Run 'replyguy init' to create a default configuration file."
        )
    })?;

    // Check for config upgrade opportunity before `run`
    if matches!(&cli.command, Commands::Run(_)) && std::io::stdin().is_terminal() {
        commands::upgrade::check_before_run(&cli.config).await?;
    }

    match cli.command {
        Commands::Init(_) | Commands::Upgrade(_) | Commands::Settings(_) => unreachable!(),
        Commands::Run(args) => {
            commands::run::execute(&config, args.status_interval).await?;
        }
        Commands::Auth(args) => {
            commands::auth::execute(&config, args.mode.as_deref()).await?;
        }
        Commands::Test(_args) => {
            commands::test::execute(&config, &cli.config).await?;
        }
        Commands::Discover(_args) => {
            eprintln!("discover: not yet available (requires WP08 merge)");
        }
        Commands::Mentions(_args) => {
            eprintln!("mentions: not yet available (requires WP08 merge)");
        }
        Commands::Post(_args) => {
            eprintln!("post: not yet available (requires WP09 merge)");
        }
        Commands::Thread(_args) => {
            eprintln!("thread: not yet available (requires WP09 merge)");
        }
        Commands::Score(_args) => {
            eprintln!("score: not yet available (requires WP06 merge)");
        }
        Commands::Stats(_args) => {
            commands::stats::execute(&config).await?;
        }
        Commands::Approve(_args) => {
            commands::approve::execute(&config).await?;
        }
    }

    Ok(())
}
