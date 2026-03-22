# Session 05: Draft Insertion And Suggestion Controls

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Make accepting a related-note suggestion feel precise and reversible so users can shape tweets and threads without losing authorship.

Repository anchors
- dashboard/src/lib/components/composer/
- dashboard/src/lib/api/types.ts
- dashboard/src/lib/api/client.ts
- dashboard/tests/unit/
- crates/tuitbot-server/src/routes/content/compose/
- crates/tuitbot-server/tests/compose_contract_tests.rs
- docs/roadmap/backlink-synthesizer/end-to-end-ux-journey.md
- docs/roadmap/backlink-synthesizer/ghostwriter-entry-flow.md
- docs/roadmap/backlink-synthesizer/graph-api-contract.md

Tasks
1. Implement the acceptance flow for related-note suggestions so the user can preview, accept, undo, replace, or ignore a suggestion without regenerating everything.
2. Add thread-aware controls that can target a slot such as opening hook, tweet 2, tweet 4 pro-tip, or closing takeaway, with clear feedback about what changed.
3. Make accepted suggestions visible inside the draft as contextual inserts rather than invisible prompt mutations.
4. Ensure citation chips and provenance reflect both the original selected note and any accepted related-note suggestions.
5. Protect the user from destructive updates by preserving the previous draft state and offering an easy revert path.
6. Add unit tests and decision docs for insertion behavior.

Deliverables
- docs/roadmap/backlink-synthesizer/draft-insertion-model.md
- docs/roadmap/backlink-synthesizer/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Suggestion acceptance is visible, reversible, and slot-aware.
- Users can improve a draft incrementally instead of feeling forced into full regeneration.
- Provenance remains correct after undo, replace, and multi-suggestion flows.
```
