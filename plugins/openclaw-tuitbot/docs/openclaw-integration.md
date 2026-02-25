# OpenClaw + Tuitbot Integration Guide

## Overview

The Tuitbot OpenClaw plugin bridges all 46 Tuitbot MCP tools into the OpenClaw agent platform. It provides layered safety filters, risk-aware tool categorization, and structured error handling out of the box.

**Key safety feature:** Mutations are disabled by default. The plugin starts in read-only mode — you must explicitly opt in to posting, liking, following, and approval actions.

## Installation

1. Install the plugin into your OpenClaw project:

```bash
cd your-openclaw-project
npm install @tuitbot/openclaw-plugin
```

2. Register in your OpenClaw config:

```json
{
  "plugins": [
    {
      "id": "tuitbot",
      "config": {}
    }
  ]
}
```

## Configuration Reference

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `tuitbotBinaryPath` | `string` | `"tuitbot"` | Path to the tuitbot binary |
| `configPath` | `string` | `~/.tuitbot/config.toml` | Path to tuitbot config file |
| `allowedTools` | `string[]` | _(all)_ | Name-based allowlist (most restrictive filter) |
| `enableMutations` | `boolean` | `false` | Enable mutation tools (post, reply, like, follow, approve) |
| `allowCategories` | `string[]` | _(all)_ | Category inclusion filter: `read`, `mutation`, `composite`, `ops` |
| `denyCategories` | `string[]` | _(none)_ | Category exclusion filter |
| `maxRiskLevel` | `string` | `"high"` | Risk ceiling: `low`, `medium`, `high` |

## Configuration Examples

### Default (Read-Only)

Safe startup — only read and ops tools are available:

```json
{
  "plugins": [{
    "id": "tuitbot",
    "config": {}
  }]
}
```

### Full Access

All tools enabled, including high-risk mutations:

```json
{
  "plugins": [{
    "id": "tuitbot",
    "config": {
      "enableMutations": true
    }
  }]
}
```

### Analytics Only

Only read-category tools for dashboards and reporting:

```json
{
  "plugins": [{
    "id": "tuitbot",
    "config": {
      "allowCategories": ["read"]
    }
  }]
}
```

### Risk-Capped Mutations

Enable mutations but block high-risk actions (posting, replying, mass-approve):

```json
{
  "plugins": [{
    "id": "tuitbot",
    "config": {
      "enableMutations": true,
      "maxRiskLevel": "medium"
    }
  }]
}
```

### Explicit Allowlist

Register only specific tools by name:

```json
{
  "plugins": [{
    "id": "tuitbot",
    "config": {
      "allowedTools": ["get_stats", "get_discovery_feed", "score_tweet"]
    }
  }]
}
```

## Error Handling

Tool execution returns an `EnrichedToolResult` with structured error information:

```ts
interface EnrichedToolResult {
  data: unknown;
  success: boolean;
  errorMessage?: string;   // Actionable, human-readable
  errorCode?: string;       // Machine-readable code
  retryable?: boolean;      // Whether the operation can be retried
  meta?: { tool_version: string; elapsed_ms: number; mode?: string };
}
```

### Error Codes

| Code | Message | Retryable? |
|------|---------|------------|
| `x_rate_limited` | X API rate limit hit. Wait before retrying. | Yes |
| `x_auth_expired` | X API authentication expired. Re-authenticate with `tuitbot auth`. | No |
| `x_auth_missing` | X API credentials not configured. Run `tuitbot auth` to set up. | No |
| `x_forbidden` | X API returned 403 Forbidden. Check account permissions. | No |
| `x_not_found` | The requested X resource was not found. | No |
| `llm_not_configured` | LLM provider not configured. Set up the [llm] section in config.toml. | No |
| `llm_generation_failed` | LLM generation failed. Check provider connectivity and API key. | Yes |
| `policy_denied_blocked` | This tool is blocked by MCP policy configuration. | No |
| `policy_denied_approval` | This action requires approval. Submit via the approval queue. | No |
| `safety_duplicate` | Duplicate content detected. This reply was already posted. | No |
| `safety_rate_limit` | Internal rate limit reached. Wait before posting again. | Yes |
| `safety_banned_phrase` | Content contains a banned phrase. Edit and retry. | No |

## Layered Safety

The plugin enforces five layers of filtering, applied in order:

1. **Name allowlist** — If `allowedTools` is set, only those exact tool names pass. Most restrictive.
2. **Mutation gate** — Mutations and policy-gated composites require `enableMutations: true`. This is the primary safety switch.
3. **Category allowlist** — If `allowCategories` is set, only tools in those categories pass.
4. **Category denylist** — If `denyCategories` is set, tools in those categories are blocked.
5. **Risk ceiling** — If `maxRiskLevel` is set, tools above that risk level are blocked.

Unknown tools (not in the catalog) pass all filters by default for forward-compatibility with new Tuitbot versions.

## Tool Categories

| Category | Description | Example Tools |
|----------|-------------|---------------|
| `read` | Data retrieval, content generation (no side effects) | `get_stats`, `generate_reply`, `score_tweet` |
| `ops` | System health and status | `health_check`, `get_mode`, `get_capabilities` |
| `composite` | Multi-step workflows | `find_reply_opportunities`, `propose_and_queue_replies` |
| `mutation` | Actions with side effects on X or the approval queue | `x_post_tweet`, `x_like_tweet`, `approve_all` |
