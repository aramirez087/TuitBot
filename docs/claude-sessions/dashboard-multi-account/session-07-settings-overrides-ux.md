# Session 07: Settings Overrides UX

Paste this into a new Claude Code session:

```md
Continue from Session 06 artifacts.

Continuity
- Preserve the Session 02 settings-scope contract and the Session 05 account-switch invalidation behavior.

Mission
Make the Settings experience truly account-scoped by editing and visualizing per-account overrides instead of a singleton config file.

Repository anchors
- `docs/roadmap/dashboard-multi-account/settings-scope-matrix.md`
- `docs/roadmap/dashboard-multi-account/account-management-flow.md`
- `dashboard/src/lib/stores/settings.ts`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/routes/(app)/settings/SaveBar.svelte`
- `dashboard/src/routes/(app)/settings/XApiSection.svelte`
- `dashboard/src/routes/(app)/settings/LlmProviderSection.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`
- `dashboard/src/lib/api/client.ts`
- `dashboard/src/lib/api/types.ts`

Tasks
1. Update the settings store and API types to consume effective-config metadata and show whether values are inherited or overridden for the selected account.
2. Add reset-to-base affordances for overrideable fields and block or redirect edits to instance-scoped fields according to the Session 02 contract.
3. Keep save and validate flows account-aware and ensure account switches discard or protect dirty drafts without cross-account leakage.
4. Add targeted coverage for override badges, reset behavior, and dirty-form edge cases during account switching.
5. Write `docs/roadmap/dashboard-multi-account/settings-override-ux.md` and `docs/roadmap/dashboard-multi-account/session-07-handoff.md`.

Deliverables
- `docs/roadmap/dashboard-multi-account/settings-override-ux.md`
- `docs/roadmap/dashboard-multi-account/session-07-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `npm --prefix dashboard run check`
- `npm --prefix dashboard run build`

Exit criteria
- A non-default account clearly shows inherited versus overridden settings.
- Users can reset account-specific overrides without editing the base config by hand.
- Account switching no longer risks saving one account's draft into another account's settings.
```
