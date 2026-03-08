# Session 06: Provenance Through Drafts And Posting

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Continuity
- Treat provenance as a persisted contract that survives draft edits, approval, scheduling, posting, and analytics.

Mission
- Carry note and fragment provenance end to end so every generated draft or post can be traced back to source material.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/product-model.md`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `crates/tuitbot-core/src/storage/approval_queue/mod.rs`
- `crates/tuitbot-core/src/storage/approval_queue/queries.rs`
- `crates/tuitbot-core/src/storage/threads.rs`

Tasks
1. Add structured provenance fields or link tables for drafts, approval items, original tweets, and threads; do not rely on a loose `source` string.
2. Accept selected note or fragment refs from generation-entry APIs and persist them through draft and approval creation.
3. Ensure publish paths can recover originating note refs without reparsing prompt text or guessing from content.
4. Keep existing manual and legacy draft creation flows compatible by making provenance optional where needed.
5. Document provenance payloads, storage rules, and compatibility behavior.

Deliverables
- `migrations/20260308020000_vault_provenance_links.sql`
- `crates/tuitbot-core/migrations/20260308020000_vault_provenance_links.sql`
- `crates/tuitbot-server/src/routes/content/drafts.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-core/src/storage/scheduled_content.rs`
- `crates/tuitbot-core/src/storage/approval_queue/queries.rs`
- `crates/tuitbot-core/src/storage/threads.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/provenance-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-06-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Provenance now survives the full draft-to-post path, existing manual content still works, and later loop-back can rely on stored refs instead of heuristics.
```
