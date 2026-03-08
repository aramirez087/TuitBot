# Session 02: Draft Domain And Schema

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Read the charter, UX blueprint, and technical architecture before editing schema or storage code.

Mission
- Implement the durable draft domain model that can support a unified workspace, organization metadata, and revision history without breaking existing content flows.

Repository anchors
- `docs/roadmap/draft-studio-beyond-typefully/charter.md`
- `docs/roadmap/draft-studio-beyond-typefully/technical-architecture.md`
- `migrations/20260222000006_scheduled_content.sql`
- `migrations/20260226000011_composer_mode.sql`
- `crates/tuitbot-core/migrations/20260222000006_scheduled_content.sql`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `crates/tuitbot-server/src/routes/content/drafts.rs`

Tasks
1. Extend `scheduled_content` instead of creating a parallel primary table unless a blocker is discovered and documented.
2. Add additive columns needed for draft title, scratchpad/notes, archive semantics, and open/save metadata.
3. Add normalized tables for tags, revisions, and activity with content-oriented names, not UI-specific names.
4. Mirror all migrations in both `migrations/` and `crates/tuitbot-core/migrations/`.
5. Update storage structs and helpers so legacy draft list/create/edit flows still work against migrated data.
6. Document the final schema, lifecycle states, and compatibility rules.

Deliverables
- `migrations/20260308000100_draft_studio_foundation.sql`
- `crates/tuitbot-core/migrations/20260308000100_draft_studio_foundation.sql`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `docs/roadmap/draft-studio-beyond-typefully/data-model.md`
- `docs/roadmap/draft-studio-beyond-typefully/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Existing draft, scheduled, and posted records still load; the schema supports tags, revisions, and activity without follow-up redesign; the docs match the migration exactly.
```
