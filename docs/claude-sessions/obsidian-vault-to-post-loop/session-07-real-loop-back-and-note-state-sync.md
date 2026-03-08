# Session 07: Real Loop Back And Note State Sync

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Continuity
- Only claim loop-back behavior that is powered by persisted provenance and successful posting events.

Mission
- Implement real, idempotent loop-back from posted content to source notes and keep note metadata synchronized safely.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/provenance-contract.md`
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-core/src/workflow/publish.rs`
- `crates/tuitbot-core/src/storage/threads.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`

Tasks
1. Use persisted note or fragment provenance to locate the source note for posted tweets or threads and write back metadata idempotently.
2. Expand loop-back metadata so it can represent tweet or thread status, URLs, timestamps, and later analytics updates without destroying existing frontmatter.
3. Keep local files writable but leave Google Drive and manual sources explicitly read-only, with clear no-op behavior where loop-back is unsupported.
4. Prevent self-induced ingest storms by pairing loop-back writes with safe cooldown or re-ingest handling.
5. Add integration coverage that exercises real provenance-based loop-back rather than direct helper-only tests.
6. Document the frontmatter contract and source-type limitations.

Deliverables
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`
- `crates/tuitbot-core/src/automation/approval_poster.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/loopback-contract.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-07-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Loop-back is now real, provenance-driven, source-aware, idempotent, and documented with its exact local-vs-remote behavior.
```
