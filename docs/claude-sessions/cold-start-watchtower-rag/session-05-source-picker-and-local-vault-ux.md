# Session 05: Source Picker And Local Vault UX

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.
Continuity
- Read docs/roadmap/cold-start-watchtower-rag/session-04-handoff.md, docs/roadmap/cold-start-watchtower-rag/rag-ranking.md, and the files modified in Sessions 02 through 04.

Mission
Add the desktop and dashboard UX for selecting, saving, and inspecting a local content-source folder for the Watchtower.

Repository anchors
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/api.ts
- dashboard/src-tauri/src/lib.rs
- dashboard/src-tauri/Cargo.toml
- dashboard/package.json
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/settings.rs

Tasks
1. Expose the new content-source settings through the existing init and patch settings flows so the frontend can read and persist a local vault path.
2. Add a dashboard step or settings surface for source management that lets users review the current source and manually enter a folder path when native desktop capabilities are unavailable.
3. Add a Tauri folder-picker command using the dialog plugin so desktop users can choose an Obsidian vault without typing raw paths.
4. Wire the UI to save the selected path back into config using the existing API client and show basic validation or error feedback.
5. Add focused coverage for config round-tripping and document any desktop-only behavior the web build cannot offer.

Deliverables
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/lib/api.ts
- dashboard/src-tauri/src/lib.rs
- dashboard/src-tauri/Cargo.toml
- dashboard/package.json
- crates/tuitbot-core/src/config/types.rs
- crates/tuitbot-server/src/routes/settings.rs
- docs/roadmap/cold-start-watchtower-rag/session-05-handoff.md

Quality gates
cargo fmt --all && cargo fmt --all --check
RUSTFLAGS="-D warnings" cargo test --workspace
cargo clippy --workspace -- -D warnings
cd dashboard && npm run check

Exit criteria
- Desktop users can pick a folder through the Tauri shell and persist it.
- Browser users still have a usable manual-path fallback.
- The chosen local source path round-trips cleanly through the config API.
```
