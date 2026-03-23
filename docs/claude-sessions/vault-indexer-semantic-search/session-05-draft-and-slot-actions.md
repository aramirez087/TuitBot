# Session 05: Draft And Slot Actions

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Connect semantic evidence to tweet and thread editing so users can strengthen whole drafts or specific slots while keeping provenance and undo honest.

Repository anchors
- dashboard/src/lib/components/composer/ComposerInspector.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte
- dashboard/src/lib/components/composer/CitationChips.svelte
- dashboard/src/lib/stores/draftInsertStore.ts
- dashboard/src/lib/utils/threadLaneActions.ts
- dashboard/src/lib/api/types.ts
- crates/tuitbot-server/src/routes/content/compose/mod.rs
- crates/tuitbot-core/src/storage/provenance.rs
- docs/roadmap/vault-indexer-semantic-search/ghostwriter-evidence-interaction.md
- docs/roadmap/vault-indexer-semantic-search/retrieval-ranking-spec.md

Tasks
1. Let semantic evidence target either whole-draft improvement or a specific tweet or thread slot using the current focused-block and undoable-insert mechanics.
2. Preserve provenance and citation data for semantic hits, including source node, source chunk, and match reason, without inventing false certainty about what was used.
3. Make tweet and thread actions feel native: one-click whole-tweet support, focused-card targeting, slot labels, and reversible edits in thread mode.
4. Add tests for undo behavior, provenance persistence, and slot targeting, and document the action model.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/provenance-and-slot-actions.md
- docs/roadmap/vault-indexer-semantic-search/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Evidence can strengthen tweet or thread content without replacing user text unexpectedly.
- Undo and provenance remain consistent across whole-draft and slot-targeted actions.
- Citation UI makes semantic support traceable instead of mysterious.
```
