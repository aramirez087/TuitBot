# Session 03: Draft API And Sync Contract

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Read the data model and only expose API surface that is directly supported by the migrated schema.

Mission
- Implement the server and client contract for Draft Studio CRUD, selection, filters, autosave patches, and workflow transitions.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/data-model.md`
- `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/routes/content/mod.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `dashboard/src/lib/api/client.ts`
- `crates/tuitbot-server/tests/compose_contract_tests.rs`

Tasks
1. Add endpoints and handlers for collection query, single draft fetch, blank or seeded draft creation, metadata patching, content patching, duplicate, archive, and tab/status transitions.
2. Separate autosave patch semantics from publish and schedule actions so the UI can save often without workflow side effects.
3. Expose revision and activity read endpoints needed later, even if the UI waits until Session 09.
4. Keep current draft routes backward compatible where feasible and document any compatibility shim.
5. Update dashboard client types and methods to cover the new contract cleanly.
6. Add integration tests covering draft lifecycle, filters, and backward compatibility.

Deliverables
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/routes/content/mod.rs`
- `dashboard/src/lib/api/client.ts`
- `crates/tuitbot-server/tests/draft_studio_api_tests.rs`
- `docs/roadmap/draft-studio-beyond-typefully/api-sync-contract.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-03-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- The API can represent one canonical draft workspace end to end, autosave writes are side-effect free, and the client contract is explicit enough for the frontend sessions to build against directly.
```
