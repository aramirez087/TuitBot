# Operations

## Deployment patterns

### Long-running daemon

- Run under `systemd`, `tmux`, or equivalent supervisor.
- Set restart policy to `on-failure`.

### Tick-based scheduling

- Use cron/systemd timer/launchd/OpenClaw.
- Run every 15-30 minutes.
- Pipe JSON output to logs.

## Logging

- Use JSON output where supported.
- Capture stdout/stderr to centralized logging.
- Track failed loops and repeated skips.

## Backup and recovery

- Snapshot SQLite DB regularly.
- Back up config and token material securely.
- Validate restore procedure before incident.

## Upgrades

```bash
tuitbot update
```

This single command:

1. Checks GitHub Releases for a newer `tuitbot` binary.
2. Downloads, verifies SHA256, and atomically replaces the running binary.
3. Detects new config features and walks you through adding them.

Use `--check` to see if an update is available without installing. Use `--config-only` to skip the binary update. Use `--non-interactive` in CI to apply config defaults automatically.

After updating, run `tuitbot test` to validate auth and connectivity.
