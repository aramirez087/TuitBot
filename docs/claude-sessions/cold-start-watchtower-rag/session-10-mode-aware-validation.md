# Session 10: Mode Aware Validation

Paste this into a new Claude Code session:

```md
Continue from Session 09 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/session-09-handoff.md, docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md, and the files modified in Sessions 08 and 09.

Mission
Validate that source selection now behaves correctly across desktop, self-host, and cloud modes, then issue a corrected go or no-go recommendation.

Repository anchors
- docs/architecture.md
- docs/configuration.md
- docs/roadmap/BACKLOG-cloud-hosted-tier.md
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/settings.rs
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/api.ts
- dashboard/src-tauri/src/lib.rs

Tasks
1. Run the required Rust and dashboard quality gates and fix any regressions introduced by the capability refactor.
2. Perform a manual shakeout for three cases: desktop with native picker, self-host browser with manual local path, and cloud with connector-only source selection.
3. Confirm that local-folder configs remain valid where supported and are rejected with clear messaging in cloud mode.
4. Update the validation report so it reflects the corrected deployment-aware source behavior and any remaining risks.

Deliverables
- docs/architecture.md
- docs/configuration.md
- docs/roadmap/cold-start-watchtower-rag/validation-report.md
- docs/roadmap/cold-start-watchtower-rag/session-10-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check

Exit criteria
- The quality gates pass without suppressing warnings.
- The manual validation explicitly covers desktop, self-host, and cloud source UX paths.
- The updated validation report ends with a clear go or no-go recommendation.
```
