# Session 01 — Coupling Audit

> Per-tool coupling classification with exact files/modules causing coupling.
> Goal: identify which tools can operate independently vs. which are tightly bound to AppState.

## Coupling Classifications

### 1. `stateless_ready` — Config-only, no DB, no network (3 tools)

These tools read only from the immutable `Config` struct. They can run in any context.

| Tool | Handler | Deps | Notes |
|------|---------|------|-------|
| `score_tweet` | `tools/scoring.rs:36` | `Config` → `ScoringEngine` | Pure computation. Reads `config.scoring.*` + `config.business.product_keywords`. Sync (no async). |
| `get_config` | `tools/config.rs:11` | `Config` | Clones config, redacts secrets via `safety::redact::mask_secret()`. |
| `validate_config` | `tools/config.rs:33` | `Config` | Calls `config.validate()` — returns validation errors. |
| `get_mode` | `server.rs:352` | `Config` | Reads `config.mode` + `config.effective_approval_mode()`. |

### 2. `config_coupled` — Config + DB reads for state checks (2 tools)

| Tool | Handler | Deps | Coupling Source |
|------|---------|------|-----------------|
| `health_check` | `tools/health.rs:27` | `Config`, `DbPool`, `LlmProvider` | DB connectivity test via `storage::analytics::get_follower_snapshots()`. LLM health via `llm_provider.health_check()`. |
| `get_policy_status` | `tools/policy_gate.rs:191` | `Config`, `DbPool` | Reads `config.mcp_policy.*` + `storage::rate_limits::get_all_rate_limits()` for current usage. |

### 3. `db_coupled` (read-only) — Read from SQLite, no mutations (17 tools)

All analytics, discovery, context, telemetry, and monitoring tools. Read-only DB access.

| Tool | Handler | Storage Modules Used |
|------|---------|---------------------|
| `get_stats` | `tools/analytics.rs:55` | `storage::analytics::{get_analytics_summary, get_follower_snapshots}` |
| `get_follower_trend` | `tools/analytics.rs:105` | `storage::analytics::get_follower_snapshots` |
| `suggest_topics` | `tools/analytics.rs:137` | `storage::analytics::get_top_topics` |
| `get_action_log` | `tools/actions.rs:24` | `storage::action_log::get_actions_since` |
| `get_action_counts` | `tools/actions.rs:64` | `storage::action_log::get_action_counts_since` |
| `get_recent_replies` | `tools/replies.rs:27` | `storage::replies::get_replies_since` |
| `get_reply_count_today` | `tools/replies.rs:65` | `storage::replies::count_replies_today` |
| `list_target_accounts` | `tools/targets.rs:25` | `storage::target_accounts::get_active_target_accounts` |
| `list_unreplied_tweets` | `tools/discovery.rs:47` | `storage::tweets::get_unreplied_tweets_above_score` |
| `get_discovery_feed` | `tools/discovery.rs:70` | `storage::tweets::get_unreplied_tweets_above_score` |
| `get_author_context` | `tools/context.rs:13` | `tuitbot_core::context::author::get_author_context` → (DB reads internally) |
| `recommend_engagement_action` | `tools/context.rs:29` | `tuitbot_core::context::engagement::recommend_engagement` → (DB reads internally) |
| `topic_performance_snapshot` | `tools/context.rs:56` | `tuitbot_core::context::topics::get_topic_snapshot` |
| `get_capabilities` | `tools/capabilities.rs:64` | `storage::cursors::get_cursor_with_timestamp`, `storage::rate_limits::get_all_rate_limits` |
| `get_rate_limits` | `tools/rate_limits.rs:24` | `storage::rate_limits::get_all_rate_limits` |
| `list_pending_approvals` | `tools/approval.rs:52` | `storage::approval_queue::get_pending` |
| `get_pending_count` | `tools/approval.rs:75` | `storage::approval_queue::pending_count` |

### 4. `db_coupled` (read-only, telemetry/usage) — Observability reads (3 tools)

| Tool | Handler | Storage Modules Used |
|------|---------|---------------------|
| `get_x_usage` | `tools/x_actions/read.rs:164` | `storage::x_api_usage::{get_usage_summary, get_daily_usage, get_endpoint_breakdown}` |
| `get_mcp_tool_metrics` | `tools/telemetry.rs:14` | `storage::mcp_telemetry::{get_metrics_since, get_summary}` |
| `get_mcp_error_breakdown` | `tools/telemetry.rs:46` | `storage::mcp_telemetry::get_error_breakdown` |

### 5. `db_coupled` (mutations) — Write to approval queue (3 tools)

| Tool | Handler | Storage Modules Used | Mutation Target |
|------|---------|---------------------|-----------------|
| `approve_item` | `tools/approval.rs:99` | `storage::approval_queue::update_status_with_review` | `approval_queue` table |
| `reject_item` | `tools/approval.rs:127` | `storage::approval_queue::update_status_with_review` | `approval_queue` table |
| `approve_all` | `tools/approval.rs:155` | `storage::approval_queue::batch_approve` | `approval_queue` table |

### 6. `x_api_coupled` (read-only) — X API reads, no DB (5 tools)

| Tool | Handler | XApiClient Method | Extra Deps |
|------|---------|-------------------|------------|
| `get_tweet_by_id` | `tools/x_actions/read.rs:11` | `get_tweet()` | — |
| `x_get_user_by_username` | `tools/x_actions/read.rs:30` | `get_user_by_username()` | — |
| `x_search_tweets` | `tools/x_actions/read.rs:49` | `search_tweets()` | — |
| `x_get_user_tweets` | `tools/x_actions/read.rs:107` | `get_user_tweets()` | — |
| `x_upload_media` | `tools/x_actions/media.rs:15` | `upload_media()` | `tokio::fs::read()` for file I/O |

### 7. `x_api_coupled` (read-only, needs `authenticated_user_id`) — (2 tools)

| Tool | Handler | XApiClient Method | Extra Deps |
|------|---------|-------------------|------------|
| `x_get_user_mentions` | `tools/x_actions/read.rs:77` | `get_mentions(user_id, ...)` | `authenticated_user_id` |
| `x_get_home_timeline` | `tools/x_actions/read.rs:134` | `get_home_timeline(user_id, ...)` | `authenticated_user_id` |

### 8. `x_api_coupled` (mutations, policy-gated) — X API writes (10 tools)

All pass through `policy_gate::check_policy()` before executing. All call `McpPolicyEvaluator::record_mutation()` after success.

| Tool | Handler | XApiClient Method | Needs `authenticated_user_id`? |
|------|---------|-------------------|------|
| `x_post_tweet` | `tools/x_actions/write.rs:14` | `post_tweet()` / `post_tweet_with_media()` | No |
| `x_reply_to_tweet` | `tools/x_actions/write.rs:52` | `reply_to_tweet()` / `reply_to_tweet_with_media()` | No |
| `x_quote_tweet` | `tools/x_actions/write.rs:99` | `quote_tweet()` | No |
| `x_delete_tweet` | `tools/x_actions/write.rs:141` | `delete_tweet()` | No |
| `x_post_thread` | `tools/x_actions/write.rs:183` | `post_tweet()` + `reply_to_tweet()` loop | No |
| `x_like_tweet` | `tools/x_actions/engage.rs:13` | `like_tweet(user_id, tweet_id)` | **Yes** |
| `x_follow_user` | `tools/x_actions/engage.rs:55` | `follow_user(user_id, target_id)` | **Yes** |
| `x_unfollow_user` | `tools/x_actions/engage.rs:97` | `unfollow_user(user_id, target_id)` | **Yes** |
| `x_retweet` | `tools/x_actions/engage.rs:139` | `retweet(user_id, tweet_id)` | **Yes** |
| `x_unretweet` | `tools/x_actions/engage.rs:181` | `unretweet(user_id, tweet_id)` | **Yes** |

### 9. `db_mutation_coupled` (policy-gated, scheduled content) — (1 tool)

| Tool | Handler | Storage Modules | Policy |
|------|---------|-----------------|--------|
| `compose_tweet` | `server.rs:375` | `storage::scheduled_content::{insert, insert_draft}`, `McpPolicyEvaluator::record_mutation` | `policy_gate::check_policy()` |

### 10. `llm_coupled` — Require LLM provider (3 tools)

| Tool | Handler | LLM Usage | Extra Deps |
|------|---------|-----------|------------|
| `generate_reply` | `tools/content.rs:54` | `ContentGenerator::generate_reply()` via `ArcProvider` wrapper | Config (business profile) |
| `generate_tweet` | `tools/content.rs:95` | `ContentGenerator::generate_tweet()` via `ArcProvider` wrapper | Config (business profile) |
| `generate_thread` | `tools/content.rs:131` | `ContentGenerator::generate_thread()` via `ArcProvider` wrapper | Config (business profile) |

### 11. `workflow_coupled` — Multi-dependency composite tools (4 tools)

These orchestrate multiple subsystems in a single tool call.

| Tool | Handler | X API | DB | LLM | Policy | Scoring |
|------|---------|-------|-----|-----|--------|---------|
| `find_reply_opportunities` | `tools/composite/find_opportunities.rs:16` | `search_tweets()` | `storage::tweets::insert_discovered_tweet()`, `storage::replies::has_replied_to()` | — | — | `ScoringEngine` |
| `draft_replies_for_candidates` | `tools/composite/draft_replies.rs:30` | — | `storage::tweets::get_tweet_by_id()` | `generate_reply_with_archetype()` | — | — |
| `propose_and_queue_replies` | `tools/composite/propose_queue.rs:20` | `reply_to_tweet()` (if not approval mode) | `storage::tweets::*`, `storage::approval_queue::enqueue()` | `generate_reply()` (fallback) | `policy_gate::check_policy()` | — |
| `generate_thread_plan` | `tools/composite/thread_plan.rs:42` | — | `storage::mcp_telemetry::log_telemetry()` | `generate_thread_with_structure()` | — | — |

---

## Coupling Summary

| Classification | Count | Key Coupling Source |
|---|---|---|
| **stateless_ready** | 4 | `Config` only — `score_tweet`, `get_config`, `validate_config`, `get_mode` |
| **config_coupled** | 2 | `Config` + DB reads — `health_check`, `get_policy_status` |
| **db_coupled** (read-only) | 20 | `storage::*` modules — analytics, discovery, context, telemetry, approval reads, capabilities |
| **db_coupled** (mutations) | 3 | `storage::approval_queue` — `approve_item`, `reject_item`, `approve_all` |
| **x_api_coupled** (read) | 7 | `XApiClient` trait — reads only, some need `authenticated_user_id` |
| **x_api_coupled** (mutations) | 10 | `XApiClient` + `policy_gate` + `McpPolicyEvaluator` — all policy-gated writes |
| **db_mutation_coupled** | 1 | `compose_tweet` — `scheduled_content` + policy gate |
| **llm_coupled** | 3 | `LlmProvider` trait — content generation |
| **workflow_coupled** | 4 | Composite tools — orchestrate X API + scoring + LLM + DB + policy |
| **Total** | **52** (2 tools counted in multiple categories for composite) |

---

## Module-Level Coupling Map

### Database (`tuitbot_core::storage::*`)

| Storage Module | File | Used By Tools |
|---|---|---|
| `analytics` | `storage/analytics.rs` | `get_stats`, `get_follower_trend`, `suggest_topics`, `health_check` |
| `action_log` | `storage/action_log.rs` | `get_action_log`, `get_action_counts` |
| `replies` | `storage/replies.rs` | `get_recent_replies`, `get_reply_count_today`, `find_reply_opportunities` |
| `tweets` | `storage/tweets.rs` | `list_unreplied_tweets`, `get_discovery_feed`, `find_reply_opportunities`, `draft_replies_for_candidates`, `propose_and_queue_replies` |
| `target_accounts` | `storage/target_accounts.rs` | `list_target_accounts` |
| `approval_queue` | `storage/approval_queue/` | `list_pending_approvals`, `get_pending_count`, `approve_item`, `reject_item`, `approve_all`, `propose_and_queue_replies`, `compose_tweet` (via policy gate) |
| `rate_limits` | `storage/rate_limits.rs` | `get_rate_limits`, `get_capabilities`, `get_policy_status` |
| `cursors` | `storage/cursors.rs` | `get_capabilities` |
| `x_api_usage` | `storage/x_api_usage.rs` | `get_x_usage` |
| `mcp_telemetry` | `storage/mcp_telemetry.rs` | `get_mcp_tool_metrics`, `get_mcp_error_breakdown`, composite tools (recording) |
| `scheduled_content` | `storage/scheduled_content.rs` | `compose_tweet` |

### Config (`tuitbot_core::config::Config`)

Immutable after initialization. All tools receive `&Config` — no coupling beyond read access.

### X API (`tuitbot_core::x_api::XApiClient`)

Trait object via `state.x_client: Option<Box<dyn XApiClient>>`. 17 tools depend on it. All mutation tools also depend on `state.authenticated_user_id` for engagement actions (like, follow, retweet).

### LLM (`tuitbot_core::llm::LlmProvider`)

Trait object via `state.llm_provider: Option<Box<dyn LlmProvider>>`. Content tools wrap `Arc<AppState>` in `ArcProvider` adapter to satisfy `LlmProvider` trait bounds for `ContentGenerator`.

### Policy (`tuitbot_core::mcp_policy::McpPolicyEvaluator`)

Static methods: `evaluate()`, `record_mutation()`, `log_decision()`. Used by `tools/policy_gate.rs` which gates all mutation tools.

### State (`tuitbot_mcp::state::AppState`)

```rust
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
    pub llm_provider: Option<Box<dyn LlmProvider>>,
    pub x_client: Option<Box<dyn XApiClient>>,
    pub authenticated_user_id: Option<String>,
}
```

All tools access deps through `SharedState = Arc<AppState>`. This is the **primary coupling bottleneck** — even tools that only need `Config` receive the full `AppState`.

---

## Decoupling Opportunities (for Session 03)

1. **api_client_lane tools** (17 tools): Currently access `XApiClient` through `state.x_client` which also bundles DB pool, config, LLM provider. These could operate with just `XApiClient` + `Config` (for policy config).

2. **Read-only X API tools** (7 tools): Need only `XApiClient` trait object. Zero DB dependency. Prime candidates for standalone extraction.

3. **Mutation X API tools** (10 tools): Need `XApiClient` + policy gate. Policy gate currently needs `DbPool` (for `record_mutation` and `get_all_rate_limits`). Decoupling requires making policy recording optional or injecting a trait.

4. **Content generation tools** (3 tools): Need only `LlmProvider` + `Config.business`. The `ArcProvider` wrapper couples them to full `AppState`.
