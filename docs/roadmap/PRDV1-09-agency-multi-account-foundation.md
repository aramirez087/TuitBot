# PRDV1-09: Agency / Multi-Account Foundation

## Goal

Lay the minimal foundation for “agency mode” and future multi-tenant
productization without forcing full cloud tenancy in this phase.

## Already built (do not redo)

- Single-account workflow is stable and feature-rich.
- MCP policy and dashboard governance foundation exists.
- Separate cloud-tier backlog already exists in `BACKLOG-cloud-hosted-tier.md`.

## Scope for this phase

1. Multi-account support in self-host context.
2. Per-account policies and quotas.
3. Basic role model for dashboard/API:
   - `admin`
   - `approver`
   - `viewer`

## Non-goal for this phase

- Full SaaS tenant architecture, billing, Stripe, hosted auth stack.

## Primary code touchpoints

- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/storage/*` (account scoping/migrations)
- `crates/tuitbot-server/src/auth/*`
- `crates/tuitbot-server/src/routes/*`
- `dashboard/src/lib/api.ts`
- `dashboard/src/routes/(app)/*`
- `migrations/*`

## Implementation tasks

1. Data model foundation.
   - add `account_id` scoping to relevant tables (approval, activity, telemetry, content).
   - create migration strategy for existing single-account rows.
2. Config structure upgrade.
   - account list + active account context.
   - per-account limits and brand/language policy binding.
3. API account context.
   - account selection on API calls (header/query/session-scoped).
   - enforce account isolation in storage queries.
4. Role-based permissions.
   - approval actions restricted to `admin`/`approver`.
   - viewer is read-only.
5. Dashboard account switcher and role-aware UI.
   - account picker
   - disable protected actions for viewers.
6. Agency policy template.
   - default quotas per account/client.

## Acceptance criteria

- User can manage multiple X accounts from one deployment without cross-account leakage.
- Approval and mutation operations are role-gated.
- Per-account quotas and policies are independently configurable.
- Existing single-account install migrates cleanly to default account.

## Verification commands

```bash
cargo test -p tuitbot-core
cargo test -p tuitbot-server
cd dashboard && npm run check
```

## Hand-off note

Once this is complete, revisit
`docs/roadmap/BACKLOG-cloud-hosted-tier.md` for full hosted multi-tenant buildout.
