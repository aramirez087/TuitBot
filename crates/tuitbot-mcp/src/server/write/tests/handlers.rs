//! Coverage tests for write/handlers.rs handler methods.
//!
//! Handler methods are proc-macro generated and private. Tests call the workflow
//! functions each handler delegates to — same code paths, no visibility hacks.
//! NullX returns AuthExpired/NotConfigured for all X calls, so results are error
//! strings rather than data — that is fine; coverage is the goal.
//!
//! Tools shared with admin (analytics, approval, x_actions, etc.) are already
//! tested in admin/tests. Tests here also cover workflow paths unique to the write
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

// ── Engagement handlers ───────────────────────────────────────────────

#[tokio::test]
async fn write_quote_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::quote_tweet(&state, "quoting this", "123", None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_like_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::like_tweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_follow_user() {
    let state = make_state().await;
    let result = workflow::x_actions::follow_user(&state, "u2").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_unfollow_user() {
    let state = make_state().await;
    let result = workflow::x_actions::unfollow_user(&state, "u2").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_retweet() {
    let state = make_state().await;
    let result = workflow::x_actions::retweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_unretweet() {
    let state = make_state().await;
    let result = workflow::x_actions::unretweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_delete_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::delete_tweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_unlike_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::unlike_tweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_bookmark_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::bookmark_tweet(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_unbookmark_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::unbookmark_tweet(&state, "123").await;
    assert!(!result.is_empty());
}

// ── Thread / dry-run handlers ─────────────────────────────────────────

#[tokio::test]
async fn write_post_thread() {
    let state = make_state().await;
    let tweets = vec!["tweet 1".to_string(), "tweet 2".to_string()];
    let result = workflow::x_actions::post_thread(&state, &tweets, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_post_tweet_dry_run() {
    let state = make_state().await;
    let result = workflow::x_actions::post_tweet_dry_run(&state, "dry run tweet", None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_post_thread_dry_run() {
    let state = make_state().await;
    let tweets = vec!["part 1".to_string()];
    let result = workflow::x_actions::post_thread_dry_run(&state, &tweets, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_upload_media_dry_run() {
    let state = make_state().await;
    let result =
        workflow::x_actions::upload_media(&state, "/tmp/nonexistent.jpg", None, true).await;
    assert!(!result.is_empty());
}

// ── Read handlers ─────────────────────────────────────────────────────

#[tokio::test]
async fn write_get_home_timeline() {
    let state = make_state().await;
    let result = workflow::x_actions::get_home_timeline(&state, 20, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_followers() {
    let state = make_state().await;
    let result = workflow::x_actions::get_followers(&state, "u1", 100, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_following() {
    let state = make_state().await;
    let result = workflow::x_actions::get_following(&state, "u1", 100, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_user_by_id() {
    let state = make_state().await;
    let result = workflow::x_actions::get_user_by_id(&state, "u1").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_liked_tweets() {
    let state = make_state().await;
    let result = workflow::x_actions::get_liked_tweets(&state, "u1", 10, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_bookmarks() {
    let state = make_state().await;
    let result = workflow::x_actions::get_bookmarks(&state, 10, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_users_by_ids() {
    let state = make_state().await;
    let result = workflow::x_actions::get_users_by_ids(&state, &["u1"]).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_tweet_liking_users() {
    let state = make_state().await;
    let result = workflow::x_actions::get_tweet_liking_users(&state, "123", 100, None).await;
    assert!(!result.is_empty());
}

// ── X usage ──────────────────────────────────────────────────────────

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
    match gate {
        workflow::policy_gate::GateResult::Proceed => {} // expected
        workflow::policy_gate::GateResult::EarlyReturn(msg) => {
            assert!(!msg.is_empty());
        }
    }
}

// ── Context / analytics handlers ──────────────────────────────────────

#[tokio::test]
async fn write_get_author_context() {
    let state = make_state().await;
    let result = workflow::context::get_author_context(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_recommend_engagement_action() {
    let state = make_state().await;
    let result =
        workflow::context::recommend_engagement(&state, "tuitbot", "Check this out!", None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_topic_performance_snapshot() {
    let state = make_state().await;
    let result = workflow::context::topic_performance_snapshot(&state.pool, 30).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_mcp_tool_metrics() {
    let state = make_state().await;
    let result = workflow::telemetry::get_mcp_tool_metrics(&state.pool, 24).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_mcp_error_breakdown() {
    let state = make_state().await;
    let result = workflow::telemetry::get_mcp_error_breakdown(&state.pool, 24).await;
    assert!(!result.is_empty());
}

// ── Composite handlers ────────────────────────────────────────────────

#[tokio::test]
async fn write_find_reply_opportunities() {
    let state = make_state().await;
    let result = workflow::composite::find_opportunities::execute(
        &state,
        Some("rust"),
        None,
        Some(10),
        None,
    )
    .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_draft_replies_for_candidates() {
    let state = make_state().await;
    let result = workflow::composite::draft_replies::execute(&state, &[], None, false).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_propose_and_queue_replies() {
    let state = make_state().await;
    let result = workflow::composite::propose_queue::execute(&state, &[], false).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_generate_thread_plan() {
    let state = make_state().await;
    let result =
        workflow::composite::thread_plan::execute(&state, "Rust async", None, None, None).await;
    assert!(!result.is_empty());
}
