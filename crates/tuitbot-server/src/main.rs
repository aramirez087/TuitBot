//! Tuitbot API server binary.
//!
//! Starts an HTTP server bridging tuitbot-core's storage layer to a REST API
//! for the desktop dashboard.

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use tuitbot_core::config::Config;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::storage;

use tuitbot_server::auth;
use tuitbot_server::state::AppState;
use tuitbot_server::ws::WsEvent;

/// Tuitbot API server — serves the dashboard REST API.
#[derive(Parser)]
#[command(name = "tuitbot-server", version, about)]
struct Cli {
    /// Port to listen on.
    #[arg(long, default_value = "3001")]
    port: u16,

    /// Path to the tuitbot configuration file.
    #[arg(long, default_value = "~/.tuitbot/config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (respects RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    // Derive database path from config directory.
    let config_path = std::path::PathBuf::from(storage::expand_tilde(&cli.config));
    let db_dir = config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let db_path = db_dir.join("tuitbot.db");

    tracing::info!(
        db = %db_path.display(),
        port = cli.port,
        "starting tuitbot server"
    );

    let pool = storage::init_db(&db_path.to_string_lossy()).await?;

    // Ensure the API token file exists and read it.
    let api_token = auth::ensure_api_token(db_dir)?;
    tracing::info!(token_path = %db_dir.join("api_token").display(), "API token ready");

    // Create the broadcast channel for WebSocket events.
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let data_dir = db_dir.to_path_buf();

    // Try to initialize content generator from config (optional — AI assist endpoints need it).
    let content_generator = match Config::load(Some(&cli.config)) {
        Ok(config) => match create_provider(&config.llm) {
            Ok(provider) => {
                tracing::info!("LLM provider initialized for AI assist endpoints");
                Some(Arc::new(ContentGenerator::new(provider, config.business)))
            }
            Err(e) => {
                tracing::info!(error = %e, "LLM provider not configured — AI assist endpoints disabled");
                None
            }
        },
        Err(e) => {
            tracing::info!(error = %e, "Config not loaded — AI assist endpoints disabled");
            None
        }
    };

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir,
        event_tx,
        api_token,
        runtime: Mutex::new(None),
        content_generator,
    });

    let router = tuitbot_server::build_router(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", cli.port)).await?;
    tracing::info!("listening on http://127.0.0.1:{}", cli.port);
    axum::serve(listener, router).await?;

    Ok(())
}
