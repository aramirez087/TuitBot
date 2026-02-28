/// Tuitbot CLI - Autonomous X growth assistant.
///
/// Entry point for the tuitbot binary. Parses CLI arguments,
/// initializes logging, and dispatches to subcommand handlers.
mod commands;
mod deps;
pub mod output;

use std::io::IsTerminal;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use tuitbot_core::config::Config;

/// Autonomous X growth assistant
#[derive(Parser)]
#[command(name = "tuitbot")]
#[command(version)]
#[command(about = "Autonomous X growth assistant")]
#[command(after_help = "\
Quick start:
  1. tuitbot init     — interactive setup wizard
  2. tuitbot auth     — authenticate with X
  3. tuitbot test     — validate configuration
  4. tuitbot run      — start the agent")]
struct Cli {
    /// Path to config.toml
    #[arg(
        short = 'c',
        long,
        global = true,
        default_value = "~/.tuitbot/config.toml"
    )]
    config: String,

    /// Enable verbose logging (debug level)
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Output format (text or json) for machine-readable output
    #[arg(long, global = true, default_value = "text", value_parser = ["text", "json"])]
    output: String,

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
    /// Check for updates and upgrade binary + config
    Update(commands::UpdateArgs),
    /// Configure new features added since last setup
    #[command(hide = true)]
    Upgrade(commands::UpgradeArgs),
    /// Run each enabled loop once and exit (for external schedulers)
    Tick(commands::TickArgs),
    /// MCP server for AI agent integration
    Mcp(commands::McpArgs),
    /// Create a database backup
    Backup(commands::BackupArgs),
    /// Restore database from a backup
    Restore(commands::RestoreArgs),
}

#[tokio::main]
async fn main() {
    // Restore default SIGPIPE handling so piped commands (e.g. `| head`)
    // terminate this process cleanly instead of triggering a panic.
    output::reset_sigpipe();

    let result = run().await;
    match result {
        Ok(()) => {}
        Err(e) if output::is_broken_pipe(&e) => {
            // Consumer closed the pipe — exit silently with success.
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e:#}");
            std::process::exit(1);
        }
    }
}

async fn run() -> anyhow::Result<()> {
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
        EnvFilter::new("tuitbot=debug,tuitbot_core=debug,info")
    } else if cli.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("tuitbot=info,tuitbot_core=info,warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(cli.verbose)
        .compact()
        .init();

    let output_format = commands::OutputFormat::from_str(&cli.output);

    // Handle `init`, `update`, `upgrade`, and `settings` before general config
    // loading (they manage their own config lifecycle).
    if let Commands::Init(args) = cli.command {
        return commands::init::execute(args.force, args.non_interactive, args.advanced).await;
    }
    if let Commands::Update(args) = cli.command {
        return commands::update::execute(
            args.non_interactive,
            args.check,
            args.config_only,
            &cli.config,
        )
        .await;
    }
    if let Commands::Upgrade(args) = cli.command {
        return commands::upgrade::execute(args.non_interactive, &cli.config).await;
    }
    if let Commands::Settings(args) = cli.command {
        return commands::settings::execute(args, &cli.config, output_format).await;
    }
    if let Commands::Backup(args) = cli.command {
        return commands::backup::execute(args).await;
    }
    if let Commands::Restore(args) = cli.command {
        return commands::restore::execute(args).await;
    }
    if let Commands::Mcp(ref args) = cli.command {
        return match &args.command {
            commands::McpSubcommand::Manifest { ref profile } => {
                commands::mcp::print_manifest(profile)
            }
            commands::McpSubcommand::Serve { ref profile } => {
                commands::mcp::execute_serve(profile).await
            }
            commands::McpSubcommand::Setup => commands::mcp::execute_setup().await,
        };
    }

    // Load configuration.
    let config = match Config::load(Some(&cli.config)) {
        Ok(c) => c,
        Err(e) => {
            // If the default config path doesn't exist and we're in an
            // interactive terminal, offer to run init instead of erroring.
            let expanded = tuitbot_core::startup::expand_tilde(&cli.config);
            if cli.config == "~/.tuitbot/config.toml"
                && !expanded.exists()
                && std::io::stdin().is_terminal()
            {
                eprintln!("No configuration found.\n");
                let run_init = dialoguer::Confirm::new()
                    .with_prompt("Run setup wizard now?")
                    .default(true)
                    .interact()
                    .unwrap_or(false);

                if run_init {
                    return commands::init::execute(false, false, false).await;
                }
            }

            return Err(anyhow::anyhow!(
                "Failed to load configuration: {e}\n\
                 Hint: Run 'tuitbot init' to create a default configuration file."
            ));
        }
    };

    // Check for config upgrade opportunity before `run`
    if matches!(&cli.command, Commands::Run(_)) && std::io::stdin().is_terminal() {
        commands::update::check_before_run(&cli.config).await?;
    }

    match cli.command {
        Commands::Init(_)
        | Commands::Update(_)
        | Commands::Upgrade(_)
        | Commands::Settings(_)
        | Commands::Backup(_)
        | Commands::Restore(_)
        | Commands::Mcp(_) => {
            unreachable!()
        }
        Commands::Run(args) => {
            commands::run::execute(&config, args.status_interval).await?;
        }
        Commands::Tick(args) => {
            let mut config = config;
            if args.require_approval {
                config.approval_mode = true;
            }
            commands::tick::execute(&config, args, output_format).await?;
        }
        Commands::Auth(args) => {
            commands::auth::execute(&config, args.mode.as_deref()).await?;
        }
        Commands::Test(_args) => {
            commands::test::execute(&config, &cli.config, output_format).await?;
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
            commands::stats::execute(&config, output_format).await?;
        }
        Commands::Approve(args) => {
            commands::approve::execute(&config, args, output_format).await?;
        }
    }

    Ok(())
}
