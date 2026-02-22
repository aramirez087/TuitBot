---
name: tuitbot
description: >
  Autonomous X (Twitter) growth assistant for founders and indie hackers.
  Discovers relevant conversations, scores tweets, generates AI-powered
  replies/tweets/threads, monitors target accounts, and tracks analytics.
  Supports human-in-the-loop approval mode.
version: 0.1.0
homepage: https://github.com/aramirez087/tuitbot
metadata:
  openclaw:
    skillKey: tuitbot
    requires:
      bins: [tuitbot]
      env: [TUITBOT_X_API__CLIENT_ID]
    primaryEnv: TUITBOT_LLM__API_KEY
    emoji: "\U0001F916"
---

# Tuitbot Skill

## When to use

Use this skill when the user wants to:

- Grow their X (Twitter) account autonomously
- Discover relevant tweets and conversations in their niche
- Generate and post AI-powered replies, tweets, or threads
- Monitor target accounts and build relationships
- Review and approve queued posts (approval mode)
- View analytics on follower growth and content performance
- Configure posting limits, scoring thresholds, or brand voice

## Installation

```bash
cargo install --git https://github.com/your-org/tuitbot --bin tuitbot
```

Or build from source:

```bash
git clone https://github.com/your-org/tuitbot
cd tuitbot
cargo install --path crates/tuitbot-cli
```

## Setup flow

### 1. Initialize configuration

```bash
tuitbot init --non-interactive
```

This creates `~/.tuitbot/config.toml` with default values.

### 2. Configure settings

Set required values using `tuitbot settings --set`:

```bash
tuitbot settings --set business.product_name=YourProduct
tuitbot settings --set business.product_description="One-line description"
tuitbot settings --set business.product_keywords="keyword1, keyword2"
tuitbot settings --set business.industry_topics="topic1, topic2"
tuitbot settings --set llm.provider=openai
tuitbot settings --set llm.model=gpt-4o-mini
```

Secrets can also be set via environment variables:

```bash
export TUITBOT_LLM__API_KEY="sk-..."
export TUITBOT_X_API__CLIENT_ID="your-client-id"
```

### 3. Authenticate with X

```bash
tuitbot auth
```

This prints a URL. The user opens it in any browser (laptop, phone), authorizes, then pastes back the callback URL. Works on headless servers and VPS — no local browser needed. Tokens are saved to `~/.tuitbot/tokens.json`.

> **Note:** This step requires user interaction (visit URL, paste code). It cannot be fully automated.

### 4. Validate setup

```bash
tuitbot test --output json
```

Returns:
```json
{"passed": true, "checks": [{"label": "Configuration", "passed": true, "message": "loaded from ~/.tuitbot/config.toml"}, ...]}
```

## Available commands

All commands support `--output json` for machine-readable output.

### `tuitbot test --output json`

Validate configuration, credentials, and connectivity.

```json
{"passed": true, "checks": [{"label": "Configuration", "passed": true, "message": "..."}, {"label": "Business profile", "passed": true, "message": "..."}, {"label": "X API auth", "passed": true, "message": "..."}, {"label": "LLM provider", "passed": true, "message": "..."}, {"label": "Database", "passed": true, "message": "..."}]}
```

Exit code 1 if any check fails.

### `tuitbot settings --show --output json`

Show current configuration as JSON (secrets redacted).

```json
{"x_api": {"client_id": "abc", "client_secret": "***REDACTED***"}, "business": {...}, "llm": {"provider": "openai", "api_key": "***REDACTED***", ...}, ...}
```

### `tuitbot settings --set KEY=VALUE`

Set a configuration value directly. Examples:

```bash
tuitbot settings --set scoring.threshold=70
tuitbot settings --set limits.max_replies_per_day=10
tuitbot settings --set approval_mode=true
```

### `tuitbot stats --output json`

Show analytics dashboard.

```json
{"follower_trend": [{"date": "2025-01-15", "follower_count": 1200, "following_count": 300, "tweet_count": 500}], "net_follower_change": 50, "top_topics": [{"topic": "rust", "format": "tip", "total_posts": 5, "avg_performance": 85.0}], "engagement": {"avg_reply_score": 73.5, "avg_tweet_score": 81.2}, "content_measured": {"replies": 25, "tweets": 18}}
```

### `tuitbot tick --require-approval`

Force approval mode on for a single tick (queue posts for human review). This is automatically enabled when running inside OpenClaw (any `OPENCLAW_*` env var detected), but can also be set explicitly.

### `tuitbot run`

Start the autonomous agent. This is a **long-running process** that runs 6 concurrent loops (discovery, mentions, content, threads, target monitoring, analytics).

```bash
# Start in background
nohup tuitbot run > /dev/null 2>&1 &

# Or with tmux
tmux new -d -s tuitbot 'tuitbot run'
```

Press Ctrl+C for graceful shutdown (30s timeout).

### `tuitbot discover --dry-run`

Run one discovery cycle without posting. Useful for testing scoring.

### `tuitbot post --dry-run`

Generate a tweet without posting.

### `tuitbot thread --dry-run`

Generate a thread without posting.

## MCP server

Tuitbot includes a built-in MCP (Model Context Protocol) server for structured tool access over stdio. This is the preferred integration method for AI agents — no CLI output parsing required.

### Starting the server

```bash
tuitbot mcp serve
```

With a custom config path:

```bash
tuitbot -c /path/to/config.toml mcp serve
```

The server communicates via JSON-RPC 2.0 over stdin/stdout (newline-delimited).

### Available MCP tools

| Category | Tools |
|---|---|
| **Analytics** | `get_stats`, `get_follower_trend` |
| **Action Log** | `get_action_log`, `get_action_counts` |
| **Rate Limits** | `get_rate_limits` |
| **Replies** | `get_recent_replies`, `get_reply_count_today` |
| **Targets** | `list_target_accounts` |
| **Discovery** | `list_unreplied_tweets` |
| **Scoring** | `score_tweet` (6-signal engine) |
| **Approval Queue** | `list_pending_approvals`, `get_pending_count`, `approve_item`, `reject_item`, `approve_all` |
| **Content Generation** | `generate_reply`, `generate_tweet`, `generate_thread` (requires LLM provider) |
| **Capabilities** | `get_capabilities` (tier, rate limits, recommended max actions) |
| **Config & Health** | `get_config`, `validate_config`, `health_check` |

### MCP vs CLI

- **MCP**: Typed JSON inputs/outputs, tool discovery via `tools/list`, no shell parsing. Best for programmatic agent integration.
- **CLI with `--output json`**: Simpler to invoke ad-hoc but requires spawning a process per command and parsing stdout.

When running inside OpenClaw via the plugin (`plugins/openclaw-tuitbot/`), MCP tools are automatically bridged into native OpenClaw tools prefixed with `tuitbot_` (e.g., `tuitbot_health_check`, `tuitbot_get_stats`).

## Approval workflow

When `approval_mode = true`, all generated content is queued for review instead of being posted immediately.

### List pending items

```bash
tuitbot approve --list --output json
```

Returns:
```json
[{"id": 1, "action_type": "reply", "target_tweet_id": "123", "target_author": "@user", "generated_content": "Great point!", "topic": "rust", "archetype": "AgreeAndExpand", "score": 85.0, "created_at": "2025-01-15T10:30:00Z"}]
```

### Approve a specific item

```bash
tuitbot approve --approve 1 --output json
```

Returns:
```json
{"id": 1, "status": "approved"}
```

### Reject a specific item

```bash
tuitbot approve --reject 1 --output json
```

Returns:
```json
{"id": 1, "status": "rejected"}
```

### Approve all pending items

```bash
tuitbot approve --approve-all --output json
```

Returns:
```json
[{"id": 1, "status": "approved"}, {"id": 2, "status": "approved"}]
```

## Configuration reference

Key settings that can be modified via `tuitbot settings --set`:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `business.product_name` | string | "" | Your product name |
| `business.product_keywords` | CSV | [] | Discovery keywords |
| `business.industry_topics` | CSV | [] | Content generation topics |
| `llm.provider` | string | "" | AI provider: openai, anthropic, ollama |
| `llm.model` | string | "" | Model name (e.g., gpt-4o-mini) |
| `scoring.threshold` | 0-100 | 60 | Minimum score to trigger a reply |
| `limits.max_replies_per_day` | int | 5 | Maximum replies per day |
| `limits.max_tweets_per_day` | int | 6 | Maximum tweets per day |
| `limits.product_mention_ratio` | 0.0-1.0 | 0.2 | Fraction of replies mentioning product |
| `approval_mode` | bool | false | Queue posts for human review |
| `schedule.timezone` | string | UTC | IANA timezone for active hours |
| `schedule.active_hours_start` | 0-23 | 8 | Start of posting window |
| `schedule.active_hours_end` | 0-23 | 22 | End of posting window |

Environment variables use `TUITBOT_` prefix with `__` separator: `TUITBOT_LLM__API_KEY`, `TUITBOT_SCORING__THRESHOLD`.

Special env var: `TUITBOT_APPROVAL_MODE=true|false` overrides the `approval_mode` config value. When any `OPENCLAW_*` env var is present, approval mode is automatically enabled unless `TUITBOT_APPROVAL_MODE=false` explicitly opts out.

## OpenClaw plugin

Tuitbot ships with an OpenClaw plugin at `plugins/openclaw-tuitbot/` that bridges MCP tools into native OpenClaw tool registrations. When the plugin is loaded, approval mode is enabled by default for safety. See `plugins/openclaw-tuitbot/openclaw.plugin.json` for configuration schema.

## Limitations

- **`tuitbot run` is long-running**: Start it as a background process. It does not return until stopped.
- **`tuitbot auth` requires user interaction**: OAuth 2.0 PKCE flow requires the user to visit a URL and paste back a code. Works on headless servers (VPS, SSH, OpenClaw) — no local browser needed — but cannot be fully automated.
- **X API tier limits**: Discovery and replies require a paid X API tier or pay-per-use credits. Posting tweets/threads works on the Free tier.
- **Rate limits**: The agent respects X API rate limits and configurable posting limits. It will not exceed them.
