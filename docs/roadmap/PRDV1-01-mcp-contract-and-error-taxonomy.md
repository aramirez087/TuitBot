# PRDV1-01: MCP Contract Hardening (Strict Envelope + Typed Errors)

## Goal

Make MCP output deterministic and product-grade: every tool returns the v1
envelope and all failures use typed, machine-actionable errors.

## Why now

Policy, dashboard observability, and AI integrators all depend on a stable MCP
 contract. Current repo still mixes migrated and legacy response shapes.

## Already built (do not redo)

- `ToolResponse` exists in `crates/tuitbot-mcp/src/tools/response.rs`.
- Some tools already migrated (`x_actions`, `health`, `capabilities`, parts of analytics/discovery/approval).
- Telemetry and eval harness already exist.

## Gaps to implement

1. Enforce envelope on **all** MCP tools.
2. Extend error payload schema to include:
   - `rate_limit_reset` (optional unix seconds/ISO string)
   - `policy_decision` (optional, for policy-denied/routed calls)
3. Remove legacy response branches from docs and tool outputs.
4. Make unknown/untyped errors explicit and measurable.

## Primary code touchpoints

- `crates/tuitbot-mcp/src/tools/response.rs`
- `crates/tuitbot-mcp/src/tools/*.rs` (especially `actions.rs`, `replies.rs`, `rate_limits.rs`, `targets.rs`, `config.rs`, `scoring.rs`, `content.rs`, `approval.rs`, `analytics.rs`, `discovery.rs`)
- `crates/tuitbot-mcp/src/server.rs`
- `crates/tuitbot-mcp/src/tools/policy_gate.rs`
- `crates/tuitbot-mcp/src/tools/x_actions.rs`
- `docs/mcp-reference.md`
- `docs/operations.md`

## Implementation tasks

1. Upgrade envelope schema.
   - Add optional fields to `ToolError`.
   - Add helper constructors for common classes: db/policy/x/network/validation.
2. Migrate every tool function to `ToolResponse` output.
   - No direct `serde_json::to_string_pretty(...)` outputs.
   - No raw `"Error: ..."` strings.
3. Standardize mode/approval metadata.
   - Attach `ToolMeta::with_mode(...)` everywhere where mode is known.
4. Normalize policy-denied and rate-limited outputs.
   - Include `policy_decision`.
   - Include `rate_limit_reset` when available.
5. Update docs.
   - Remove “migrated vs non-migrated” wording.
   - Publish strict schema and detection guidance.
6. Add regression tests.
   - Per-tool happy path and failure shape.
   - Contract tests to assert top-level keys and error typing.

## Acceptance criteria

- 100% MCP tools return `{success,data,error,meta}`.
- No MCP tool returns plain strings or legacy JSON payloads.
- Error payload includes `retryable` plus new optional fields where applicable.
- `docs/mcp-reference.md` documents only the strict envelope behavior.

## Verification commands

```bash
cargo test -p tuitbot-mcp
cargo test -p tuitbot-mcp eval_harness -- --nocapture
```

## Out of scope

- Adding brand/QA logic (covered in PRDV1-06).
- Adding new X API capabilities (covered in PRDV1-04).
