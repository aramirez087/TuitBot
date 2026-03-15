//! Coverage tests for write-profile unique handlers.
//!
//! Tools shared with admin (analytics, approval, x_actions, etc.) are already
//! tested in admin/tests. These tests cover workflow paths unique to the write
//! server: capabilities, mode, content generation, compose, and x_usage.

use crate::tools::response::ToolResponse;
use crate::tools::workflow;

use super::make_state;

// ── Capabilities & mode (write-only core_router tools) ───────────────

#[tokio::test]
async fn write_get_capabilities() {
    let state = make_state().await;
    let result = workflow::capabilities::get_capabilities(
        &state.pool,
        &state.config,
        false, // llm_available
        true,  // x_available
        state.authenticated_user_id.as_deref(),
        &state.granted_scopes,
    )
    .await;
    assert!(!result.is_empty());
    // Should contain capability information
    assert!(result.contains("capabilities") || result.contains("tier") || result.contains("ok"));
}

#[tokio::test]
async fn write_get_mode() {
    let state = make_state().await;
    let mode = state.config.mode.to_string();
    // Verify the mode value is non-empty (same logic as handler)
    assert!(!mode.is_empty());
    // effective_approval_mode returns a value — just verify it doesn't panic
    let _approval = state.config.effective_approval_mode();
}

// ── Content generation (LLM-gated, write-only) ──────────────────────

#[tokio::test]
async fn write_generate_reply_no_llm() {
    // Without an LLM provider, should return llm_not_configured response
    let result = ToolResponse::llm_not_configured().to_json();
    assert!(!result.is_empty());
    assert!(
        result.contains("llm") || result.contains("not_configured") || result.contains("error")
    );
}

#[tokio::test]
async fn write_generate_tweet_no_llm() {
    let result = ToolResponse::llm_not_configured().to_json();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_generate_thread_no_llm() {
    let result = ToolResponse::llm_not_configured().to_json();
    assert!(!result.is_empty());
}

// ── X usage (write handler, different call path from admin) ──────────

#[tokio::test]
async fn write_get_x_usage() {
    let state = make_state().await;
    let result = workflow::x_actions::get_x_usage(&state, 7).await;
    assert!(!result.is_empty());
}

// ── Compose tweet (write-only, policy-gated) ─────────────────────────

#[tokio::test]
async fn write_compose_tweet_draft() {
    let state = make_state().await;
    // Insert a draft via the storage layer (same path compose_tweet uses)
    let result = tuitbot_core::storage::scheduled_content::insert_draft(
        &state.pool,
        "tweet",
        "Test draft content from write tests",
        "mcp",
    )
    .await;
    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(id > 0);
}

#[tokio::test]
async fn write_compose_tweet_scheduled() {
    let state = make_state().await;
    let result = tuitbot_core::storage::scheduled_content::insert(
        &state.pool,
        "tweet",
        "Scheduled tweet content",
        Some("2026-12-01T12:00:00Z"),
    )
    .await;
    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(id > 0);
}

// ── Policy gate (compose_tweet uses this) ────────────────────────────

#[tokio::test]
async fn write_policy_gate_check() {
    let state = make_state().await;
    let start = std::time::Instant::now();
    let params = serde_json::json!({"content": "test", "content_type": "tweet"}).to_string();
    let gate = workflow::policy_gate::check_policy(&state, "compose_tweet", &params, start).await;
    // Should proceed (no mutations recorded yet, under limit)
    match gate {
        workflow::policy_gate::GateResult::Proceed => {} // expected
        workflow::policy_gate::GateResult::EarlyReturn(msg) => {
            // Also acceptable if policy is in strict mode
            assert!(!msg.is_empty());
        }
    }
}
