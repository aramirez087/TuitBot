/// ReplyGuy CLI - Autonomous X growth assistant.
///
/// Entry point for the replyguy binary. Parses CLI arguments,
/// initializes logging, and dispatches to subcommand handlers.
mod commands;

use clap::Parser;

/// Autonomous X growth assistant
#[derive(Parser)]
#[command(name = "replyguy")]
#[command(version)]
#[command(about = "Autonomous X growth assistant")]
struct Cli {
    /// Path to config.toml
    #[arg(short = 'c', long, default_value = "~/.replyguy/config.toml")]
    config: String,

    /// Enable verbose logging (debug level)
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
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
    /// Score a specific tweet
    Score(commands::ScoreArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing subscriber based on verbosity flags
    let log_level = if cli.quiet {
        "error"
    } else if cli.verbose {
        "debug"
    } else {
        "warn"
    };

    tracing_subscriber::fmt().with_env_filter(log_level).init();

    match cli.command {
        Commands::Run(_args) => {
            println!("run not implemented yet");
        }
        Commands::Auth(_args) => {
            println!("auth not implemented yet");
        }
        Commands::Test(_args) => {
            println!("test not implemented yet");
        }
        Commands::Discover(_args) => {
            println!("discover not implemented yet");
        }
        Commands::Mentions(_args) => {
            println!("mentions not implemented yet");
        }
        Commands::Post(_args) => {
            println!("post not implemented yet");
        }
        Commands::Thread(_args) => {
            println!("thread not implemented yet");
        }
        Commands::Score(_args) => {
            println!("score not implemented yet");
        }
    }

    Ok(())
}
