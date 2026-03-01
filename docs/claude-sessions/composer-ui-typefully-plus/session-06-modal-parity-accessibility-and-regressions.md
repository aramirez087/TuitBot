# Session 06: Modal Parity Accessibility And Regressions

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Bring every compose entry point, shortcut, and responsive state into parity so the new home composer feels native everywhere and regressions are closed before ship.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/CommandPalette.svelte
- dashboard/src/lib/utils/shortcuts.ts
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte

Tasks
1. Ensure calendar, drafts, discovery, and global shortcut entry points reuse the same compose-workspace behavior as the new home page, not an older modal-only path.
2. Reconcile command palette terminology and actions with the new editor model so actions read as split below, merge, move, preview, inspector, schedule, and publish.
3. Verify autosave, recovery, media attachment, voice cues, AI improve, from notes, focus mode, preview, and the mobile inspector drawer behave the same in modal and full-page contexts.
4. Tighten accessibility with correct focus order, aria-live reorder announcements, keyboard hints, 44px touch targets, and no pointer-only controls.
5. Polish performance and motion by avoiding layout thrash, debouncing expensive preview work if needed, and keeping transitions subtle.
6. Document any remaining deliberate scope cuts before final validation.

Deliverables
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/CommandPalette.svelte
- dashboard/src/lib/utils/shortcuts.ts
- docs/roadmap/composer-ui-typefully-plus/session-06-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The user can enter compose from any existing path without dropping to an older UX.
- Keyboard, accessibility, and mobile behavior are consistent across modal and full-page contexts.
- Remaining risks are explicitly documented for the validation session.
```
