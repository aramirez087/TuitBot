# Session 03: Drive Connection Backend

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.
Continuity
- Read `docs/roadmap/deployment-aware-content-source-setup/source-connection-contract.md` and `docs/roadmap/deployment-aware-content-source-setup/session-02-handoff.md` first.

Mission
Implement the authenticated backend flow for linking, inspecting, refreshing, and disconnecting a user-owned Google Drive connection without exposing secrets.

Repository anchors
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/tests/api_tests.rs`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-core/src/source/google_drive.rs`

Tasks
1. Add authenticated API endpoints for starting a Drive link flow, completing it, reading connection status, and disconnecting a linked account.
2. Persist refresh-token material and account identity safely, and return only redacted connection metadata to clients.
3. Wire the new routes into the server with the existing auth model and clear error handling for expired, revoked, or duplicate connections.
4. Add focused API tests for the happy path, reconnects, invalid callbacks, and disconnect behavior.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/drive-connection-flow.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The backend can manage a linked Drive account end to end, all exposed responses are secret-safe, and Session 04 can consume stored credentials.
```
