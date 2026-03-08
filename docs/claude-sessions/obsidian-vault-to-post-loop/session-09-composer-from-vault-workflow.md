# Session 09: Composer From Vault Workflow

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Continuity
- Build on the existing composer rather than replacing it, but make vault selection feel first-class beside manual notes.

Mission
- Implement the composer-side From Vault workflow with search, selection, citations, and safe generation or insertion behavior.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/ux-blueprint.md`
- `docs/roadmap/obsidian-vault-to-post-loop/vault-api-contract.md`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/InspectorContent.svelte`
- `dashboard/src/lib/components/FromNotesPanel.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`

Tasks
1. Add a From Vault flow alongside From Notes so users can search notes or fragments, inspect excerpts, and choose source material deliberately.
2. Let users generate tweet or thread drafts from selected vault refs and preserve those refs in draft state for later provenance and loop-back.
3. Show citations or source chips in the composer in a way that is visible but does not clutter normal writing.
4. Keep undo, replacement safety, keyboard access, and mobile behavior as strong as the current composer.
5. Document the interaction model and any new draft metadata shown in the UI.

Deliverables
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/lib/components/composer/InspectorContent.svelte`
- `dashboard/src/lib/components/FromNotesPanel.svelte`
- `dashboard/src/lib/components/composer/ComposerInspector.svelte`
- `docs/roadmap/obsidian-vault-to-post-loop/composer-vault-workflow.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-09-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Users can intentionally compose from vault material, see what they used, and recover safely from replacements without losing trust in the editor.
```
