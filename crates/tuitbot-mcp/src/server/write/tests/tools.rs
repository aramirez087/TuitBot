//! Coverage tests for write/tools.rs handler methods.
//!
//! Handler methods are private (proc-macro generated). Tests call the workflow
//! functions that each handler delegates to — same code paths, no visibility hacks.

use crate::tools::workflow;

use super::make_state;

#[tokio::test]
async fn write_get_stats() {
    let state = make_state().await;
    let result = workflow::analytics::get_stats(&state.pool, 7, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_follower_trend() {
    let state = make_state().await;
    let result = workflow::analytics::get_follower_trend(&state.pool, 30, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_action_log() {
    let state = make_state().await;
    let result = workflow::actions::get_action_log(&state.pool, 24, None, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_action_counts() {
    let state = make_state().await;
    let result = workflow::actions::get_action_counts(&state.pool, 24, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_recent_mutations() {
    let state = make_state().await;
    let result =
        workflow::mutation_audit::get_recent_mutations(&state.pool, 50, None, None, &state.config)
            .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_mutation_detail() {
    let state = make_state().await;
    let result =
        workflow::mutation_audit::get_mutation_detail(&state.pool, "nonexistent", &state.config)
            .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_rate_limits() {
    let state = make_state().await;
    let result = workflow::rate_limits::get_rate_limits(&state.pool, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_recent_replies() {
    let state = make_state().await;
    let result = workflow::replies::get_recent_replies(&state.pool, 24, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_reply_count_today() {
    let state = make_state().await;
    let result = workflow::replies::get_reply_count_today(&state.pool, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_list_target_accounts() {
    let state = make_state().await;
    let result = workflow::targets::list_target_accounts(&state.pool, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_list_unreplied_tweets() {
    let state = make_state().await;
    let result = workflow::discovery::list_unreplied_tweets(&state.pool, 0.5, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_score_tweet() {
    let state = make_state().await;
    let input = crate::tools::scoring::ScoreTweetInput {
        text: "hello world",
        author_username: "tuitbot",
        author_followers: 1000,
        likes: 5,
        retweets: 2,
        replies: 1,
        created_at: "2026-01-01T00:00:00Z",
    };
    let result = crate::tools::scoring::score_tweet(&state.config, &input);
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_list_pending_approvals() {
    let state = make_state().await;
    let result = workflow::approval::list_pending(&state.pool, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_pending_count() {
    let state = make_state().await;
    let result = workflow::approval::get_pending_count(&state.pool, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_approve_item() {
    let state = make_state().await;
    let result = workflow::approval::approve_item(&state.pool, 99999, &state.config, false).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_reject_item() {
    let state = make_state().await;
    let result = workflow::approval::reject_item(&state.pool, 99999, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_approve_all() {
    let state = make_state().await;
    let result = workflow::approval::approve_all(&state.pool, &state.config, false).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_policy_status() {
    let state = make_state().await;
    let result = workflow::policy_gate::get_policy_status(&state).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_health_check() {
    let state = make_state().await;
    let result =
        workflow::health::health_check(&state.pool, state.llm_provider.as_deref(), &state.config)
            .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_discovery_feed() {
    let state = make_state().await;
    let result =
        workflow::discovery::list_unreplied_tweets_with_limit(&state.pool, 0.0, 20, &state.config)
            .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_suggest_topics() {
    let state = make_state().await;
    let result = workflow::analytics::get_top_topics(&state.pool, 10, &state.config).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_tweet_by_id() {
    let state = make_state().await;
    let result = workflow::x_actions::get_tweet_by_id(&state, "123").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_user_by_username() {
    let state = make_state().await;
    let result = workflow::x_actions::get_user_by_username(&state, "tuitbot").await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_search_tweets() {
    let state = make_state().await;
    let result = workflow::x_actions::search_tweets(&state, "rust", 10, None, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_user_mentions() {
    let state = make_state().await;
    let result = workflow::x_actions::get_user_mentions(&state, None, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_get_user_tweets() {
    let state = make_state().await;
    let result = workflow::x_actions::get_user_tweets(&state, "u1", 10, None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_post_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::post_tweet(&state, "test tweet", None).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn write_reply_to_tweet() {
    let state = make_state().await;
    let result = workflow::x_actions::reply_to_tweet(&state, "reply", "123", None).await;
    assert!(!result.is_empty());
}

// ── Config tools (non-workflow, called directly) ──────────────────────

#[test]
fn write_get_config() {
    let config = tuitbot_core::config::Config::default();
    let result = crate::tools::config::get_config(&config);
    assert!(!result.is_empty());
}

#[test]
fn write_validate_config() {
    let config = tuitbot_core::config::Config::default();
    let result = crate::tools::config::validate_config(&config);
    assert!(!result.is_empty());
}

// ── WriteMcpServer construction & ServerHandler ──────────────────────

#[tokio::test]
async fn write_server_construction() {
    let state = make_state().await;
    let _server = super::super::WriteMcpServer::new(state);
}

#[tokio::test]
async fn write_server_info_has_instructions() {
    use rmcp::ServerHandler;
    let state = make_state().await;
    let server = super::super::WriteMcpServer::new(state);
    let info = server.get_info();
    assert!(info.instructions.is_some());
    let instructions = info.instructions.unwrap();
    assert!(
        instructions.contains("Write"),
        "instructions should mention Write"
    );
}

#[tokio::test]
async fn write_server_info_has_tool_capabilities() {
    use rmcp::ServerHandler;
    let state = make_state().await;
    let server = super::super::WriteMcpServer::new(state);
    let info = server.get_info();
    assert!(info.capabilities.tools.is_some());
}

#[tokio::test]
async fn write_server_clones() {
    let state = make_state().await;
    let server = super::super::WriteMcpServer::new(state);
    let _clone = server.clone();
}

// ── Capabilities workflow ────────────────────────────────────────────

#[tokio::test]
async fn write_get_capabilities() {
    let state = make_state().await;
    let result = workflow::capabilities::get_capabilities(
        &state.pool,
        &state.config,
        state.llm_provider.is_some(),
        state.x_client.is_some(),
        state.authenticated_user_id.as_deref(),
        &state.granted_scopes,
    )
    .await;
    assert!(!result.is_empty());
}

// ── Mode tool ────────────────────────────────────────────────────────

#[test]
fn write_mode_and_approval() {
    let config = tuitbot_core::config::Config::default();
    let mode = config.mode.to_string();
    let approval = config.effective_approval_mode();
    assert!(!mode.is_empty());
    // approval_mode is a bool
    let _ = approval;
}
