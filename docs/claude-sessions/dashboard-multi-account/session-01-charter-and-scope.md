# Session 01: Charter And Scope

Paste this into a new Claude Code session:

```md
Continuity
- This is the first implementation session for this epic.

Mission
Audit the existing multi-account foundation and produce the charter, scope matrix, and execution plan for full dashboard multi-account support.

Repository anchors
- `crates/tuitbot-core/migrations/20260227000015_multi_account_foundation.sql`
- `crates/tuitbot-core/src/storage/accounts.rs`
- `crates/tuitbot-server/src/account.rs`
- `crates/tuitbot-server/src/routes/accounts.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/routes/scraper_session.rs`
- `crates/tuitbot-server/src/main.rs`
- `dashboard/src/lib/stores/accounts.ts`
- `dashboard/src/lib/components/AccountSwitcher.svelte`
- `dashboard/src/routes/(app)/+layout.svelte`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `docs/architecture.md`

Tasks
1. Audit what already exists for the account registry, account header routing, account-scoped tables, and the partial dashboard switcher.
2. Identify every singleton seam blocking full multi-account: effective config loading, token and scraper-session files, runtime bootstrap, watchtower/content sources, websocket/UI invalidation, and settings UX.
3. Decide and document the contract for account-scoped vs instance-scoped settings, default-account backward compatibility, and the preferred in-dashboard credential-linking flow.
4. Write `docs/roadmap/dashboard-multi-account/charter.md` with the problem statement, current state, design decisions, UX goals, risks, and implementation slices for later sessions.
5. Write `docs/roadmap/dashboard-multi-account/implementation-plan.md` as a concise sequence map naming each later session, its code/doc targets, and its main regression risks.
6. Write `docs/roadmap/dashboard-multi-account/session-01-handoff.md` with what was audited, decisions made, open issues, and exact inputs for Session 02.

Deliverables
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/implementation-plan.md`
- `docs/roadmap/dashboard-multi-account/session-01-handoff.md`

Exit criteria
- The charter is specific to this repository rather than generic product planning.
- Singleton seams and scope boundaries are explicit enough that later sessions can implement without re-deciding core contracts.
- The handoff names concrete files and tests Session 02 must use.
```
