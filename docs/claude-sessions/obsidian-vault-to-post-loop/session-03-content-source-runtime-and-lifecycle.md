# Session 03: Content Source Runtime And Lifecycle

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Treat source semantics as product behavior, not just internal config details.

Mission
- Make content-source behavior truthful, live-reloadable, and consistent across local folders, Google Drive, and manual ingest.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/charter.md`
- `docs/roadmap/obsidian-vault-to-post-loop/implementation-plan.md`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-server/src/main.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/routes/settings.rs`

Tasks
1. Define and implement explicit source lifecycle semantics for enabled state, initial sync, change detection mode, poll interval, and runtime status.
2. Remove the misleading contract where local sources need `watch=true` to ingest at all and remote sources ignore the same toggle.
3. Apply content-source config changes without requiring a full server restart by reloading or restarting Watchtower safely from settings flows.
4. Add or refine source-status and reindex APIs needed by later dashboard work.
5. Document the lifecycle contract, migration rules, and exact local-vs-remote semantics.

Deliverables
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-server/src/main.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/source-lifecycle.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Source toggles and runtime behavior now match the UI and docs, config updates apply to the running watcher, and status or reindex hooks exist for later surfaces.
```
