# Session 03: Hook Miner Extraction Engine

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Build the Hook Miner backend so vault-backed drafting returns three evidence-first angles with rationale and citations.

Repository anchors
- crates/tuitbot-core/src/content/generator/mod.rs
- crates/tuitbot-core/src/content/generator/parser.rs
- crates/tuitbot-core/src/context/retrieval.rs
- crates/tuitbot-server/src/routes/assist/hooks.rs
- crates/tuitbot-server/src/routes/rag_helpers.rs
- crates/tuitbot-server/src/lib.rs
- docs/roadmap/hook-miner-forge-loop/hook-miner-contract.md

Tasks
1. Add additive domain and API types for mined signals and angles with exact enums `contradiction`, `data_point`, `aha_moment`, `story`, `listicle`, and `hot_take`.
2. Implement a backend flow that mines evidence from the selected note context plus accepted related-note context, then synthesizes exactly three canonical angles from that evidence.
3. Add `POST /api/assist/hook-miner`; accept `topic`, optional `selected_node_ids`, and optional `session_id`, reusing the current selection-first RAG resolution rules.
4. Return stable signal ids, seed text, rationale, supporting signal ids, and citation-ready metadata without exposing raw note bodies beyond current privacy rules.
5. Implement weak-signal behavior: if fewer than two credible signals survive parsing, return an explicit fallback state and keep `/api/assist/hooks` untouched.
6. Add tests for normal, sparse, privacy-safe, and account-isolated responses.
7. Document the response shape and evidence-mining rules.

Deliverables
- docs/roadmap/hook-miner-forge-loop/hook-miner-api-contract.md
- docs/roadmap/hook-miner-forge-loop/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The API returns 3 mined angles with rationale and citations.
- Sparse notes degrade to an explicit fallback state instead of a vague failure.
- The generic hook endpoint still works unchanged.
```
