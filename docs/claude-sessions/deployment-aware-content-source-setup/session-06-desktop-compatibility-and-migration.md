# Session 06: Desktop Compatibility And Migration

Paste this into a new Claude Code session:

```md
Continue from Session 05 artifacts.
Continuity
- Read `docs/roadmap/deployment-aware-content-source-setup/frontend-flow.md` and `docs/roadmap/deployment-aware-content-source-setup/session-05-handoff.md` first.

Mission
Finish the backward-compatibility, upgrade, and documentation work so existing installs survive the new deployment-aware source model cleanly.

Repository anchors
- `config.example.toml`
- `docs/configuration.md`
- `docs/getting-started.md`
- `docs/lan-mode.md`
- `crates/tuitbot-core/src/config/tests.rs`
- `crates/tuitbot-server/tests/api_tests.rs`
- `dashboard/src/lib/components/onboarding/SourcesStep.svelte`
- `dashboard/src/routes/(app)/settings/ContentSourcesSection.svelte`

Tasks
1. Preserve existing desktop `local_fs` workflows, including Tauri browse behavior and previously saved local source configs.
2. Implement and document the migration path for legacy service-account-based Drive sources and any self-host installs that currently rely on manual local paths.
3. Update docs and examples so the recommended setup is explicit for desktop, LAN, and cloud deployments.
4. Add regression coverage for upgrades, legacy config parsing, and mixed old-plus-new source states.

Deliverables
- `docs/roadmap/deployment-aware-content-source-setup/migration-plan.md`
- `docs/roadmap/deployment-aware-content-source-setup/session-06-handoff.md`

Quality gates
- `cargo fmt --all && cargo fmt --all --check`
- `RUSTFLAGS="-D warnings" cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

Exit criteria
- Existing users have a documented and tested upgrade path, and Session 07 can validate both fresh installs and upgraded installs end to end.
```
