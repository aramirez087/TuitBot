//! Tests for the universal X API request layer.
//!
//! Covers: host allowlist, header blocklist, path validation, SSRF guards,
//! retry behavior, pagination, JSON/non-JSON response handling, and
//! request family classification.

use super::*;

// ── Host allowlist tests ────────────────────────────────────────────

#[test]
fn allowed_hosts_accepted() {
    for host in &[
        "api.x.com",
        "upload.x.com",
        "upload.twitter.com",
        "ads-api.x.com",
    ] {
        let result = validate_and_build_url(Some(host), "/2/tweets");
        assert!(result.is_ok(), "expected {host} to be allowed");
        assert_eq!(result.unwrap(), format!("https://{host}/2/tweets"));
    }
}

#[test]
fn ads_api_host_accepted() {
    let result = validate_and_build_url(Some("ads-api.x.com"), "/12/accounts");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://ads-api.x.com/12/accounts");
}

#[test]
fn ads_api_host_case_insensitive() {
    let result = validate_and_build_url(Some("ADS-API.X.COM"), "/12/accounts");
    assert!(result.is_ok());
}

#[test]
fn default_host_is_api_x_com() {
    let result = validate_and_build_url(None, "/2/tweets");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://api.x.com/2/tweets");
}

#[test]
fn blocked_host_rejected() {
    let cases = [
        "evil.com",
        "api.twitter.com",
        "example.org",
        "localhost",
        "internal.corp",
        "api.x.com.evil.com",
    ];
    for host in cases {
        let result = validate_and_build_url(Some(host), "/2/tweets");
        assert!(result.is_err(), "expected {host} to be blocked");
        assert!(
            result.unwrap_err().contains("not in the allowlist"),
            "error should mention allowlist"
        );
    }
}

#[test]
fn host_case_insensitive() {
    let result = validate_and_build_url(Some("API.X.COM"), "/2/tweets");
    assert!(result.is_ok());
}

// ── SSRF guards ─────────────────────────────────────────────────────

#[test]
fn ipv4_literal_blocked() {
    let result = validate_and_build_url(Some("127.0.0.1"), "/2/tweets");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("IP-literal"));
}

#[test]
fn ipv6_literal_blocked() {
    let result = validate_and_build_url(Some("::1"), "/2/tweets");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("IP-literal"));
}

#[test]
fn ipv6_bracket_literal_blocked() {
    let result = validate_and_build_url(Some("[::1]"), "/2/tweets");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("IP-literal"));
}

#[test]
fn private_ipv4_blocked() {
    let result = validate_and_build_url(Some("10.0.0.1"), "/2/tweets");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("IP-literal"));
}

// ── Path validation ─────────────────────────────────────────────────

#[test]
fn valid_paths_accepted() {
    let paths = [
        "/2/tweets",
        "/2/tweets/123",
        "/2/users/by/username/elonmusk",
        "/1.1/media/upload.json",
        "/2/tweets/search/recent",
    ];
    for path in paths {
        assert!(validate_path(path).is_ok(), "expected {path} to be valid");
    }
}

#[test]
fn empty_path_rejected() {
    assert!(validate_path("").is_err());
}

#[test]
fn path_without_leading_slash_rejected() {
    assert!(validate_path("2/tweets").is_err());
}

#[test]
fn path_traversal_rejected() {
    assert!(validate_path("/2/tweets/../../../etc/passwd").is_err());
    assert!(validate_path("/2/..").is_err());
}

#[test]
fn path_with_query_rejected() {
    assert!(validate_path("/2/tweets?id=123").is_err());
}

#[test]
fn path_with_fragment_rejected() {
    assert!(validate_path("/2/tweets#section").is_err());
}

#[test]
fn path_with_control_chars_rejected() {
    assert!(validate_path("/2/tweets\x00").is_err());
    assert!(validate_path("/2/tweets\n").is_err());
}

// ── Header validation ───────────────────────────────────────────────

#[test]
fn allowed_headers_accepted() {
    let headers = vec![
        ("Accept".to_string(), "application/json".to_string()),
        ("X-Custom".to_string(), "value".to_string()),
    ];
    assert!(validate_headers(&headers).is_ok());
}

#[test]
fn empty_headers_accepted() {
    assert!(validate_headers(&[]).is_ok());
}

#[test]
fn authorization_header_blocked() {
    let headers = vec![("Authorization".to_string(), "Bearer token".to_string())];
    let err = validate_headers(&headers).unwrap_err();
    assert!(err.contains("Authorization"));
}

#[test]
fn host_header_blocked() {
    let headers = vec![("Host".to_string(), "evil.com".to_string())];
    let err = validate_headers(&headers).unwrap_err();
    assert!(err.contains("Host"));
}

#[test]
fn cookie_header_blocked() {
    let headers = vec![("Cookie".to_string(), "session=abc".to_string())];
    let err = validate_headers(&headers).unwrap_err();
    assert!(err.contains("Cookie"));
}

#[test]
fn transfer_encoding_header_blocked() {
    let headers = vec![("Transfer-Encoding".to_string(), "chunked".to_string())];
    let err = validate_headers(&headers).unwrap_err();
    assert!(err.contains("Transfer-Encoding"));
}

#[test]
fn header_check_case_insensitive() {
    let headers = vec![("authorization".to_string(), "Bearer x".to_string())];
    assert!(validate_headers(&headers).is_err());

    let headers = vec![("AUTHORIZATION".to_string(), "Bearer x".to_string())];
    assert!(validate_headers(&headers).is_err());
}

#[test]
fn multiple_blocked_headers_all_reported() {
    let headers = vec![
        ("Authorization".to_string(), "Bearer x".to_string()),
        ("Cookie".to_string(), "a=b".to_string()),
    ];
    let err = validate_headers(&headers).unwrap_err();
    assert!(err.contains("Authorization"));
    assert!(err.contains("Cookie"));
}

// ── Response building ───────────────────────────────────────────────

#[test]
fn build_success_response_parses_json() {
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h
        },
        body: r#"{"data":{"id":"123","text":"hello"}}"#.to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 0, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["status"], 200);
    assert_eq!(parsed["data"]["json"]["data"]["id"], "123");
    assert!(!parsed["data"]["body_text"].as_str().unwrap().is_empty());
}

#[test]
fn build_success_response_non_json_body() {
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "text/plain".to_string());
            h
        },
        body: "plain text response".to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 0, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["success"], true);
    assert!(parsed["data"]["json"].is_null());
    assert_eq!(parsed["data"]["body_text"], "plain text response");
}

#[test]
fn build_success_response_with_retries() {
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 200,
        headers: HashMap::new(),
        body: "{}".to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 2, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["meta"]["retry_count"], 2);
}

#[test]
fn build_success_response_without_retries_omits_count() {
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 200,
        headers: HashMap::new(),
        body: "{}".to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 0, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["meta"].get("retry_count").is_none());
}

// ── Rate limit metadata ─────────────────────────────────────────────

#[test]
fn extract_rate_limit_meta_from_headers() {
    let mut headers = HashMap::new();
    headers.insert("x-rate-limit-remaining".to_string(), "50".to_string());
    headers.insert("x-rate-limit-reset".to_string(), "1700000000".to_string());

    let meta = extract_rate_limit_meta(&headers);
    assert!(meta.is_some());
    let meta = meta.unwrap();
    assert_eq!(meta.remaining, Some(50));
    assert_eq!(meta.reset_at, Some(1700000000));
}

#[test]
fn extract_rate_limit_meta_absent_when_no_headers() {
    let headers = HashMap::new();
    assert!(extract_rate_limit_meta(&headers).is_none());
}

// ── Blocked response shape ──────────────────────────────────────────

#[test]
fn blocked_response_shape() {
    let json_str = blocked_response("test block", Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"]["code"], "x_request_blocked");
    assert!(!parsed["error"]["retryable"].as_bool().unwrap());
    assert!(parsed["error"]["message"]
        .as_str()
        .unwrap()
        .contains("test block"));
}

// ── HTTP error status preserved ─────────────────────────────────────

#[test]
fn http_error_status_returned_as_success_tool_response() {
    // When the HTTP call succeeds but returns 4xx/5xx, the tool still
    // returns success=true because the *tool* worked — the caller
    // inspects the status field.
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 404,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h
        },
        body: r#"{"detail":"Not Found"}"#.to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 0, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["data"]["status"], 404);
    assert_eq!(parsed["data"]["json"]["detail"], "Not Found");
}

// ── Rate limit in response data ─────────────────────────────────────

#[test]
fn rate_limit_in_response_data() {
    let raw = tuitbot_core::x_api::types::RawApiResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h.insert("x-rate-limit-remaining".to_string(), "99".to_string());
            h.insert("x-rate-limit-reset".to_string(), "1700000000".to_string());
            h
        },
        body: "{}".to_string(),
        rate_limit: None,
    };
    let json_str = build_success_response(raw, 0, Instant::now());
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["data"]["rate_limit"]["remaining"], 99);
    assert_eq!(parsed["data"]["rate_limit"]["reset_at"], 1700000000);
}

// ── Request family classification ───────────────────────────────────

#[test]
fn classify_public_api_default_host() {
    assert_eq!(
        classify_request_family(None, "/2/tweets"),
        RequestFamily::PublicApi
    );
}

#[test]
fn classify_public_api_explicit_host() {
    assert_eq!(
        classify_request_family(Some("api.x.com"), "/2/tweets/123"),
        RequestFamily::PublicApi
    );
}

#[test]
fn classify_dm_conversations() {
    assert_eq!(
        classify_request_family(Some("api.x.com"), "/2/dm_conversations"),
        RequestFamily::DirectMessage
    );
    assert_eq!(
        classify_request_family(None, "/2/dm_conversations/123/messages"),
        RequestFamily::DirectMessage
    );
}

#[test]
fn classify_dm_events() {
    assert_eq!(
        classify_request_family(None, "/2/dm_events"),
        RequestFamily::DirectMessage
    );
    assert_eq!(
        classify_request_family(Some("api.x.com"), "/2/dm_conversations/with/456/dm_events"),
        RequestFamily::DirectMessage
    );
}

#[test]
fn classify_ads_api() {
    assert_eq!(
        classify_request_family(Some("ads-api.x.com"), "/12/accounts"),
        RequestFamily::Ads
    );
    assert_eq!(
        classify_request_family(Some("ads-api.x.com"), "/12/accounts/abc/campaigns"),
        RequestFamily::Ads
    );
}

#[test]
fn classify_ads_api_case_insensitive() {
    assert_eq!(
        classify_request_family(Some("ADS-API.X.COM"), "/12/accounts"),
        RequestFamily::Ads
    );
}

#[test]
fn classify_compliance() {
    assert_eq!(
        classify_request_family(None, "/2/compliance/jobs"),
        RequestFamily::EnterpriseAdmin
    );
    assert_eq!(
        classify_request_family(None, "/2/compliance/jobs/123"),
        RequestFamily::EnterpriseAdmin
    );
}

#[test]
fn classify_usage() {
    assert_eq!(
        classify_request_family(None, "/2/usage/tweets"),
        RequestFamily::EnterpriseAdmin
    );
}

#[test]
fn classify_media_upload() {
    assert_eq!(
        classify_request_family(Some("upload.x.com"), "/1.1/media/upload.json"),
        RequestFamily::MediaUpload
    );
    assert_eq!(
        classify_request_family(Some("upload.twitter.com"), "/1.1/media/upload.json"),
        RequestFamily::MediaUpload
    );
}

#[test]
fn classify_lists_as_public_api() {
    assert_eq!(
        classify_request_family(None, "/2/lists/123"),
        RequestFamily::PublicApi
    );
}

#[test]
fn request_family_display() {
    assert_eq!(RequestFamily::PublicApi.to_string(), "public_api");
    assert_eq!(RequestFamily::DirectMessage.to_string(), "direct_message");
    assert_eq!(RequestFamily::Ads.to_string(), "ads");
    assert_eq!(
        RequestFamily::EnterpriseAdmin.to_string(),
        "enterprise_admin"
    );
    assert_eq!(RequestFamily::MediaUpload.to_string(), "media_upload");
}

#[test]
fn request_family_serializes_to_snake_case() {
    let json = serde_json::to_string(&RequestFamily::DirectMessage).unwrap();
    assert_eq!(json, "\"direct_message\"");
    let json = serde_json::to_string(&RequestFamily::EnterpriseAdmin).unwrap();
    assert_eq!(json, "\"enterprise_admin\"");
}

// ── Enterprise host with path restrictions still enforced ───────────

#[test]
fn ads_host_still_validates_path() {
    // Path traversal must be blocked even on enterprise hosts.
    let result = validate_and_build_url(Some("ads-api.x.com"), "/12/../etc/passwd");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("traversal"));
}

#[test]
fn ads_host_query_in_path_blocked() {
    let result = validate_and_build_url(Some("ads-api.x.com"), "/12/accounts?id=1");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("query"));
}

#[test]
fn ads_host_control_chars_blocked() {
    let result = validate_and_build_url(Some("ads-api.x.com"), "/12/accounts\x00");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("control"));
}
