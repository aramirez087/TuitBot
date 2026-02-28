# Session 06: Google Drive Source Adapter

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/session-05-handoff.md and the source-management files updated in Session 05.

Mission
Add Google Drive as a provider-backed content source while preserving the same ingest and Winning DNA pipeline used by local folders.

Repository anchors
- crates/tuitbot-core/src/automation/watchtower.rs
- crates/tuitbot-core/src/storage/watchtower.rs
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/mod.rs
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/api.ts
- dashboard/src/routes/onboarding/+page.svelte

Tasks
1. Refactor source handling so local filesystem watching and remote sync both feed one shared ingest queue and one source-state model.
2. Add a Google Drive adapter that polls a configured folder, detects changed .md or .txt files, fetches content, and records stable provider IDs in source_context.
3. Extend configuration and settings APIs to support a concrete google_drive source type with the credentials and folder identifier needed for polling.
4. Add dashboard controls for configuring a Google Drive source, keeping provider-specific fields isolated from the local-folder path flow.
5. Add tests for provider selection, remote change deduplication, and ingest parity between local and Google Drive sources.

Deliverables
- crates/tuitbot-core/src/automation/watchtower.rs
- crates/tuitbot-core/src/storage/watchtower.rs
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/settings.rs
- crates/tuitbot-server/src/routes/mod.rs
- dashboard/src/lib/api.ts
- dashboard/src/routes/onboarding/+page.svelte
- docs/roadmap/cold-start-watchtower-rag/google-drive-integration.md
- docs/roadmap/cold-start-watchtower-rag/session-06-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check

Exit criteria
- A configured Google Drive source reaches the same ingest pipeline as a local source.
- Remote file changes are deduplicated by provider ID and content hash.
- Local-folder behavior remains intact after the provider abstraction.
```
