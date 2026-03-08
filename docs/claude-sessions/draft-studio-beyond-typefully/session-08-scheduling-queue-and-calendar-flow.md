# Session 08: Scheduling Queue And Calendar Flow

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Continuity
- Treat draft, scheduled, and posted as workflow states on one document, not as separate tools with duplicated compose state.

Mission
- Unify draft, scheduled, and posted workflows so movement between them feels like changing state on one document.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/ux-blueprint.md`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/lib/stores/calendar.ts`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`

Tasks
1. Wire schedule, unschedule, publish-now, and edit-scheduled flows through Draft Studio using the same draft identity.
2. Make Scheduled and Posted tabs first-class views with actionable metadata, not read-only leftovers.
3. Connect the calendar so opening a scheduled item lands in the same draft workspace and preserves selection context.
4. Keep quick compose from the calendar available, but route it through canonical draft creation instead of a separate unsaved composer state.
5. Document lifecycle transitions, race conditions, and edge cases.

Deliverables
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/lib/stores/calendar.ts`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `docs/roadmap/draft-studio-beyond-typefully/scheduling-flow.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-08-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Draft to scheduled to posted is observable in one system, calendar edits open the same record, and no route creates hidden orphan drafts.
```
