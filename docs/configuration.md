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

## Production Guidance

- Keep secrets out of shell history.
- Store config on persistent volume.
- Back up SQLite DB before major upgrades.
- Prefer approval mode when testing new prompts.
- Use Composer mode for new accounts until you trust the AI's output quality.
