# Session 02: Universal Request Foundation

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission: Implement the enterprise safety foundation for universal requests so DM/Ads/Admin coverage can be added without weakening security or auditability.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/charter.md`
- `docs/roadmap/x-enterprise-api-parity/session-01-handoff.md`
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs`
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/tests.rs`
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-mcp/src/requests.rs`
- `crates/tuitbot-core/src/mcp_policy/types.rs`
- `crates/tuitbot-core/src/storage/mutation_audit.rs`
- `crates/tuitbot-mcp/src/server/admin.rs`

Tasks:
1. Extend host/path safety rules to support enterprise domains and families, including `ads-api.x.com`, while preserving SSRF and header protections.
2. Integrate universal mutation-capable request tools with MCP policy evaluation and mutation audit recording.
3. Add explicit request-family metadata (public API vs DM vs Ads vs enterprise-admin) to improve policy categorization and reporting.
4. Update manifest metadata so universal tools expose elevated-access and policy/audit behavior accurately.
5. Add/expand tests for host allowlist, path restrictions, policy enforcement, and audit insertion for universal request flows.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/session-02-security-foundation.md`
- `docs/roadmap/x-enterprise-api-parity/session-02-handoff.md`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- Universal request mutations are policy-gated and audit-recorded.
- Safety tests cover allowed and blocked hosts/paths for enterprise families.
- Session-02 handoff lists exact extension points for DM typed tools.
```
