//! Tuitbot API server binary.
//!
//! Starts an HTTP server bridging tuitbot-core's storage layer to a REST API
//! for the desktop dashboard.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;
use tuitbot_core::auth::passphrase;
use tuitbot_core::config::Config;
use tuitbot_core::content::ContentGenerator;
use tuitbot_core::llm::factory::create_provider;
use tuitbot_core::storage;
use tuitbot_core::storage::accounts::DEFAULT_ACCOUNT_ID;

use tuitbot_core::net::local_ip;
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

    /// Host address to bind to. Use 0.0.0.0 for LAN access.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Path to the tuitbot configuration file.
    #[arg(long, default_value = "~/.tuitbot/config.toml")]
    config: String,

    /// Reset the web login passphrase and print the new one.
    #[arg(long)]
    reset_passphrase: bool,
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
        host = %cli.host,
        port = cli.port,
        "starting tuitbot server"
    );

    let pool = storage::init_db(&db_path.to_string_lossy()).await?;

    // Ensure the API token file exists and read it.
    let api_token = auth::ensure_api_token(db_dir)?;
    tracing::info!(token_path = %db_dir.join("api_token").display(), "API token ready");

    // Handle passphrase for web/LAN auth.
    let passphrase_hash = if cli.reset_passphrase {
        let new_passphrase = passphrase::reset_passphrase(db_dir)?;
        println!("\n  Web login passphrase (reset): {new_passphrase}\n");
        tracing::info!("Passphrase has been reset");
        passphrase::load_passphrase_hash(db_dir)?
    } else {
        match passphrase::ensure_passphrase(db_dir)? {
            Some(new_passphrase) => {
                println!("\n  Web login passphrase: {new_passphrase}");
                println!("  (save this — it won't be shown again)\n");
            }
            None => {
                tracing::info!("Passphrase already configured");
            }
        }
        passphrase::load_passphrase_hash(db_dir)?
    };

    // Create the broadcast channel for WebSocket events.
    let (event_tx, _) = tokio::sync::broadcast::channel::<WsEvent>(256);

    let data_dir = db_dir.to_path_buf();

    // Load config for server settings and content generator.
    let loaded_config = Config::load(Some(&cli.config)).ok();

    // Determine effective bind host/port: CLI flags override config values.
    let bind_host = if cli.host != "127.0.0.1" {
        cli.host.clone()
    } else {
        loaded_config
            .as_ref()
            .map(|c| c.server.host.clone())
            .unwrap_or_else(|| cli.host.clone())
    };
    let bind_port = if cli.port != 3001 {
        cli.port
    } else {
        loaded_config
            .as_ref()
            .map(|c| c.server.port)
            .unwrap_or(cli.port)
    };

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

    let mut content_generators = HashMap::new();
    if let Some(cg) = content_generator {
        content_generators.insert(DEFAULT_ACCOUNT_ID.to_string(), cg);
    }

    let state = Arc::new(AppState {
        db: pool,
        config_path,
        data_dir,
        event_tx,
        api_token,
        passphrase_hash: tokio::sync::RwLock::new(passphrase_hash),
        bind_host: bind_host.clone(),
        bind_port,
        login_attempts: Mutex::new(HashMap::new()),
        runtimes: Mutex::new(HashMap::new()),
        content_generators: Mutex::new(content_generators),
        circuit_breaker: None,
    });

    let router = tuitbot_server::build_router(state);

    // Warn about network exposure when binding to 0.0.0.0.
    if bind_host == "0.0.0.0" {
        tracing::warn!("Binding to 0.0.0.0 — server accessible from LAN");
        if let Some(ip) = local_ip() {
            println!("  Dashboard: http://{}:{}", ip, bind_port);
        }
    }

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", bind_host, bind_port)).await?;
    tracing::info!("listening on http://{}:{}", bind_host, bind_port);
    axum::serve(listener, router).await?;

    Ok(())
}
