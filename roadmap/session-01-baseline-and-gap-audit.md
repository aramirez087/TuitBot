# Session 01 Prompt - Baseline and Gap Audit

## Copy/paste prompt

You are the lead architect and principal Rust engineer for this repo (`/Users/aramirez/Code/ReplyGuy`).  
Mission: establish hard truth on current MCP capability, coupling, and missing X API surface before any major refactor.

Critical constraints:

- No backward compatibility requirements. Optimize for best final architecture.
- Do not produce strategy fluff. Produce auditable artifacts and concrete backlog.
- Use code as source of truth over docs/claims.

Work required:

1. Inventory all exposed MCP tools from `crates/tuitbot-mcp/src/server.rs`.
2. Map each tool to one of three product lanes:
- `api_client_lane` (general X client tools)
- `workflow_lane` (growth/approval/scoring/composite workflows)
- `platform_lane` (telemetry/config/health/policy)
3. Build an endpoint coverage matrix against X API v2 capability categories:
- auth/me
- tweet CRUD
- search
- timelines
- users lookup
- follows graph
- likes
- bookmarks
- media
- usage/rate visibility
- pagination support
4. Run a coupling audit and tag each tool:
- `db_coupled`
- `config_coupled`
- `workflow_coupled`
- `stateless_ready`
5. Produce a prioritized engineering backlog with severity, impact, and implementation order.
6. Fix any obvious code-doc mismatch that is trivial and safe in this session.

Required artifacts to create:

- `roadmap/artifacts/session-01-tool-inventory.md`
- `roadmap/artifacts/session-01-coverage-matrix.md`
- `roadmap/artifacts/session-01-coupling-audit.md`
- `roadmap/artifacts/session-01-priority-backlog.md`
- `roadmap/artifacts/session-01-handoff.md`

Definition of done:

- Coverage matrix clearly separates `implemented`, `partial`, `missing`.
- Coupling audit names exact files/modules causing coupling.
- Backlog has a strict top-to-bottom execution order for sessions 02-10.
- Handoff includes test/lint command results and any unresolved risks.

Validation commands:

- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

At the end, summarize only:

1. What is strong already.
2. What must be rebuilt.
3. What Session 02 will implement first.
