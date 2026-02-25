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

## MCP Observability & Quality Gates

### Telemetry

Every MCP tool invocation is recorded in the `mcp_telemetry` table with:

- `tool_name` — which tool was called
- `category` — tool category (`composite`, `composite_mutation`, `mutation`, `analytics`, etc.)
- `latency_ms` — wall-clock execution time
- `success` — whether the call succeeded
- `error_code` — machine-readable error code on failure
- `policy_decision` — policy gate outcome (`allow`, `deny`, `route_to_approval`, `dry_run`)

Query telemetry via MCP tools:

- `get_mcp_tool_metrics` — time-windowed aggregates per tool (call counts, success rates, latency percentiles)
- `get_mcp_error_breakdown` — error distribution grouped by tool and error code

### Quality Gates

The eval harness (`cargo test -p tuitbot-mcp eval_harness`) enforces two quality gates:

| Gate | Threshold | Description |
|------|-----------|-------------|
| Schema validation | >= 95% | All MCP tool responses must conform to the `ToolResponse` envelope (`success` key present). As of v1.0, all tools return the envelope. |
| Unknown errors | <= 5% | Error responses must use typed error codes — unclassified errors indicate missing handling |

**Running the eval harness:**

```bash
cargo test -p tuitbot-mcp eval_harness -- --nocapture
```

This executes three scenarios:

1. **Scenario A** — Raw direct reply: draft a reply and queue it
2. **Scenario B** — Composite flow: find opportunities → draft replies → queue replies
3. **Scenario C** — Policy-blocked mutation: verify denied calls are captured in telemetry

Results are written to `docs/roadmap/artifacts/task-07-eval-results.json` and `task-07-eval-summary.md`.

### CI Integration

Add to your CI pipeline after the standard test suite:

```bash
# Quality gates (fail build if thresholds breached)
cargo test -p tuitbot-mcp eval_harness
```

The eval harness asserts quality gates directly — test failure means a gate was breached.
