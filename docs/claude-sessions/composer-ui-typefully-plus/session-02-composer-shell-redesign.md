# Session 02: Composer Shell Redesign

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission

Rebuild the compose shell into a cleaner, calmer writing surface that feels closer to a premium drafting app than a generic dashboard modal.

Repository anchors
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ComposerShell.svelte
- dashboard/src/lib/components/composer/VoiceContextPanel.svelte
- dashboard/src/lib/components/composer/ThreadPreviewRail.svelte
- dashboard/src/app.css
- docs/roadmap/composer-ui-typefully-plus/ui-architecture.md

Tasks
1. Implement the new shell layout from the Session 01 spec: stronger writing canvas hierarchy, reduced chrome, cleaner header, and responsive desktop/mobile composition zones.
2. Separate primary writing controls from secondary controls so the composer feels focused by default while still exposing power actions fast.
3. Introduce any new shell subcomponents needed to keep files maintainable and the interaction model legible.
4. Preserve all existing open/close, focus mode, autosave, and submit behavior while changing the presentation and layout.
5. Update roadmap artifacts if the implementation reveals a necessary layout adjustment.

Deliverables
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ComposerShell.svelte
- dashboard/src/lib/components/composer/ComposerCanvas.svelte
- dashboard/src/lib/components/composer/ComposerHeaderBar.svelte
- docs/roadmap/composer-ui-typefully-plus/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check
- cd dashboard && npm run build

Exit criteria
- The composer opens into a visibly cleaner structure with less dashboard-like noise.
- Existing compose workflows still function without regression.
- New shell pieces map cleanly to the Session 01 architecture.
```
