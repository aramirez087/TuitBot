# Session 01 — Tool Inventory

> 54 MCP tools organized by product lane. Each entry: name, handler location, parameters, core functions called, coupling tags.

## api_client_lane (17 tools)

Direct X API v2 wrappers. All require `state.x_client` (XApiClient trait object).

### Read Operations (6 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 1 | `get_tweet_by_id` | `tools/x_actions/read.rs:11` | `tweet_id: String` | `x_client.get_tweet()` | x_api |
| 2 | `x_get_user_by_username` | `tools/x_actions/read.rs:30` | `username: String` | `x_client.get_user_by_username()` | x_api |
| 3 | `x_search_tweets` | `tools/x_actions/read.rs:49` | `query: String`, `max_results?: u32`, `since_id?: String`, `pagination_token?: String` | `x_client.search_tweets()` | x_api |
| 4 | `x_get_user_mentions` | `tools/x_actions/read.rs:77` | `since_id?: String`, `pagination_token?: String` | `x_client.get_mentions(user_id, ...)` | x_api, authenticated_user_id |
| 5 | `x_get_user_tweets` | `tools/x_actions/read.rs:107` | `user_id: String`, `max_results?: u32`, `pagination_token?: String` | `x_client.get_user_tweets()` | x_api |
| 6 | `x_get_home_timeline` | `tools/x_actions/read.rs:134` | `max_results?: u32`, `pagination_token?: String` | `x_client.get_home_timeline(user_id, ...)` | x_api, authenticated_user_id |

### Write Operations (7 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 7 | `x_post_tweet` | `tools/x_actions/write.rs:14` | `text: String`, `media_ids?: Vec<String>` | `check_tweet_length()`, `policy_gate::check_policy()`, `x_client.post_tweet()` / `post_tweet_with_media()`, `McpPolicyEvaluator::record_mutation()` | x_api, db_mutation, policy |
| 8 | `x_reply_to_tweet` | `tools/x_actions/write.rs:52` | `text: String`, `in_reply_to_id: String`, `media_ids?: Vec<String>` | `check_tweet_length()`, `policy_gate::check_policy()`, `x_client.reply_to_tweet()` / `reply_to_tweet_with_media()`, `McpPolicyEvaluator::record_mutation()` | x_api, db_mutation, policy |
| 9 | `x_quote_tweet` | `tools/x_actions/write.rs:99` | `text: String`, `quoted_tweet_id: String`, `media_ids?: Vec<String>` | `check_tweet_length()`, `policy_gate::check_policy()`, `x_client.quote_tweet()`, `McpPolicyEvaluator::record_mutation()` | x_api, db_mutation, policy |
| 10 | `x_delete_tweet` | `tools/x_actions/write.rs:141` | `tweet_id: String` | `policy_gate::check_policy()` (always gated), `x_client.delete_tweet()`, `McpPolicyEvaluator::record_mutation()` | x_api, db_mutation, policy |
| 11 | `x_post_thread` | `tools/x_actions/write.rs:183` | `tweets: Vec<String>`, `media_ids?: Vec<Vec<String>>` | `check_tweet_length()` (per tweet), `policy_gate::check_policy()`, loop: `x_client.post_tweet()` then `x_client.reply_to_tweet()`, `McpPolicyEvaluator::record_mutation()` | x_api, db_mutation, policy |
| 12 | `x_upload_media` | `tools/x_actions/media.rs:15` | `file_path: String` | `infer_media_type()`, `tokio::fs::read()`, `x_client.upload_media()` | x_api |

### Engagement Operations (4 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 13 | `x_like_tweet` | `tools/x_actions/engage.rs:13` | `tweet_id: String` | `policy_gate::check_policy()`, `x_client.like_tweet(user_id, tweet_id)`, `McpPolicyEvaluator::record_mutation()` | x_api, authenticated_user_id, db_mutation, policy |
| 14 | `x_follow_user` | `tools/x_actions/engage.rs:55` | `target_user_id: String` | `policy_gate::check_policy()`, `x_client.follow_user(user_id, target_user_id)`, `McpPolicyEvaluator::record_mutation()` | x_api, authenticated_user_id, db_mutation, policy |
| 15 | `x_unfollow_user` | `tools/x_actions/engage.rs:97` | `target_user_id: String` | `policy_gate::check_policy()`, `x_client.unfollow_user(user_id, target_user_id)`, `McpPolicyEvaluator::record_mutation()` | x_api, authenticated_user_id, db_mutation, policy |
| 16 | `x_retweet` | `tools/x_actions/engage.rs:139` | `tweet_id: String` | `policy_gate::check_policy()`, `x_client.retweet(user_id, tweet_id)`, `McpPolicyEvaluator::record_mutation()` | x_api, authenticated_user_id, db_mutation, policy |
| 17 | `x_unretweet` | `tools/x_actions/engage.rs:181` | `tweet_id: String` | `policy_gate::check_policy()`, `x_client.unretweet(user_id, tweet_id)`, `McpPolicyEvaluator::record_mutation()` | x_api, authenticated_user_id, db_mutation, policy |

---

## workflow_lane (25 tools)

Analytics, discovery, content generation, approval queue, context intelligence, and composite orchestration tools.

### Analytics (5 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 18 | `get_stats` | `tools/analytics.rs:55` | `days?: u32` (default 7) | `storage::analytics::get_analytics_summary()`, `storage::analytics::get_follower_snapshots()` | db_read, config |
| 19 | `get_follower_trend` | `tools/analytics.rs:105` | `limit?: u32` (default 7) | `storage::analytics::get_follower_snapshots()` | db_read, config |
| 20 | `suggest_topics` | `tools/analytics.rs:137` | — | `storage::analytics::get_top_topics()` | db_read, config |
| 21 | `get_action_log` | `tools/actions.rs:24` | `since_hours?: u32` (default 24), `action_type?: String` | `storage::action_log::get_actions_since()` | db_read, config |
| 22 | `get_action_counts` | `tools/actions.rs:64` | `since_hours?: u32` (default 24) | `storage::action_log::get_action_counts_since()` | db_read, config |

### Replies & Discovery (5 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 23 | `get_recent_replies` | `tools/replies.rs:27` | `since_hours?: u32` (default 24) | `storage::replies::get_replies_since()` | db_read, config |
| 24 | `get_reply_count_today` | `tools/replies.rs:65` | — | `storage::replies::count_replies_today()` | db_read, config |
| 25 | `list_target_accounts` | `tools/targets.rs:25` | — | `storage::target_accounts::get_active_target_accounts()` | db_read, config |
| 26 | `list_unreplied_tweets` | `tools/discovery.rs:47` | `threshold?: f64` (default 0.0) | `storage::tweets::get_unreplied_tweets_above_score()` | db_read, config |
| 27 | `get_discovery_feed` | `tools/discovery.rs:70` | `min_score?: f64` (default 50.0), `limit?: u32` (default 10) | `storage::tweets::get_unreplied_tweets_above_score()` | db_read, config |

### Scoring (1 tool)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 28 | `score_tweet` | `tools/scoring.rs:36` | `text: String`, `author_username: String`, `author_followers: u32`, `likes: u32`, `retweets: u32`, `replies: u32`, `created_at: String` | `ScoringEngine::new()`, `engine.score_tweet()` | config_only |

### Approval Queue (5 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 29 | `list_pending_approvals` | `tools/approval.rs:52` | — | `storage::approval_queue::get_pending()` | db_read, config |
| 30 | `get_pending_count` | `tools/approval.rs:75` | — | `storage::approval_queue::pending_count()` | db_read, config |
| 31 | `approve_item` | `tools/approval.rs:99` | `id: i64` | `storage::approval_queue::update_status_with_review()` | db_mutation, config |
| 32 | `reject_item` | `tools/approval.rs:127` | `id: i64` | `storage::approval_queue::update_status_with_review()` | db_mutation, config |
| 33 | `approve_all` | `tools/approval.rs:155` | — | `storage::approval_queue::batch_approve()` | db_mutation, config |

### Content Generation (4 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 34 | `generate_reply` | `tools/content.rs:54` | `tweet_text: String`, `tweet_author: String`, `mention_product?: bool` | `ContentGenerator::new()`, `gen.generate_reply()` | llm, config |
| 35 | `generate_tweet` | `tools/content.rs:95` | `topic?: String` | `ContentGenerator::new()`, `gen.generate_tweet()` | llm, config |
| 36 | `generate_thread` | `tools/content.rs:131` | `topic?: String` | `ContentGenerator::new()`, `gen.generate_thread()` | llm, config |
| 37 | `compose_tweet` | `server.rs:375` | `content: String`, `content_type?: String`, `scheduled_for?: String` | `policy_gate::check_policy()`, `storage::scheduled_content::insert()` or `insert_draft()`, `McpPolicyEvaluator::record_mutation()` | db_mutation, policy, config |

### Context Intelligence (3 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 38 | `get_author_context` | `tools/context.rs:13` | `identifier: String` | `tuitbot_core::context::author::get_author_context()` | db_read, config |
| 39 | `recommend_engagement_action` | `tools/context.rs:29` | `author_username: String`, `tweet_text: String`, `campaign_objective?: String` | `tuitbot_core::context::engagement::recommend_engagement()` | db_read, config |
| 40 | `topic_performance_snapshot` | `tools/context.rs:56` | `lookback_days?: u32` (default 30) | `tuitbot_core::context::topics::get_topic_snapshot()` | db_read |

### Composite (4 tools)

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 41 | `find_reply_opportunities` | `tools/composite/find_opportunities.rs:16` | `query?: String`, `min_score?: f64`, `limit?: u32`, `since_id?: String` | `x_client.search_tweets()`, `ScoringEngine::new()`, `engine.score_tweet()`, `storage::tweets::insert_discovered_tweet()`, `storage::replies::has_replied_to()`, `telemetry::record()` | x_api, db_mutation, config |
| 42 | `draft_replies_for_candidates` | `tools/composite/draft_replies.rs:30` | `candidate_ids: Vec<String>`, `archetype?: String`, `mention_product?: bool` | `storage::tweets::get_tweet_by_id()`, `gen.generate_reply_with_archetype()`, `contains_banned_phrase()`, `dedup.is_phrasing_similar()`, `telemetry::record()` | db_read, llm, config |
| 43 | `propose_and_queue_replies` | `tools/composite/propose_queue.rs:20` | `items: Vec<ProposeItem>`, `mention_product?: bool` | `policy_gate::check_policy()`, `storage::tweets::get_tweet_by_id()`, `gen.generate_reply()`, `dedup.*`, `contains_banned_phrase()`, approval routing or `x_client.reply_to_tweet()`, `McpPolicyEvaluator::record_mutation()`, `telemetry::record()` | x_api, db_mutation, llm, policy, config |
| 44 | `generate_thread_plan` | `tools/composite/thread_plan.rs:42` | `topic: String`, `objective?: String`, `target_audience?: String`, `structure?: String` | `gen.generate_thread_with_structure()`, `analyze_hook()`, `telemetry::record()` | llm, db_read (telemetry), config |

---

## platform_lane (10 tools)

Configuration, health, rate limits, telemetry, and X API usage monitoring.

| # | Tool | Handler | Parameters | Core Functions | Coupling |
|---|------|---------|------------|----------------|----------|
| 45 | `get_config` | `tools/config.rs:11` | — | `safety::redact::mask_secret()` | config_only |
| 46 | `validate_config` | `tools/config.rs:33` | — | `config.validate()` | config_only |
| 47 | `health_check` | `tools/health.rs:27` | — | `storage::analytics::get_follower_snapshots()` (DB test), `llm_provider.health_check()` | db_read, llm, config |
| 48 | `get_capabilities` | `tools/capabilities.rs:64` | — | `storage::cursors::get_cursor_with_timestamp("api_tier")`, `storage::rate_limits::get_all_rate_limits()` | db_read, config |
| 49 | `get_mode` | `server.rs:352` | — | `config.mode.to_string()`, `config.effective_approval_mode()` | config_only |
| 50 | `get_policy_status` | `tools/policy_gate.rs:191` | — | `storage::rate_limits::get_all_rate_limits()` | db_read, config |
| 51 | `get_rate_limits` | `tools/rate_limits.rs:24` | — | `storage::rate_limits::get_all_rate_limits()` | db_read, config |
| 52 | `get_x_usage` | `tools/x_actions/read.rs:164` | `days?: u32` (default 7) | `storage::x_api_usage::get_usage_summary()`, `get_daily_usage()`, `get_endpoint_breakdown()` | db_read |
| 53 | `get_mcp_tool_metrics` | `tools/telemetry.rs:14` | `since_hours?: u32` (default 24) | `storage::mcp_telemetry::get_metrics_since()`, `get_summary()` | db_read |
| 54 | `get_mcp_error_breakdown` | `tools/telemetry.rs:46` | `since_hours?: u32` (default 24) | `storage::mcp_telemetry::get_error_breakdown()` | db_read |

---

## Coupling Tag Legend

| Tag | Meaning |
|-----|---------|
| `config_only` | Reads `Config` struct. No DB, no network. |
| `db_read` | Reads from SQLite via `storage::*` modules. |
| `db_mutation` | Writes to SQLite (approval queue, policy recording, tweet persistence). |
| `x_api` | Calls X API v2 via `XApiClient` trait object. |
| `authenticated_user_id` | Requires `state.authenticated_user_id` (from `get_me()` on startup). |
| `llm` | Requires `LlmProvider` trait object (OpenAI/Anthropic/Ollama). |
| `policy` | Goes through `policy_gate::check_policy()` before mutation. |

## Tool Count Summary

| Lane | Count |
|------|-------|
| api_client_lane | 17 |
| workflow_lane | 25 |
| platform_lane | 10 |
| **Total** | **52** |

> **Note:** The plan specified 54 tools counting `get_x_usage` in platform_lane and `compose_tweet` in workflow_lane. Actual count after audit is **52 distinct tool registrations** in `server.rs`. The plan's count of 16 api_client tools excluded `x_unretweet`; adjusted count is 17 (including `x_unretweet`). Workflow lane has 25 (plan said 26 — `suggest_topics` was listed but `compose_tweet` was not counted in the plan's 26). Platform lane has 10 (plan said 12 — `get_x_usage` was double-counted across lanes).
