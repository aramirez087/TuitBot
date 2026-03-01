# Session 03: Reset Command and Live Reload

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Convert passphrase reset into a low-noise maintenance path and make running services honor out-of-band resets immediately.

Repository anchors
- docs/roadmap/passphrase-lifecycle-ux/charter.md
- crates/tuitbot-server/src/main.rs
- crates/tuitbot-server/src/auth/routes.rs
- crates/tuitbot-server/src/routes/lan.rs
- crates/tuitbot-core/src/auth/passphrase.rs
- crates/tuitbot-server/tests/fresh_install_auth.rs
- docs/lan-mode.md

Tasks
1. Move `--reset-passphrase` into an early fast path that skips DB initialization, API token creation, LLM setup, watcher startup, and socket binding.
2. Ensure the CLI path prints the new passphrase and exits without the misleading startup logs or `Address already in use` failure shown in the bug report.
3. Update authentication so the running server refreshes `passphrase_hash` from disk when it changes, making a CLI reset effective on the next login without restarting the service, while keeping the authenticated LAN reset endpoint correct.
4. Add focused regression coverage for the reload behavior and update operator docs to match the new reset semantics.

Deliverables
- crates/tuitbot-server/src/main.rs
- crates/tuitbot-server/src/auth/routes.rs
- crates/tuitbot-server/src/routes/lan.rs
- crates/tuitbot-server/tests/fresh_install_auth.rs
- docs/lan-mode.md
- docs/roadmap/passphrase-lifecycle-ux/session-03-handoff.md

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- `tuitbot-server --reset-passphrase` succeeds even while the long-running service already owns the configured port.
- A CLI reset is honored by the next browser login without restarting the server.
- The docs describe the reset command without stale or misleading behavior.
```
