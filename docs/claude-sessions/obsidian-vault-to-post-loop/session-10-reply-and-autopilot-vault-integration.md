# Session 10: Reply And Autopilot Vault Integration

Paste this into a new Claude Code session:

```md
Continue from Session 09 artifacts.

Continuity
- Treat reply quality and automation parity as part of the core feature, not a later polish pass.

Mission
- Apply vault-aware context and provenance consistently across reply surfaces, discovery workflows, and automation paths.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/retrieval-contract.md`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `crates/tuitbot-core/src/workflow/draft.rs`
- `crates/tuitbot-core/src/automation/discovery_loop.rs`
- `crates/tuitbot-core/src/automation/target_loop.rs`
- `crates/tuitbot-core/src/automation/mentions_loop.rs`

Tasks
1. Route manual reply assist and discovery reply compose through the vault-aware context builder where that context improves output.
2. Update automation reply paths to use the same retrieval rules unless a documented product reason forbids it.
3. Preserve product-mention and safety semantics while making reply provenance or citations available where useful.
4. Ensure queued replies and autopilot drafts can carry provenance through to approval and posting when vault context is used.
5. Document the final reply-surface behavior and any intentionally different cases.

Deliverables
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `crates/tuitbot-core/src/workflow/draft.rs`
- `crates/tuitbot-core/src/automation/discovery_loop.rs`
- `crates/tuitbot-core/src/automation/target_loop.rs`
- `crates/tuitbot-core/src/automation/mentions_loop.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/reply-integration.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-10-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Vault context is no longer a composer-only partial feature and reply or automation flows now follow one coherent contract.
```
