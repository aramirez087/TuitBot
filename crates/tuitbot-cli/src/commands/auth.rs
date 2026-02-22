//! Implementation of the `tuitbot auth` command.
//!
//! Walks the user through the OAuth 2.0 PKCE authentication flow
//! with the X API. Supports both manual code-entry and local
//! callback server modes. Manual mode is the default and works
//! on headless servers (VPS, SSH, OpenClaw).

use std::io::Write;
use tuitbot_core::config::Config;
use tuitbot_core::startup::{
    build_auth_url, build_redirect_uri, exchange_auth_code, extract_auth_code, generate_pkce,
    save_tokens_to_file, token_file_path, verify_credentials,
};

/// Execute the `tuitbot auth` command.
///
/// Determines the auth mode from the CLI flag or config, runs the
/// appropriate PKCE flow, saves tokens, and verifies credentials.
pub async fn execute(config: &Config, mode_override: Option<&str>) -> anyhow::Result<()> {
    // 1. Validate client_id.
    if config.x_api.client_id.is_empty() {
        anyhow::bail!(
            "X API client_id not configured.\n\
             Set it in your config file under [x_api] or via TUITBOT_X_API__CLIENT_ID env var.\n\
             Get your client_id from https://developer.x.com/en/portal/dashboard"
        );
    }

    // 2. Determine auth mode.
    let mode = mode_override.unwrap_or(&config.auth.mode);
    let redirect_uri = build_redirect_uri(&config.auth.callback_host, config.auth.callback_port);

    // 3. Generate PKCE challenge.
    let pkce = generate_pkce();
    let auth_url = build_auth_url(
        &config.x_api.client_id,
        &redirect_uri,
        &pkce.state,
        &pkce.challenge,
    );

    // 4. Run the auth flow based on mode.
    let code = match mode {
        "local_callback" => {
            if is_headless_environment() {
                eprintln!("Headless environment detected — using manual authentication.\n");
                run_manual_mode(&auth_url)?
            } else {
                run_callback_mode(
                    &auth_url,
                    &config.auth.callback_host,
                    config.auth.callback_port,
                )
                .await?
            }
        }
        _ => run_manual_mode(&auth_url)?,
    };

    // 5. Exchange the authorization code for tokens.
    eprintln!("\nExchanging authorization code for tokens...");
    let tokens = exchange_auth_code(
        &config.x_api.client_id,
        &code,
        &redirect_uri,
        &pkce.verifier,
    )
    .await?;

    // 6. Save tokens to disk.
    save_tokens_to_file(&tokens)?;
    let token_path = token_file_path();
    tracing::info!(path = %token_path.display(), "Tokens saved");

    // 7. Verify credentials.
    eprintln!("Verifying credentials...");
    let username = verify_credentials(&tokens.access_token).await?;

    eprintln!(
        "\nAuthenticated as @{username}. Tokens saved to {}",
        token_path.display()
    );

    Ok(())
}

/// Manual mode: print the authorization URL and prompt for the code.
///
/// Designed as the primary headless-friendly auth flow. Works from any
/// terminal — local, SSH, VPS, or OpenClaw. The user opens the URL on
/// any device with a browser, authorizes, then copies the code back.
fn run_manual_mode(auth_url: &str) -> anyhow::Result<String> {
    let token_path = token_file_path();

    eprintln!("=== X API Authentication ===\n");
    eprintln!("1. Open this URL in any browser (laptop, phone, etc.):\n");
    eprintln!("   {auth_url}\n");
    eprintln!("2. Log in to X and authorize the application.");
    eprintln!(
        "3. After authorizing, your browser will redirect to a page that\n   \
         won't load — this is normal. Copy the ENTIRE URL from the address bar.\n   \
         It looks like: http://127.0.0.1:8080/callback?code=...&state=..."
    );
    eprintln!("\nTokens will be saved to: {}\n", token_path.display());
    eprintln!("Paste the full callback URL (or just the code):");

    eprint!("> ");
    std::io::stderr().flush().ok();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("failed to read input: {e}"))?;

    let code = extract_auth_code(&input);
    if code.is_empty() {
        anyhow::bail!("No authorization code provided.");
    }

    Ok(code)
}

/// Callback mode: start a local HTTP server, open the browser, and wait.
async fn run_callback_mode(auth_url: &str, host: &str, port: u16) -> anyhow::Result<String> {
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        anyhow::anyhow!(
            "Failed to start callback server on {addr}: {e}\n\
             Try a different port in your config or use --mode manual"
        )
    })?;

    eprintln!("Callback server listening on {addr}");
    eprintln!("Opening authorization URL in your browser...\n");

    // Open the auth URL in the default browser.
    // If the browser fails to open, fall back to manual mode so the user
    // doesn't wait 120 seconds for a callback that will never arrive.
    if let Err(e) = open::that(auth_url) {
        eprintln!(
            "Could not open browser automatically: {e}\n\
             Falling back to manual authentication.\n"
        );
        drop(listener);
        return run_manual_mode(auth_url);
    }

    // Wait for the callback with a 120-second timeout.
    let code = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        wait_for_callback(&listener),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "Timed out waiting for authorization callback (120s).\n\
             Try again without --mode local_callback (manual mode is the default)."
        )
    })??;

    Ok(code)
}

/// Wait for a single HTTP callback request and extract the authorization code.
async fn wait_for_callback(listener: &tokio::net::TcpListener) -> anyhow::Result<String> {
    let (mut stream, _addr) = listener.accept().await?;

    // Read the HTTP request.
    let mut buf = vec![0u8; 4096];
    let n = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Extract the first line: GET /callback?code=XXX&state=YYY HTTP/1.1
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    // Check for access_denied error.
    if path.contains("error=access_denied") {
        // Send error response.
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h2>Authorization Denied</h2>\
            <p>You denied the authorization request. You can close this tab.</p>\
            </body></html>";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .ok();
        anyhow::bail!("Authorization denied by user.");
    }

    // Extract the code parameter.
    let code = extract_auth_code(path);
    if code.is_empty() {
        // Send error response.
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h2>Error</h2>\
            <p>No authorization code found in the callback.</p>\
            </body></html>";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .ok();
        anyhow::bail!("No authorization code found in callback request.");
    }

    // Send success response.
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h2>Authorization Successful</h2>\
        <p>You can close this tab and return to the terminal.</p>\
        </body></html>";
    tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
        .await
        .ok();

    Ok(code)
}

/// Detect headless environments where a local callback server is unreachable.
///
/// Returns `true` when running over SSH, inside OpenClaw, or when no display
/// server is available on Linux (no X11 or Wayland).
fn is_headless_environment() -> bool {
    // SSH session.
    if std::env::var("SSH_CONNECTION").is_ok() || std::env::var("SSH_TTY").is_ok() {
        return true;
    }

    // Running inside OpenClaw agent.
    if std::env::vars().any(|(k, _)| k.starts_with("OPENCLAW_")) {
        return true;
    }

    // Linux without a display server.
    #[cfg(target_os = "linux")]
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        return true;
    }

    false
}
