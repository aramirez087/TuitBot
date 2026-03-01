# Session 04: Premium Actions Prompts And Polish

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Upgrade the composer chrome, prompts, preview, and assist surfaces so the home experience feels more premium and more helpful than Typefully.

Repository anchors
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/composer/ComposerHeaderBar.svelte
- dashboard/src/lib/components/composer/ComposerCanvas.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/composer/ThreadPreviewRail.svelte
- dashboard/src/lib/components/composer/VoiceContextPanel.svelte
- dashboard/src/lib/components/FromNotesPanel.svelte
- dashboard/src/lib/components/TimePicker.svelte

Tasks
1. Replace modal-era chrome with a premium home action bar:
   - left side shows account identity and subtle draft state
   - right side shows warm `Schedule` and cool `Publish` pill CTAs
   - preview, focus, inspector, and help live as quiet icon tools after the CTAs
2. Make preview an intentional companion instead of a cramped always-on pane:
   - desktop supports a writing-dominant mode and a live preview mode or rail
   - mobile stacks preview below the active draft or inside the inspector drawer
   - preview stays X-accurate and updates in real time
3. Add a contextual prompt module below the active draft that appears when the draft is empty, the user is idle, or the user asks for inspiration; include replace, examples, refresh, and dismiss actions.
4. Add a dismissible getting-started tips tray that teaches split-thread, media drop, and cue or AI shortcuts without ever stealing focus or blocking typing.
5. Refine `ComposerInspector` so AI, voice, notes, and scheduling are stronger than Typefully's side affordances but remain secondary until invoked.
6. Make the page feel slicker than the benchmark through spacing, restrained motion, and clear CTA hierarchy, not generic dashboard styling.

Deliverables
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/composer/ComposerHeaderBar.svelte
- dashboard/src/lib/components/composer/ComposerCanvas.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/home/ComposerPromptCard.svelte
- dashboard/src/lib/components/home/ComposerTipsTray.svelte
- docs/roadmap/composer-ui-typefully-plus/session-04-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The top-right `Schedule` and `Publish` CTAs read as the primary actions on the page.
- Prompts and tips help the user start faster but disappear cleanly when not needed.
- Preview, AI, and scheduling are easy to access without cluttering the writing lane.
```
