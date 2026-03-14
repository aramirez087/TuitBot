//! Startup resolution of GraphQL query IDs and transaction ID generator.
//!
//! Fetches X's homepage and JS bundles to auto-detect the current
//! query IDs for all GraphQL operations. Also initializes the
//! `ClientTransaction` for generating valid `x-client-transaction-id` headers.

use std::collections::HashMap;

use crate::x_client_transaction::ClientTransaction;

use super::{FALLBACK_CREATE_TWEET_QUERY_ID, OPERATION_NAMES};

/// Result of startup resolution: query IDs + transaction generator.
pub(crate) struct ResolvedTransport {
    /// All resolved GraphQL query IDs keyed by operation name.
    pub(crate) query_ids: HashMap<String, String>,
    /// Transaction ID generator (extracted from X's homepage).
    pub(crate) transaction: Option<ClientTransaction>,
}

/// Resolve all query IDs and initialize the transaction ID generator.
pub(crate) async fn resolve_transport() -> ResolvedTransport {
    let query_ids = resolve_query_ids().await;
    let transaction = resolve_client_transaction().await;

    ResolvedTransport {
        query_ids,
        transaction,
    }
}

/// Resolve all GraphQL query IDs from X's JS bundles in a single pass.
async fn resolve_query_ids() -> HashMap<String, String> {
    let mut ids = HashMap::new();

    // Env-var override for CreateTweet always wins.
    if let Ok(id) = std::env::var("TUITBOT_CREATE_TWEET_QUERY_ID") {
        tracing::info!(query_id = %id, "Using CreateTweet query ID from env var");
        ids.insert("CreateTweet".to_string(), id);
    }

    match detect_all_query_ids_from_bundles().await {
        Ok(detected) => {
            let count = detected.len();
            for (op, id) in detected {
                // Don't overwrite env-var override
                ids.entry(op).or_insert(id);
            }
            tracing::info!(count, "Auto-detected GraphQL query IDs from X web client");
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to auto-detect query IDs from X web client"
            );
        }
    }

    // Ensure CreateTweet always has a fallback.
    ids.entry("CreateTweet".to_string()).or_insert_with(|| {
        tracing::warn!(
            fallback = FALLBACK_CREATE_TWEET_QUERY_ID,
            "Using fallback CreateTweet query ID"
        );
        FALLBACK_CREATE_TWEET_QUERY_ID.to_string()
    });

    ids
}

/// Initialize the `ClientTransaction` for generating valid transaction IDs.
async fn resolve_client_transaction() -> Option<ClientTransaction> {
    match tokio::task::spawn_blocking(|| {
        let client = reqwest::blocking::Client::builder()
            .user_agent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
                 AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/134.0.0.0 Safari/537.36",
            )
            .build()
            .ok()?;
        ClientTransaction::new(&client).ok()
    })
    .await
    {
        Ok(Some(ct)) => {
            tracing::info!("Initialized x-client-transaction-id generator");
            Some(ct)
        }
        Ok(None) => {
            tracing::warn!(
                "Failed to initialize x-client-transaction-id generator; \
                 requests will omit the header"
            );
            None
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "spawn_blocking for ClientTransaction panicked"
            );
            None
        }
    }
}

/// Fetch X's homepage, find JS bundle URLs, and extract query IDs for all operations.
async fn detect_all_query_ids_from_bundles() -> Result<HashMap<String, String>, String> {
    let client = rquest::Client::builder()
        .emulation(rquest_util::Emulation::Chrome134)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let html = client
        .get("https://x.com")
        .send()
        .await
        .map_err(|e| format!("failed to fetch x.com: {e}"))?
        .text()
        .await
        .map_err(|e| format!("failed to read x.com response: {e}"))?;

    let script_urls = extract_script_urls(&html);

    if script_urls.is_empty() {
        return Err("no JS bundle URLs found in x.com HTML".to_string());
    }

    tracing::debug!(count = script_urls.len(), "Found JS bundle URLs to scan");

    let mut all_ids = HashMap::new();

    for url in &script_urls {
        let js = match client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        // Scan for all operations in a single pass through this bundle.
        for &op in OPERATION_NAMES {
            if all_ids.contains_key(op) {
                continue;
            }
            if let Some(id) = extract_query_id_for_operation(&js, op) {
                all_ids.insert(op.to_string(), id);
            }
        }

        // Early exit if we found all operations.
        if all_ids.len() == OPERATION_NAMES.len() {
            break;
        }
    }

    if all_ids.is_empty() {
        return Err("no query IDs found in any JS bundle".to_string());
    }

    Ok(all_ids)
}

/// Extract `<script src="...">` URLs from HTML that look like X's JS bundles.
pub(crate) fn extract_script_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for segment in html.split("src=\"") {
        if let Some(end) = segment.find('"') {
            let url = &segment[..end];
            if url.ends_with(".js") && (url.contains("twimg.com") || url.contains("x.com")) {
                urls.push(url.to_string());
            }
        }
    }
    urls
}

/// Search a JS bundle for a GraphQL operation and extract its query ID.
pub(crate) fn extract_query_id_for_operation(js: &str, operation: &str) -> Option<String> {
    let op_pattern = format!("\"{}\"", operation);

    for (idx, _) in js.match_indices(&op_pattern) {
        let start = idx.saturating_sub(200);
        let window = &js[start..idx];

        if let Some(id) = extract_query_id_value(window) {
            return Some(id);
        }

        let end = (idx + op_pattern.len() + 200).min(js.len());
        let window = &js[idx..end];

        if let Some(id) = extract_query_id_value(window) {
            return Some(id);
        }
    }
    None
}

/// Extract a queryId value from a JS snippet like `queryId:"abc123"`.
pub(crate) fn extract_query_id_value(snippet: &str) -> Option<String> {
    let marker = "queryId:\"";
    let pos = snippet.rfind(marker)?;
    let after = &snippet[pos + marker.len()..];
    let end = after.find('"')?;
    let id = &after[..end];
    if !id.is_empty()
        && id.len() < 64
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Some(id.to_string())
    } else {
        None
    }
}
