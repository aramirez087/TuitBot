# Session 04: Shortcuts And Thread Behavior

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
- Align shortcut behavior to the new interaction model and eliminate the destructive `Cmd/Ctrl+J` path.

Repository anchors
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/CommandPalette.svelte`
- `dashboard/src/lib/components/home/ComposerTipsTray.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`

Tasks
1. Audit shortcut ownership between the workspace, focused editors, thread cards, and browser defaults.
2. Make split-into-thread use `Cmd/Ctrl+Enter` consistently in behavior, labels, tooltips, and palette actions.
3. Remove the direct full-draft rewrite behavior from `Cmd/Ctrl+J`; it may only act on an explicit selection or surface AI affordances, but it must never wipe or replace the draft implicitly.
4. Update the shortcut catalog, command palette, tips tray, and any header copy so visible shortcut guidance matches reality.
5. Add a regression matrix that covers tweet mode, thread mode, modal compose, and home compose.

Deliverables
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/TweetEditor.svelte`
- `dashboard/src/lib/components/composer/ThreadFlowLane.svelte`
- `dashboard/src/lib/components/CommandPalette.svelte`
- `dashboard/src/lib/components/home/ComposerTipsTray.svelte`
- `dashboard/src/lib/utils/shortcuts.ts`
- `docs/roadmap/composer-ui-typefully-redesign/shortcut-regression-matrix.md`
- `docs/roadmap/composer-ui-typefully-redesign/session-04-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- `Cmd/Ctrl+Enter` reliably inserts a thread break in the editor contexts where it is advertised.
- `Cmd/Ctrl+J` can no longer wipe or rewrite the whole draft without an explicit selection.
- Every visible shortcut hint matches the implemented behavior.
```
