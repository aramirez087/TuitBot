# MCP Reference

Tuitbot ships with an MCP server so AI agents can call tools with typed inputs.
The server exposes **64 tools** across three lanes — from raw X API access to
full autonomous growth workflows.

## Quick Start

```bash
# Default: full workflow profile (64 tools)
tuitbot mcp serve

# API-only profile (34 tools)
tuitbot mcp serve --profile api

# With custom config
tuitbot -c /path/to/config.toml mcp serve
```

## Three MCP Lanes

TuitBot's MCP server offers three integration lanes, each serving a different use case:

| Lane | Command | Tools | Prerequisites | Use Case |
|------|---------|-------|---------------|----------|
| **1 — Official API** | `tuitbot mcp serve --profile api` | 34 | X API tokens (`tuitbot auth`) | Drop-in replacement for thin X wrappers with policy safety |
| **2 — Scraper** | `tuitbot mcp serve --profile api` | 34 | `provider_backend = "scraper"` in config | Read-heavy agents without official API tokens |
| **3 — Workflow** | `tuitbot mcp serve` | 64 | X API tokens + LLM provider + SQLite DB | Full autonomous growth co-pilot (default) |

### Choosing a Lane

| Question | Answer | Lane |
|----------|--------|------|
| Need direct X API access only? | Yes | Lane 1 |
| No official X API tokens? | Yes | Lane 2 |
| Need analytics, content generation, approval workflows? | Yes | Lane 3 |
| Want composite multi-step workflows? | Yes | Lane 3 |
| Replacing a thin X MCP wrapper? | Yes | Lane 1 or Lane 3 |
| Default / unsure? | — | Lane 3 (default) |

## Claude Code Configuration

**Lane 3 — Workflow (default, recommended):**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve"]
    }
  }
}
```

**Lane 1 — API only:**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve", "--profile", "api"]
    }
  }
}
```

---

## Response Envelope (v1.0)

Every MCP tool wraps its output in a unified JSON envelope with `success`,
`data`, `error`, and `meta` fields.

### Success Example

```json
{
  "success": true,
  "data": {
    "tier": "Basic",
    "can_reply": true
  },
  "meta": {
    "tool_version": "1.0",
    "elapsed_ms": 12,
    "mode": "autopilot",
    "approval_mode": false
  }
}
```

### Error Example

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "x_rate_limited",
    "message": "Rate limit exceeded",
    "retryable": true,
    "rate_limit_reset": "2026-02-25T13:00:00Z"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 3 }
}
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | Whether the tool call succeeded |
| `data` | `any` | Tool payload (object, array, or null on error) |
| `error` | `object?` | Present only on failure |
| `error.code` | `string` | Machine-readable code (e.g. `db_error`) |
| `error.message` | `string` | Human-readable description |
| `error.retryable` | `bool` | Whether the caller may retry |
| `error.rate_limit_reset` | `string?` | ISO-8601 timestamp when rate limit resets |
| `error.policy_decision` | `string?` | Policy decision: `"denied"`, `"routed_to_approval"` |
| `meta` | `object?` | Execution metadata (optional) |
| `meta.tool_version` | `string` | Envelope schema version |
| `meta.elapsed_ms` | `u64` | Wall-clock execution time in ms |
| `meta.mode` | `string?` | Operating mode (`autopilot` / `composer`) |
| `meta.approval_mode` | `bool?` | Effective approval mode flag |

---

## Error Codes (28)

All error codes are typed enum variants (`ErrorCode`) with compile-time exhaustiveness.

### X API Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `x_rate_limited` | X API rate limit hit (HTTP 429) | Yes |
| `x_auth_expired` | OAuth token expired (HTTP 401) | No |
| `x_forbidden` | Forbidden / tier restriction (HTTP 403) | No |
| `x_account_restricted` | Account suspended or limited | No |
| `x_network_error` | Network connectivity issue | Yes |
| `x_not_configured` | X API client not available (no tokens) | No |
| `x_api_error` | Other X API errors | Yes |

### Provider Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `scraper_mutation_blocked` | Mutation attempted via scraper with mutations disabled | No |

### Database Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `db_error` | Database operation failed | Yes |

### Validation Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `validation_error` | Input validation failed | No |
| `invalid_input` | Malformed request parameters | No |
| `tweet_too_long` | Tweet text exceeds 280 characters | No |

### LLM Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `llm_error` | LLM generation failed | Yes |
| `llm_not_configured` | LLM provider not set up | No |

### Media Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `unsupported_media_type` | File type not supported for upload | No |
| `file_read_error` | Could not read media file from disk | No |
| `media_upload_error` | Media upload to X failed | No |

### Thread Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `thread_partial_failure` | Some tweets in a thread posted, others failed | Yes |

### Policy Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `policy_error` | Policy evaluation failed (DB error) | Yes |
| `policy_denied_blocked` | Tool is in `blocked_tools` configuration | No |
| `policy_denied_rate_limited` | Hourly MCP mutation rate limit exceeded | No |
| `policy_denied_hard_rule` | Blocked by hard safety rule | No |
| `policy_denied_user_rule` | Blocked by user-configured rule | No |

### Context Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `context_error` | Author context retrieval failed | No |
| `recommendation_error` | Engagement recommendation failed | No |
| `topic_error` | Topic performance query failed | No |

### Resource Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `not_found` | Requested resource not found | No |

### Internal Errors

| Code | Meaning | Retryable |
|------|---------|-----------|
| `serialization_error` | Response serialization failed | No |

---

## Read Tools (14)

These tools provide read-only access to X API v2. Available in both API and Workflow profiles (except `x_get_me` which is API-only).

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `get_tweet_by_id` | Fetch a single tweet by ID | `tweet_id` (required) | Both |
| `x_get_user_by_username` | Look up a user by @username | `username` (required) | Both |
| `x_search_tweets` | Search recent tweets | `query` (required), `max_results` (optional, 10-100), `since_id` (optional) | Both |
| `x_get_user_mentions` | Get mentions of the authenticated user | `since_id` (optional) | Both |
| `x_get_user_tweets` | Get recent tweets from a user | `user_id` (required), `max_results` (optional, 5-100) | Both |
| `x_get_home_timeline` | Get authenticated user's home timeline | `max_results` (optional), `since_id` (optional) | Both |
| `x_get_followers` | Get followers of a user | `user_id` (required), `max_results` (optional) | Both |
| `x_get_following` | Get accounts a user follows | `user_id` (required), `max_results` (optional) | Both |
| `x_get_user_by_id` | Look up a user by numeric ID | `user_id` (required) | Both |
| `x_get_liked_tweets` | Get tweets liked by a user | `user_id` (required), `max_results` (optional) | Both |
| `x_get_bookmarks` | Get authenticated user's bookmarks | `max_results` (optional) | Both |
| `x_get_users_by_ids` | Batch look up users by IDs | `user_ids` (required, array) | Both |
| `x_get_tweet_liking_users` | Get users who liked a tweet | `tweet_id` (required), `max_results` (optional) | Both |
| `x_get_me` | Get authenticated user's own profile | None | API only |

### Example: Get a tweet

```json
// Request
{ "tweet_id": "1234567890" }

// Response
{
  "success": true,
  "data": {
    "id": "1234567890",
    "text": "Hello world",
    "author_id": "987654321",
    "created_at": "2026-02-24T12:00:00.000Z",
    "public_metrics": {
      "retweet_count": 5,
      "reply_count": 2,
      "like_count": 10
    }
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 245 }
}
```

---

## Write Tools (6)

Mutation tools for posting, replying, quoting, and deleting tweets. Policy-gated — may route to approval queue.

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_post_tweet` | Post a new tweet | `text` (required), `media_ids` (optional) | Both |
| `x_reply_to_tweet` | Reply to an existing tweet | `text` (required), `in_reply_to_id` (required) | Both |
| `x_quote_tweet` | Post a quote tweet | `text` (required), `quoted_tweet_id` (required) | Both |
| `x_delete_tweet` | Delete an owned tweet | `tweet_id` (required) | Both |
| `x_post_thread` | Post a multi-tweet thread | `tweets` (required, array of text), `media_ids` (optional) | Both |
| `compose_tweet` | Create a draft or scheduled tweet | `content` (required), `content_type` (optional), `scheduled_for` (optional) | Workflow only |

---

## Engage Tools (8)

Social engagement actions. All are mutations and policy-gated.

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_like_tweet` | Like a tweet | `tweet_id` (required) | Both |
| `x_unlike_tweet` | Unlike a tweet | `tweet_id` (required) | Both |
| `x_follow_user` | Follow a user | `target_user_id` (required) | Both |
| `x_unfollow_user` | Unfollow a user | `target_user_id` (required) | Both |
| `x_retweet` | Retweet a tweet | `tweet_id` (required) | Both |
| `x_unretweet` | Undo a retweet | `tweet_id` (required) | Both |
| `x_bookmark_tweet` | Bookmark a tweet | `tweet_id` (required) | Both |
| `x_unbookmark_tweet` | Remove a bookmark | `tweet_id` (required) | Both |

### Example: Like a tweet

```json
// Request
{ "tweet_id": "1234567890" }

// Response
{
  "success": true,
  "data": { "liked": true, "tweet_id": "1234567890" },
  "meta": { "tool_version": "1.0", "elapsed_ms": 312 }
}
```

---

## Media Tools (1)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_upload_media` | Upload an image/video for tweet attachment | `file_path` (required), `media_type` (optional) | Both |

---

## Utility Tools (6)

Available in both profiles. No X client required (except `health_check` which needs DB).

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `score_tweet` | Score a tweet using 6-signal heuristic | `tweet` (required object) | Both |
| `get_config` | Get current configuration | None | Both |
| `validate_config` | Validate configuration file | None | Both |
| `get_capabilities` | Get server capabilities and provider info | None | Both |
| `health_check` | Check server and database health | None | Both |
| `get_mode` | Get current operating mode | None | Both |

---

## Workflow-Only Tools (31)

These tools are available only in the Workflow profile (`tuitbot mcp serve`, the default). They provide analytics, content generation, approval workflows, discovery, and composite multi-step operations.

### Analytics (7)

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_stats` | Aggregate growth statistics | `days` (optional) |
| `get_follower_trend` | Historical follower count data | `days` (optional) |
| `get_action_log` | Recent automation actions | `limit` (optional) |
| `get_action_counts` | Action counts by type and period | `period` (optional) |
| `get_recent_replies` | Recent replies posted by TuitBot | `limit` (optional) |
| `get_reply_count_today` | Today's reply count | None |
| `get_x_usage` | X API usage statistics | None |

### Approval Queue (5)

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_pending_approvals` | List items awaiting approval | `limit` (optional) |
| `get_pending_count` | Count of pending approval items | None |
| `approve_item` | Approve and execute a queued item | `id` (required) |
| `reject_item` | Reject a queued item | `id` (required) |
| `approve_all` | Approve and execute all queued items | None |

### Content Generation (4)

| Tool | Description | Parameters |
|------|-------------|------------|
| `generate_reply` | Generate a contextual reply draft | `tweet_id` (required), `context` (optional) |
| `generate_tweet` | Generate an original tweet draft | `topic` (optional), `style` (optional) |
| `generate_thread` | Generate a multi-tweet thread draft | `topic` (required), `num_tweets` (optional) |
| `suggest_topics` | Get topic suggestions from performance data | None |

### Discovery (3)

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_target_accounts` | List monitored target accounts | None |
| `list_unreplied_tweets` | Find tweets not yet replied to | `limit` (optional), `min_score` (optional) |
| `get_discovery_feed` | Scored tweets from the discovery feed | `limit` (optional), `min_score` (optional) |

### Policy (2)

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_rate_limits` | Current rate limit status | None |
| `get_policy_status` | MCP mutation policy settings and usage | None |

### Context Intelligence (3)

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_author_context` | Profile and interaction history for a tweet author | `author_id` (required) |
| `recommend_engagement_action` | AI-recommended engagement for a tweet | `tweet_id` (required) |
| `topic_performance_snapshot` | Performance metrics by topic | `days` (optional) |

### Telemetry (2)

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_mcp_tool_metrics` | Per-tool invocation metrics | `days` (optional) |
| `get_mcp_error_breakdown` | Error frequency by code | `days` (optional) |

### Composite Workflows (4)

Multi-step operations that replace complex agent orchestration loops:

| Tool | Description | Parameters |
|------|-------------|------------|
| `find_reply_opportunities` | Discover high-scoring tweets for engagement | `query` (optional), `min_score` (optional), `limit` (optional) |
| `draft_replies_for_candidates` | Generate reply drafts for tweet candidates | `tweet_ids` (required, array) |
| `propose_and_queue_replies` | Submit drafted replies to approval queue or execute | `drafts` (required, array) |
| `generate_thread_plan` | Plan a multi-tweet thread structure | `topic` (required), `target_tweets` (optional) |

---

## Provider Selection

### Configuration

```toml
[x_api]
provider_backend = "scraper"          # "x_api" (default) or "scraper"
scraper_allow_mutations = false       # default: mutations blocked
```

### Capabilities Output

The `get_capabilities` tool reports the active provider:

```json
{
  "success": true,
  "data": {
    "provider": {
      "backend": "x_api",
      "mutations_available": true,
      "risk_level": "standard",
      "data_confidence": "high",
      "unsupported_methods": [],
      "note": "Official X API via OAuth 2.0."
    }
  }
}
```

Scraper backend:

```json
{
  "provider": {
    "backend": "scraper",
    "mutations_available": false,
    "risk_level": "elevated",
    "data_confidence": "medium",
    "unsupported_methods": [
      "get_user_mentions", "get_home_timeline", "get_me", "get_bookmarks"
    ],
    "note": "Scraper backend (read-only). Mutations blocked by default."
  }
}
```

---

## Policy Engine

### Configuration

```toml
[mcp_policy]
enforce_for_mutations = true
require_approval_for = ["x_post_tweet", "x_reply_to_tweet"]
blocked_tools = []
dry_run_mutations = false
max_mutations_per_hour = 20
```

### Policy Error Responses

Rate-limited:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "policy_denied_rate_limited",
    "message": "Policy denied: rate limited",
    "retryable": false,
    "rate_limit_reset": "2026-02-25T13:00:00Z",
    "policy_decision": "denied"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 3 }
}
```

Routed to approval queue:

```json
{
  "success": true,
  "data": {
    "routed_to_approval": true,
    "approval_queue_id": 42,
    "reason": "tool 'x_post_tweet' requires approval"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 5 }
}
```

Dry-run mode:

```json
{
  "success": true,
  "data": {
    "dry_run": true,
    "would_execute": "x_post_tweet",
    "params": "{\"text\":\"Hello!\"}"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 2 }
}
```

---

## Capability Matrix: TuitBot vs Thin X MCP Wrappers

| Capability | TuitBot MCP | Thin X Wrapper |
|------------|-------------|----------------|
| Direct X read tools | 14 tools | Yes |
| Direct X write/engage/media tools | 14 tools | Yes |
| Three integration lanes (API/Scraper/Workflow) | Yes | No |
| Scraper backend (no API tokens required) | Yes | No |
| Centralized mutation policy engine | Yes — per-tool blocking, approval routing, dry-run, rate limits | No |
| Approval queue routing | Yes — configurable via `require_approval_for` | No |
| Dry-run mode | Yes — `dry_run_mutations = true` | No |
| Hourly mutation rate limiting | Yes — `max_mutations_per_hour` | No |
| Composite goal-oriented workflows | 4 tools (find → draft → queue, thread planning) | No |
| Context intelligence | 3 tools (author profiling, recommendations, topic analysis) | No |
| Growth analytics via MCP | 7 tools | No |
| Content generation (LLM-powered) | 4 tools | No |
| Structured response envelope | v1.0 — all 64 tools return `success`, `data`, `error`, `meta` | Varies |
| Typed error taxonomy | 28 error codes with `retryable`, `rate_limit_reset`, `policy_decision` | Limited |
| Per-invocation telemetry | Yes — latency, success, error code, policy decision | No |
| Operating mode awareness | Yes — Autopilot / Composer mode-specific behavior | No |

---

## Migrating from a Thin X MCP Wrapper

### Step 1: Install and configure

```bash
cargo install tuitbot-cli --locked
tuitbot init        # creates ~/.tuitbot/config.toml
tuitbot auth        # OAuth 2.0 PKCE flow for X
```

### Step 2: Start the MCP server

```bash
tuitbot mcp serve                  # Lane 3: Workflow (default)
tuitbot mcp serve --profile api    # Lane 1: API only
```

### Step 3: Map your tool calls

| Thin Wrapper Tool | TuitBot Equivalent | Notes |
|-------------------|--------------------|-------|
| `search_tweets` | `x_search_tweets` | Same parameters; returns v1.0 envelope |
| `post_tweet` | `x_post_tweet` | Policy-gated; may route to approval queue |
| `reply_to_tweet` | `x_reply_to_tweet` | Policy-gated |
| `quote_tweet` | `x_quote_tweet` | Policy-gated |
| `delete_tweet` | `x_delete_tweet` | Policy-gated |
| `like_tweet` | `x_like_tweet` | Policy-gated |
| `unlike_tweet` | `x_unlike_tweet` | Policy-gated |
| `follow_user` | `x_follow_user` | Policy-gated |
| `unfollow_user` | `x_unfollow_user` | Policy-gated |
| `retweet` | `x_retweet` | Policy-gated |
| `unretweet` | `x_unretweet` | Policy-gated |
| `bookmark` | `x_bookmark_tweet` | Policy-gated |
| `unbookmark` | `x_unbookmark_tweet` | Policy-gated |
| `get_tweet` | `get_tweet_by_id` | Direct read |
| `get_user` | `x_get_user_by_username` | Direct read |
| `get_user_by_id` | `x_get_user_by_id` | Direct read |
| `get_mentions` | `x_get_user_mentions` | Direct read |
| `get_user_tweets` | `x_get_user_tweets` | Direct read |
| `get_timeline` | `x_get_home_timeline` | Direct read |
| `get_followers` | `x_get_followers` | Direct read |
| `get_following` | `x_get_following` | Direct read |
| `get_likes` | `x_get_liked_tweets` | Direct read |
| `get_bookmarks` | `x_get_bookmarks` | Direct read |
| `get_me` | `x_get_me` | API profile only |
| `upload_media` | `x_upload_media` | Policy-gated |

### Step 4: Configure safety policy (recommended)

```toml
[mcp_policy]
enforce_for_mutations = true
require_approval_for = ["x_post_tweet", "x_reply_to_tweet"]
dry_run_mutations = false
max_mutations_per_hour = 20
```

Start with `dry_run_mutations = true` to verify agent behavior.

### Step 5: Upgrade to composite workflows (optional)

1. `find_reply_opportunities` — discover high-scoring tweets
2. `draft_replies_for_candidates` — generate contextual replies
3. `propose_and_queue_replies` — submit to approval queue or execute
4. `generate_thread_plan` — plan multi-tweet threads

---

## Operational Notes

- MCP server uses same config and DB as CLI.
- Use approval mode if agent autonomy should be constrained. In Composer mode, approval mode is always on.
- Prefer Composer mode for agents that should assist rather than act autonomously.
- Prefer JSON outputs for deterministic agent behavior.
- The `--profile api` flag is useful for agents that only need X API access without workflow overhead.
- Scraper backend carries elevated risk of account restrictions — use for read-heavy, experimental integrations only.
