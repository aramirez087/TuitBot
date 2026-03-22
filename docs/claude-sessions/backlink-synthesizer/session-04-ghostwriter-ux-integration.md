# Session 04: Ghostwriter UX Integration

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Mission
Integrate the entry flow from Ghostwriter selection into composer so the user instantly understands what was selected, what related context is available, and what to do next.

Repository anchors
- dashboard/src/lib/api/types.ts
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/components/composer/
- dashboard/tests/unit/
- crates/tuitbot-server/src/routes/vault/selections.rs
- crates/tuitbot-server/src/ws.rs
- docs/roadmap/backlink-synthesizer/epic-charter.md
- docs/roadmap/backlink-synthesizer/graph-api-contract.md
- docs/roadmap/backlink-synthesizer/retrieval-ranking-spec.md

Tasks
1. Design and implement the entry state after a note or block is sent from Obsidian, with a clear source summary, selected-text preview, and a visible “finding related notes” phase.
2. Add first-run, loading, empty, and degraded states so the user always knows whether the system is working, has no related notes, or fell back to the selected note only.
3. Show suggestion cards with note title, short snippet, reason badge, and explicit actions such as use as pro-tip, use as example, use as counterpoint, or dismiss.
4. Keep the current compose flow intact with an obvious session-level toggle to disable related-note synthesis without losing the original selection.
5. Preserve provenance and citations when a suggestion is accepted.
6. Add unit tests for the new entry-flow states and document the interaction model.

Deliverables
- docs/roadmap/backlink-synthesizer/ghostwriter-entry-flow.md
- docs/roadmap/backlink-synthesizer/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- A first-time user can understand the flow without external docs.
- Empty and degraded states are calm and actionable instead of vague.
- Provenance and existing compose behavior remain intact.
```
