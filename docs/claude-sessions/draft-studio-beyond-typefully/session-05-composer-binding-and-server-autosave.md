# Session 05: Composer Binding And Server Autosave

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Continuity
- Treat the selected server draft record as the canonical document and use local autosave only as a crash-safety layer.

Mission
- Refactor the composer so a selected server draft record, not local component-only state, is the source of truth.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md`
- `docs/roadmap/draft-studio-beyond-typefully/workspace-shell.md`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/utils/composerAutosave.ts`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/ComposeModal.svelte`

Tasks
1. Make `ComposeWorkspace` able to hydrate from and emit changes to the selected draft record, including thread blocks, media, schedule, and metadata it needs.
2. Add debounced server autosave with explicit sync badge or status and preserve local autosave only as crash recovery fallback.
3. Keep undo and recovery safe across draft switches, reloads, and failed saves.
4. Prevent cross-draft leakage: switching drafts must fully replace editor state, media preview state, timers, and recovery banners.
5. Document save semantics, failure handling, and the server-vs-local precedence rules.

Deliverables
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/utils/composerAutosave.ts`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/drafts/DraftSyncBadge.svelte`
- `docs/roadmap/draft-studio-beyond-typefully/autosave-and-sync.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-05-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Editing a selected draft saves back to the server, switching drafts is reliable, save failures are visible and non-destructive, and local recovery never overwrites a newer server draft silently.
```
