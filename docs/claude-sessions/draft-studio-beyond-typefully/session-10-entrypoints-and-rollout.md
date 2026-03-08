# Session 10: Entrypoints And Rollout

Paste this into a new Claude Code session:

```md
Continue from Session 09 artifacts.

Continuity
- Remove the remaining split-brain entry points so writing always flows through the Draft Studio model.

Mission
- Clean up legacy entry points, navigation, docs, and rollout details so the shipped product describes one coherent writing system.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/charter.md`
- `docs/roadmap/draft-studio-beyond-typefully/scheduling-flow.md`
- `docs/roadmap/draft-studio-beyond-typefully/history-and-restore.md`
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/Sidebar.svelte`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `docs/composer-mode.md`

Tasks
1. Eliminate stale copy and legacy affordances that imply a separate draft editor or unsaved home-composer workflow.
2. Decide and implement the home-surface behavior using one canonical draft model only: resume last draft, open new draft, or route into Draft Studio.
3. Update navigation labels, empty states, and docs so product language matches the new workflow.
4. Add lightweight telemetry or logging where useful to observe save failures, restore actions, and draft-state transitions.
5. Document rollout notes, compatibility assumptions, and any deferred cleanup that is intentionally out of scope.

Deliverables
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/Sidebar.svelte`
- `docs/composer-mode.md`
- `docs/roadmap/draft-studio-beyond-typefully/entrypoints-and-rollout.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-10-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- There is no user-visible duplicate draft flow, docs match behavior, and every major entry point lands in the canonical draft experience.
```
