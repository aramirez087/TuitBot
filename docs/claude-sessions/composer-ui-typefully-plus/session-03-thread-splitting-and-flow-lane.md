# Session 03: Thread Splitting And Flow Lane

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Turn thread writing into a fast continuous lane that beats Typefully on split speed, clarity, and keyboard flow while preserving current data contracts.

Repository anchors
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte
- dashboard/src/lib/components/composer/ThreadFlowCard.svelte
- dashboard/src/lib/components/ThreadComposer.svelte
- dashboard/src/lib/components/composer/TweetEditor.svelte
- dashboard/src/lib/components/CommandPalette.svelte
- dashboard/src/lib/utils/tweetLength.ts

Tasks
1. Migrate thread mode in `ComposeWorkspace` off the old stacked `ThreadComposer` flow and onto `ThreadFlowLane` and `ThreadFlowCard`, removing the form-like "card stack" feel.
2. Make thread splitting explicit and fast:
   - `Cmd/Ctrl+Enter` splits the active block at the cursor
   - double-empty-line or paragraph-aware paste can create new blocks when the input clearly represents a thread
   - backspace at offset 0 merges with the previous block
   - `Alt+Up` and `Alt+Down` reorder the active block without losing focus
3. Make the visuals match the benchmark image but cleaner:
   - a persistent left spine line connecting blocks
   - compact avatar anchors or neutral markers aligned to that spine
   - no boxed cards, no heavy borders, and no chunky bottom action rows
   - between-block tools hidden until hover or focus, with character count, reorder handle, merge, and remove
4. Replace any remaining "Add tweet card" language in the UI and command palette with language that fits the new model such as "Split thread" or "Add post below".
5. Preserve `ThreadBlock[]`, per-block media attachment, autosave, validation, and a textarea-based fallback if contenteditable-level complexity is unnecessary.
6. Document cursor-management edge cases and the exact follow-up needed for preview, prompts, and CTA polish.

Deliverables
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte
- dashboard/src/lib/components/composer/ThreadFlowCard.svelte
- dashboard/src/lib/components/CommandPalette.svelte
- docs/roadmap/composer-ui-typefully-plus/session-03-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- A three-post thread can be written and reordered without clicking a persistent "add tweet" button.
- Split, merge, reorder, media, and validation all work without breaking autosave or submit behavior.
- The lane looks and feels like a writing surface, not a form builder.
```
