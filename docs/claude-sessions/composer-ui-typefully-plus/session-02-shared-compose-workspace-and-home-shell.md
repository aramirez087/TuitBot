# Session 02: Shared Compose Workspace And Home Shell

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Mission
Extract a reusable compose workspace and make the app home route render a composer-first page shell instead of analytics cards.

Repository anchors
- dashboard/src/routes/(app)/+page.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ComposerShell.svelte
- dashboard/src/lib/components/composer/ComposerHeaderBar.svelte
- dashboard/src/lib/components/composer/ComposerCanvas.svelte
- dashboard/src/lib/stores/analytics.ts
- dashboard/src/lib/components/StatCard.svelte
- dashboard/src/lib/components/FollowerChart.svelte
- dashboard/src/lib/components/TopTopics.svelte
- dashboard/src/lib/components/RecentPerformance.svelte

Tasks
1. Extract the current analytics dashboard markup from `dashboard/src/routes/(app)/+page.svelte` into `dashboard/src/lib/components/home/AnalyticsHome.svelte` without changing its behavior.
2. Extract the stateful compose body from `dashboard/src/lib/components/ComposeModal.svelte` into `dashboard/src/lib/components/composer/ComposeWorkspace.svelte` so the same editor surface can render full-page and inside the modal.
3. Refactor `dashboard/src/lib/components/ComposeModal.svelte` into a thin modal wrapper that only owns backdrop, focus trap, recovery banner, and close behavior while delegating the writing UI to `ComposeWorkspace`.
4. Replace the current root route with a composer-first home page that mounts `ComposeWorkspace` directly in page context and keeps writing above the fold.
5. Match the home shell to the target visual frame:
   - full-page canvas instead of a centered modal
   - immediate focus into the first writing area
   - a top-right action cluster position reserved even if the final CTA styling lands in a later session
   - a wider centered writing lane than the old modal
   - reserved space below the draft for prompts and tips modules
6. Keep the existing global compose trigger and calendar compose flow working by ensuring modal callers still use the same shared workspace code path.

Deliverables
- dashboard/src/lib/components/home/AnalyticsHome.svelte
- dashboard/src/lib/components/composer/ComposeWorkspace.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/routes/(app)/+page.svelte
- docs/roadmap/composer-ui-typefully-plus/session-02-handoff.md

Quality gates
- cd dashboard && npm run check
- cd dashboard && npm run build
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings

Exit criteria
- The home route opens to a full-page compose surface, not the analytics overview.
- The modal compose flow still works and reuses the shared workspace.
- The old analytics home content still exists through `AnalyticsHome.svelte` for the later preference session.
```
