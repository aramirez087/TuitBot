# Session 06: Hook-First Compose Flow

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/hook-generation-contract.md` and `docs/roadmap/obsidian-ghostwriter-edge/session-05-handoff.md`.

Mission
Integrate hook generation into compose so users can compare hook options, choose one, and generate a stronger tweet or thread from Ghostwriter context.

Repository anchors
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/VaultFooter.svelte`
- `dashboard/src/lib/components/composer/VaultHighlights.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte`
- `dashboard/src/lib/stores/draftStudio.svelte.ts`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`

Tasks
1. Add a hook review step to the Ghostwriter flow with clear compare, choose, regenerate, and fallback behavior.
2. Use the chosen hook to drive downstream tweet or thread generation in compose and draft-studio entry paths.
3. Preserve source context during retries so users can iterate on hooks without losing their selected block or citations.
4. Add unit coverage for hook selection, retry flows, and mode switching between tweet and thread.

Deliverables
- `dashboard/src/lib/components/composer/FromVaultPanel.svelte`
- `dashboard/src/lib/components/composer/VaultFooter.svelte`
- `dashboard/src/lib/components/composer/VaultHighlights.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/drafts/DraftStudioShell.svelte`
- `dashboard/src/lib/components/drafts/DraftDetailsPanel.svelte`
- `dashboard/src/lib/stores/draftStudio.svelte.ts`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `docs/roadmap/obsidian-ghostwriter-edge/hook-first-workflow.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-06-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run test:unit:run`

Exit criteria
- A user can generate five hooks, choose one, and produce a grounded draft without losing Ghostwriter context.
- Session 07 can propagate that state through the rest of the product.
```
