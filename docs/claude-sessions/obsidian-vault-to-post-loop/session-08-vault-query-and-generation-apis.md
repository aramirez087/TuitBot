# Session 08: Vault Query And Generation APIs

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Continuity
- Expose vault capabilities through explicit APIs; do not make the dashboard depend on storage internals or implicit prompt behavior.

Mission
- Add the server APIs needed to search notes, preview fragments, and drive vault-aware generation requests from the UI.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/retrieval-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/provenance-contract.md`
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `crates/tuitbot-server/src/state.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`

Tasks
1. Add authenticated APIs for vault source status, note or fragment search, note detail or preview, and selected-reference retrieval.
2. Extend generation-entry APIs to accept optional selected vault refs while keeping existing request shapes backward compatible.
3. Return citation and provenance metadata in generation responses wherever later UI surfaces need to show what was used.
4. Keep error behavior privacy-safe and avoid returning raw note bodies where summaries or selected fragments are sufficient.
5. Add contract tests and document the final API shapes.

Deliverables
- `crates/tuitbot-server/src/routes/assist.rs`
- `crates/tuitbot-server/src/routes/mod.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `docs/roadmap/obsidian-vault-to-post-loop/vault-api-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-08-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The dashboard can search and select vault content through stable APIs, generation contracts remain backward compatible, and response metadata supports later UI work.
```
