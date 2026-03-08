# Session 09: Activity History And Restore

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Continuity
- Make restore safer than any existing undo path by preserving the current state before any destructive revert.

Mission
- Add revision history and activity visibility so AI edits and structural changes are reversible, auditable, and safer than the current baseline.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/data-model.md`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `docs/roadmap/draft-studio-beyond-typefully/autosave-and-sync.md`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`

Tasks
1. Persist meaningful revisions and activity events for autosave checkpoints, AI transforms, schedule changes, publish transitions, and restore actions.
2. Add read endpoints and UI for a history panel with timestamps, source labels, and one-click restore.
3. Make restore safe: confirm before destructive revert, create a new revision on restore, and never lose the current state without a breadcrumb.
4. Surface AI-generated changes clearly enough that users can trust undo and restore.
5. Document the revision model, retention behavior, and restore safeguards.

Deliverables
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/tests/draft_history_api_tests.rs`
- `dashboard/src/lib/components/drafts/DraftHistoryPanel.svelte`
- `dashboard/src/lib/stores/draftStudio.ts`
- `docs/roadmap/draft-studio-beyond-typefully/history-and-restore.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-09-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Users can inspect and restore prior states, restore is non-lossy, and history covers both manual and AI-driven changes.
```
