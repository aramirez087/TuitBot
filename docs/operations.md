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

```bash
tuitbot backup                     # create backup to ~/.tuitbot/backups/
tuitbot backup --list              # list existing backups
tuitbot backup --prune 5           # keep 5 most recent
tuitbot restore ./backup.db        # restore with confirmation
tuitbot restore ./backup.db --validate-only  # validate only
```

Pre-migration backups are created automatically on startup when the DB already exists.

## Runbooks

Step-by-step operational guides are available in [`docs/runbooks/`](runbooks/README.md):

- [Incident Response](runbooks/incident-response.md) — general triage
- [Auth Expiry](runbooks/auth-expiry.md) — token refresh failures
- [Rate Limit Storms](runbooks/rate-limit-storms.md) — circuit breaker events
- [Backup & Restore](runbooks/backup-restore.md) — scheduled backups, recovery
- [Database Maintenance](runbooks/database-maintenance.md) — WAL, VACUUM, cleanup

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

## Architecture Layer Guide

Tuitbot's core follows a three-layer architecture: Toolkit, Workflow, and Autopilot. Each layer has specific operational characteristics.

### Profile Selection Guide

Choose the right MCP profile based on your use case:

| Scenario | Recommended Profile | Rationale |
|----------|-------------------|-----------|
| AI agent that reads tweets and scores them | `api-readonly` | Full read access, zero mutation risk |
| Agent that needs config/health checks only | `readonly` | Minimal 14-tool surface |
| Growth co-pilot that drafts and queues content | `write` | All 104 tools including content gen and approval |
| Debugging raw X API responses | `admin` | Universal request tools for ad-hoc endpoints |
| New integration, testing phase | `api-readonly` | Start read-only, upgrade to `write` after validation |
| Production autonomous agent | `write` with `approval_mode = true` | Human review before posting |

**Progression path:** Start with `readonly` or `api-readonly` to verify connectivity and tool behavior. Graduate to `write` with `dry_run_mutations = true` to preview mutations without executing them. Enable live mutations only after reviewing dry-run output.

### Safe Mutation Checklist

Before enabling mutations for any consumer (MCP agent, automation, CLI):

1. **Enable approval mode** — all mutations route to the approval queue for human review:
   ```toml
   [general]
   approval_mode = true
   ```

2. **Start with dry-run** — mutations are logged but not executed:
   ```toml
   [mcp_policy]
   dry_run_mutations = true
   ```

3. **Set conservative rate limits** — default is 20 mutations/hour:
   ```toml
   [mcp_policy]
   max_mutations_per_hour = 10
   ```

4. **Require approval for critical tools** — route specific tools through the queue:
   ```toml
   [mcp_policy]
   require_approval_for = ["x_post_tweet", "x_reply_to_tweet", "x_post_thread"]
   ```

5. **Block tools you don't need** — remove tools from the available surface:
   ```toml
   [mcp_policy]
   blocked_tools = ["x_delete_tweet", "x_follow_user", "x_unfollow_user"]
   ```

6. **Review the approval queue** before approving:
   ```bash
   tuitbot approve --list
   ```

### Layer-Specific Operational Notes

**Toolkit layer** (`core::toolkit/`):
- Stateless — safe to call in any context without initialization overhead
- No rate limiting or policy enforcement at this layer
- Errors are typed (`ToolkitError`) and map directly to MCP error codes
- Use for testing, debugging, and admin operations

**Workflow layer** (`core::workflow/`):
- Requires DB connection and optionally LLM provider
- Policy enforcement and safety checks live here
- Composite operations (discover → draft → queue) are deterministic
- All workflow errors propagate through `WorkflowError`

**Autopilot layer** (`core::automation/`):
- Scheduled loops with jitter and circuit breaking
- Loops call workflow and toolkit — never X API directly
- Mode-aware: Autopilot mode runs all loops, Composer mode runs read-only + posting
- Graceful shutdown via `CancellationToken`

---

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

---

## MCP Profile Verification Runbook

Use this runbook to verify MCP profile integrity after deployments, profile changes, or CI failures.

### 1. Quick Profile Smoke Test

Confirm each profile exposes the expected number of tools:

```bash
# Write profile (expect 104)
cargo run -p tuitbot-cli -- mcp manifest --profile write --format json | jq '.tool_count'

# Admin profile (expect 108)
cargo run -p tuitbot-cli -- mcp manifest --profile admin --format json | jq '.tool_count'

# Read-only profile (expect 14)
cargo run -p tuitbot-cli -- mcp manifest --profile readonly --format json | jq '.tool_count'

# API read-only profile (expect 40)
cargo run -p tuitbot-cli -- mcp manifest --profile api-readonly --format json | jq '.tool_count'
```

### 2. Confirm Read-Only Guarantee

Verify that read-only profiles contain zero mutation tools:

```bash
# Both should output 0
cargo run -p tuitbot-cli -- mcp manifest --profile readonly --format json \
  | jq '[.tools[] | select(.mutation == true)] | length'

cargo run -p tuitbot-cli -- mcp manifest --profile api-readonly --format json \
  | jq '[.tools[] | select(.mutation == true)] | length'
```

### 3. Verify Manifest Sync

Ensure committed manifests match the current binary output:

```bash
bash scripts/check-mcp-manifests.sh
# Expected: exit 0, "All manifests in sync."
```

If drift is detected, regenerate: `bash scripts/generate-mcp-manifests.sh`

### 4. CI Gate Reference

| Gate | What it validates | Local run command |
|------|-------------------|-------------------|
| Boundary tests (32) | Profile isolation, mutation denylists, lane constraints | `cargo test -p tuitbot-mcp boundary` |
| Conformance tests | 27 kernel tools present with correct schemas | `cargo test -p tuitbot-mcp conformance_all_kernel_tools` |
| Golden snapshots | Response schema drift detection | `cargo test -p tuitbot-mcp golden_snapshot_matches` |
| Eval harness | 4 scenarios × 4 quality gates | `cargo test -p tuitbot-mcp eval_harness` |
| Manifest sync | Committed JSON matches binary output | `bash scripts/check-mcp-manifests.sh` |

### 5. Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Wrong tool count | Tool added/removed without profile update | Update `router.rs` profile registration, re-run boundary tests |
| Mutation tool in read-only | Tool incorrectly tagged or registered | Check `is_mutation()` in tool definition, verify profile exclusion |
| Manifest drift | Binary updated but manifests not regenerated | Run `bash scripts/generate-mcp-manifests.sh` and commit |
| Boundary test failure | Profile invariant violated | Read test assertion message — it names the exact tool/profile mismatch |
