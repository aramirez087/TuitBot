# Session 12: Desktop Obsidian Integration And Polish

Paste this into a new Claude Code session:

```md
Continue from Session 11 artifacts.

Continuity
- Focus this session on desktop-grade vault affordances that materially improve the Obsidian workflow without breaking self-hosted or cloud paths.

Mission
- Add the Obsidian-specific desktop touches that make the feature feel native for local-vault users.

Repository anchors
- `docs/roadmap/obsidian-vault-to-post-loop/ux-blueprint.md`
- `dashboard/src-tauri/src/lib.rs`
- `dashboard/src-tauri/src/main.rs`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/components/onboarding/LocalFolderInput.svelte`

Tasks
1. Add desktop affordances to open a source note from citations or vault views, using the best available local integration path and a graceful fallback when Obsidian-specific deep links are unavailable.
2. Make local-vault flows feel intentional with clearer labels, note-open actions, and any minimal metadata or config needed for Obsidian interoperability.
3. Keep all desktop-only behavior guarded so web, self-hosted, and cloud builds remain safe.
4. Document the desktop integration behavior, fallbacks, and any operator setup required.

Deliverables
- `dashboard/src-tauri/src/lib.rs`
- `dashboard/src-tauri/src/main.rs`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `docs/roadmap/obsidian-vault-to-post-loop/obsidian-desktop-integration.md`
- `docs/roadmap/obsidian-vault-to-post-loop/session-12-handoff.md`

Quality gates
- `npm --prefix dashboard run check`
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Desktop users can move cleanly between Tuitbot and their source notes, platform guards are correct, and the behavior is documented precisely.
```
