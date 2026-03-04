# Session 03: Add Composer RAG Context Resolver

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/core-contract.md
- Read docs/roadmap/composer-auto-vault-context/session-02-handoff.md

Mission
Implement one reusable server-side helper that resolves optional composer RAG context from the Watchtower-backed vault data.

Repository anchors
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-core/src/context/winning_dna.rs
- crates/tuitbot-core/src/content/generator/mod.rs

Tasks
1. Add an assist-layer helper that calls `build_draft_context()` with the generator-derived keyword set and the existing winning-DNA constants instead of new magic numbers.
2. Make the helper return an optional prompt block and fail open on storage or retrieval errors, logging the failure without breaking generation.
3. Keep the implementation centralized so tweet, thread, and improve handlers can share it without duplicating logic.
4. Document the helper contract, fallback behavior, and any logging decisions.
5. Write the handoff with exact integration points for Session 04.

Deliverables
- docs/roadmap/composer-auto-vault-context/server-rag-contract.md
- docs/roadmap/composer-auto-vault-context/session-03-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- `assist.rs` contains one shared composer-RAG resolver and no endpoint yet duplicates raw `build_draft_context()` plumbing.
```
