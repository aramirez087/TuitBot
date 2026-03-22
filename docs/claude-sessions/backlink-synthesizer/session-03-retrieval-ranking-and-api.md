# Session 03: Retrieval Ranking And API

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Build graph-aware retrieval and expose it through privacy-safe APIs that explain why each related note was selected.

Repository anchors
- crates/tuitbot-core/src/context/retrieval.rs
- crates/tuitbot-core/src/context/winning_dna/analysis.rs
- crates/tuitbot-server/src/routes/rag_helpers.rs
- crates/tuitbot-server/src/routes/vault/mod.rs
- crates/tuitbot-server/src/routes/vault/selections.rs
- crates/tuitbot-server/tests/assist_rag_tests.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs
- docs/roadmap/backlink-synthesizer/graph-rag-architecture.md
- docs/roadmap/backlink-synthesizer/graph-storage-contract.md

Tasks
1. Implement graph candidate expansion around a selected note using direct backlinks, outbound links, and shared tags, with note-centric fallback when graph data is sparse.
2. Add a ranking policy that favors strong direct relationships, preserves diversity across neighbor notes, and limits token budget by capping fragments per related note.
3. Surface related-note suggestions through additive API shapes that include reason labels, matched tags, suggestion intent such as pro-tip or counterpoint, and citation-ready metadata.
4. Add server-side response shapes for the full flow state: no related notes, unresolved links, sparse graph fallback, and accepted-suggestion provenance.
5. Ensure existing assist and selection flows can consume graph retrieval without breaking current note-biased behavior.
6. Add tests for ranking, fallback behavior, provenance, and account isolation.
7. Document the ranking rules and API contract.

Deliverables
- docs/roadmap/backlink-synthesizer/retrieval-ranking-spec.md
- docs/roadmap/backlink-synthesizer/graph-api-contract.md
- docs/roadmap/backlink-synthesizer/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The backend can return graph suggestions with human-readable reasons like backlink, linked note, or shared tag.
- The API contract is rich enough for a polished UI without extra hidden client heuristics.
- Existing assist routes still succeed when graph retrieval returns nothing.
- Returned items are provenance-ready and privacy-safe.
```
