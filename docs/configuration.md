# Configuration

> **Note**: While power-users and CLI users can edit following paths directly via terminal, the recommended method is using the **Visual Settings UI Editor** inside the Desktop App dashboard under the "Settings" tab.

## Config file location

Default path:

- `~/.tuitbot/config.toml`

Custom path:

```bash
tuitbot -c /path/to/config.toml <command>
```

## High-Level Sections

- `mode`: operating mode (`autopilot` or `composer`).
- `x_api`: OAuth/app settings for X integration.
- `llm`: provider and model settings.
- `limits`: rate and behavior guardrails.
- `intervals`: loop timing.
- `scheduling`: active hours and slot windows.
- `features`: loop enable/disable switches.

## Operating Mode

Tuitbot supports two operating modes that control how much autonomy the agent has:

| Mode | Behavior |
|---|---|
| `autopilot` (default) | All automation loops run. Posts automatically when `approval_mode = false`, or queues for review when `approval_mode = true`. |
| `composer` | Autonomous loops disabled. Discovery runs read-only. Approval mode is always on. You write content with AI Assist and the Discovery Feed. |

```toml
mode = "composer"
```

```bash
export TUITBOT_MODE=composer
```

Setting `mode = "composer"` implies `approval_mode = true` — you do not need to set both. See the [Composer Mode guide](composer-mode.md) for full details.

## Safety Defaults

The default profile is intentionally conservative:

- reply caps
- per-author caps
- banned phrase checks
- active-hours enforcement

## Validation

```bash
tuitbot settings --show
tuitbot test
```

Use these commands after every config change.

## MCP Mutation Policy

The `[mcp_policy]` section controls how MCP mutation tools (post, reply, like, follow, etc.) are gated before execution. This is the safety layer between AI agents and real X API actions.

```toml
[mcp_policy]
enforce_for_mutations = true
require_approval_for = ["post_tweet", "reply_to_tweet", "follow_user", "like_tweet"]
blocked_tools = []
dry_run_mutations = false
max_mutations_per_hour = 20
```

| Field | Default | Description |
|---|---|---|
| `enforce_for_mutations` | `true` | Master switch. Set to `false` to disable all policy checks. |
| `require_approval_for` | `["post_tweet", "reply_to_tweet", "follow_user", "like_tweet"]` | Tools routed to the approval queue. |
| `blocked_tools` | `[]` | Tools completely blocked. Cannot overlap with `require_approval_for`. |
| `dry_run_mutations` | `false` | Return dry-run responses without executing. |
| `max_mutations_per_hour` | `20` | Aggregate hourly rate limit for all MCP mutations. |

**Evaluation order** (safest wins): disabled? > blocked? > dry_run? > rate limited? > requires approval? > allow.

**Composer mode**: All mutations require approval regardless of `require_approval_for`.

**Environment variable overrides**:

```bash
export TUITBOT_MCP_POLICY__ENFORCE_FOR_MUTATIONS=true
export TUITBOT_MCP_POLICY__REQUIRE_APPROVAL_FOR="post_tweet,reply_to_tweet"
export TUITBOT_MCP_POLICY__BLOCKED_TOOLS="follow_user"
export TUITBOT_MCP_POLICY__DRY_RUN_MUTATIONS=false
export TUITBOT_MCP_POLICY__MAX_MUTATIONS_PER_HOUR=20
```

## Production Guidance

- Keep secrets out of shell history.
- Store config on persistent volume.
- Back up SQLite DB before major upgrades.
- Prefer approval mode when testing new prompts.
- Use Composer mode for new accounts until you trust the AI's output quality.
