# Session 04: Ghostwriter Evidence Panel

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Add the semantic evidence search experience to Ghostwriter so users can move from idea to support without leaving the current compose flow.

Repository anchors
- CLAUDE.md
- docs/composer-mode.md
- dashboard/src/lib/components/composer/FromVaultPanel.svelte
- dashboard/src/lib/components/composer/VaultSelectionReview.svelte
- dashboard/src/lib/components/composer/HookPicker.svelte
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- dashboard/tests/unit/
- docs/roadmap/vault-indexer-semantic-search/ghostwriter-evidence-ux.md
- docs/roadmap/vault-indexer-semantic-search/search-api-contract.md

Tasks
1. Add the evidence search surface inside the existing Ghostwriter sidebar without breaking the current selection, hook, or manual vault path.
2. Seed queries automatically from the current context such as selected text, chosen hook, tweet draft, or focused thread card, while allowing manual edits and scope changes.
3. Implement loading, indexing, stale, empty, and degraded states so index freshness is legible but low-noise.
4. Let users pin and unpin evidence for later generation steps, keep graph suggestions understandable, add unit tests, and document the interaction model.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/ghostwriter-evidence-interaction.md
- docs/roadmap/vault-indexer-semantic-search/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- A user can find relevant support from the sidebar without losing their place in compose.
- Automatic query seeding feels helpful instead of surprising.
- Empty and degraded states preserve trust and current workflow continuity.
```
