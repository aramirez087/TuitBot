# PRDV1-02: Policy Engine v2 (Governance Core)

## Goal

Make TuitBot the safest X MCP: richer policy logic, template profiles, and
deterministic routing (allow/deny/require-approval/dry-run) by context.

## Already built (do not redo)

- v1 evaluator exists: `McpPolicyEvaluator`.
- Current config has `blocked_tools`, `require_approval_for`, `dry_run_mutations`, hourly cap.
- MCP policy endpoints exist (`/api/mcp/policy` GET/PATCH).

## Gaps to implement

1. Policy conditions by:
   - tool
   - category (read/write/engage/media/thread/delete)
   - mode (`autopilot`/`composer`)
   - language
   - schedule window
   - account scope (single-account now, account-key-ready schema)
2. Policy actions:
   - `allow`
   - `deny`
   - `require_approval`
   - `dry_run_only`
3. Advanced limits:
   - per hour/day
   - per author
   - per keyword/topic
   - per engagement type (`like`, `retweet`, `reply`, `quote`, `delete`)
4. Product templates:
   - `safe_default` (recommended)
   - `growth_aggressive` (explicit opt-in)
   - `agency_mode` (quota-oriented defaults)

## Primary code touchpoints

- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/config/defaults.rs`
- `crates/tuitbot-core/src/mcp_policy/evaluator.rs`
- `crates/tuitbot-mcp/src/tools/policy_gate.rs`
- `crates/tuitbot-server/src/routes/mcp.rs`
- `dashboard/src/lib/stores/mcp.ts`
- `dashboard/src/routes/(app)/mcp/+page.svelte`
- migration files under `migrations/`

## Implementation tasks

1. Expand config model.
   - Introduce v2 policy structs (`rules`, `limits`, `template`, `version`).
   - Keep backward compatibility by mapping old fields into v2 defaults.
2. Rewrite evaluator order with explicit precedence.
   - Hard-deny rules first.
   - Safety quotas and per-dimension caps.
   - Composer override behavior.
   - Final action decision.
3. Add policy reason typing.
   - Return structured reason codes for dashboards and telemetry.
4. Add template apply endpoint.
   - `POST /api/mcp/policy/templates/{template_name}`.
   - Keep PATCH endpoint for manual edits.
5. Add dashboard controls.
   - Template switcher.
   - Rule editor for categories and caps.
   - Warnings for aggressive template.
6. Add tests.
   - Policy matrix tests (mode x tool x language x limit).
   - Serialization/backward-compat tests for config.

## Hard rules required by PRD

- `delete_tweet` always `require_approval`.
- Delete actions have cooldown and max-per-day cap.
- `retweet/like` allowed but constrained by keyword/topic and rate caps.

## Acceptance criteria

- Policy decisions are deterministic and explainable with typed reasons.
- Template apply + manual patch both work.
- Composer mode keeps “all mutations reviewed” unless explicitly relaxed.
- Telemetry captures decision + reason for every mutation.

## Verification commands

```bash
cargo test -p tuitbot-core mcp_policy
cargo test -p tuitbot-server mcp
cargo test -p tuitbot-mcp policy_gate
```

## Out of scope

- Reviewer identity and approval notes UI (PRDV1-03).
- Bilingual QA flags (PRDV1-06).
