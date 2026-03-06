# Session 04: Runtime And Watchtower Isolation

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Carry forward the credential-isolation contract and do not reintroduce default-account-only service initialization.

Mission
Make runtime, assist, and watchtower services consume account-specific effective config and isolated credentials instead of default-account singletons.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md`
- `crates/tuitbot-server/src/main.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/routes/runtime.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/source/google_drive/mod.rs`
- `docs/architecture.md`

Tasks
1. Remove default-account-only initialization of content generators and other lazy services in `main.rs`; build and cache them per account through `AppState`.
2. Make runtime start, stop, and status plus any lazy assist or discovery generators load the selected account's effective config and credentials.
3. Ensure watchtower or content-source execution uses the selected account's config and credentials without cross-account leakage while preserving backward compatibility.
4. Add regression tests for two accounts with different overrides, content sources, and credentials showing isolated runtime state.
5. Write `docs/roadmap/dashboard-multi-account/runtime-isolation-plan.md` and `docs/roadmap/dashboard-multi-account/session-04-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/runtime-isolation-plan.md`
- `docs/roadmap/dashboard-multi-account/session-04-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Runtime, assist, and source-processing services are keyed by the active account rather than assumed-default globals.
- A second account can run with different config and credentials without mutating the default account's runtime behavior.
- Tests and docs cover the new service-lifecycle contract.
```
