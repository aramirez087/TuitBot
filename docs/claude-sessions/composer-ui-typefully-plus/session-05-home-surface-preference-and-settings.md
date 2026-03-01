# Session 05: Home Surface Preference And Settings

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Mission
Add a durable Settings-controlled home surface preference with composer as the default and analytics as the user-selectable fallback.

Repository anchors
- dashboard/src/lib/stores/persistence.ts
- dashboard/src/routes/(app)/+page.svelte
- dashboard/src/routes/(app)/settings/+page.svelte
- dashboard/src/lib/components/Sidebar.svelte
- dashboard/src/lib/stores/settings.ts
- dashboard/src/lib/components/home/AnalyticsHome.svelte
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte

Tasks
1. Create `dashboard/src/lib/stores/homeSurface.ts` that exposes `homeSurface`, `homeSurfaceReady`, `loadHomeSurface`, and `setHomeSurface`, defaulting to `composer`.
2. Expand `dashboard/src/lib/stores/persistence.ts` so UI preferences persist in both Tauri and browser dev through the Tauri store when available and a `localStorage` fallback otherwise.
3. Add `dashboard/src/routes/(app)/settings/WorkspaceSection.svelte` with a clear "Default landing page" control and options for `Composer home` and `Analytics overview`, with copy that makes composer the recommended default.
4. Wire the Settings page to render the new workspace section without mixing this UI preference into backend automation config and without requiring backend schema changes.
5. Update the home route so it waits for the preference to load, then renders `ComposeWorkspace` or `AnalyticsHome` accordingly, with composer chosen when no prior preference exists.
6. Preserve a direct path to analytics even when composer is default, and avoid flicker, redirect loops, or stale state after the user changes the setting.

Deliverables
- dashboard/src/lib/stores/homeSurface.ts
- dashboard/src/lib/stores/persistence.ts
- dashboard/src/routes/(app)/settings/WorkspaceSection.svelte
- dashboard/src/routes/(app)/settings/+page.svelte
- dashboard/src/routes/(app)/+page.svelte
- docs/roadmap/composer-ui-typefully-plus/session-05-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- Fresh installs land on the composer home by default.
- Changing the setting reliably flips the home surface between composer and analytics.
- The preference persists in desktop and browser development flows.
```
