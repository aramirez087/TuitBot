# Session 04: Wire Tweet and Thread Assist

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/server-rag-contract.md
- Read docs/roadmap/composer-auto-vault-context/session-03-handoff.md

Mission
Route automatic vault context into composer tweet and thread generation without changing the client contract.

Repository anchors
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-core/src/content/generator/mod.rs
- dashboard/src/lib/api.ts

Tasks
1. Update `/api/assist/tweet` to resolve composer RAG context and call `generate_tweet_with_context()`.
2. Update `/api/assist/thread` to resolve composer RAG context and call `generate_thread_with_context()`.
3. Preserve the existing JSON response shapes and maintain success behavior when the context helper returns `None`.
4. Add concise implementation notes describing how tweet and thread now inherit the same vault-awareness pattern as autopilot drafting.
5. Write the handoff with the exact remaining work for `/api/assist/improve`.

Deliverables
- docs/roadmap/composer-auto-vault-context/tweet-thread-wiring.md
- docs/roadmap/composer-auto-vault-context/session-04-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Composer tweet and thread assist calls inject automatic vault context when available and remain backward-compatible when it is absent.
```
