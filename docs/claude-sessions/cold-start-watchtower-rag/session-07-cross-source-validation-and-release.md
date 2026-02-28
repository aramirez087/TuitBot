# Session 07: Cross-Source Validation And Release

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/session-06-handoff.md, docs/roadmap/cold-start-watchtower-rag/google-drive-integration.md, and the files modified in Sessions 02 through 06.

Mission
Validate the full Cold Start workflow across local folders and Google Drive, then issue a documented go or no-go release recommendation.

Repository anchors
- docs/architecture.md
- docs/configuration.md
- crates/tuitbot-core/src/automation/watchtower.rs
- crates/tuitbot-core/src/storage/watchtower.rs
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/lib/api.ts
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src-tauri/src/lib.rs

Tasks
1. Run the required Rust and dashboard quality gates and fix any regressions introduced by the new source-management work.
2. Perform an end-to-end shakeout covering a Tauri folder-pick flow, local filesystem ingest, manual POST /api/ingest, a simulated Google Drive sync, seed creation, and source-file loop-back where applicable.
3. Update architecture and configuration docs so the provider model, local-path setup, Google Drive setup, and operational limits are explicit.
4. Produce a go or no-go validation report with unresolved risks, rollback notes, and exact follow-up work if the epic is not release-ready.

Deliverables
- docs/architecture.md
- docs/configuration.md
- docs/roadmap/cold-start-watchtower-rag/validation-report.md
- docs/roadmap/cold-start-watchtower-rag/session-07-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check

Exit criteria
- The quality gates pass without suppressing warnings.
- The documented manual test covers both local-folder and Google Drive source paths.
- The validation report ends with a clear go or no-go recommendation.
```
