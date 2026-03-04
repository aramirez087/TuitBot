# Session 02: Extend Core Generator Contract

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/charter.md
- Read docs/roadmap/composer-auto-vault-context/implementation-plan.md
- Read docs/roadmap/composer-auto-vault-context/session-01-handoff.md

Mission
Add the core primitives needed for server-side composer RAG without changing public HTTP payloads.

Repository anchors
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-core/src/config/tests.rs
- crates/tuitbot-core/src/content/generator/mod.rs
- crates/tuitbot-core/src/content/generator/tests.rs
- crates/tuitbot-core/src/workflow/draft.rs

Tasks
1. Add a reusable business-profile helper for draft-context keywords so composer and reply workflows stop duplicating keyword assembly.
2. Expose the minimum `ContentGenerator` surface needed for server routes to obtain draft-context keywords from the same business profile used for prompt generation.
3. Extend draft improvement to accept both a user directive and an optional RAG prompt block, while preserving the existing `improve_draft()` call pattern as a compatibility wrapper if possible.
4. Refactor the reply draft workflow to use the new shared keyword helper so behavior stays aligned across automation and composer paths.
5. Add or update unit tests covering keyword assembly and dual-context improve prompt composition.

Deliverables
- docs/roadmap/composer-auto-vault-context/core-contract.md
- docs/roadmap/composer-auto-vault-context/session-02-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Core code exposes a stable way to assemble composer RAG inputs, tests cover the new behavior, and existing callers remain source-compatible or are updated cleanly.
```
