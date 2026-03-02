# Session 03: Full-Screen X Preview

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
- Add a full-screen high-fidelity preview mode that mirrors X without pulling the writing canvas back into side-by-side preview mode.

Repository anchors
- `dashboard/src/routes/(app)/+page.svelte`
- `dashboard/src/lib/components/ComposeModal.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/ThreadPreviewRail.svelte`

Tasks
1. Build a dedicated full-screen preview mode that renders the draft in an X-accurate layout and feels distinct from edit mode.
2. Move the current realistic preview rendering out of the inline editor path and into a dedicated preview component layered over the same compose session.
3. Wire the home and modal composer actions so users can enter preview and return without losing draft state or editor position.
4. Reuse the existing compose state source of truth instead of duplicating serialization or branching submit logic.
5. Document the transition and state-restoration rules for single tweets, multi-post threads, and scheduled drafts.

Deliverables
- `dashboard/src/lib/components/composer/ComposerPreviewSurface.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/HomeComposerHeader.svelte`
- `dashboard/src/lib/components/composer/ThreadPreviewRail.svelte`
- `docs/roadmap/composer-ui-typefully-redesign/session-03-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Preview opens as a full-screen dedicated mode, not inline compose chrome.
- Returning to compose restores the same draft and leaves the editor ready to continue without route navigation.
- The preview rendering is clearly closer to X than the compose surface while sharing the same draft data.
```
