# Session 08: Local-First Privacy Mode

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/privacy-and-deployment-matrix.md`, `docs/roadmap/obsidian-ghostwriter-edge/native-workflow-polish.md`, and `docs/roadmap/obsidian-ghostwriter-edge/session-07-handoff.md`.

Mission
Implement the deployment-aware privacy behavior so Desktop plus local vault can honestly claim local-first Ghostwriter handling while other modes are explicitly constrained.

Repository anchors
- `dashboard/src-tauri/src/lib.rs`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/api/client.ts`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/sources.rs`
- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/storage/watchtower/sources.rs`
- `plugins/obsidian-tuitbot/src/main.ts`

Tasks
1. Implement the local-first path defined in Session 01 for Desktop plus `local_fs`, keeping the exposed surface area minimal.
2. Add deployment gating and copy so Self-host and Cloud modes do not over-claim local handling they cannot guarantee.
3. Tighten the settings and runtime behavior around vault-source eligibility, local transport availability, and failure modes.
4. Add tests for deployment gating, allowed local entry points, and privacy-safe server behavior.

Deliverables
- `dashboard/src-tauri/src/lib.rs`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/api/client.ts`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/sources.rs`
- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/config/types.rs`
- `plugins/obsidian-tuitbot/src/main.ts`
- `docs/roadmap/obsidian-ghostwriter-edge/local-first-implementation.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-08-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run test:unit:run`
- `npm --prefix plugins/obsidian-tuitbot run build`

Exit criteria
- Desktop plus local vault has a defensible local-first Ghostwriter path.
- Other deployment modes are explicit and honest about their privacy envelope.
- Session 09 can focus on end-to-end validation and release judgment.
```
