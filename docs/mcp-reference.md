# MCP Reference

Tuitbot ships with an MCP server so AI agents can call tools with typed inputs.
The server exposes up to **140 tools** across four profiles — from minimal
read-only surfaces to the full autonomous growth co-pilot with enterprise
API coverage (DMs, Ads, Compliance, Stream Rules).

## Quick Start

```bash
# Write profile (112 tools, default)
tuitbot mcp serve

# Admin profile (139 tools — adds Ads, Compliance, Stream Rules, universal request)
tuitbot mcp serve --profile admin

# Read-only profile (14 tools)
tuitbot mcp serve --profile readonly

# API read-only profile (45 tools)
tuitbot mcp serve --profile api-readonly

# With custom config
tuitbot -c /path/to/config.toml mcp serve
```

## Machine-Readable Manifests

Profile-specific tool manifests are generated from source and committed as JSON:

| Profile | File | Tools |
|---------|------|-------|
| `write` | [`docs/generated/mcp-manifest-write.json`](generated/mcp-manifest-write.json) | 112 |
| `admin` | [`docs/generated/mcp-manifest-admin.json`](generated/mcp-manifest-admin.json) | 139 |
| `readonly` | [`docs/generated/mcp-manifest-readonly.json`](generated/mcp-manifest-readonly.json) | 14 |
| `api-readonly` | [`docs/generated/mcp-manifest-api-readonly.json`](generated/mcp-manifest-api-readonly.json) | 45 |

These files include tool names, categories, mutation flags, dependency
requirements, profiles, and possible error codes. Regenerate after any tool or
profile change with `bash scripts/generate-mcp-manifests.sh`.

## MCP Profiles

TuitBot's MCP server offers four profiles, each exposing a curated set of tools:

| Profile | Command | Tools | Use Case |
|---------|---------|-------|----------|
| **Write** (default) | `tuitbot mcp serve` | 112 | Standard operating profile — reads, writes, DMs, analytics, content gen, approval workflows, generated X API tools |
| **Admin** | `tuitbot mcp serve --profile admin` | 139 | Superset of Write — adds Ads API (16 tools), Compliance (4 tools), Stream Rules (3 tools), and universal request tools |
| **Read-only** | `tuitbot mcp serve --profile readonly` | 14 | Minimal safe surface — utility, config, health, scoring tools only |
| **API read-only** | `tuitbot mcp serve --profile api-readonly` | 45 | X API reads + DM reads + utility tools — no mutations, no workflow tools |

### Choosing a Profile

| Question | Answer | Profile |
|----------|--------|---------|
| Need full growth co-pilot with content gen and approval workflows? | Yes | `write` |
| Need arbitrary X API endpoint access (power users / debugging)? | Yes | `admin` |
| Need X API reads without any mutation risk? | Yes | `api-readonly` |
| Need minimal tooling for config validation and health checks? | Yes | `readonly` |
| Replacing a thin X MCP wrapper? | Yes | `write` or `api-readonly` |
| Default / unsure? | — | `write` (default) |

### Why Read-Only TuitBot?

Read-only profiles (`readonly` and `api-readonly`) give AI agents structured access to TuitBot's intelligence without any mutation risk:

1. **Typed schemas + structured errors** — every tool returns a v1.0 envelope (`success`, `data`, `error`, `meta`) with 28 typed error codes.
2. **Rate-limit awareness** — built-in backoff and retry logic; never burns your API quota.
3. **Stable output formats** — machine-readable manifests and deterministic JSON for reliable agent parsing.
4. **Reliability** — retry, backoff, and pagination built into every tool invocation.
5. **Higher-level read intelligence** — scoring, health checks, config validation, and X API reads without mutation risk.

Read-only profiles are safe by construction: mutation tools are never registered on the server, not merely policy-blocked.

## Claude Code Configuration

**Write profile (default, recommended):**

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

**Admin profile (adds universal X API request tools):**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve", "--profile", "admin"]
    }
  }
}
```

**Read-only profile:**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve", "--profile", "readonly"]
    }
  }
}
```

**API read-only profile:**

```json
{
  "mcpServers": {
    "tuitbot": {
      "command": "tuitbot",
      "args": ["mcp", "serve", "--profile", "api-readonly"]
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
| `compose_tweet` | Create a draft or scheduled tweet | `content` (required), `content_type` (optional), `scheduled_for` (optional) | Write + Admin |

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

## Write-Profile Tools (30)

These tools are available in the Write and Admin profiles (`tuitbot mcp serve`, the default). They provide analytics, content generation, approval workflows, discovery, and composite multi-step operations.

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

## Direct Message Tools (8)

DM tools provide typed access to the X v2 Direct Message API. Read tools are available from API-readonly and above; mutation tools require Write or Admin profiles. All DM tools require `dm.read` and `users.read` OAuth scopes; mutations additionally require `dm.write`.

### DM Reads (5)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_v2_dm_conversations` | List DM conversations | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` (all optional) | ApiRO, Write, Admin |
| `x_v2_dm_conversation_by_id` | Get a specific DM conversation | `id` (required), `dm_event_fields`, `expansions` (optional) | ApiRO, Write, Admin |
| `x_v2_dm_events_by_conversation` | List DM events in a conversation | `id` (required), `max_results`, `pagination_token`, `dm_event_fields`, `expansions` (optional) | ApiRO, Write, Admin |
| `x_v2_dm_events_by_participant` | List DM events with a participant | `participant_id` (required), `max_results`, `pagination_token`, `dm_event_fields`, `expansions` (optional) | ApiRO, Write, Admin |
| `x_v2_dm_events` | List all DM events | `max_results`, `pagination_token`, `dm_event_fields`, `expansions` (all optional) | ApiRO, Write, Admin |

### DM Mutations (3)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_v2_dm_send_in_conversation` | Send a message in an existing conversation | `id` (required), `text` (required), `attachments` (optional) | Write, Admin |
| `x_v2_dm_send_to_participant` | Send a message to a user | `participant_id` (required), `text` (required), `attachments` (optional) | Write, Admin |
| `x_v2_dm_create_group` | Create a group DM conversation | `participant_ids` (required, array), `text` (required), `attachments` (optional) | Write, Admin |

**Safety controls:** DM mutations are policy-gated (approval routing, rate limiting, dry-run mode) and recorded in the mutation audit log. DM reads have zero mutation risk.

**Prerequisites:** Your X API app must have DM access enabled. Request the `dm.read` and `dm.write` OAuth scopes during `tuitbot auth`.

---

## Ads / Campaign Tools (16)

Ads tools provide typed access to the X Ads API v12 via `ads-api.x.com`. All 16 tools are **Admin-only** because Ads API access requires a separate developer account approval and mutations can incur financial costs.

### Ads Reads (9)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_ads_accounts` | List Ads accounts | `account_id` (optional) | Admin |
| `x_ads_account_by_id` | Get a specific Ads account | `account_id` (required) | Admin |
| `x_ads_campaigns` | List campaigns in an account | `account_id` (required) | Admin |
| `x_ads_campaign_by_id` | Get a specific campaign | `account_id` (required), `campaign_id` (required) | Admin |
| `x_ads_line_items` | List line items in an account | `account_id` (required) | Admin |
| `x_ads_promoted_tweets` | List promoted tweets | `account_id` (required) | Admin |
| `x_ads_targeting_criteria` | List targeting criteria | `account_id` (required) | Admin |
| `x_ads_analytics` | Get campaign analytics | `account_id` (required) | Admin |
| `x_ads_funding_instruments` | List funding instruments | `account_id` (required) | Admin |

### Ads Mutations (7)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_ads_campaign_create` | Create a campaign | `account_id` (required), campaign fields | Admin |
| `x_ads_campaign_update` | Update a campaign | `account_id` (required), `campaign_id` (required), fields to update | Admin |
| `x_ads_campaign_delete` | Delete a campaign | `account_id` (required), `campaign_id` (required) | Admin |
| `x_ads_line_item_create` | Create a line item | `account_id` (required), line item fields | Admin |
| `x_ads_promoted_tweet_create` | Create a promoted tweet | `account_id` (required), promotion fields | Admin |
| `x_ads_targeting_create` | Create targeting criteria | `account_id` (required), targeting fields | Admin |
| `x_ads_targeting_delete` | Delete targeting criteria | `account_id` (required), `id` (required) | Admin |

**Safety controls:** Ads mutations are policy-gated and audit-logged. Financial guardrails (spend limits, budget caps) are managed in the X Ads dashboard, not in TuitBot.

**Prerequisites:** Your X developer account must have [Ads API access](https://developer.x.com/en/docs/twitter-ads-api/getting-started). Ads tools will return `x_forbidden` if your credentials lack Ads API authorization.

**Host routing:** All Ads tools route to `ads-api.x.com` (added to the host allowlist alongside `api.x.com`, `upload.x.com`, and `upload.twitter.com`).

---

## Compliance & Stream Rules Tools (7)

Compliance and Stream Rules tools provide typed access to the X v2 Compliance and Filtered Stream APIs. All 7 tools are **Admin-only** and require elevated API access.

### Compliance (4)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_v2_compliance_jobs` | List compliance jobs | query fields (optional) | Admin |
| `x_v2_compliance_job_by_id` | Get a specific compliance job | `id` (required) | Admin |
| `x_v2_compliance_job_create` | Create a compliance job | job fields (required) | Admin |
| `x_v2_usage_tweets` | Get tweet usage statistics | query fields (optional) | Admin |

### Stream Rules (3)

| Tool | Description | Parameters | Profile |
|------|-------------|------------|---------|
| `x_v2_stream_rules_list` | List filtered stream rules | None | Admin |
| `x_v2_stream_rules_add` | Add filtered stream rules | `rules` (required, array) | Admin |
| `x_v2_stream_rules_delete` | Delete filtered stream rules | `rule_ids` (required, array) | Admin |

**Design note:** Stream rule *delete* uses POST with a `delete` body payload (X API design quirk). TuitBot handles this internally — callers provide `rule_ids` and the tool constructs the correct request body.

**What is NOT included:** The filtered stream connection endpoint (`GET /2/tweets/search/stream`) is not implemented. It is a long-lived SSE connection that does not fit the MCP request/response model. Only stream rule CRUD is supported.

**Prerequisites:** Compliance tools require `compliance.write` scope. Stream rules require `tweet.read` scope. Both require elevated API access (Enterprise or Academic tier).

---

## Admin-Only Tools — Universal Request (4)

These universal request tools are available only in the Admin profile (`--profile admin`). They allow access to any X API endpoint reachable with your credentials and are intended for power users, debugging, and ad-hoc API exploration.

| Tool | Description | Parameters |
|------|-------------|------------|
| `x_get` | Send a GET request to any X API endpoint | `endpoint` (required), `query_params` (optional) |
| `x_post` | Send a POST request to any X API endpoint | `endpoint` (required), `body` (optional) |
| `x_put` | Send a PUT request to any X API endpoint | `endpoint` (required), `body` (optional) |
| `x_delete` | Send a DELETE request to any X API endpoint | `endpoint` (required) |

These tools bypass the typed tool layer and send raw requests to the X API. They are excluded from the Write profile to prevent unintended mutations through unstructured API calls.

### Admin Profile Scope

The Admin profile is a **superset of the Write profile**. It adds 27 Admin-only tools:

- **16 Ads API tools** — campaign reads + mutations via `ads-api.x.com`
- **4 Compliance tools** — GDPR compliance job management
- **3 Stream Rules tools** — filtered stream rule CRUD
- **4 Universal request tools** — raw HTTP access to any allowed host

**What "admin" means:**
- Full access to all 112 Write-profile tools (reads, writes, DMs, engagements, analytics, content generation, approval workflows, discovery, policy, telemetry, composite workflows, and generated spec-pack tools).
- Plus 27 Admin-only tools covering Ads, Compliance, Stream Rules, and universal request access.
- Universal request tools are constrained to approved hosts (`api.x.com`, `upload.x.com`, `upload.twitter.com`, `ads-api.x.com`) — no arbitrary outbound HTTP.
- All typed mutations (DM, Ads, Compliance, Stream Rules) are policy-gated with approval routing, rate limiting, and dry-run mode.
- Universal request mutations are constrained by SSRF guards, path validation, and header blocklist but are **not** currently subject to the MCP policy engine. Policy integration is planned as a post-launch enhancement.
- All tool invocations are logged to the `mcp_telemetry` table.

**What "admin" does NOT mean:**
- It does not grant X platform admin privileges (account suspension, content moderation at scale, etc.).
- It does not bypass X API tier restrictions — your API plan's rate limits and endpoint access still apply.
- It does not disable TuitBot's safety policy engine for typed tools. Note: universal request mutations bypass the policy engine (constrained instead by host allowlist and SSRF guards).
- Ads API tools will fail with `x_forbidden` if your developer account lacks Ads API authorization.

**When to use Admin:**
- Managing X Ads campaigns, compliance jobs, or filtered stream rules.
- Exploring X API endpoints that lack a dedicated typed tool.
- Debugging API responses at the raw HTTP level.
- One-off operations that don't justify a dedicated tool.

**When not to use Admin:**
- Standard growth operations — the Write profile covers all normal workflows including DMs.
- Untrusted agents — use `write`, `api-readonly`, or `readonly` to limit blast radius.

---

## API Coverage Boundaries

TuitBot provides maximum coverage of the **X API v2 public surface** plus enterprise APIs (DMs, Ads, Compliance, Stream Rules). The following surfaces are explicitly out of scope:

### Not Supported

| Surface | Status | Reason |
|---------|--------|--------|
| **X Premium/Enterprise-only endpoints** | Partial | Some v2 endpoints (e.g., full-archive search) require Premium or Enterprise API plans. TuitBot's typed tools cover standard-tier endpoints plus Compliance and Stream Rules for enterprise users. If your plan grants access to additional endpoints, the admin-profile universal request tools can reach them. |
| **X API v1.1 (legacy)** | Not targeted | TuitBot targets v2 exclusively. The universal request tools accept v1.1 paths but provide no v1.1-specific handling, pagination, or error mapping. |
| **Account administration** | Not available | Suspend/unsuspend accounts, manage app permissions, or modify developer portal settings. These are platform-level operations, not API operations. |
| **Filtered stream connections** | Not available | The long-lived SSE endpoint (`GET /2/tweets/search/stream`) does not fit MCP's request/response model. Stream rule CRUD is supported. |

### Supported Surface Summary

TuitBot's **140 tools** (73 curated L1 + 67 generated L2) cover the following areas:

| Area | Typed Tools | Coverage | Profile |
|------|-------------|----------|---------|
| Tweet reads (lookup, search, timelines, counts) | 26 | Comprehensive | ApiRO+ |
| Tweet writes (post, reply, quote, delete, threads) | 11 | Comprehensive | Write+ |
| Engagements (like, retweet, bookmark, follow, pin) | 10 | Comprehensive | Write+ |
| User reads (lookup, followers, following) | 6 | Comprehensive | ApiRO+ |
| Direct Messages (conversations, events, send) | 8 | Comprehensive | ApiRO+ (reads), Write+ (mutations) |
| Lists (CRUD, members, followers, pins) | 15 | Comprehensive | ApiRO+ (reads), Write+ (mutations) |
| Moderation (mutes, blocks, hide replies) | 8 | Comprehensive | Write+ |
| Spaces (lookup, search, buyers, tweets) | 6 | Comprehensive | ApiRO+ |
| Ads / Campaign (accounts, campaigns, targeting, analytics) | 16 | Comprehensive | Admin only |
| Compliance (jobs, usage) | 4 | Comprehensive | Admin only |
| Stream Rules (list, add, delete) | 3 | Comprehensive | Admin only |
| Media (upload) | 1 | Basic | Write+ |
| Universal request (any allowed host) | 4 | Escape hatch | Admin only |

TuitBot aims for **maximum API coverage** — every X API v2 endpoint plus enterprise surfaces (DMs, Ads, Compliance) should have either a typed tool or be reachable via the admin-profile universal request tools.

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
| Generated X API tools (Layer 2) | 67 tools | No |
| Enterprise API tools (DM, Ads, Compliance, Stream) | 31 tools | No |
| Universal X API request tools (admin) | 4 tools | No |
| Four profiles (write/admin/readonly/api-readonly) | Yes | No |
| Scraper backend (no API tokens required) | Yes | No |
| Centralized mutation policy engine | Yes — per-tool blocking, approval routing, dry-run, rate limits | No |
| Approval queue routing | Yes — configurable via `require_approval_for` | No |
| Dry-run mode | Yes — `dry_run_mutations = true` | No |
| Hourly mutation rate limiting | Yes — `max_mutations_per_hour` | No |
| Composite goal-oriented workflows | 4 tools (find → draft → queue, thread planning) | No |
| Context intelligence | 3 tools (author profiling, recommendations, topic analysis) | No |
| Growth analytics via MCP | 7 tools | No |
| Content generation (LLM-powered) | 4 tools | No |
| Structured response envelope | v1.0 — all 140 tools return `success`, `data`, `error`, `meta` | Varies |
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
tuitbot mcp serve                          # Write profile (default, 112 tools)
tuitbot mcp serve --profile admin          # Admin profile (139 tools, adds Ads/Compliance/Stream/universal request)
tuitbot mcp serve --profile api-readonly   # API read-only (45 tools, no mutations)
tuitbot mcp serve --profile readonly       # Read-only (14 tools, minimal surface)
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
- Use `--profile api-readonly` for agents that only need X API reads without workflow overhead, or `--profile readonly` for the minimal safe surface.
- Use `--profile admin` when you need Ads API, Compliance, Stream Rules, or universal request access. The default `write` profile covers all standard operations including DMs.
- Enterprise API tools (Ads, Compliance, Stream Rules) require additional X API access. See [API Coverage Boundaries](#api-coverage-boundaries) for details.
- Scraper backend carries elevated risk of account restrictions — use for read-heavy, experimental integrations only.

---

## MCP Profiles — Release Checklist

### Completed Tasks

1. Four MCP profiles (`write`/112, `admin`/139, `readonly`/14, `api-readonly`/45) with curated tool routing — read-only profiles are safe by construction (mutation tools not registered); admin tools structurally absent from write profile.
2. `mcp manifest` CLI command for machine-readable profile introspection (`--format json|table`).
3. Generated JSON manifest artifacts in `docs/generated/` (`write.json`, `admin.json`, `readonly.json`, `api-readonly.json`).
4. Boundary tests covering isolation, mutation denylists, lane constraints, dependency validation, error codes, and admin-only tool exclusion from write profile.
5. Conformance tests (27 kernel + 31 spec) with golden fixture snapshots for schema drift detection.
6. Eval harness (4 scenarios, 4 quality gates) for continuous profile validation.
7. Manifest-sync CI job with `scripts/check-mcp-manifests.sh` drift guard.
8. Full documentation update — MCP reference, CLI reference, README, and CHANGELOG.
9. Operator runbook for profile verification (see `docs/operations.md`).
10. Final validation pass — all quality gates green.
11. Session 4: Spec pack and tool generation pipeline — 67 generated endpoint tools from `EndpointDef` spec (36 original + 31 enterprise).
12. Session 5: Strict 4-profile model — renamed Full→Write, added Admin profile with universal request tools, structural enforcement via separate server structs.
13. X Enterprise API Parity (Sessions 01-07): 31 new enterprise tools — DM (8), Ads (16), Compliance (4), Stream Rules (3). Host allowlist extended with `ads-api.x.com`. 59 conformance tests. Full documentation alignment.

### Known Limitations

1. Write/Engage tool tables use "Profile: Both" column — refers to Write+Admin profiles. Cosmetic; no runtime impact.
2. `mkdocs build --strict` not enforced in CI — docs validated manually.
3. No live X API integration test — all tests use mock providers. Safety enforced structurally.

### Rollback Strategy

1. `git revert <sha>` on the merge commit to main.
2. Verify with `cargo test -p tuitbot-mcp boundary` — boundary tests must still pass.
3. Partial rollback: users switch `--profile` flag to a working profile.
4. Regenerate manifests after rollback: `bash scripts/generate-mcp-manifests.sh`.
