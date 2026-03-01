# Session 03: Dashboard Danger Zone

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.
Continuity
- Read `docs/roadmap/factory-reset-danger-zone/reset-contract.md` and `docs/roadmap/factory-reset-danger-zone/session-02-handoff.md` first.

Mission
Add a clearly marked Settings danger zone that requires typed confirmation and sends the user back to onboarding immediately after a successful reset.

Repository anchors
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/routes/(app)/settings/LanAccessSection.svelte`
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/auth.ts`
- `dashboard/src/lib/stores/settings.ts`
- `dashboard/src/routes/+layout.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`

Tasks
1. Add a dedicated danger-zone section to Settings navigation and content, visually separated from normal settings.
2. Implement a typed confirmation UX using the exact phrase from the charter; do not use the existing timer-only double-click pattern.
3. Add the API client method for the reset route, submit the confirmation phrase, and surface loading and error states clearly.
4. After success, clear local web-session state without breaking Tauri bearer mode, reset relevant in-memory UI stores, and route to `/onboarding`.
5. Document the UX flow and any known limitations that remain because the server does not hot-restart subsystems.

Deliverables
- `dashboard/src/routes/(app)/settings/DangerZoneSection.svelte`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/auth.ts`
- `dashboard/src/lib/stores/settings.ts`
- `docs/roadmap/factory-reset-danger-zone/frontend-flow.md`
- `docs/roadmap/factory-reset-danger-zone/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cd dashboard && npm run check && npm run build`

Exit criteria
- The danger zone is unmistakable, reset cannot trigger without exact typed confirmation, and both web and Tauri flows land in onboarding after success.
```
