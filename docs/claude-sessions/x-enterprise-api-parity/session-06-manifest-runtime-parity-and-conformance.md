# Session 06: Manifest Runtime Parity And Conformance

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission: Ensure every manifest-declared enterprise tool is actually callable and covered by deterministic conformance checks.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-05-handoff.md`
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs`
- `crates/tuitbot-mcp/src/tools/conformance_tests/coverage.rs`
- `crates/tuitbot-mcp/src/tools/conformance_tests/read.rs`
- `crates/tuitbot-mcp/src/tools/conformance_tests/write.rs`
- `crates/tuitbot-mcp/src/tools/conformance_tests/engage.rs`
- `scripts/run-conformance.sh`
- `scripts/check-mcp-manifests.sh`

Tasks:
1. Add a structural parity test that fails if any manifest tool lacks a registered MCP handler in the profiles that declare it.
2. Add deterministic conformance tests for DM, Ads/Campaign, and enterprise-admin tool families (mock-based).
3. Update coverage report generation to include new families and profile deltas clearly.
4. Regenerate `docs/generated/coverage-report.json` and `docs/generated/coverage-report.md`.
5. Regenerate profile manifests and verify drift checks pass.

Deliverables:
- `docs/generated/coverage-report.json`
- `docs/generated/coverage-report.md`
- `docs/roadmap/x-enterprise-api-parity/session-06-parity-report.md`
- `docs/roadmap/x-enterprise-api-parity/session-06-handoff.md`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- Manifest/runtime parity check passes for all profiles.
- Coverage report includes DM + Ads/Campaign + enterprise-admin tools.
- Session-06 handoff lists only documentation and release-alignment tasks for Session 07.
```
