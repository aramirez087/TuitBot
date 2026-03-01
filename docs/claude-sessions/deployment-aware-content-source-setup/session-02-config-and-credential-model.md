# Session 02: Config And Credential Model

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.
Continuity
- Read `docs/roadmap/deployment-aware-content-source-setup/charter.md` and `docs/roadmap/deployment-aware-content-source-setup/session-01-handoff.md` first.

Mission
Implement the shared configuration, capability, and persistence contract that makes remote account-linked sync the default non-desktop path.

Repository anchors
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/config/tests.rs`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/tests/api_tests.rs`
- `dashboard/src/lib/api.ts`

Tasks
1. Replace the service-account-path-centric Google Drive config shape with a connector-friendly credential reference model while preserving legacy decode support where possible.
2. Add the persistence layer for remote sync connections and their non-secret metadata, including any required SQL migration files.
3. Update deployment capability and settings contracts so desktop, self-host, and cloud can advertise different preferred source defaults without hard-coding UI guesses.
4. Extend validation and serialization tests to cover the new config shape, legacy config compatibility, and capability reporting.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/source-connection-contract.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The codebase has a stable connector contract for later sessions, legacy config behavior is explicit, and Session 03 can build on it without redefining storage.
```
