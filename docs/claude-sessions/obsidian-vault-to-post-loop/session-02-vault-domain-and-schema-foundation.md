# Session 02: Vault Domain And Schema Foundation

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Read the charter, product model, and implementation plan before editing schema or storage code.

Mission
- Implement the durable account-scoped vault domain model that can support fragments, provenance, and loop-back without breaking existing content flows.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/product-model.md`
- `migrations/20260228000019_watchtower_ingestion.sql`
- `crates/tuitbot-core/migrations/20260228000019_watchtower_ingestion.sql`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-core/src/storage/mod.rs`
- `crates/tuitbot-core/src/storage/threads.rs`

Tasks
1. Make vault/source storage truly account-scoped instead of default-account-only, updating APIs rather than relying on global rows.
2. Add additive schema for durable vault documents or fragments and structured provenance links; do not overload `draft_seeds` as the primary long-term note model.
3. Add the minimum additive columns or link tables needed so drafts, approval items, and posted originals can later point back to source material.
4. Mirror every migration in both migration directories and preserve compatibility for existing `content_nodes`, `draft_seeds`, and original tweet history.
5. Update storage structs and helpers so old callers still work while new account-aware APIs exist for later sessions.
6. Document the final schema, lifecycle states, and compatibility rules.

Deliverables
- `migrations/20260308010000_vault_domain_foundation.sql`
- `crates/tuitbot-core/migrations/20260308010000_vault_domain_foundation.sql`
- `crates/tuitbot-core/src/storage/watchtower/mod.rs`
- `crates/tuitbot-core/src/storage/mod.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/data-model.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Vault data is no longer implicitly global, the schema supports fragments and provenance without follow-up redesign, and compatibility behavior is documented exactly.
```
