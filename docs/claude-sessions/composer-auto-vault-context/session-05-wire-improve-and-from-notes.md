# Session 05: Wire Improve and From Notes

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Continuity
- Read docs/roadmap/composer-auto-vault-context/tweet-thread-wiring.md
- Read docs/roadmap/composer-auto-vault-context/session-04-handoff.md

Mission
Integrate automatic vault context into draft improvement while preserving current inline-improve and from-notes semantics.

Repository anchors
- crates/tuitbot-server/src/routes/assist.rs
- crates/tuitbot-core/src/content/generator/mod.rs
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- docs/composer-mode.md

Tasks
1. Update `/api/assist/improve` to resolve automatic vault context and pass it through the new core improve path together with any user-supplied `context`.
2. Preserve the current meaning of `context` for quick cues and from-notes instructions; automatic RAG must be additive, not substitutive.
3. Confirm no frontend payload change is required by reading the current composer callers, and only touch frontend code if a real contract mismatch is discovered.
4. Record how inline improve, selected-text improve, and from-notes flows behave after the backend change.
5. Write the handoff with exact coverage gaps for the testing session.

Deliverables
- docs/roadmap/composer-auto-vault-context/improve-flow.md
- docs/roadmap/composer-auto-vault-context/session-05-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- `/api/assist/improve` now combines user intent with automatic vault context and existing composer flows keep their current request payloads.
```
