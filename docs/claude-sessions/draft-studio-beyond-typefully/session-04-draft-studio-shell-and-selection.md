# Session 04: Draft Studio Shell And Selection

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Build the route shell against the new API contract instead of layering more logic into the old CRUD page.

Mission
- Replace the current drafts page with a full Draft Studio shell that makes one selected draft the center of the route.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/ux-blueprint.md`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/components/Sidebar.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/routes/(app)/+page.svelte`

Tasks
1. Replace the old `/drafts` CRUD view with a three-zone shell: left rail, center composer surface, right details area or drawer.
2. Create a `draftStudio` store that loads collection results, selected draft id, current tab, filters, and pending sync state from the new API.
3. Make selection reload-safe and shareable through URL params or other stable route state, not hidden component memory.
4. Preserve deliberate empty, loading, and error states; a blank draft should feel like a document, not an empty CRUD card.
5. Document shell behavior, route state, and remaining polish risks.

Deliverables
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/Sidebar.svelte`
- `docs/roadmap/draft-studio-beyond-typefully/workspace-shell.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-04-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- `/drafts` no longer renders the old card list, refresh preserves selection, and the new shell makes the draft/composer relationship immediately understandable.
```
