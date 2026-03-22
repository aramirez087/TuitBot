# Session 07: Thread Publish Normalization

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Mission
Normalize Ghostwriter thread publishing so Forge can treat a thread as one source-note outcome with reliable persistence and provenance.

Repository anchors
- crates/tuitbot-core/src/automation/approval_poster.rs
- crates/tuitbot-core/src/storage/threads.rs
- crates/tuitbot-server/src/routes/content/compose/transforms.rs
- crates/tuitbot-server/src/routes/content/drafts.rs
- crates/tuitbot-server/src/routes/approval/handlers.rs
- docs/roadmap/hook-miner-forge-loop/forge-thread-contract.md

Tasks
1. Treat approved and scheduled `thread` content as a first-class reply chain instead of flattening it into tweet-only assumptions.
2. When a Ghostwriter thread posts, persist one `threads` row, one `thread_tweets` row per posted tweet, and one `original_tweets` row for the root tweet so current topic and analytics behavior still have a canonical key.
3. Copy provenance from the approval or scheduled entity to both the new thread entity and the root `original_tweet`.
4. Use the root tweet id as the canonical note-sync key while preserving child tweet ids for later aggregation.
5. Ensure direct compose, approval poster, and scheduled publish share the same normalization rules as far as the current architecture allows.
6. Add tests for successful thread posting, partial failure behavior, provenance propagation, and scheduled thread bridging.
7. Document the normalization contract and remaining limitations.

Deliverables
- docs/roadmap/hook-miner-forge-loop/thread-publish-normalization.md
- docs/roadmap/hook-miner-forge-loop/session-07-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Ghostwriter threads persist enough identifiers for later Forge aggregation.
- Provenance survives the thread publish path.
- The root tweet id is stable and explicit.
```
