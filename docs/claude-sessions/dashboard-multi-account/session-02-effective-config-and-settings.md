# Session 02: Effective Config And Settings

Paste this into a new Claude Code session:

```md
Continue from Session 01 artifacts.

Continuity
- Use the Session 01 roadmap artifacts as the only source of prior-session context.

Mission
Implement account-aware effective config loading and settings API behavior so each account sees its own merged configuration.

Repository anchors
- `docs/roadmap/dashboard-multi-account/charter.md`
- `docs/roadmap/dashboard-multi-account/implementation-plan.md`
- `crates/tuitbot-server/src/routes/settings.rs`
- `crates/tuitbot-server/src/account.rs`
- `crates/tuitbot-server/src/state.rs`
- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/config/types.rs`
- `crates/tuitbot-core/src/config/validation.rs`
- `crates/tuitbot-core/src/storage/accounts.rs`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`
- `dashboard/src/lib/stores/settings.ts`

Tasks
1. Add backend helpers that load `config.toml`, merge the selected account's `config_overrides`, and produce an effective config plus override metadata.
2. Enforce the Session 01 scope contract when validating and patching settings so account-scoped fields persist to `accounts.config_overrides` while instance-scoped fields stay install-wide.
3. Update `GET /api/settings`, `PATCH /api/settings`, `POST /api/settings/validate`, and related TypeScript types to be account-aware through `AccountContext`.
4. Keep the default account backward-compatible and make null-clears or remove-override behavior explicit and tested.
5. Add backend and TypeScript coverage for merge precedence, forbidden instance-scoped edits, and fallback to the base config.
6. Write `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md` and `docs/roadmap/dashboard-multi-account/session-02-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md`
- `docs/roadmap/dashboard-multi-account/session-02-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`

Exit criteria
- Selected-account settings reads return an effective config instead of the raw singleton file.
- Saving settings from a non-default account no longer mutates install-wide fields outside the approved scope contract.
- Tests cover default-account and second-account behavior.
```
