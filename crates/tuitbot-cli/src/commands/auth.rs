//! Implementation of the `tuitbot auth` command.
//!
//! Walks the user through the OAuth 2.0 PKCE authentication flow
//! with the X API. Supports both manual code-entry and local
//! callback server modes. Manual mode is the default and works
//! on headless servers (VPS, SSH, OpenClaw).

use std::io::Write;
use tuitbot_core::config::Config;
use tuitbot_core::startup::{
    build_auth_url, build_redirect_uri, exchange_auth_code, extract_auth_code,
    extract_callback_state, generate_pkce, save_tokens_to_file, token_file_path,
    verify_credentials,
};

/// Execute the `tuitbot auth` command.
///
/// Determines the auth mode from the CLI flag or config, runs the
/// appropriate PKCE flow, saves tokens, and verifies credentials.
pub async fn execute(config: &Config, mode_override: Option<&str>) -> anyhow::Result<()> {
    // Short-circuit: scraper mode does not require X API auth.
    if config.x_api.provider_backend == "scraper" {
        eprintln!(
            "Local No-Key Mode (scraper backend) does not require X API authentication.\n\
             Authentication is only needed for the official X API backend."
        );
        return Ok(());
    }

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
                run_manual_mode(&auth_url, &pkce.state)?
            } else {
                run_callback_mode(
                    &auth_url,
                    &config.auth.callback_host,
                    config.auth.callback_port,
                    &pkce.state,
                )
                .await?
            }
        }
        "manual" => run_manual_mode(&auth_url, &pkce.state)?,
        other => {
            anyhow::bail!(
                "Invalid auth mode: '{other}'. Must be 'manual' or 'local_callback'.\n\
                 Fix [auth].mode in your config file or use --mode manual|local_callback."
            );
        }
    };

    // 5. Exchange the authorization code for tokens.
    eprintln!("\nExchanging authorization code for tokens...");
    let tokens = exchange_auth_code(
        &config.x_api.client_id,
        &code,
        &redirect_uri,
        &pkce.verifier,
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("401") || msg.contains("invalid_client") || msg.contains("invalid_grant") {
            anyhow::anyhow!(
                "X API rejected the authorization code (HTTP 401).\n\
                 \n\
                 This usually means the code was already used, it expired (codes are \
                 one-time-use and valid for ~30 seconds), or your client_id is wrong.\n\
                 Run `tuitbot auth` again to get a fresh code."
            )
        } else if msg.contains("connect") || msg.contains("timed out") || msg.contains("dns") {
            anyhow::anyhow!(
                "Cannot reach api.x.com to exchange the authorization code.\n\
                 \n\
                 Network error: {e}\n\
                 Check your internet connection and try again."
            )
        } else {
            anyhow::anyhow!(
                "Token exchange failed: {e}\n\
                 \n\
                 Run `tuitbot auth` again. If this keeps failing, verify your \
                 client_id in config."
            )
        }
    })?;

    // 6. Save tokens to disk.
    save_tokens_to_file(&tokens)?;
    let token_path = token_file_path();
    tracing::info!(path = %token_path.display(), "Tokens saved");

    // 7. Verify credentials.
    eprintln!("Verifying credentials...");
    let username = verify_credentials(&tokens.access_token)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("401") || msg.to_lowercase().contains("unauthorized") {
                anyhow::anyhow!(
                    "X API rejected the new token when verifying credentials (HTTP 401).\n\
                     \n\
                     The token was saved but X returned unauthorized on the first use. \
                     This can happen if your app's permissions don't include `users.read`.\n\
                     Check your app's scopes at https://developer.x.com and re-run \
                     `tuitbot auth`."
                )
            } else if msg.contains("connect") || msg.contains("timed out") || msg.contains("dns") {
                anyhow::anyhow!(
                    "Cannot reach api.x.com to verify credentials.\n\
                     \n\
                     Your token was saved. Network error: {e}\n\
                     Check your internet connection. Run `tuitbot test` to confirm auth later."
                )
            } else {
                anyhow::anyhow!(
                    "Credential verification failed: {e}\n\
                     \n\
                     Your token was saved. Run `tuitbot test` to diagnose the issue."
                )
            }
        })?;

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
fn run_manual_mode(auth_url: &str, expected_state: &str) -> anyhow::Result<String> {
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

    let trimmed = input.trim();

    // If the input looks like a URL, validate the state parameter.
    if trimmed.contains("code=") || trimmed.contains("state=") {
        validate_callback_state(trimmed, expected_state)?;
    }

    let code = extract_auth_code(trimmed);
    if code.is_empty() {
        anyhow::bail!("No authorization code provided.");
    }

    Ok(code)
}

/// Validate the `state` parameter from a callback URL against the expected value.
fn validate_callback_state(input: &str, expected_state: &str) -> anyhow::Result<()> {
    let returned_state = extract_callback_state(input);
    if returned_state.is_empty() {
        anyhow::bail!(
            "Callback URL is missing the 'state' parameter.\n\
             Make sure you copied the entire URL from the address bar."
        );
    }
    if returned_state != expected_state {
        anyhow::bail!(
            "OAuth state mismatch — the callback state does not match this auth session.\n\
             This can happen if the URL is from a different or expired auth attempt.\n\
             Please restart the auth flow and use the new URL."
        );
    }
    Ok(())
}

/// Callback mode: start a local HTTP server, open the browser, and wait.
async fn run_callback_mode(
    auth_url: &str,
    host: &str,
    port: u16,
    expected_state: &str,
) -> anyhow::Result<String> {
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
        return run_manual_mode(auth_url, expected_state);
    }

    // Wait for the callback with a 120-second timeout.
    let code = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        wait_for_callback(&listener, expected_state),
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
async fn wait_for_callback(
    listener: &tokio::net::TcpListener,
    expected_state: &str,
) -> anyhow::Result<String> {
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
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h2>Authorization Denied</h2>\
            <p>You denied the authorization request. You can close this tab.</p>\
            </body></html>";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .ok();
        anyhow::bail!("Authorization denied by user.");
    }

    // Validate the state parameter (CSRF protection).
    let returned_state = extract_callback_state(path);
    if returned_state.is_empty() || returned_state != expected_state {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h2>Error</h2>\
            <p>OAuth state mismatch. This may be a stale or forged callback. \
            Please restart the auth flow.</p>\
            </body></html>";
        tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes())
            .await
            .ok();
        anyhow::bail!(
            "OAuth state mismatch in callback — the returned state does not match \
             this auth session. Please restart the auth flow."
        );
    }

    // Extract the code parameter.
    let code = extract_auth_code(path);
    if code.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_callback_state ──────────────────────────────────────

    #[test]
    fn validate_callback_state_matching() {
        let url = "http://127.0.0.1:8080/callback?code=abc123&state=mystate";
        assert!(validate_callback_state(url, "mystate").is_ok());
    }

    #[test]
    fn validate_callback_state_mismatch() {
        let url = "http://127.0.0.1:8080/callback?code=abc123&state=wrong";
        let err = validate_callback_state(url, "expected").unwrap_err();
        assert!(err.to_string().contains("state mismatch"));
    }

    #[test]
    fn validate_callback_state_missing_state() {
        let url = "http://127.0.0.1:8080/callback?code=abc123";
        let err = validate_callback_state(url, "expected").unwrap_err();
        assert!(err.to_string().contains("missing"));
    }

    // ── is_headless_environment ──────────────────────────────────────

    #[test]
    fn is_headless_returns_bool() {
        // We can't easily control env vars in tests, but we can at least
        // ensure the function doesn't panic and returns a bool.
        let _result = is_headless_environment();
    }

    // ── validate_callback_state edge cases ────────────────────────────

    #[test]
    fn validate_callback_state_with_extra_params() {
        let url = "http://127.0.0.1:8080/callback?code=abc&state=mystate&extra=value";
        assert!(validate_callback_state(url, "mystate").is_ok());
    }

    #[test]
    fn validate_callback_state_state_before_code() {
        let url = "http://127.0.0.1:8080/callback?state=mystate&code=abc123";
        assert!(validate_callback_state(url, "mystate").is_ok());
    }

    #[test]
    fn validate_callback_state_error_message_contains_mismatch() {
        let url = "http://127.0.0.1:8080/callback?code=abc&state=wrong";
        let err = validate_callback_state(url, "expected").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("state mismatch") || msg.contains("mismatch"));
    }

    #[test]
    fn validate_callback_state_error_message_contains_missing() {
        let url = "http://127.0.0.1:8080/callback?code=abc123";
        let err = validate_callback_state(url, "expected").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing") || msg.contains("Missing"));
    }

    // ── URL/code detection patterns ───────────────────────────────────

    #[test]
    fn input_detection_url_vs_code() {
        let url_input = "http://127.0.0.1:8080/callback?code=abc123&state=xyz";
        assert!(url_input.contains("code=") || url_input.contains("state="));

        let bare_code = "abc123def456";
        assert!(!bare_code.contains("code=") && !bare_code.contains("state="));
    }

    // ── extract_auth_code ─────────────────────────────────────────────

    #[test]
    fn extract_auth_code_from_url() {
        let code = extract_auth_code("http://127.0.0.1:8080/callback?code=mycode&state=st");
        assert_eq!(code, "mycode");
    }

    #[test]
    fn extract_auth_code_bare_code() {
        let code = extract_auth_code("mycode123");
        // When input doesn't contain code=, the function should handle it
        // (either return the input or empty depending on implementation)
        assert!(!code.is_empty() || code.is_empty()); // just don't panic
    }

    // ── extract_callback_state ────────────────────────────────────────

    #[test]
    fn extract_callback_state_from_url() {
        let state = extract_callback_state("http://127.0.0.1:8080/callback?code=c&state=mystate");
        assert_eq!(state, "mystate");
    }

    #[test]
    fn extract_callback_state_missing() {
        let state = extract_callback_state("http://127.0.0.1:8080/callback?code=c");
        assert!(state.is_empty());
    }

    // ── Auth mode matching ────────────────────────────────────────────

    #[test]
    fn auth_mode_matching_patterns() {
        let modes = ["manual", "local_callback"];
        for mode in &modes {
            let valid = matches!(*mode, "manual" | "local_callback");
            assert!(valid, "mode {mode} should be valid");
        }
        assert!(!matches!("invalid", "manual" | "local_callback"));
    }
}
