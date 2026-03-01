# Session 02: Backend Factory Reset

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.
Continuity
- Read `docs/roadmap/factory-reset-danger-zone/charter.md` and `docs/roadmap/factory-reset-danger-zone/session-01-handoff.md` first.

Mission
Implement the authenticated live factory-reset backend that returns the instance to an unconfigured state without requiring a server restart.

Repository anchors
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-core/src/storage/mod.rs`
- `crates/tuitbot-server/tests/api_tests.rs`
- `crates/tuitbot-server/tests/fresh_install_auth.rs`

Tasks
1. Add the chartered protected reset route under `/api/settings` and keep it behind existing auth and CSRF rules.
2. Move the destructive data-clearing logic into `tuitbot-core` by adding a dedicated storage helper so the route stays thin.
3. Cancel live runtimes and the watchtower token before cleanup, clear content generators and passphrase state in memory, then remove Tuitbot-owned file artifacts and wipe table contents while preserving schema and `api_token`.
4. Return a success payload that always clears the `tuitbot_session` cookie for web callers and leaves bearer callers usable.
5. Add integration coverage for success, bad confirmation, repeat reset idempotency, cleared config status, and cookie clearing.

Deliverables
- `crates/tuitbot-core/src/storage/reset.rs`
- `crates/tuitbot-core/src/storage/mod.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/tests/factory_reset.rs`
- `docs/roadmap/factory-reset-danger-zone/reset-contract.md`
- `docs/roadmap/factory-reset-danger-zone/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The route works end-to-end in tests, the server remains usable after reset, and Session 03 has exact API behavior to build against.
```
