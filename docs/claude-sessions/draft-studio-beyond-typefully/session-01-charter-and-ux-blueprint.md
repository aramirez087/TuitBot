# Session 01: Charter And UX Blueprint

Paste this into a new Claude Code session:

```md
Continuity
- Start from current repository state only.
- Read the listed anchors and existing composer roadmaps before deciding anything.

Mission
- Define the north-star UX and technical plan for a unified Draft Studio that makes `/drafts` the canonical writing workspace and beats Typefully on clarity, safety, and speed.

Repository anchors
- `dashboard/src/routes/(app)/drafts/+page.svelte`
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/routes/(app)/content/+page.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/Sidebar.svelte`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `docs/composer-mode.md`
- `docs/roadmap/composer-ui-typefully-redesign/charter.md`
- `docs/roadmap/composer-ui-typefully-redesign/ui-architecture.md`

Tasks
1. Audit the current split across home compose, drafts CRUD, calendar compose, and backend draft storage.
2. Write a charter that locks these defaults: one canonical draft workspace, server-backed draft records as source of truth, local autosave as crash recovery only, and no second independent home-composer state.
3. Define the UX blueprint for empty state, new draft, draft selection, thread editing, schedule, archive, restore, keyboard navigation, and mobile behavior.
4. Call out differentiators that must exceed Typefully: explicit sync state, safer AI undo, revision restore, and frictionless draft-to-scheduled-to-posted transitions.
5. Produce a phased technical architecture aligned to Sessions 02-11.
6. Keep code changes minimal; this session is planning and documentation first.

Deliverables
- `docs/roadmap/draft-studio-beyond-typefully/charter.md`
- `docs/roadmap/draft-studio-beyond-typefully/ux-blueprint.md`
- `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-01-handoff.md`

Quality gates
- No broad code changes expected. Verify every referenced path exists and every later-session dependency is explicit.

Exit criteria
- The documents contain no TBDs, choose concrete product defaults, and give later sessions enough detail to implement without re-planning.
```
