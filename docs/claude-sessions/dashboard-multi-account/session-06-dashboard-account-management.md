# Session 06: Dashboard Account Management

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.

Continuity
- Build on the stabilized shell and switching flow instead of introducing a separate account state system.

Mission
Add dashboard account-management flows so users can create, rename, archive, and inspect accounts from the UI instead of relying on hidden APIs.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/frontend-switching-flow.md`
- `dashboard/src/lib/stores/accounts.ts`
- `dashboard/src/lib/components/AccountSwitcher.svelte`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `crates/tuitbot-server/src/routes/accounts.rs`

Tasks
1. Add an account-roster surface reachable from the dashboard with list, create, rename, archive, and profile-refresh actions.
2. Ensure create-account flow chooses sane defaults, selects the new account immediately, and handles empty-state or first-add flows cleanly.
3. Guard against destructive actions on the current or default account and reflect backend validation errors inline.
4. Add targeted coverage for create, rename, archive, and sync-profile UX.
5. Write `docs/roadmap/dashboard-multi-account/account-management-flow.md` and `docs/roadmap/dashboard-multi-account/session-06-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/account-management-flow.md`
- `docs/roadmap/dashboard-multi-account/session-06-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run build`

Exit criteria
- Users can fully manage the account roster from the dashboard.
- New accounts become immediately selectable and archived accounts disappear cleanly from active pickers.
- Error handling and empty states are production-ready rather than hidden console failures.
```
