# Session 09: Forge Settings And Prompt UX

Paste this into a new Claude Code session:

```md
Continue from Session 08 artifacts.

Mission
Add the settings, one-time prompt behavior, and user-facing copy that make Forge analytics sync understandable and consent-driven.

Repository anchors
- dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte
- dashboard/src/lib/stores/settings.ts
- dashboard/src/lib/stores/activity.ts
- dashboard/src/routes/(app)/activity/+page.svelte
- crates/tuitbot-server/src/routes/settings/mod.rs
- crates/tuitbot-server/src/routes/settings/handlers.rs
- crates/tuitbot-core/src/config/types.rs
- docs/configuration.md
- docs/roadmap/hook-miner-forge-loop/forge-sync-architecture.md

Tasks
1. Add a new source setting `analytics_sync_enabled` with default `false` and keep `loop_back_enabled` as the separate immediate publish-history switch.
2. Update config, validation, API, and settings UI so the distinction between publish writeback and analytics sync is explicit.
3. Show a one-time post-publish prompt when the just-published content has eligible `local_fs` provenance, `loop_back_enabled` is true, and `analytics_sync_enabled` is still false.
4. If the publish happened outside the current draft view, surface one pending prompt on the next relevant dashboard load instead of spamming repeated prompts.
5. Make “Not now” suppress repeats until the user revisits Settings or explicitly re-enables prompting state.
6. Write clear copy about local-only writes, delayed stats arrival, and unsupported source types.
7. Add frontend and settings-route tests for the new flag and prompt behavior.

Deliverables
- docs/roadmap/hook-miner-forge-loop/settings-and-copy-notes.md
- docs/roadmap/hook-miner-forge-loop/session-09-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- npm --prefix dashboard run check
- npm --prefix dashboard run test:unit:run

Exit criteria
- Users can understand the difference between immediate writeback and ongoing analytics sync.
- Eligible users see the prompt once, not repeatedly.
- Unsupported sources are explained instead of silently failing.
```
