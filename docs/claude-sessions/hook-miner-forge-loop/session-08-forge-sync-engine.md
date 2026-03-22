# Session 08: Forge Sync Engine

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Mission
Extend the current loopback path into an opt-in analytics sync engine that enriches source-note frontmatter when metrics arrive.

Repository anchors
- crates/tuitbot-core/src/automation/analytics_loop.rs
- crates/tuitbot-core/src/automation/adapters/storage.rs
- crates/tuitbot-core/src/automation/watchtower/loopback.rs
- crates/tuitbot-core/src/storage/analytics/
- crates/tuitbot-core/src/storage/threads.rs
- docs/roadmap/hook-miner-forge-loop/forge-frontmatter-contract.md
- docs/roadmap/hook-miner-forge-loop/forge-thread-contract.md

Tasks
1. Preserve the current immediate publish writeback and add a later analytics-enrichment path instead of replacing the existing writer.
2. Extend measurement so Ghostwriter thread child tweets can be aggregated for Forge, using additive storage keyed by tweet id without regressing current tweet metrics.
3. Build an idempotent sync step that finds eligible `local_fs` notes from provenance, matches the existing `tuitbot` entry by root `tweet_id`, updates entry metrics, and refreshes the top-level summary fields.
4. Aggregate thread metrics across root and child tweet ids before writing entry metrics or note-level summaries.
5. Respect `analytics_sync_enabled`, skip Google Drive and Cloud paths, and fail open when a source note is missing or non-writable.
6. Add tests for single-tweet sync, thread aggregation, duplicate protection, non-local skips, and stale-note handling.
7. Document the sync architecture and operational behavior.

Deliverables
- docs/roadmap/hook-miner-forge-loop/forge-sync-architecture.md
- docs/roadmap/hook-miner-forge-loop/session-08-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Notes update after metrics are measured without duplicate entries.
- Thread chains sync aggregated metrics correctly.
- Non-local sources remain non-writable and harmless.
```
