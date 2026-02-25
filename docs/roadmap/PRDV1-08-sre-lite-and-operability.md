# PRDV1-08: SRE-lite and Operability

## Goal

Make self-host operation reliable and predictable: health checks, safety
autopause, backup/recovery, and upgrade preflight with rollback support.

## Already built (do not redo)

- Basic `/api/health` endpoint exists.
- Runtime start/stop/status endpoints exist.
- Operations doc has high-level guidance.

## Gaps to implement

1. Deeper health/status surface.
   - DB health
   - runtime loop health
   - auth health
   - X API readiness
2. Circuit breaker behavior.
   - if 429/403 spikes, autopause mutating loops
   - emit alert event and action log record
3. Backup and recovery commands.
   - scheduled backup command/script
   - restore command with validation
4. Migration preflight.
   - automatic backup before schema migration
   - fail-safe rollback instructions
5. Runbooks.
   - incident response, restart, auth expiry, rate-limit storms

## Primary code touchpoints

- `crates/tuitbot-server/src/routes/health.rs`
- `crates/tuitbot-server/src/routes/runtime.rs`
- `crates/tuitbot-core/src/automation/*`
- `crates/tuitbot-core/src/storage/*`
- `crates/tuitbot-cli/src/commands/*` (new backup/status commands)
- `docs/operations.md`

## Implementation tasks

1. Expand health endpoint(s).
   - include component statuses and degradation reasons.
2. Add circuit-breaker module.
   - monitor rolling error windows for mutating endpoints.
   - trigger autopause and cooldown timer.
3. Add runtime alert signaling.
   - websocket + action log + API status field.
4. Implement backup/restore tooling.
   - `tuitbot backup create`
   - `tuitbot backup restore --file ...`
   - optional retention policy for backup folder.
5. Add upgrade preflight hook.
   - backup DB before migrations.
   - abort on failed backup.
6. Write runbooks with exact commands.

## Acceptance criteria

- Operator can identify unhealthy subsystem from health payload.
- Repeated 429/403 conditions trigger automatic mutation pause.
- Backup and restore are documented and tested.
- Migrations cannot run without successful preflight backup.

## Verification commands

```bash
cargo test -p tuitbot-core automation
cargo test -p tuitbot-server health
cargo test -p tuitbot-cli
```

## Out of scope

- Cloud multi-tenant infra rollout (covered in cloud backlog docs).
- Agency roles and quotas (PRDV1-09).
