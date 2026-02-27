# Session 03: DM API Tooling

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission: Add first-class DM API tool coverage (read + send) to the MCP system with strict scope, policy, and manifest metadata.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-02-handoff.md`
- `crates/tuitbot-mcp/src/spec/endpoints.rs`
- `crates/tuitbot-mcp/src/spec/params.rs`
- `crates/tuitbot-mcp/src/spec/generator.rs`
- `crates/tuitbot-mcp/src/spec/tests.rs`
- `crates/tuitbot-mcp/src/requests.rs`
- `crates/tuitbot-mcp/src/server/admin.rs`
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs`
- `crates/tuitbot-core/src/x_api/scopes.rs`

Tasks:
1. Add a DM endpoint set to the spec pack covering conversation discovery, message/event retrieval, and message send mutation.
2. Implement request models and admin-profile tool handlers for the DM endpoint set, reusing the hardened universal request foundation.
3. Assign correct categories, scopes, profile availability, mutation flags, and error codes in manifest metadata.
4. Update scope diagnostics so missing DM scopes produce explicit degraded-feature messages.
5. Add/adjust tests for spec validation, boundary isolation, and DM tool manifest inclusion.
6. Regenerate committed manifest artifacts after changes.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/session-03-dm-endpoint-matrix.md`
- `docs/roadmap/x-enterprise-api-parity/session-03-handoff.md`
- `docs/generated/mcp-manifest-admin.json`
- `docs/generated/mcp-manifest-write.json`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- DM tools exist in manifest with accurate scope and mutation metadata.
- DM mutations use policy + audit pathways defined in Session 02.
- Session-03 handoff identifies exact files for Ads/Campaign expansion.
```
