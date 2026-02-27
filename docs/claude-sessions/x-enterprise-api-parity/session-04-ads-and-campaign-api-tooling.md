# Session 04: Ads And Campaign API Tooling

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission: Add typed Ads/Campaign API tool coverage in admin profile with strict host, scope, and mutation controls.

Repository anchors:
- `docs/roadmap/x-enterprise-api-parity/session-03-handoff.md`
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/mod.rs`
- `crates/tuitbot-mcp/src/tools/workflow/x_actions/x_request/tests.rs`
- `crates/tuitbot-mcp/src/spec/endpoints.rs`
- `crates/tuitbot-mcp/src/spec/tests.rs`
- `crates/tuitbot-mcp/src/requests.rs`
- `crates/tuitbot-mcp/src/server/admin.rs`
- `crates/tuitbot-mcp/src/tools/manifest.rs`
- `crates/tuitbot-mcp/src/tools/boundary_tests.rs`
- `docs/configuration.md`

Tasks:
1. Extend endpoint support for Ads host usage (`ads-api.x.com`) while preserving current SSRF and header defenses.
2. Add a concrete Ads/Campaign endpoint family (accounts, campaigns, ad groups, analytics) with explicit read vs mutation classification.
3. Implement admin tool handlers and request schemas for the Ads/Campaign family using the hardened request pathway.
4. Mark Ads/Campaign mutations as elevated-access and ensure policy + mutation audit applies.
5. Add and update tests for allowlist behavior, manifest inclusion, and admin-profile isolation.
6. Regenerate committed manifest artifacts after changes.

Deliverables:
- `docs/roadmap/x-enterprise-api-parity/session-04-ads-endpoint-matrix.md`
- `docs/roadmap/x-enterprise-api-parity/session-04-handoff.md`
- `docs/generated/mcp-manifest-admin.json`

Quality gates:
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria:
- Ads/Campaign tools appear in admin manifest with correct safety metadata.
- Ads host restrictions are covered by positive and negative tests.
- Session-04 handoff defines the enterprise-admin/compliance endpoint set for Session 05.
```
