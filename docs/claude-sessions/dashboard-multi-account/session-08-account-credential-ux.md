# Session 08: Account Credential UX

Paste this into a new Claude Code session:

```md
Continue from Session 07 artifacts.

Continuity
- Reuse the credential backend from Session 03 and the account shell from Sessions 05-07 rather than bypassing them.

Mission
Implement dashboard credential-linking flows so each account can connect X API or scraper credentials without leaving the app.

Repository anchors
- `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md`
- `docs/roadmap/dashboard-multi-account/settings-override-ux.md`
- `dashboard/src/routes/(app)/settings/XApiSection.svelte`
- `dashboard/src/routes/(app)/settings/BrowserSessionSection.svelte`
- `dashboard/src/lib/stores/accounts.ts`
- `dashboard/src/lib/stores/runtime.ts`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `crates/tuitbot-server/src/routes/accounts.rs`
- `crates/tuitbot-server/src/routes/scraper_session.rs`
- `crates/tuitbot-server/src/lib.rs`

Tasks
1. Add any missing account-scoped credential-status endpoints and TypeScript types for X OAuth tokens, scraper sessions, and profile-sync status.
2. Build selected-account X access UI that can start or finish OAuth linking, import or remove scraper sessions, and show last-linked status without mutating other accounts.
3. Ensure credential changes refresh current-account profile data, runtime posting capability, and composer affordances immediately after success.
4. Add targeted coverage for connect, reconnect, disconnect, and cross-account isolation.
5. Write `docs/roadmap/dashboard-multi-account/x-access-account-flow.md` and `docs/roadmap/dashboard-multi-account/session-08-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/x-access-account-flow.md`
- `docs/roadmap/dashboard-multi-account/session-08-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run build`

Exit criteria
- Each account can manage its own X credentials from the dashboard.
- Credential actions update the selected account's status immediately without bleeding into other accounts.
- The UI makes per-account credential state understandable before users try to post.
```
