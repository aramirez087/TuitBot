# Session 09: Capability Based Source UX

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/deployment-capability-matrix.md, docs/roadmap/cold-start-watchtower-rag/session-08-handoff.md, and the capability contract changes made in Session 08.

Mission
Implement a mode-aware source configuration UX so desktop shows a native folder picker, self-host keeps a manual path option, and cloud onboarding is connector-first.

Repository anchors
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/api.ts
- dashboard/src-tauri/src/lib.rs
- dashboard/src-tauri/Cargo.toml
- dashboard/package.json
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/settings.rs

Tasks
1. Update the dashboard API client to fetch and cache source capabilities before rendering source controls.
2. Split the source-management UI so local folder selection appears only when local_folder is supported, manual path entry appears only when manual_local_path is supported, and cloud users are steered toward connectors such as Google Drive.
3. Keep the Tauri dialog-based folder picker strictly behind the desktop capability path and remove any implication that it is available in cloud-hosted browsing contexts.
4. Add explicit UX copy for unsupported source types so users understand why a local filesystem option is unavailable in cloud mode.
5. Add focused coverage for capability-driven rendering and any desktop-only command wiring that changed.

Deliverables
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/api.ts
- dashboard/src-tauri/src/lib.rs
- dashboard/src-tauri/Cargo.toml
- dashboard/package.json
- crates/tuitbot-server/src/routes/runtime.rs
- crates/tuitbot-server/src/routes/settings.rs
- docs/roadmap/cold-start-watchtower-rag/session-09-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check

Exit criteria
- Desktop users still get a native folder-picker flow.
- Self-hosted browser users can configure a local path without a native picker.
- Cloud users are not shown local filesystem affordances.
```
