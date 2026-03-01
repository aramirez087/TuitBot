# Session 04: Inspector Actions And Polish

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission

Add the polished secondary experience around the writing canvas so scheduling, metadata, AI actions, and guidance feel contextual and premium instead of noisy.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/lib/components/composer/VoiceContextPanel.svelte
- dashboard/src/lib/components/composer/ThreadPreviewRail.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- docs/composer-mode.md

Tasks
1. Move secondary controls into a cleaner contextual rail or inspector model so the main canvas stays focused while schedule, mode-specific actions, and AI helpers remain easy to reach.
2. Refine the notes, voice, and assist surfaces so they feel integrated with the composer rather than layered on top as separate mini-tools.
3. Add lightweight guidance and empty-state cues that teach power behavior without interrupting drafting.
4. Ensure the redesigned composer still works cleanly in mobile widths and when opened from the calendar flow.
5. Update docs to reflect the delivered UI model, shortcuts, and any changed affordances.

Deliverables
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/lib/components/composer/VoiceContextPanel.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- docs/composer-mode.md
- docs/roadmap/composer-ui-typefully-plus/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check
- cd dashboard && npm run build

Exit criteria
- The composer's secondary controls feel contextual, compact, and visually subordinate to writing.
- The docs match the shipped UI.
- The calendar entry point and mobile layout remain solid.
```
