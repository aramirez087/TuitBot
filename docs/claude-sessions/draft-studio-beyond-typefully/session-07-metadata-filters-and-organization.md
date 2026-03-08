# Session 07: Metadata Filters And Organization

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Continuity
- Keep metadata editing low-friction and subordinate to the writing canvas rather than turning the route back into a form.

Mission
- Ship the per-draft organizational tools that make the workspace meaningfully more powerful than a plain integrated draft list.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/ux-blueprint.md`
- `docs/roadmap/draft-studio-beyond-typefully/data-model.md`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/api/client.ts`

Tasks
1. Add internal title, tags, scratchpad or notes, source badge, and ready-state affordances to the details panel.
2. Support filtering and sorting by tag, last edited, scheduled time, source, and workflow state.
3. Keep metadata editing live and inline; do not add a separate modal form for routine organization tasks.
4. Make the details panel responsive for desktop and mobile.
5. Document the organization model and any constraints or non-goals.

Deliverables
- `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte`
- `dashboard/src/lib/components/drafts/DraftFilterBar.svelte`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/api/client.ts`
- `docs/roadmap/draft-studio-beyond-typefully/metadata-and-filters.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-07-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Metadata is editable without leaving the draft, filters survive reload and selection changes, and notes do not compete with the writing canvas for primary attention.
```
