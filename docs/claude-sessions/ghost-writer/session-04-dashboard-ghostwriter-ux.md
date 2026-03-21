# Session 04: Dashboard Ghostwriter UX

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/block-contracts.md` and `docs/roadmap/obsidian-ghostwriter-edge/session-03-handoff.md`.

Mission
Turn the current vault panel into a true Ghostwriter workspace for exact block selection, ingress from Obsidian, and provenance-aware composition.

Repository anchors
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/VaultNoteList.svelte`
- `dashboard/src/lib/components/composer/VaultFooter.svelte`
- `dashboard/src/lib/components/composer/InspectorContent.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/CitationChips.svelte`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `dashboard/tests/unit/VaultHighlights.test.ts`

Tasks
1. Rework the vault flow so users can see and select exact source blocks instead of thinking in whole-note terms.
2. Support Ghostwriter ingress payloads from Obsidian as first-class compose state rather than forcing manual re-selection in the dashboard.
3. Tighten the CTA copy and interaction model so “send block”, “use source”, and citation actions are obvious to a power user.
4. Add unit coverage for selection state, ingress hydration, and citation affordances.

Deliverables
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/VaultNoteList.svelte`
- `dashboard/src/lib/components/composer/VaultFooter.svelte`
- `dashboard/src/lib/components/composer/InspectorContent.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/CitationChips.svelte`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `dashboard/tests/unit/VaultHighlights.test.ts`
- `docs/roadmap/obsidian-ghostwriter-edge/dashboard-ghostwriter-ux.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-04-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run test:unit:run`

Exit criteria
- The dashboard has a concrete Ghostwriter interaction model for exact blocks.
- Session 05 can add hook generation without revisiting selection UX or ingress state.
```
