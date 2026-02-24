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

- `x_api`: OAuth/app settings for X integration.
- `llm`: provider and model settings.
- `limits`: rate and behavior guardrails.
- `intervals`: loop timing.
- `scheduling`: active hours and slot windows.
- `features`: loop enable/disable switches.

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
