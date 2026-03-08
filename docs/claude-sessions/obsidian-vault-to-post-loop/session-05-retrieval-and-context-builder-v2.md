# Session 05: Retrieval And Context Builder V2

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Continuity
- Keep winning-tweet ancestry and vault retrieval as one combined context system, not two disconnected prompt hacks.

Mission
- Build the account-filtered retrieval engine that returns exact vault fragments with citations and feeds a stronger context builder than the current seed-only path.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/fragment-indexing.md`
- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-core/src/storage/analytics/ancestors.rs`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`

Tasks
1. Add retrieval over stored vault fragments, favoring account isolation, keyword relevance, recency, and optionally selected-note bias.
2. Return structured citation records alongside prompt text so later API and UI work can expose the exact source material used.
3. Replace the current global cold-start seed fallback with the new fragment retrieval path wherever that is now the better source of truth.
4. Keep winning ancestors in the context model and define how they combine with vault citations without bloating prompts.
5. Add tests that prove account filtering, citation payload shape, and mixed ancestor-plus-fragment behavior.
6. Document retrieval ranking, prompt shaping, and guardrails.

Deliverables
- `crates/tuitbot-core/src/context/winning_dna.rs`
- `crates/tuitbot-core/src/storage/analytics/ancestors.rs`
- `crates/tuitbot-server/tests/assist_rag_tests.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/retrieval-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-05-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Retrieval is account-safe, fragment-based, citation-capable, and materially stronger than the current note-seed fallback.
```
