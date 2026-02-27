//! Policy-gated mutation variants of universal X API requests.
//!
//! Wraps the raw HTTP execution with the unified mutation gateway
//! (policy + idempotency + audit) so that every `x_post`, `x_put`, and
//! `x_delete` via the admin server is traceable and policy-checked.

use std::time::Instant;

use serde_json::Value;

use crate::contract::error::provider_error_to_response;
use crate::provider::retry::{with_retry, RetryPolicy};
use crate::provider::x_api::map_x_error;
use crate::state::SharedState;
use crate::tools::response::ToolResponse;
use crate::tools::workflow::policy_gate::{
    complete_gateway_failure, complete_gateway_success, run_gateway, GatewayResult,
};

use super::{
    blocked_response, classify_request_family, extract_rate_limit_meta, validate_and_build_url,
    validate_headers, XRequestResponse,
};

/// Execute a POST request with policy evaluation and mutation audit recording.
///
/// This is the enterprise-safe variant of the raw `x_post`. It runs the
/// unified mutation gateway (policy + idempotency + audit) before executing
/// the HTTP request.
pub async fn x_post_audited(
    state: &SharedState,
    path: &str,
    host: Option<&str>,
    query: Option<&[(String, String)]>,
    body: Option<&str>,
    headers: Option<&[(String, String)]>,
) -> String {
    execute_audited_mutation(state, "POST", "x_post", path, host, query, body, headers).await
}

/// Execute a PUT request with policy evaluation and mutation audit recording.
pub async fn x_put_audited(
    state: &SharedState,
    path: &str,
    host: Option<&str>,
    query: Option<&[(String, String)]>,
    body: Option<&str>,
    headers: Option<&[(String, String)]>,
) -> String {
    execute_audited_mutation(state, "PUT", "x_put", path, host, query, body, headers).await
}

/// Execute a DELETE request with policy evaluation and mutation audit recording.
pub async fn x_delete_audited(
    state: &SharedState,
    path: &str,
    host: Option<&str>,
    query: Option<&[(String, String)]>,
    headers: Option<&[(String, String)]>,
) -> String {
    execute_audited_mutation(
        state, "DELETE", "x_delete", path, host, query, None, headers,
    )
    .await
}

/// Unified audited mutation execution for universal request tools.
///
/// Validates host/path/headers, runs the mutation gateway (policy + dedup +
/// audit), executes the HTTP request, and completes the audit trail.
#[allow(clippy::too_many_arguments)]
async fn execute_audited_mutation(
    state: &SharedState,
    method: &str,
    tool_name: &str,
    path: &str,
    host: Option<&str>,
    query: Option<&[(String, String)]>,
    body: Option<&str>,
    headers: Option<&[(String, String)]>,
) -> String {
    let start = Instant::now();

    // 1. Validate URL before touching the gateway.
    let url = match validate_and_build_url(host, path) {
        Ok(u) => u,
        Err(msg) => return blocked_response(&msg, start),
    };

    // 2. Validate headers.
    if let Some(h) = headers {
        if let Err(msg) = validate_headers(h) {
            return blocked_response(&msg, start);
        }
    }

    // 3. Build a params summary for audit (includes family classification).
    let family = classify_request_family(host, path);
    let params_json = serde_json::json!({
        "method": method,
        "path": path,
        "host": host.unwrap_or("api.x.com"),
        "family": family.to_string(),
        "body_preview": body.map(|b| {
            if b.len() > 200 { format!("{}…", &b[..200]) } else { b.to_string() }
        }),
    })
    .to_string();

    // 4. Run the mutation gateway (policy + idempotency + audit).
    let ticket = match run_gateway(state, tool_name, &params_json, start).await {
        GatewayResult::Proceed(t) => t,
        GatewayResult::EarlyReturn(r) => return r,
    };

    // 5. Ensure X client is available.
    let client = match state.x_client.as_ref() {
        Some(c) => c.as_ref(),
        None => {
            let _ = complete_gateway_failure(state, &ticket, "X API client not configured", start)
                .await;
            return crate::tools::workflow::x_actions::not_configured_response(start);
        }
    };

    // 6. Execute the HTTP request with retry.
    let retry_policy = RetryPolicy::default();
    let (result, retry_count) = with_retry(&retry_policy, || async {
        client
            .raw_request(method, &url, query, body, headers)
            .await
            .map_err(|e| map_x_error(&e))
    })
    .await;

    match result {
        Ok(raw) => {
            let result_data = serde_json::json!({
                "status": raw.status,
                "family": family.to_string(),
                "path": path,
            });
            let mut meta = complete_gateway_success(state, &ticket, &result_data, start).await;
            if retry_count > 0 {
                meta = meta.with_retry_count(retry_count);
            }

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

            ToolResponse::success(resp_data).with_meta(meta).to_json()
        }
        Err(e) => {
            let msg = e.to_string();
            let meta = complete_gateway_failure(state, &ticket, &msg, start).await;
            let mut resp = provider_error_to_response(&e, start);
            // Inject correlation_id and retry count.
            if let Ok(mut v) = serde_json::from_str::<Value>(&resp) {
                if let Some(m) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
                    m.insert(
                        "correlation_id".into(),
                        Value::String(ticket.correlation_id.clone()),
                    );
                    if retry_count > 0 {
                        m.insert("retry_count".into(), retry_count.into());
                    }
                }
                if let Ok(patched) = serde_json::to_string_pretty(&v) {
                    resp = patched;
                }
            }
            let _ = meta; // already used above for gateway recording
            resp
        }
    }
}
