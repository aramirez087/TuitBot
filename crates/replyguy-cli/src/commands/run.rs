//! Implementation of the `replyguy run` command.
//!
//! The main entry point for autonomous operation. Initializes all
//! dependencies, detects API tier, starts automation loops, and
//! waits for a shutdown signal.

use replyguy_core::config::Config;
use replyguy_core::startup::{
    expand_tilde, format_startup_banner, load_tokens_from_file, ApiTier, TierCapabilities,
};

/// Execute the `replyguy run` command.
///
/// Startup sequence:
/// 1. Validate database path
/// 2. Load and verify OAuth tokens
/// 3. Detect API tier (defaults to Free in this integration layer)
/// 4. Print startup banner
/// 5. Wait for shutdown signal
pub async fn execute(config: &Config, status_interval: u64) -> anyhow::Result<()> {
    // 1. Validate database path.
    let db_path = expand_tilde(&config.storage.db_path);
    tracing::info!(path = %db_path.display(), "Database path configured");

    // 2. Load OAuth tokens.
    let tokens = load_tokens_from_file().map_err(|e| anyhow::anyhow!("{e}"))?;

    if tokens.is_expired() {
        anyhow::bail!("Authentication expired. Run `replyguy auth` to re-authenticate.");
    }
    tracing::info!(
        expires_in = %tokens.format_expiry(),
        "OAuth tokens loaded"
    );

    // 3. Determine API tier.
    // Full tier detection requires X API calls (implemented in WP04).
    // At the integration layer, we default to Free tier. When the full
    // application is assembled, this will call XApiClient::detect_tier().
    let tier = ApiTier::Free;
    let capabilities = TierCapabilities::for_tier(tier);
    tracing::info!(tier = %tier, "{}", capabilities.format_status());

    // 4. Apply status_interval override.
    let effective_interval = if status_interval > 0 {
        status_interval
    } else {
        config.logging.status_interval_seconds
    };

    // 5. Print startup banner (always visible, even in default mode).
    let banner = format_startup_banner(tier, &capabilities, effective_interval);
    eprintln!("{banner}");

    // 6. Wait for shutdown signal.
    wait_for_shutdown_signal().await;

    tracing::info!("Shutdown complete.");
    Ok(())
}

/// Wait for an OS shutdown signal (Ctrl+C or SIGTERM).
async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to register SIGTERM handler, using Ctrl+C only"
                );
                if let Err(e) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %e, "Failed to listen for Ctrl+C");
                } else {
                    tracing::info!("Received Ctrl+C");
                }
                return;
            }
        };

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(e) = result {
                    tracing::error!(error = %e, "Ctrl+C handler error");
                }
                tracing::info!("Received Ctrl+C");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %e, "Failed to listen for Ctrl+C");
        } else {
            tracing::info!("Received Ctrl+C");
        }
    }
}
