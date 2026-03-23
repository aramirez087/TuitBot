# Session 03: Hybrid Retrieval And API Contract

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Mission
Implement the hybrid retrieval and API layer that turns indexed vault content into fast, explainable semantic evidence results.

Repository anchors
- crates/tuitbot-server/src/routes/vault/mod.rs
- crates/tuitbot-server/src/routes/vault/selections.rs
- crates/tuitbot-server/src/routes/rag_helpers.rs
- crates/tuitbot-core/src/context/retrieval.rs
- crates/tuitbot-core/src/context/graph_expansion.rs
- crates/tuitbot-core/src/storage/watchtower/chunks.rs
- dashboard/src/lib/api/client.ts
- dashboard/src/lib/api/types.ts
- docs/roadmap/vault-indexer-semantic-search/semantic-index-architecture.md
- docs/roadmap/vault-indexer-semantic-search/indexer-lifecycle.md

Tasks
1. Add a semantic search endpoint and any supporting status contracts needed by Ghostwriter, with privacy-safe snippets, account scoping, explicit scope filters, and bounded limits.
2. Blend semantic ranking with current graph and keyword signals so results can explain whether they matched semantically, structurally, lexically, or through a hybrid score.
3. Extend request and response types so selection review, hook generation, tweet editing, and slot-targeted thread actions can issue evidence queries without bespoke APIs for each surface.
4. Add tests for account isolation, stale-index fallback, empty queries, limit enforcement, and ranking explanations, and document the contract.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/search-api-contract.md
- docs/roadmap/vault-indexer-semantic-search/retrieval-ranking-spec.md
- docs/roadmap/vault-indexer-semantic-search/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check

Exit criteria
- Semantic search works with the chosen index and falls back cleanly when needed.
- Every result carries enough reason data for user-facing explanation.
- Existing vault search and graph suggestion routes remain backward compatible.
```
