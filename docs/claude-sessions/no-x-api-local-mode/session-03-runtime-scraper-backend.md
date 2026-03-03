# Session 03: Runtime Scraper Backend

Paste this into a new Claude Code session:

```md
Continuity
Continue from Session 02 artifacts.
Read `docs/roadmap/no-x-api-local-mode/session-02-handoff.md` and `docs/roadmap/no-x-api-local-mode/settings-flow.md` before changing code.

Mission
Make the product runtime honor `provider_backend = "scraper"` so local users can run core discovery and publishing workflows without an X API key while writes still fail closed when the transport is unsafe.

Repository anchors
- `crates/tuitbot-core/src/x_api/mod.rs`
- `crates/tuitbot-core/src/automation/mod.rs`
- `crates/tuitbot-core/src/automation/adapters/x_api.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-core/src/workflow/discover.rs`
- `crates/tuitbot-core/src/workflow/publish.rs`
- `crates/tuitbot-server/src/main.rs`
- `crates/tuitbot-mcp/src/provider/scraper.rs`

Tasks
1. Introduce `crates/tuitbot-core/src/x_api/local_mode.rs` and implement the subset of `XApiClient` the product runtime uses for discovery, mentions, profile lookup, tweet posting, and reply posting.
2. Use `x_api.provider_backend` to select between the current official client and `LocalModeXClient` in the runtime code paths that currently assume paid API access.
3. Mirror the MCP scraper backend's error semantics and data-shape discipline where useful, but keep the product implementation inside `tuitbot-core` so the app does not depend on MCP crates.
4. Enforce `scraper_allow_mutations`: when false, keep write flows queue-only or return actionable blocked errors; when true, require an explicit local transport health check before any live mutation attempt.
5. Make automation loops and server startup degrade gracefully in scraper mode so unsupported operations surface clear status instead of crashing the process.
6. Add tests for backend selection, read-path behavior, mutation gating, and backward compatibility of the paid `x_api` path.
7. Write `docs/roadmap/no-x-api-local-mode/runtime-backend-plan.md` with the shipped runtime behavior and end with a handoff.

Deliverables
- `docs/roadmap/no-x-api-local-mode/runtime-backend-plan.md`
- `crates/tuitbot-core/src/x_api/local_mode.rs`
- `crates/tuitbot-core/src/x_api/mod.rs`
- `crates/tuitbot-core/src/automation/mod.rs`
- `crates/tuitbot-core/src/automation/adapters/x_api.rs`
- `crates/tuitbot-core/src/workflow/discover.rs`
- `crates/tuitbot-core/src/workflow/publish.rs`
- `crates/tuitbot-server/src/main.rs`
- `docs/roadmap/no-x-api-local-mode/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The runtime follows the selected backend without requiring X API credentials in scraper mode.
- Writes fail closed or queue when scraper mutations are disabled or unhealthy.
- Existing paid X API behavior remains backward compatible.
```
