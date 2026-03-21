# Session 09: Validation And Release Readiness

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Continuity
- Load every roadmap artifact under `docs/roadmap/obsidian-ghostwriter-edge/`, especially `epic-charter.md`, `local-first-implementation.md`, `privacy-and-deployment-matrix.md`, and `session-08-handoff.md`.

Mission
Validate the full Ghostwriter edge implementation, close remaining consistency gaps, and produce a clear go or no-go release assessment.

Repository anchors
- `docs/roadmap/obsidian-ghostwriter-edge/`
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src-tauri/src/lib.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-core/src/content/generator/mod.rs`
- `plugins/obsidian-tuitbot/src/main.ts`

Tasks
1. Run the full validation sweep across Rust, dashboard, and Obsidian plugin surfaces and fix only real regressions or consistency gaps.
2. Verify the end-to-end journeys for block send, hook-first drafting, provenance continuity, and local-first privacy against the charter.
3. Produce the QA matrix and release-readiness report with explicit residual risks and rollback concerns.
4. Call out any non-blocking follow-up work separately instead of hiding it inside the go or no-go decision.

Deliverables
- `docs/roadmap/obsidian-ghostwriter-edge/qa-matrix.md`
- `docs/roadmap/obsidian-ghostwriter-edge/release-readiness.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-09-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run test:unit:run`
- `npm --prefix dashboard run build`
- `npm --prefix plugins/obsidian-tuitbot run build`

Exit criteria
- The feature set is validated against the charter with no unresolved P0 or P1 gaps.
- The roadmap folder contains a clear go or no-go recommendation and residual risk summary.
- A maintainer can make a release decision without any prior-session memory.
```
