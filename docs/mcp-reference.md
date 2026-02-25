# MCP Reference

Tuitbot ships with an MCP server so AI agents can call tools with typed inputs.

## Run MCP server

```bash
tuitbot mcp serve
```

With custom config:

```bash
tuitbot -c /path/to/config.toml mcp serve
```

## Tool categories

- Analytics
- Action log
- Rate limits
- Replies and discovery
- Targets
- Scoring
- Approval queue
- Content generation
- Config and health
- Composer mode

## Claude Code example

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

## Composer mode tools

These tools support user-driven workflows in Composer mode:

| Tool | Description | Parameters |
|---|---|---|
| `get_mode` | Returns the current operating mode (`autopilot` or `composer`) | None |
| `compose_tweet` | Generate a tweet using AI Assist | `topic` (required), `format` (optional) |
| `get_discovery_feed` | Retrieve scored tweets from the Discovery Feed | `limit` (optional), `min_score` (optional) |
| `suggest_topics` | Get topic suggestions based on profile and performance data | `count` (optional) |

## Response Envelope (v1.0)

Migrated tools wrap their output in a unified JSON envelope. Non-migrated tools
continue to return their original JSON shape. Agents can detect the envelope by
checking for the top-level `"success"` key.

### Example

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

### Field reference

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | Whether the tool call succeeded |
| `data` | `any` | Tool payload (object, array, or null on error) |
| `error` | `object?` | Present only on failure |
| `error.code` | `string` | Machine-readable code (e.g. `db_error`) |
| `error.message` | `string` | Human-readable description |
| `error.retryable` | `bool` | Whether the caller may retry |
| `meta` | `object?` | Execution metadata (optional) |
| `meta.tool_version` | `string` | Envelope schema version |
| `meta.elapsed_ms` | `u64` | Wall-clock execution time in ms |
| `meta.mode` | `string?` | Operating mode (`autopilot` / `composer`) |
| `meta.approval_mode` | `bool?` | Effective approval mode flag |

### Detection strategy

Check for the top-level `"success"` key. If present, the response uses the
envelope schema. If absent, treat it as a legacy (non-migrated) response.

### Migrated tools (v1.0)

| Tool | Error codes |
|------|-------------|
| `get_capabilities` | — (always succeeds) |
| `health_check` | — (always succeeds; degradation in data) |
| `get_stats` | `db_error` |
| `list_pending_approvals` | `db_error` |
| `get_discovery_feed` | `db_error` |
| All Direct X API tools | `x_not_configured`, `x_rate_limited`, `x_auth_expired`, `x_forbidden`, `x_account_restricted`, `x_network_error`, `x_api_error` |

## Direct X API Tools

These tools give agents direct access to X API v2 endpoints. They require
the MCP server to have valid OAuth tokens (`tuitbot auth`). Check availability
via `get_capabilities` → `direct_tools`.

### Read Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_tweet_by_id` | Fetch a single tweet by ID | `tweet_id` (required) |
| `x_get_user_by_username` | Look up a user profile by @username | `username` (required) |
| `x_search_tweets` | Search recent tweets (Basic/Pro tier) | `query` (required), `max_results` (optional, 10-100), `since_id` (optional) |
| `x_get_user_mentions` | Get mentions of the authenticated user | `since_id` (optional) |
| `x_get_user_tweets` | Get recent tweets from a user | `user_id` (required), `max_results` (optional, 5-100) |

### Mutation Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `x_post_tweet` | Post a new tweet | `text` (required) |
| `x_reply_to_tweet` | Reply to an existing tweet | `text` (required), `in_reply_to_id` (required) |
| `x_quote_tweet` | Post a quote tweet | `text` (required), `quoted_tweet_id` (required) |
| `x_like_tweet` | Like a tweet | `tweet_id` (required) |
| `x_follow_user` | Follow a user | `target_user_id` (required) |
| `x_unfollow_user` | Unfollow a user | `target_user_id` (required) |

### Error Codes

| Code | Meaning | Retryable |
|------|---------|-----------|
| `x_not_configured` | X API client not available (no tokens) | No |
| `x_rate_limited` | X API rate limit hit (HTTP 429) | Yes |
| `x_auth_expired` | OAuth token expired (HTTP 401) | No |
| `x_forbidden` | Forbidden / tier restriction (HTTP 403) | No |
| `x_account_restricted` | Account suspended or limited | No |
| `x_network_error` | Network connectivity issue | Yes |
| `x_api_error` | Other X API errors | No |

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

## Policy Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_policy_status` | Get current MCP mutation policy settings and rate limit usage | None |

### Policy Error Codes

Mutation tools may return these additional error codes when policy enforcement is enabled:

| Code | Meaning | Retryable |
|------|---------|-----------|
| `policy_denied_blocked` | Tool is in `blocked_tools` configuration | No |
| `policy_denied_rate_limited` | Hourly MCP mutation rate limit exceeded | No |
| `policy_error` | Policy evaluation failed (DB error) | Yes |

When a mutation is routed to the approval queue, the response is a success envelope:

```json
{
  "success": true,
  "data": {
    "routed_to_approval": true,
    "approval_queue_id": 42,
    "reason": "tool 'post_tweet' requires approval"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 5 }
}
```

When dry-run mode is active:

```json
{
  "success": true,
  "data": {
    "dry_run": true,
    "would_execute": "post_tweet",
    "params": "{\"text\":\"Hello!\"}"
  },
  "meta": { "tool_version": "1.0", "elapsed_ms": 2 }
}
```

## Operational notes

- MCP server uses same config and DB as CLI.
- Use approval mode if agent autonomy should be constrained. In Composer mode, approval mode is always on.
- Prefer Composer mode for agents that should assist rather than act autonomously.
- Prefer JSON outputs for deterministic agent behavior.
