# Session 04: Markdown Fragment Indexing

Paste this into a new Claude Code session:

```md
Continue from Session 03 artifacts.

Continuity
- Preserve Obsidian note structure and favor stable fragment identities over opaque whole-note blobs.

Mission
- Extract durable note fragments during ingest so retrieval can cite specific headings or blocks instead of vague note-level seeds.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/data-model.md`
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/automation/watchtower/loopback.rs`
- `crates/tuitbot-core/src/source/local_fs.rs`
- `crates/tuitbot-core/src/source/google_drive/mod.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`

Tasks
1. Parse markdown into stable note fragments using headings and text blocks while preserving path, title, tags, and frontmatter context.
2. Store enough metadata to reconstruct citations and later deep links without reparsing raw files on every assist call.
3. Keep `.txt` ingestion working with a sensible plain-text fallback fragment strategy.
4. Preserve current dedup or update semantics so re-ingest updates changed fragments predictably.
5. Add focused tests for fragment extraction, update behavior, and mixed-source ingest.
6. Document extraction rules, fragment identity rules, and fallback behavior.

Deliverables
- `crates/tuitbot-core/src/automation/watchtower/mod.rs`
- `crates/tuitbot-core/src/source/tests/integration.rs`
- `docs/roadmap/obsidian-vault-to-post-loop/fragment-indexing.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-04-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Ingested vault content now has stable, queryable fragments with predictable updates and documented structure for later retrieval work.
```
