# Session 02: Live Canvas Surface

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
- Rework the compose surface so the editor itself visually reads like the post output while preserving existing compose contracts.

Repository anchors
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowCard.svelte`
- `dashboard/src/lib/components/home/ComposerTipsTray.svelte`
- `dashboard/src/lib/components/home/ComposerPromptCard.svelte`

Tasks
1. Replace the boxed textarea feel in tweet and thread modes with a calmer live-post canvas that uses subtle separators, inline metadata, and lower chrome.
2. Refactor `TweetEditor.svelte`, `ThreadFlowLane.svelte`, and `ThreadFlowCard.svelte` so thread drafting feels continuous and post-like while still emitting the same `ThreadBlock[]`.
3. Simplify `HomeComposerHeader.svelte` and `ComposeWorkspace.svelte` so the default compose surface no longer treats preview as inline companion chrome and instead leaves room for a dedicated full-screen preview mode.
4. Rework or remove the current tips and prompt modules if they fight the cleaner hierarchy.
5. Preserve autosave, inspector access, schedule/publish flows, media attachment, and accessibility cues.

Deliverables
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowCard.svelte`
- `dashboard/src/lib/components/home/ComposerTipsTray.svelte`
- `dashboard/src/lib/components/home/ComposerPromptCard.svelte`
- `docs/roadmap/composer-ui-typefully-redesign/session-02-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The home composer reads as a live post canvas on first paint.
- Thread drafting no longer looks like stacked form cards.
- Existing compose flows still work without backend changes.
```
