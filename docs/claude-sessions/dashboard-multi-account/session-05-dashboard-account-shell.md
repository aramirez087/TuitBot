# Session 05: Dashboard Account Shell

Paste this into a new Claude Code session:

```md
Continue from Session 04 artifacts.

Continuity
- Use the runtime and config contracts already documented under `docs/roadmap/dashboard-multi-account/`.

Mission
Build the dashboard shell that boots, persists, and switches active account context without stale data leaking across routes.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/runtime-isolation-plan.md`
- `dashboard/src/routes/+layout.svelte`
- `dashboard/src/routes/(app)/+layout.svelte`
- `dashboard/src/lib/api/http.ts`
- `dashboard/src/lib/stores/accounts.ts`
- `dashboard/src/lib/stores/websocket.ts`
- `dashboard/src/lib/components/Sidebar.svelte`
- `dashboard/src/lib/components/AccountSwitcher.svelte`
- `dashboard/src/lib/components/composer/ComposeWorkspace.svelte`

Tasks
1. Replace the current account bootstrap with validation against `/api/accounts`, safe fallback to default or first active account, and explicit loading or error states.
2. When the active account changes, invalidate account-scoped stores, reconnect live data as needed, and prevent stale content from the previous account from flashing.
3. Upgrade the sidebar and shell for zero, one, or many accounts and keep the active account identity visible across the app and composer.
4. Add targeted coverage for bootstrap, invalid persisted account ids, and switch-driven refetch behavior.
5. Write `docs/roadmap/dashboard-multi-account/frontend-switching-flow.md` and `docs/roadmap/dashboard-multi-account/session-05-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/frontend-switching-flow.md`
- `docs/roadmap/dashboard-multi-account/session-05-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run build`

Exit criteria
- Persisted account selection is validated on boot and stale ids no longer strand the UI.
- Switching accounts refreshes the visible data model without showing stale content from the prior account.
- The shell UX is usable for first account, single account, and multi-account states.
```
