//! OAuth 2.0 PKCE client setup and authentication flows.

use std::io::Write;

use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};

use crate::error::XApiError;
use crate::x_api::scopes::REQUIRED_SCOPES;

use super::{Tokens, AUTH_URL, TOKEN_URL};

/// Build the OAuth 2.0 PKCE client with the given configuration.
pub(crate) fn build_oauth_client(
    client_id: &str,
    redirect_uri: &str,
) -> Result<BasicClient, XApiError> {
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).map_err(|e| XApiError::ApiError {
        status: 0,
        message: format!("Invalid auth URL: {e}"),
    })?;

    let token_url = TokenUrl::new(TOKEN_URL.to_string()).map_err(|e| XApiError::ApiError {
        status: 0,
        message: format!("Invalid token URL: {e}"),
    })?;

    let redirect = RedirectUrl::new(redirect_uri.to_string()).map_err(|e| XApiError::ApiError {
        status: 0,
        message: format!("Invalid redirect URI: {e}"),
    })?;

    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect);

    Ok(client)
}

/// Perform OAuth 2.0 PKCE authentication in manual mode.
///
/// Prints the authorization URL and prompts the user to paste the
/// authorization code from the callback URL. Exchanges the code for tokens.
pub async fn authenticate_manual(client_id: &str) -> Result<Tokens, XApiError> {
    let redirect_uri = "http://localhost/callback";
    let client = build_oauth_client(client_id, redirect_uri)?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_builder = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);
    for scope in REQUIRED_SCOPES {
        auth_builder = auth_builder.add_scope(Scope::new(scope.to_string()));
    }
    let (auth_url, csrf_state) = auth_builder.url();

    println!("\n=== X API Authentication (Manual Mode) ===\n");
    println!("1. Open this URL in your browser:\n");
    println!("   {auth_url}\n");
    println!("2. Authorize the application");
    println!("3. Copy the authorization code from the callback URL");
    println!("   (Look for ?code=XXXXX in the URL)\n");

    let _ = csrf_state; // State validation not applicable in manual mode

    print!("Paste the authorization code: ");
    std::io::stdout().flush().map_err(|e| XApiError::ApiError {
        status: 0,
        message: format!("IO error: {e}"),
    })?;

    let mut code = String::new();
    std::io::stdin()
        .read_line(&mut code)
        .map_err(|e| XApiError::ApiError {
            status: 0,
            message: format!("Failed to read input: {e}"),
        })?;

    let code = code.trim().to_string();
    if code.is_empty() {
        return Err(XApiError::ApiError {
            status: 0,
            message: "Authorization code cannot be empty".to_string(),
        });
    }

    exchange_code(&client, &code, pkce_verifier).await
}

/// Perform OAuth 2.0 PKCE authentication with a local callback server.
///
/// Starts a temporary HTTP server, opens the browser to the authorization URL,
/// and captures the callback with the authorization code automatically.
pub async fn authenticate_callback(
    client_id: &str,
    host: &str,
    port: u16,
) -> Result<Tokens, XApiError> {
    let redirect_uri = format!("http://{host}:{port}/callback");
    let client = build_oauth_client(client_id, &redirect_uri)?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_builder = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);
    for scope in REQUIRED_SCOPES {
        auth_builder = auth_builder.add_scope(Scope::new(scope.to_string()));
    }
    let (auth_url, csrf_state) = auth_builder.url();

    // Start the temporary callback server
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| XApiError::ApiError {
            status: 0,
            message: format!(
                "Failed to bind callback server on {addr}: {e}. Try changing auth.callback_port."
            ),
        })?;

    tracing::info!("Callback server listening on {addr}");

    // Open the browser
    let url_str = auth_url.to_string();
    if let Err(e) = open::that(&url_str) {
        tracing::warn!(error = %e, "Failed to open browser automatically");
        println!("\nCould not open browser automatically.");
        println!("Please open this URL manually:\n");
        println!("   {url_str}\n");
    } else {
        println!("\nOpened authorization URL in your browser.");
        println!("Waiting for callback...\n");
    }

    // Wait for the callback with a timeout
    let callback_result = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        accept_callback(&listener, csrf_state.secret()),
    )
    .await
    .map_err(|_| XApiError::ApiError {
        status: 0,
        message: "Authentication timed out after 120 seconds".to_string(),
    })??;

    exchange_code(&client, &callback_result, pkce_verifier).await
}

/// Accept a single HTTP callback and extract the authorization code.
async fn accept_callback(
    listener: &tokio::net::TcpListener,
    expected_state: &str,
) -> Result<String, XApiError> {
    let (mut stream, _addr) = listener.accept().await.map_err(|e| XApiError::ApiError {
        status: 0,
        message: format!("Failed to accept connection: {e}"),
    })?;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| XApiError::ApiError {
            status: 0,
            message: format!("Failed to read request: {e}"),
        })?;

    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse the first line: GET /callback?code=XXX&state=YYY HTTP/1.1
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    let query_start = path.find('?').map(|i| i + 1);
    let query_string = query_start.map(|i| &path[i..]).unwrap_or("");

    let mut code = None;
    let mut state = None;

    for param in query_string.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            match key {
                "code" => code = Some(value.to_string()),
                "state" => state = Some(value.to_string()),
                _ => {}
            }
        }
    }

    // Validate state (required for CSRF protection)
    let received_state = state.ok_or_else(|| XApiError::ApiError {
        status: 0,
        message: "Missing OAuth state parameter in callback".to_string(),
    })?;
    if received_state != expected_state {
        let error_html = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h1>Authentication Failed</h1>\
            <p>State parameter mismatch. This may indicate a CSRF attack.</p>\
            <p>Please try again.</p></body></html>";
        let _ = stream.write_all(error_html.as_bytes()).await;
        return Err(XApiError::ApiError {
            status: 0,
            message: "OAuth state parameter mismatch".to_string(),
        });
    }

    let auth_code = code.ok_or_else(|| XApiError::ApiError {
        status: 0,
        message: "No authorization code in callback URL".to_string(),
    })?;

    // Send success response
    let success_html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h1>Authentication Successful!</h1>\
        <p>You can close this tab and return to the terminal.</p></body></html>";
    let _ = stream.write_all(success_html.as_bytes()).await;

    Ok(auth_code)
}

/// Exchange an authorization code for tokens using the PKCE verifier.
async fn exchange_code(
    client: &BasicClient,
    code: &str,
    pkce_verifier: oauth2::PkceCodeVerifier,
) -> Result<Tokens, XApiError> {
    let http_client = oauth2::reqwest::async_http_client;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(http_client)
        .await
        .map_err(|e| XApiError::ApiError {
            status: 0,
            message: format!("Token exchange failed: {e}"),
        })?;

    let access_token = token_result.access_token().secret().to_string();
    let refresh_token = token_result
        .refresh_token()
        .map(|rt| rt.secret().to_string())
        .unwrap_or_default();

    let expires_in = token_result
        .expires_in()
        .map(|d| d.as_secs() as i64)
        .unwrap_or(7200);

    let scopes: Vec<String> = token_result
        .scopes()
        .map(|s| s.iter().map(|scope| scope.to_string()).collect())
        .unwrap_or_else(|| REQUIRED_SCOPES.iter().map(|s| s.to_string()).collect());

    let tokens = Tokens {
        access_token,
        refresh_token,
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(expires_in),
        scopes,
    };

    tracing::info!(
        expires_at = %tokens.expires_at,
        scopes = ?tokens.scopes,
        "Authentication successful"
    );

    Ok(tokens)
}
