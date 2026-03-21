# Session 03: Backend Block Contracts

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Load `docs/roadmap/obsidian-ghostwriter-edge/ghostwriter-architecture.md`, `docs/roadmap/obsidian-ghostwriter-edge/obsidian-plugin-contract.md`, and `docs/roadmap/obsidian-ghostwriter-edge/session-02-handoff.md`.

Mission
Implement the backend contracts that receive exact Ghostwriter selections, resolve them against vault data, and preserve provenance through compose and draft workflows.

Repository anchors
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-core/src/storage/watchtower/chunks.rs`
- `crates/tuitbot-core/src/storage/watchtower/nodes.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`
- `crates/tuitbot-server/tests/compose_contract_tests.rs`

Tasks
1. Add the receiving contract for Ghostwriter selections and resolve each payload to the best available indexed block identity without weakening account isolation.
2. Extend provenance handling so exact block selections survive draft creation, compose submission, and later citation rendering.
3. Add any required additive schema or contract changes, but do not break current vault search or existing `selected_node_ids` callers.
4. Add focused tests for route behavior, provenance persistence, and malformed transport payloads.

Deliverables
- `crates/tuitbot-server/src/routes/vault.rs`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/content/compose/mod.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-core/src/context/retrieval.rs`
- `crates/tuitbot-core/src/storage/provenance.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`
- `crates/tuitbot-server/tests/compose_contract_tests.rs`
- `docs/roadmap/obsidian-ghostwriter-edge/block-contracts.md`
- `docs/roadmap/obsidian-ghostwriter-edge/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The backend can receive an exact Ghostwriter selection and return or persist stable provenance.
- Session 04 can build the dashboard UX on top of a concrete contract instead of mock behavior.
```
