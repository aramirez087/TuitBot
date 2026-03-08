# Session 06: Rail Keyboard And Multi-Draft Actions

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Continuity
- Optimize for dense scanning and keyboard-first flow without hiding save state or destructive consequences.

Mission
- Make the draft rail faster than Typefully by supporting dense scanning, keyboard navigation, and zero-confusion quick actions.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/ux-blueprint.md`
- `docs/roadmap/draft-studio-beyond-typefully/autosave-and-sync.md`
- `dashboard/src/lib/stores/draftStudio.ts`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/CommandPalette.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`

Tasks
1. Build rail tabs for Drafts, Scheduled, Posted, and Archived with counts, search, sort, and active filters.
2. Add keyboard navigation for rail traversal, open-in-place, new draft, duplicate current, archive current, restore archived, and jump back to composer.
3. Implement quick actions that never hide save state and always give an undo-safe path for destructive behavior where practical.
4. Update the command palette and shortcut catalog so workspace actions are discoverable and consistent.
5. Document the interaction model and shortcut map.

Deliverables
- `dashboard/src/lib/components/drafts/DraftRail.svelte`
- `dashboard/src/lib/components/drafts/DraftRailItem.svelte`
- `dashboard/src/lib/components/CommandPalette.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`
- `docs/roadmap/draft-studio-beyond-typefully/rail-interactions.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-06-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- A user can manage drafts without the mouse, destructive actions are explicit, and rail counts and filters stay in sync with the selected tab and store state.
```
