//! Universal X API request layer.
//!
//! Provides `x_get`, `x_post`, `x_put`, and `x_delete` — a small family of
//! MCP tools that let an AI agent call any authorized X API endpoint safely
//! and predictably, without needing a dedicated tool per endpoint.
//!
//! Safety constraints enforced by construction:
//! - Hard host allowlist: `api.x.com`, `upload.x.com`, `upload.twitter.com`,
//!   `ads-api.x.com` (enterprise Ads API)
//! - Path validation: must start with `/`, no `..` traversal, no query in path
//! - Header blocklist: `authorization`, `host`, `cookie`, `transfer-encoding`
//! - SSRF protection: reject non-HTTPS schemes, reject IP-literal hosts
//! - Mutation requests (`x_post`, `x_put`, `x_delete`) are policy-gated and
//!   audit-recorded via the unified mutation gateway

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contract::error::provider_error_to_response;
use crate::contract::PaginationInfo;
use crate::provider::retry::{with_retry, RetryPolicy};
use crate::provider::x_api::map_x_error;
use crate::state::SharedState;
use crate::tools::response::{ErrorCode, ToolMeta, ToolResponse};

// ── Host allowlist ──────────────────────────────────────────────────

/// Hard-coded allowlist of X API hosts. Requests to any other host are blocked.
const ALLOWED_HOSTS: &[&str] = &[
    "api.x.com",
    "upload.x.com",
    "upload.twitter.com",
    "ads-api.x.com", // Enterprise Ads API
];

/// Headers that callers may not override (case-insensitive).
const BLOCKED_HEADERS: &[&str] = &[
    "authorization",
    "host",
    "cookie",
    "set-cookie",
    "transfer-encoding",
    "proxy-authorization",
    "proxy-connection",
];

/// Maximum number of auto-paginated pages to fetch in a single call.
const MAX_AUTO_PAGES: u32 = 10;

// ── Request family classification (in `family.rs`) ───────────────────

mod family;
pub(crate) use family::classify_request_family;
#[cfg(test)]
pub(crate) use family::RequestFamily;

// ── Validation ──────────────────────────────────────────────────────

/// Validate that a path is safe for the X API.
///
/// Returns `Ok(())` or an error message describing the violation.
pub(crate) fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("path must not be empty".to_string());
    }
    if !path.starts_with('/') {
        return Err("path must start with '/'".to_string());
    }
    if path.contains("..") {
        return Err("path must not contain '..' traversal".to_string());
    }
    if path.contains('?') || path.contains('#') {
        return Err(
            "path must not contain query or fragment (use the query parameter)".to_string(),
        );
    }
    // Block paths with control characters.
    if path.chars().any(|c| c.is_control()) {
        return Err("path must not contain control characters".to_string());
    }
    Ok(())
}

/// Validate that a URL targets an allowed X API host.
///
/// Returns `Ok(full_url)` or an error message if the host is not allowed.
pub(crate) fn validate_and_build_url(host: Option<&str>, path: &str) -> Result<String, String> {
    validate_path(path)?;

    let effective_host = host.unwrap_or("api.x.com");

    // Block IP-literal hosts (SSRF vector).
    if effective_host.parse::<std::net::Ipv4Addr>().is_ok()
        || effective_host.parse::<std::net::Ipv6Addr>().is_ok()
        || effective_host.starts_with('[')
    {
        return Err(format!("IP-literal hosts are blocked: {effective_host}"));
    }

    // Check the allowlist.
    let host_lower = effective_host.to_ascii_lowercase();
    if !ALLOWED_HOSTS.contains(&host_lower.as_str()) {
        return Err(format!(
            "host '{effective_host}' is not in the allowlist. \
             Allowed: {}",
            ALLOWED_HOSTS.join(", ")
        ));
    }

    Ok(format!("https://{effective_host}{path}"))
}

/// Validate caller-supplied headers against the blocklist.
///
/// Returns `Ok(())` or an error listing the blocked headers.
pub(crate) fn validate_headers(headers: &[(String, String)]) -> Result<(), String> {
    let mut blocked = Vec::new();
    for (k, _) in headers {
        let lower = k.to_ascii_lowercase();
        if BLOCKED_HEADERS.contains(&lower.as_str()) {
            blocked.push(k.clone());
        }
    }
    if blocked.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "caller may not set restricted headers: {}",
            blocked.join(", ")
        ))
    }
}

// ── Structured response ─────────────────────────────────────────────

/// Structured response from a universal X API request.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct XRequestResponse {
    /// HTTP status code.
    pub status: u16,
    /// Selected response headers.
    pub headers: HashMap<String, String>,
    /// Parsed JSON body (if Content-Type is application/json), else null.
    pub json: Value,
    /// Raw body text (always populated as fallback).
    pub body_text: String,
    /// Rate limit metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitMeta>,
}

/// Rate limit metadata extracted from response headers.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RateLimitMeta {
    /// Remaining requests in the current window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u64>,
    /// Unix epoch second when the window resets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<u64>,
    /// Recommended wait in milliseconds (computed from reset_at - now).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_wait_ms: Option<u64>,
}

// ── Tool implementations ────────────────────────────────────────────

/// Bundled parameters for `execute_request` (avoids clippy::too_many_arguments).
struct RequestParams<'a> {
    method: &'a str,
    path: &'a str,
    host: Option<&'a str>,
    query: Option<&'a [(String, String)]>,
    body: Option<&'a str>,
    headers: Option<&'a [(String, String)]>,
    auto_paginate: bool,
    max_pages: Option<u32>,
}

/// Execute a GET request against the X API.
pub async fn x_get(
    state: &SharedState,
    path: &str,
    host: Option<&str>,
    query: Option<&[(String, String)]>,
    headers: Option<&[(String, String)]>,
    auto_paginate: bool,
    max_pages: Option<u32>,
) -> String {
    execute_request(
        state,
        RequestParams {
            method: "GET",
            path,
            host,
            query,
            body: None,
            headers,
            auto_paginate,
            max_pages,
        },
    )
    .await
}

// ── Policy-gated mutation variants (in `audited.rs`) ─────────────────

mod audited;
pub use audited::{x_delete_audited, x_post_audited, x_put_audited};

/// Core request execution with validation, retry, and optional pagination.
async fn execute_request(state: &SharedState, params: RequestParams<'_>) -> String {
    let start = Instant::now();
    let RequestParams {
        method,
        path,
        host,
        query,
        body,
        headers,
        auto_paginate,
        max_pages,
    } = params;

    // 1. Validate URL (host allowlist + path safety).
    let url = match validate_and_build_url(host, path) {
        Ok(u) => u,
        Err(msg) => {
            return blocked_response(&msg, start);
        }
    };

    // 2. Validate headers (blocklist check).
    if let Some(h) = headers {
        if let Err(msg) = validate_headers(h) {
            return blocked_response(&msg, start);
        }
    }

    // 3. Ensure X client is available.
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => return super::not_configured_response(start),
    };

    // 4. Execute with retry.
    if auto_paginate && method == "GET" {
        return execute_paginated(
            client,
            &url,
            query,
            headers,
            max_pages.unwrap_or(MAX_AUTO_PAGES).min(MAX_AUTO_PAGES),
            start,
        )
        .await;
    }

    let retry_policy = RetryPolicy::default();
    let (result, retry_count) = with_retry(&retry_policy, || async {
        client
            .raw_request(method, &url, query, body, headers)
            .await
            .map_err(|e| map_x_error(&e))
    })
    .await;

    match result {
        Ok(raw) => build_success_response(raw, retry_count, start),
        Err(e) => {
            let mut resp = provider_error_to_response(&e, start);
            // Inject retry count into the JSON if retries happened.
            if retry_count > 0 {
                if let Ok(mut v) = serde_json::from_str::<Value>(&resp) {
                    if let Some(meta) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
                        meta.insert("retry_count".into(), retry_count.into());
                    }
                    if let Ok(patched) = serde_json::to_string_pretty(&v) {
                        resp = patched;
                    }
                }
            }
            resp
        }
    }
}

/// Execute paginated GET, collecting pages until `next_token` is absent or
/// `max_pages` is reached.
async fn execute_paginated(
    client: &dyn tuitbot_core::x_api::XApiClient,
    base_url: &str,
    base_query: Option<&[(String, String)]>,
    headers: Option<&[(String, String)]>,
    max_pages: u32,
    start: Instant,
) -> String {
    let retry_policy = RetryPolicy::default();
    let mut all_pages: Vec<Value> = Vec::new();
    let mut next_token: Option<String> = None;
    let mut total_retries: u32 = 0;
    let mut last_rate_limit: Option<RateLimitMeta> = None;

    for page_num in 0..max_pages {
        // Build query with pagination token if we have one.
        let mut query_vec: Vec<(String, String)> = base_query
            .unwrap_or(&[])
            .iter()
            .filter(|(k, _)| k != "pagination_token" && k != "next_token")
            .cloned()
            .collect();
        if let Some(ref token) = next_token {
            query_vec.push(("pagination_token".to_string(), token.clone()));
        }

        let query_slice: Vec<(String, String)> = query_vec;
        let (result, retries) = with_retry(&retry_policy, || async {
            client
                .raw_request("GET", base_url, Some(&query_slice), None, headers)
                .await
                .map_err(|e| map_x_error(&e))
        })
        .await;
        total_retries += retries;

        match result {
            Ok(raw) => {
                last_rate_limit = extract_rate_limit_meta(&raw.headers);
                let body_json: Value = serde_json::from_str(&raw.body).unwrap_or(Value::Null);

                // Extract next_token from the response meta.
                next_token = body_json
                    .get("meta")
                    .and_then(|m| m.get("next_token"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());

                all_pages.push(serde_json::json!({
                    "page": page_num + 1,
                    "status": raw.status,
                    "data": body_json,
                }));

                // Stop if no next page.
                if next_token.is_none() {
                    break;
                }
            }
            Err(e) => {
                // Return what we have plus the error for the failed page.
                let elapsed = start.elapsed().as_millis() as u64;
                let pagination = PaginationInfo {
                    next_token: next_token.clone(),
                    result_count: all_pages.len() as u32,
                    has_more: next_token.is_some(),
                };
                let meta = ToolMeta::new(elapsed)
                    .with_retry_count(total_retries)
                    .with_pagination(pagination);

                return ToolResponse::error(
                    e.error_code(),
                    format!(
                        "Pagination stopped at page {} of {}: {}",
                        page_num + 1,
                        max_pages,
                        e
                    ),
                )
                .with_meta(meta)
                .to_json();
            }
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    let pagination = PaginationInfo {
        next_token: next_token.clone(),
        result_count: all_pages.len() as u32,
        has_more: next_token.is_some(),
    };
    let mut meta = ToolMeta::new(elapsed).with_pagination(pagination);
    if total_retries > 0 {
        meta = meta.with_retry_count(total_retries);
    }

    let mut data = serde_json::json!({
        "pages": all_pages,
        "total_pages": all_pages.len(),
    });
    if let Some(rl) = last_rate_limit {
        data["rate_limit"] = serde_json::to_value(rl).unwrap_or(Value::Null);
    }

    ToolResponse::success(data).with_meta(meta).to_json()
}

// ── Helpers ─────────────────────────────────────────────────────────

fn blocked_response(message: &str, start: Instant) -> String {
    let elapsed = start.elapsed().as_millis() as u64;
    ToolResponse::error(ErrorCode::XRequestBlocked, message)
        .with_meta(ToolMeta::new(elapsed))
        .to_json()
}

fn build_success_response(
    raw: tuitbot_core::x_api::types::RawApiResponse,
    retry_count: u32,
    start: Instant,
) -> String {
    let is_json = raw
        .headers
        .get("content-type")
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false);

    let json_body = if is_json {
        serde_json::from_str(&raw.body).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    let rate_limit = extract_rate_limit_meta(&raw.headers);

    let resp_data = XRequestResponse {
        status: raw.status,
        headers: raw.headers,
        json: json_body,
        body_text: raw.body,
        rate_limit,
    };

    let elapsed = start.elapsed().as_millis() as u64;
    let mut meta = ToolMeta::new(elapsed);
    if retry_count > 0 {
        meta = meta.with_retry_count(retry_count);
    }

    // If the HTTP status indicates an error, still return success=true
    // because the *tool* succeeded — the caller interprets the status.
    ToolResponse::success(resp_data).with_meta(meta).to_json()
}

fn extract_rate_limit_meta(headers: &HashMap<String, String>) -> Option<RateLimitMeta> {
    let remaining = headers
        .get("x-rate-limit-remaining")
        .and_then(|v| v.parse::<u64>().ok());
    let reset_at = headers
        .get("x-rate-limit-reset")
        .and_then(|v| v.parse::<u64>().ok());

    if remaining.is_none() && reset_at.is_none() {
        return None;
    }

    let recommended_wait_ms = reset_at.and_then(|reset| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        reset.checked_sub(now).map(|s| s * 1000)
    });

    Some(RateLimitMeta {
        remaining,
        reset_at,
        recommended_wait_ms,
    })
}

#[cfg(test)]
mod tests;
