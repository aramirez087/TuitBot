# Session 01: Charter And Reset Scope

Paste this into a new Claude Code session:

```md
Continuity
- Start from the current repository files only; do not assume prior artifacts.

Mission
Audit the current onboarding, auth, storage, and settings flows and lock the exact factory-reset contract before implementation.

Repository anchors
- `docs/architecture.md`
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-core/src/storage/mod.rs`
- `crates/tuitbot-core/src/auth/passphrase.rs`
- `crates/tuitbot-core/src/auth/session.rs`
- `crates/tuitbot-server/tests/fresh_install_auth.rs`
- `dashboard/src/routes/+layout.svelte`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/routes/(app)/settings/LanAccessSection.svelte`
- `dashboard/src/routes/onboarding/+page.svelte`
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/auth.ts`

Tasks
1. Document the current flows for configured and unconfigured instances, cookie auth, bearer auth, and onboarding redirects.
2. Define the destructive scope as a live reset that preserves the running server, SQLite schema, and `api_token` while clearing Tuitbot-managed data and file artifacts.
3. Decide the exact authenticated endpoint, request body, typed confirmation phrase, response shape, cookie-clearing behavior, and redirect target.
4. Explicitly exclude user-authored content source folders outside the app-managed data directory from deletion.
5. Split the work into backend and dashboard implementation steps with named files and tests.

Deliverables
- `docs/roadmap/factory-reset-danger-zone/charter.md`
- `docs/roadmap/factory-reset-danger-zone/session-01-handoff.md`

Quality gates
- No code changes are expected; if you touch code, stop and keep this session documentation-only.

Exit criteria
- The charter resolves reset scope, safety rules, endpoint contract, file plan, and exact inputs for Session 02.
```
