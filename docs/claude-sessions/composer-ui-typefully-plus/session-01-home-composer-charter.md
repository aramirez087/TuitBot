# Session 01: Home Composer Charter

Paste this into a new Claude Code session:

```md
Continuity
Start from the current repository state.

Mission
Lock the product and technical charter for a composer-first home experience that decisively beats Typefully's thread writing UX.

Repository anchors
- dashboard/src/routes/(app)/+page.svelte
- dashboard/src/lib/components/ComposeModal.svelte
- dashboard/src/lib/components/composer/ThreadFlowLane.svelte
- dashboard/src/lib/components/composer/ThreadFlowCard.svelte
- dashboard/src/lib/components/CommandPalette.svelte
- dashboard/src/lib/stores/persistence.ts
- dashboard/src/routes/(app)/settings/+page.svelte
- docs/roadmap/composer-ui-typefully-plus/charter.md
- docs/roadmap/composer-ui-typefully-plus/ui-architecture.md
- docs/roadmap/composer-ui-typefully-plus/benchmark-notes.md

Tasks
1. Audit the current home route, modal composer, thread flow components, command palette, persistence helpers, and settings layout so the plan matches the real code.
2. Update `docs/roadmap/composer-ui-typefully-plus/benchmark-notes.md` with the current competitive bar from official Typefully surfaces: continuous document editor, split thread with `Cmd/Ctrl+Enter`, plain-text thread splitting, configurable auto-split for X drafts, preview mode, and low-chrome top actions.
3. Update `docs/roadmap/composer-ui-typefully-plus/charter.md` so the goal explicitly includes a composer-first home surface, default-home behavior, and a Settings override back to analytics.
4. Update `docs/roadmap/composer-ui-typefully-plus/ui-architecture.md` to describe a shared compose workspace extracted from the modal and a dual-surface home renderer with `composer` and `analytics` modes.
5. Add `docs/roadmap/composer-ui-typefully-plus/home-surface-plan.md` with the exact acceptance criteria for the landing page:
   - full-page dark canvas with the writing lane as the first visible surface
   - centered thread lane with a left spine, avatar pucks, and low-noise separators
   - top-right `Schedule` and `Publish` pill actions with secondary icon tools
   - inline prompt module and dismissible getting-started tips that appear only when useful
   - analytics available as an alternate home surface, not the default
   - a persisted UI preference named `home_surface` with `composer` as the fresh-install default
6. Write a handoff that names the files Session 02 must extract or create first and calls out any architectural risks.

Deliverables
- docs/roadmap/composer-ui-typefully-plus/benchmark-notes.md
- docs/roadmap/composer-ui-typefully-plus/charter.md
- docs/roadmap/composer-ui-typefully-plus/ui-architecture.md
- docs/roadmap/composer-ui-typefully-plus/home-surface-plan.md
- docs/roadmap/composer-ui-typefully-plus/session-01-handoff.md

Exit criteria
- The benchmark and target experience are documented in enough detail that implementation sessions can execute without inventing the UX.
- The home-surface preference and shared compose-workspace architecture are clearly defined.
- Session 02 can start from concrete file-level instructions instead of open design questions.
```
