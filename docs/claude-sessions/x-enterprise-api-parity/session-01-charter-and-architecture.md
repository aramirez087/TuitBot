# Session 01: Charter And Architecture

Paste this into a new Claude Code session:

```md
Mission: Produce a complete, decision-finalized implementation charter to close DM + Ads/Campaign/Admin API coverage gaps in the MCP manifest and runtime.

Repository anchors:
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-mcp/src/spec/endpoints.rs`
- `crates/tuitbot-mcp/src/server/write.rs`
- `crates/tuitbot-mcp/src/server/admin.rs`
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs`
- `docs/mcp-reference.md`
- `docs/configuration.md`
- `README.md`

Tasks:
1. Audit current state: compare manifest-declared tools vs actually registered server tools and document the DM/Ads/Admin enterprise gaps with file-path evidence.
2. Lock architecture decisions (no open alternatives):
   - Keep existing profile model; expand enterprise coverage through admin profile tooling.
   - Add typed DM tools, typed Ads/Campaign tools, and typed Enterprise Admin/Compliance tools.
   - Extend universal request safety to support `ads-api.x.com` under strict allowlists.
   - Route universal mutation-capable requests through policy + mutation audit.
3. Define exact endpoint families to implement in Sessions 02-06, including scopes, mutation/read classification, and profile membership.
4. Define acceptance metrics: manifest counts by profile, coverage report deltas, and minimum new test coverage for added tool families.
5. Write a session-by-session execution map with exact file targets and verification commands for each next session.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/charter.md`
- `docs/roadmap/x-enterprise-api-parity/coverage-gap-audit.md`
- `docs/roadmap/x-enterprise-api-parity/session-01-handoff.md`

Quality gates:
- If any Rust code changed, run:
  cargo fmt --all && cargo fmt --all --check
  RUSTFLAGS="-D warnings" cargo test --workspace
  cargo clippy --workspace -- -D warnings

Exit criteria:
- Charter contains final decisions with no unresolved choices.
- Coverage gaps are explicitly mapped to concrete implementation sessions.
- Session-01 handoff includes exact entry points and commands for Session 02.
```
