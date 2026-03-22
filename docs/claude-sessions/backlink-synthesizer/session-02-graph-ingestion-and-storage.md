# Session 02: Graph Ingestion And Storage

Paste this into a new Claude Code session:

```md
Mission
Implement the additive storage and ingestion layer that turns vault notes into a resolvable note graph.

Repository anchors
- crates/tuitbot-core/src/automation/watchtower/mod.rs
- crates/tuitbot-core/src/automation/watchtower/chunker.rs
- crates/tuitbot-core/src/storage/watchtower/mod.rs
- crates/tuitbot-core/src/storage/watchtower/nodes.rs
- crates/tuitbot-core/src/storage/watchtower/tests.rs
- crates/tuitbot-core/src/storage/watchtower/tests_storage.rs
- migrations/

Tasks
1. Add additive schema for note-link edges and normalized note tags, account-scoped and cascade-safe.
2. Extend Watchtower ingest to extract wiki links and markdown note links from note bodies, then persist unresolved targets plus resolved target node IDs when possible.
3. Normalize frontmatter tags into query-friendly rows while preserving existing `content_nodes.tags` behavior.
4. Make re-ingest idempotent: stale links and tags must be replaced safely when a note changes.
5. Add focused tests for extraction, storage, account isolation, and re-resolution behavior.
6. Document the storage contract and migration choices.

Deliverables
- docs/roadmap/backlink-synthesizer/graph-storage-contract.md
- docs/roadmap/backlink-synthesizer/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The database can answer “which notes link to this note” and “which notes share tags with this note” without parsing raw markdown at request time.
- Ingest remains backward-compatible for sources with no links or tags.
- Tests prove link extraction and tag normalization are deterministic and account-scoped.
```
