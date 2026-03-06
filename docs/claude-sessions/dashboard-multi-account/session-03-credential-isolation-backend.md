# Session 03: Credential Isolation Backend

Paste this into a new Claude Code session:

```md
Continue from Session 02 artifacts.

Continuity
- Use the Session 02 roadmap artifacts and code changes as the contract for this backend slice.

Mission
Implement per-account credential storage and X access backend flows so OAuth tokens and scraper sessions stop using global singleton files.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md`
- `crates/tuitbot-server/src/routes/accounts.rs`
- `crates/tuitbot-server/src/routes/scraper_session.rs`
- `crates/tuitbot-server/src/routes/runtime.rs`
- `crates/tuitbot-server/src/routes/content/compose.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-server/src/lib.rs`
- `crates/tuitbot-core/src/startup.rs`
- `crates/tuitbot-cli/src/commands/auth.rs`
- `crates/tuitbot-server/tests/api_tests.rs`

Tasks
1. Introduce per-account asset path helpers under `data_dir/accounts/<account_id>/` for OAuth tokens, scraper sessions, and related account-owned auth artifacts with safe default-account fallback.
2. Update backend posting and auth-status code paths to resolve credentials through those helpers instead of global `tokens.json` and `scraper_session.json`.
3. Add account-specific X credential-linking endpoints, reusing existing PKCE and token-save helpers where possible, and persist linked tokens to the selected account asset path.
4. Make scraper-session endpoints account-aware and expose the per-account status needed by the dashboard.
5. Add tests for two accounts proving isolated credential files, `can_post` state, and sync-profile behavior.
6. Write `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md` and `docs/roadmap/dashboard-multi-account/session-03-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/credential-isolation-contract.md`
- `docs/roadmap/dashboard-multi-account/session-03-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Two non-default accounts can hold different credentials without reusing the same on-disk files.
- Backend status and posting eligibility are computed from the selected account's credentials.
- The new credential routes are documented and covered by tests.
```
