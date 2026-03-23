# Session 02: Indexer And Storage Foundation

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Implement the background vault indexing foundation that keeps semantic search fresh without blocking ingest or compose.

Repository anchors
- CLAUDE.md
- crates/tuitbot-core/src/automation/watchtower/mod.rs
- crates/tuitbot-core/src/automation/watchtower/chunker.rs
- crates/tuitbot-core/src/storage/watchtower/
- crates/tuitbot-core/src/llm/
- crates/tuitbot-core/migrations/
- crates/tuitbot-server/src/state.rs
- docs/roadmap/vault-indexer-semantic-search/semantic-index-architecture.md
- docs/roadmap/vault-indexer-semantic-search/implementation-map.md

Tasks
1. Add the schema, storage, and worker abstractions required by the chosen semantic index design, including dirty-state tracking, versioning, and freshness status.
2. Wire index scheduling into the existing Watchtower lifecycle so note adds, edits, deletes, and re-chunks update the semantic index incrementally and fail open.
3. Add the minimum runtime and config plumbing needed to compute embeddings and report index readiness without exposing raw content through read APIs.
4. Write focused Rust tests for migration safety, idempotent updates, delete handling, and stale-index recovery, and document the lifecycle decisions.

Deliverables
- docs/roadmap/vault-indexer-semantic-search/indexer-lifecycle.md
- docs/roadmap/vault-indexer-semantic-search/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Vault changes mark and refresh semantic records predictably.
- Index freshness can be observed without opening the database manually.
- Stale or failed indexing degrades safely to existing behavior.
```
