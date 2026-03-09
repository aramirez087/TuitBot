# Session 06: First Run Checklist And Unlocks

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Mission
Build the first-run activation experience that helps users unlock deferred capabilities after the short onboarding instead of front-loading every setup step.

Repository anchors
- docs/roadmap/x-profile-prefill-onboarding/session-05-handoff.md
- dashboard/src/routes/onboarding/+page.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/routes/(app)/settings/BrowserSessionSection.svelte
- dashboard/src/routes/(app)/settings/XApiSection.svelte
- dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte
- dashboard/src/lib/stores/accounts.ts

Tasks
1. Create a post-onboarding checklist or first-run surface that shows exactly which capabilities are unlocked, which are deferred, and what the next best action is.
2. Reuse the existing settings and credential flows where possible instead of rebuilding separate mini-wizards for X posting access, browser session import, LLM, or content sources.
3. Tailor the checklist by deployment mode so desktop and self-host users see only the steps that matter for them.
4. Keep first-value actions visible even when the user is only profile-ready or read-only, and avoid dead ends.
5. Add route-level validation and dashboard checks for checklist completeness, stale state, and unlock transitions.

Deliverables
- dashboard/src/lib/components/onboarding/ActivationChecklist.svelte
- dashboard/src/routes/(app)/content/+page.svelte
- dashboard/src/routes/onboarding/+page.svelte
- docs/roadmap/x-profile-prefill-onboarding/activation-checklist.md
- docs/roadmap/x-profile-prefill-onboarding/session-06-handoff.md

Quality gates
- cargo fmt --all && cargo fmt --all --check
- RUSTFLAGS="-D warnings" cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cd dashboard && npm run check

Exit criteria
- Users can finish the short onboarding and see a clear, deployment-aware path to unlock remaining capabilities.
- The checklist reuses existing settings surfaces instead of duplicating credential flows.
- The handoff names the exact provisioning and first-value actions Session 07 must implement.
```
