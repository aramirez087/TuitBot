//! Tests for StartupError, PKCE generation, URL building, and startup banner.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

use crate::startup::config::{ApiTier, StartupError, TierCapabilities};
use crate::startup::services::{
    build_auth_url, build_redirect_uri, extract_auth_code, format_startup_banner, generate_pkce,
    url_encode, X_AUTH_URL,
};

// ============================================================================
// StartupError
// ============================================================================

#[test]
fn startup_error_display() {
    let err = StartupError::AuthRequired;
    assert_eq!(
        err.to_string(),
        "authentication required: run `tuitbot auth` first"
    );

    let err = StartupError::AuthExpired;
    assert!(err.to_string().contains("expired"));

    let err = StartupError::Config("bad field".to_string());
    assert_eq!(err.to_string(), "configuration error: bad field");

    let err = StartupError::XApiError("timeout".to_string());
    assert_eq!(err.to_string(), "X API error: timeout");
}

// ============================================================================
// PKCE
// ============================================================================

#[test]
fn generate_pkce_produces_valid_challenge() {
    let pkce = generate_pkce();
    assert_eq!(pkce.verifier.len(), 43);
    assert_eq!(pkce.challenge.len(), 43);
    assert_eq!(pkce.state.len(), 22);
    let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.verifier.as_bytes()));
    assert_eq!(pkce.challenge, expected);
}

#[test]
fn generate_pkce_unique_each_time() {
    let a = generate_pkce();
    let b = generate_pkce();
    assert_ne!(a.verifier, b.verifier);
    assert_ne!(a.challenge, b.challenge);
    assert_ne!(a.state, b.state);
}

// ============================================================================
// URL Building
// ============================================================================

#[test]
fn build_auth_url_contains_required_params() {
    let url = build_auth_url(
        "client123",
        "http://localhost:8080/callback",
        "state456",
        "challenge789",
    );
    assert!(url.starts_with(X_AUTH_URL));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("client_id=client123"));
    assert!(url.contains("code_challenge=challenge789"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=state456"));
    assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
}

#[test]
fn build_redirect_uri_format() {
    let uri = build_redirect_uri("127.0.0.1", 8080);
    assert_eq!(uri, "http://127.0.0.1:8080/callback");
}

// ============================================================================
// extract_auth_code
// ============================================================================

#[test]
fn extract_code_from_full_url() {
    let code = extract_auth_code("http://127.0.0.1:8080/callback?code=abc123&state=xyz");
    assert_eq!(code, "abc123");
}

#[test]
fn extract_code_from_bare_code() {
    let code = extract_auth_code("  abc123  ");
    assert_eq!(code, "abc123");
}

#[test]
fn extract_code_from_url_without_state() {
    let code = extract_auth_code("http://127.0.0.1:8080/callback?code=mycode");
    assert_eq!(code, "mycode");
}

// ============================================================================
// URL Encoding
// ============================================================================

#[test]
fn url_encode_basic() {
    assert_eq!(url_encode("hello"), "hello");
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(
        url_encode("http://localhost:8080/callback"),
        "http%3A%2F%2Flocalhost%3A8080%2Fcallback"
    );
}

// ============================================================================
// Startup Banner
// ============================================================================

#[test]
fn startup_banner_free_tier() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let banner = format_startup_banner(ApiTier::Free, &caps, 300);
    assert!(banner.contains("Tuitbot v"));
    assert!(banner.contains("Tier: Free"));
    assert!(!banner.contains("mentions"));
    assert!(banner.contains("content"));
    assert!(banner.contains("threads"));
    assert!(!banner.contains("discovery"));
    assert!(banner.contains("every 300s"));
}

#[test]
fn startup_banner_basic_tier() {
    let caps = TierCapabilities::for_tier(ApiTier::Basic);
    let banner = format_startup_banner(ApiTier::Basic, &caps, 0);
    assert!(banner.contains("Tier: Basic"));
    assert!(banner.contains("discovery"));
    assert!(banner.contains("disabled"));
}

#[test]
fn startup_banner_contains_ctrl_c_hint() {
    let caps = TierCapabilities::for_tier(ApiTier::Free);
    let banner = format_startup_banner(ApiTier::Free, &caps, 0);
    assert!(banner.contains("Ctrl+C"));
}

// ============================================================================
// StartupError variants
// ============================================================================

#[test]
fn startup_error_all_variants_display() {
    let errors = vec![
        StartupError::Config("bad".to_string()),
        StartupError::AuthRequired,
        StartupError::AuthExpired,
        StartupError::TokenRefreshFailed("fail".to_string()),
        StartupError::Database("db err".to_string()),
        StartupError::LlmError("llm err".to_string()),
        StartupError::XApiError("api err".to_string()),
        StartupError::Other("other".to_string()),
    ];
    for err in &errors {
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}

// ============================================================================
// StartupError Io variant
// ============================================================================

#[test]
fn startup_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let startup_err = StartupError::Io(io_err);
    let msg = startup_err.to_string();
    assert!(msg.contains("missing"), "got: {msg}");
}

// ============================================================================
// URL encoding edge cases
// ============================================================================

#[test]
fn url_encode_special_chars() {
    assert_eq!(url_encode("a b+c"), "a%20b%2Bc");
    assert_eq!(url_encode("foo@bar"), "foo%40bar");
    assert_eq!(url_encode("~valid_chars.-"), "~valid_chars.-");
}

#[test]
fn url_encode_empty() {
    assert_eq!(url_encode(""), "");
}
