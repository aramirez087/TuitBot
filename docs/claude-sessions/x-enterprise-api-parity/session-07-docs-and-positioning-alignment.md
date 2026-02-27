# Session 07: Docs And Positioning Alignment

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission: Align product claims, MCP documentation, and configuration guidance with the delivered DM + Ads/Campaign + enterprise-admin coverage.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-06-handoff.md`
- `README.md`
- `docs/mcp-reference.md`
- `docs/configuration.md`
- `docs/cli-reference.md`
- `docs/operations.md`
- `CHANGELOG.md`
- `docs/generated/mcp-manifest-admin.json`
- `docs/generated/coverage-report.md`

Tasks:
1. Replace outdated boundary statements that claim DM/Ads/Admin APIs are unsupported.
2. Document new tool families, profile availability, scope requirements, and safety controls.
3. Update CLI/profile usage examples and operational runbook guidance for enterprise scenarios.
4. Add a concise migration note explaining what changed, how to verify manifests, and how to roll back safely.
5. Cross-check all published tool counts and examples against generated artifacts.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/session-07-docs-diff-summary.md`
- `docs/roadmap/x-enterprise-api-parity/session-07-handoff.md`

Quality gates:
- If any Rust code changed, run:
  cargo fmt --all && cargo fmt --all --check
  RUSTFLAGS="-D warnings" cargo test --workspace
  cargo clippy --workspace -- -D warnings

Exit criteria:
- No documentation contradicts runtime behavior or generated manifests.
- Enterprise API coverage is clearly described with concrete limits and safety rules.
- Session-07 handoff includes final validation checklist for Session 08.
```
