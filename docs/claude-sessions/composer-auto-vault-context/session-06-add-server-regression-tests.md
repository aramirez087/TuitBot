# Session 06: Add Server Regression Tests

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/improve-flow.md
- Read docs/roadmap/composer-auto-vault-context/session-05-handoff.md

Mission
Add regression coverage that proves composer assist now consumes automatic vault context without breaking existing contracts.

Repository anchors
- crates/tuitbot-server/tests/api_tests.rs
- crates/tuitbot-server/tests/compose_contract_tests.rs
- crates/tuitbot-core/src/source/tests/integration.rs
- crates/tuitbot-server/src/routes/assist.rs

Tasks
1. Build a test harness for assist endpoints that can register a `ContentGenerator` backed by a deterministic test `LlmProvider`.
2. Seed enough test data to exercise the composer RAG path, using the existing source-context integration tests as the reference for Watchtower-backed context setup.
3. Add assertions for tweet, thread, and improve flows that verify requests still succeed and that prompt composition includes the expected automatic context when available.
4. Add at least one fail-open regression test showing assist generation still works when no vault context exists.
5. Write a test matrix summary and the handoff for docs and release notes.

Deliverables
- docs/roadmap/composer-auto-vault-context/test-matrix.md
- docs/roadmap/composer-auto-vault-context/session-06-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Automated tests cover success-with-context and success-without-context paths for all three composer assist endpoints.
```
