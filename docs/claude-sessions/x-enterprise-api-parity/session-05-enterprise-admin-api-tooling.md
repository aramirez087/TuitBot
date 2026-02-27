# Session 05: Enterprise Admin API Tooling

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission: Add enterprise admin/compliance API coverage so manifest claims include operational admin surfaces beyond public timeline tooling.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-04-handoff.md`
- `crates/tuitbot-mcp/src/spec/endpoints.rs`
- `crates/tuitbot-mcp/src/spec/tests.rs`
- `crates/tuitbot-mcp/src/requests.rs`
- `crates/tuitbot-mcp/src/server/admin.rs`
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-core/src/mcp_policy/types.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs`
- `docs/mcp-reference.md`

Tasks:
1. Add an enterprise-admin endpoint family (compliance jobs/rules and related admin operations supported by authenticated X APIs).
2. Implement admin tool handlers and request schemas for this family, using elevated-access enforcement and hardened request validation.
3. Extend policy categorization so enterprise-admin actions are visible in policy decisions and telemetry.
4. Add mutation audit coverage and rollback guidance for new enterprise-admin mutation tools.
5. Update boundary/manifest tests to enforce admin-only availability for enterprise-admin tools.
6. Regenerate committed manifest artifacts.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/session-05-enterprise-admin-matrix.md`
- `docs/roadmap/x-enterprise-api-parity/session-05-handoff.md`
- `docs/generated/mcp-manifest-admin.json`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- Enterprise-admin tools are present in admin manifest and absent from non-admin profiles.
- Policy and audit systems classify and record enterprise-admin mutations correctly.
- Session-05 handoff includes exact runtime/manifest parity gaps for Session 06.
```
