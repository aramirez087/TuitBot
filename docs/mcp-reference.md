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

## Operational notes

- MCP server uses same config and DB as CLI.
- Use approval mode if agent autonomy should be constrained. In Composer mode, approval mode is always on.
- Prefer Composer mode for agents that should assist rather than act autonomously.
- Prefer JSON outputs for deterministic agent behavior.
