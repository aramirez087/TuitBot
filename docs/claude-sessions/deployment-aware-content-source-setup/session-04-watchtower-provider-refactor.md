# Session 04: Watchtower Provider Refactor

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.
Continuity
- Read `docs/roadmap/deployment-aware-content-source-setup/drive-connection-flow.md` and `docs/roadmap/deployment-aware-content-source-setup/session-03-handoff.md` first.

Mission
Refactor the Watchtower Google Drive provider to use linked-account credentials and keep remote sync reliable under refresh, revocation, and restart scenarios.

Repository anchors
- `crates/tuitbot-core/src/source/mod.rs`
- `crates/tuitbot-core/src/source/google_drive.rs`
- `crates/tuitbot-core/src/source/tests/unit.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/automation/watchtower/tests.rs`
- `crates/tuitbot-server/src/main.rs`

Tasks
1. Replace the provider's service-account-file dependency with a credential loader that resolves linked-account tokens from the new storage contract.
2. Implement token refresh, in-memory caching, and failure handling for revoked or expired Drive credentials.
3. Update Watchtower source registration and polling so remote sources are built from connector references and existing sync cursors remain stable.
4. Expand provider and Watchtower tests to cover token refresh, restart recovery, and graceful degradation when a connection is broken.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/watchtower-sync-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-04-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Remote Drive polling runs from linked-account credentials, failure modes are explicit, and Session 05 can switch the UI without backend gaps.
```
