# Session 01: Benchmark And Charter

Paste this into a new Claude Code session:

```md
Continuity
- This is the first implementation session for `composer-ui-typefully-redesign`.

Mission
- Audit the current composer and produce a concrete redesign charter based on a live-post writing canvas plus a dedicated full-screen X-accurate preview mode.

Repository anchors
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowCard.svelte`
- `dashboard/src/lib/components/composer/ThreadPreviewRail.svelte`
- `dashboard/src/lib/components/CommandPalette.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`

Tasks
1. Audit the home composer, modal composer, and current preview behavior against the user's stated Typefully target.
2. Document the gaps that keep the editor from feeling like the post itself while writing, including chrome density, separators, and visual rhythm.
3. Define the target preview architecture: the compose surface is the writing canvas, and the X-accurate preview is a dedicated full-screen mode reached without route-driven draft duplication.
4. Trace the current `Cmd/Ctrl+J` path end to end and document exactly why it can replace or clear content unexpectedly.
5. Produce a session roadmap that splits visual redesign, preview-surface work, shortcut fixes, and validation into clear execution slices.

Deliverables
- `docs/roadmap/composer-ui-typefully-redesign/charter.md`
- `docs/roadmap/composer-ui-typefully-redesign/benchmark-notes.md`
- `docs/roadmap/composer-ui-typefully-redesign/ui-architecture.md`
- `docs/roadmap/composer-ui-typefully-redesign/session-01-handoff.md`

Exit criteria
- The documents define the target interaction model, non-goals, acceptance criteria, and the hotspot files for implementation.
- The handoff lists open questions, the final shortcut policy to implement, and exact next-session inputs.
```
